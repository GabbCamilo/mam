//! # mam-ai-engine
//! Coordenação do Manga AI Engine com o processo Python auxiliar
//! via JSON-RPC 2.0 sobre stdio (JSONL).

pub mod protocol;

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

pub use protocol::{AiEngineError, JsonRpcRequest, JsonRpcResponse, ModeloInfo};

struct ProcessoAtivo {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

/// Gerenciador do processo auxiliar Python do Manga AI Engine.
pub struct AiEngineManager {
    executavel_python: String,
    diretorio_ai_engine: PathBuf,
    timeout: Duration,
    proximo_id: AtomicU64,
    processo: Mutex<Option<ProcessoAtivo>>,
}

impl AiEngineManager {
    /// Cria um novo gerenciador apontando para o diretório do módulo Python `ai-engine/`.
    pub fn nova(executavel_python: impl Into<String>, diretorio_ai_engine: PathBuf) -> Self {
        Self {
            executavel_python: executavel_python.into(),
            diretorio_ai_engine,
            timeout: Duration::from_secs(10),
            proximo_id: AtomicU64::new(1),
            processo: Mutex::new(None),
        }
    }

    /// Resolve o executável Python priorizando:
    /// 1. Variável de ambiente `MAM_PYTHON_EXE`
    /// 2. Runtime Python embutido ao lado do executável (`<exe_dir>/python/python.exe` ou `python/bin/python3`)
    /// 3. Fallback `python` no PATH do sistema
    pub fn detectar_executavel_python() -> String {
        if let Ok(env_py) = std::env::var("MAM_PYTHON_EXE") {
            let trimmed = env_py.trim();
            if !trimmed.is_empty() && Path::new(trimmed).exists() {
                return trimmed.to_string();
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let candidatos = [
                    exe_dir.join("python/python.exe"),
                    exe_dir.join("python/bin/python3"),
                ];
                for cand in candidatos {
                    if cand.exists() {
                        return cand.to_string_lossy().to_string();
                    }
                }
            }
        }

        "python".to_string()
    }

    /// Resolve o diretório padrão do `ai-engine` priorizando:
    /// 1. Variável de ambiente `MAM_AI_ENGINE_DIR`
    /// 2. Diretório ao lado do executável portátil (`<exe_dir>/ai-engine`)
    /// 3. Caminhos relativos ao workspace de desenvolvimento (`ai-engine`, `../ai-engine`, `../../ai-engine`)
    pub fn detectar_diretorio_padrao() -> PathBuf {
        if let Ok(env_dir) = std::env::var("MAM_AI_ENGINE_DIR") {
            let p = PathBuf::from(env_dir);
            if p.join("mam_ai_engine").exists() {
                return p;
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let candidatos_exe = [
                    exe_dir.join("ai-engine"),
                    exe_dir.join("../share/mam/ai-engine"),
                ];
                for cand in candidatos_exe {
                    if cand.join("mam_ai_engine").exists() {
                        return cand;
                    }
                }
            }
        }

        let candidatos = [
            PathBuf::from("ai-engine"),
            PathBuf::from("../ai-engine"),
            PathBuf::from("../../ai-engine"),
        ];
        for cand in candidatos {
            if cand.join("mam_ai_engine").exists() {
                return cand;
            }
        }
        PathBuf::from("ai-engine")
    }

    /// Garante que o processo Python auxiliar está iniciado.
    async fn garantir_processo<'a>(
        &self,
        guard: &'a mut Option<ProcessoAtivo>,
    ) -> Result<&'a mut ProcessoAtivo, AiEngineError> {
        if let Some(proc) = guard.as_mut() {
            if let Ok(Some(_)) = proc.child.try_wait() {
                *guard = None;
            }
        }

        if guard.is_none() {
            let mut cmd = Command::new(&self.executavel_python);
            cmd.arg("-m")
                .arg("mam_ai_engine.server")
                .env("PYTHONPATH", &self.diretorio_ai_engine)
                .env("PYTHONUNBUFFERED", "1")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            let mut child = cmd.spawn()?;
            let stdin = child
                .stdin
                .take()
                .ok_or(AiEngineError::ProcessoEncerrado)?;
            let stdout = BufReader::new(
                child
                    .stdout
                    .take()
                    .ok_or(AiEngineError::ProcessoEncerrado)?,
            );

            *guard = Some(ProcessoAtivo {
                child,
                stdin,
                stdout,
            });
        }

        guard.as_mut().ok_or(AiEngineError::ProcessoEncerrado)
    }

    /// Envia uma requisição JSON-RPC 2.0 (JSONL) e aguarda resposta com timeout.
    pub async fn chamar(&self, method: &str, params: Value) -> Result<Value, AiEngineError> {
        let id = self.proximo_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest::nova(id, method, params);
        let linha = format!("{}\n", serde_json::to_string(&req)?);

        let mut guard = self.processo.lock().await;
        let proc = self.garantir_processo(&mut guard).await?;

        proc.stdin.write_all(linha.as_bytes()).await?;
        proc.stdin.flush().await?;

        let mut resposta_raw = String::new();
        let lido = tokio::time::timeout(self.timeout, proc.stdout.read_line(&mut resposta_raw))
            .await
            .map_err(|_| AiEngineError::Timeout)??;

        if lido == 0 {
            *guard = None;
            return Err(AiEngineError::ProcessoEncerrado);
        }

        let resp: JsonRpcResponse = serde_json::from_str(resposta_raw.trim())?;
        if let Some(err) = resp.error {
            return Err(AiEngineError::Rpc {
                codigo: err.code,
                mensagem: err.message,
            });
        }

        Ok(resp.result.unwrap_or(Value::Null))
    }

    /// Verifica saúde e provedores de inferência do AI Engine.
    pub async fn ping(&self) -> Result<Value, AiEngineError> {
        self.chamar("ping", json!({})).await
    }

    /// Executa pipeline de OCR local em uma página sem alterar o CBZ original.
    pub async fn processar_ocr(
        &self,
        caminho_imagem: &Path,
        idioma_origem: &str,
    ) -> Result<Value, AiEngineError> {
        self.chamar(
            "ocr",
            json!({
                "caminho_imagem": caminho_imagem.to_string_lossy(),
                "idioma_origem": idioma_origem,
            }),
        )
        .await
    }

    /// Lista modelos opcionais instalados localmente sem realizar downloads.
    pub async fn listar_modelos(&self, diretorios: Vec<String>) -> Result<Vec<ModeloInfo>, AiEngineError> {
        let val = self
            .chamar("models_list", json!({ "diretorios": diretorios }))
            .await?;
        Ok(serde_json::from_value(val)?)
    }

    /// Solicita encerramento limpo do processo Python auxiliar.
    pub async fn shutdown(&self) -> Result<(), AiEngineError> {
        let _ = self.chamar("shutdown", json!({})).await;
        let mut guard = self.processo.lock().await;
        if let Some(mut proc) = guard.take() {
            let _ = proc.child.kill().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn comunicacao_stdio_json_rpc_com_python() {
        let dir_ai = AiEngineManager::detectar_diretorio_padrao();
        if !dir_ai.join("mam_ai_engine").exists() {
            return;
        }

        let manager = AiEngineManager::nova("python", dir_ai);
        let ping = manager.ping().await.expect("ping no ai-engine");
        assert_eq!(ping["status"], "ok");
        assert_eq!(ping["provedores"]["cpu"], true);

        let ocr = manager
            .processar_ocr(Path::new("pagina_001.jpg"), "ja")
            .await
            .expect("ocr local");
        assert_eq!(ocr["estado_revisao"], "nao_revisado");

        manager.shutdown().await.expect("shutdown limpo");
    }
}
