//! Gerenciamento de Conectores Declarativos (`connectors/*.json`) atualizáveis sem reinstalar o app.
//!
//! Prioridade de carregamento:
//! 1. `<MAM_LIBRARY>/System/connectors/sites.json` (atualizações baixadas ou importadas offline)
//! 2. `<exe_dir>/connectors/default_sites.json` (embutido no pacote portátil)
//! 3. Fallback embutido no binário (`include_str!`)

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ObraBuscaResultado, SourceError};

pub const URL_MANIFESTO_OFICIAL: &str =
    "https://raw.githubusercontent.com/GabbCamilo/mam/main/connectors/manifest.json";

const DEFAULT_SITES_EMBEDDED: &str = include_str!("../../../connectors/default_sites.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConectorSiteConfig {
    pub id: String,
    pub nome: String,
    #[serde(default = "versao_padrao")]
    pub versao: String,
    #[serde(default = "tipo_padrao")]
    pub tipo: String, // "api" | "web-cdp" | "web"
    #[serde(default)]
    pub dominios: Vec<String>,
    #[serde(default)]
    pub url_busca: Option<String>,
    #[serde(default)]
    pub padroes_url_capitulo: Vec<String>,
    #[serde(default)]
    pub seletor_link: Option<String>,
    #[serde(default)]
    pub seletor_idioma_linha: Option<String>,
    #[serde(default)]
    pub seletor_paginacao: Option<String>,
    #[serde(default = "cdp_auto")]
    pub requer_cdp: String,
    #[serde(default)]
    pub seletores_imagem: Vec<String>,
    #[serde(default)]
    pub rejeitar_tokens_imagem: Vec<String>,
}

fn versao_padrao() -> String {
    "1.0.0".to_string()
}
fn tipo_padrao() -> String {
    "web".to_string()
}
fn cdp_auto() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogoConectores {
    pub schema_version: u32,
    pub version: String,
    pub sites: Vec<ConectorSiteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestoConectores {
    pub schema_version: u32,
    pub version: String,
    pub sites_url: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub notas: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusAtualizacaoConectores {
    pub versao_atual: String,
    pub versao_remota: Option<String>,
    pub atualizacao_disponivel: bool,
    pub notas: Option<String>,
}

pub struct GerenciadorConectores;

impl GerenciadorConectores {
    /// Caminho do arquivo de conectores atualizado pelo usuário dentro da biblioteca (`System/connectors/sites.json`).
    pub fn caminho_usuario(raiz_biblioteca: &Path) -> PathBuf {
        raiz_biblioteca
            .join("System")
            .join("connectors")
            .join("sites.json")
    }

    /// Carrega o catálogo ativo respeitando a precedência:
    /// `System/connectors/sites.json` > `<exe_dir>/connectors/default_sites.json` > embutido.
    pub fn carregar(raiz_biblioteca: Option<&Path>) -> CatalogoConectores {
        if let Some(raiz) = raiz_biblioteca {
            let user_path = Self::caminho_usuario(raiz);
            if let Ok(raw) = std::fs::read_to_string(&user_path) {
                if let Ok(cat) = Self::validar_catalogo_json(&raw) {
                    return cat;
                }
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let pkg_path = dir.join("connectors").join("default_sites.json");
                if let Ok(raw) = std::fs::read_to_string(&pkg_path) {
                    if let Ok(cat) = Self::validar_catalogo_json(&raw) {
                        return cat;
                    }
                }
            }
        }

        Self::validar_catalogo_json(DEFAULT_SITES_EMBEDDED).expect("default_sites.json embutido válido")
    }

    /// Valida estrutura JSON do catálogo de conectores.
    pub fn validar_catalogo_json(raw: &str) -> Result<CatalogoConectores, SourceError> {
        let cat: CatalogoConectores = serde_json::from_str(raw)
            .map_err(|e| SourceError::Operacao(format!("JSON de conectores inválido: {}", e)))?;
        if cat.schema_version != 1 {
            return Err(SourceError::Operacao(format!(
                "Versão de schema não suportada: {}",
                cat.schema_version
            )));
        }
        if cat.sites.is_empty() {
            return Err(SourceError::Operacao(
                "Catálogo de conectores não pode estar vazio".to_string(),
            ));
        }
        for s in &cat.sites {
            if s.id.trim().is_empty() || s.nome.trim().is_empty() {
                return Err(SourceError::Operacao(
                    "Cada conector precisa de `id` e `nome` válidos".to_string(),
                ));
            }
        }
        Ok(cat)
    }

    /// Grava atomicamente um catálogo validado em `<MAM_LIBRARY>/System/connectors/sites.json`.
    pub fn salvar_catalogo_atomico(
        raiz_biblioteca: &Path,
        raw_json: &str,
        sha256_esperado: Option<&str>,
    ) -> Result<CatalogoConectores, SourceError> {
        if let Some(esperado) = sha256_esperado.filter(|s| !s.trim().is_empty()) {
            let mut hasher = Sha256::new();
            hasher.update(raw_json.as_bytes());
            let calculado = format!("{:x}", hasher.finalize());
            if !calculado.eq_ignore_ascii_case(esperado.trim()) {
                return Err(SourceError::Operacao(format!(
                    "Hash SHA-256 divergente na atualização de conectores (esperado {}, obtido {})",
                    esperado, calculado
                )));
            }
        }

        let cat = Self::validar_catalogo_json(raw_json)?;
        let destino = Self::caminho_usuario(raiz_biblioteca);
        if let Some(pai) = destino.parent() {
            std::fs::create_dir_all(pai).map_err(|e| SourceError::Operacao(e.to_string()))?;
        }
        let temp = destino.with_extension("json.mam_tmp");
        std::fs::write(&temp, raw_json.as_bytes())
            .map_err(|e| SourceError::Operacao(e.to_string()))?;
        if destino.exists() {
            let _ = std::fs::remove_file(&destino);
        }
        std::fs::rename(&temp, &destino).map_err(|e| SourceError::Operacao(e.to_string()))?;
        Ok(cat)
    }

    /// Verifica no manifesto remoto se existe uma versão mais nova de conectores.
    pub async fn verificar_atualizacao_remota(
        raiz_biblioteca: Option<&Path>,
        manifest_url: Option<&str>,
    ) -> Result<StatusAtualizacaoConectores, SourceError> {
        let atual = Self::carregar(raiz_biblioteca);
        let url = manifest_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(URL_MANIFESTO_OFICIAL);

        let client = reqwest::Client::builder()
            .user_agent("MAM-Updater/0.1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao consultar manifesto de conectores",
                resp.status()
            )));
        }
        let manifesto: ManifestoConectores = resp.json().await?;
        let disponivel = manifesto.version.trim() != atual.version.trim();

        Ok(StatusAtualizacaoConectores {
            versao_atual: atual.version,
            versao_remota: Some(manifesto.version),
            atualizacao_disponivel: disponivel,
            notas: manifesto.notas,
        })
    }

    /// Baixa o catálogo do manifesto remoto, valida schema + SHA-256 e salva atomicamente.
    pub async fn atualizar_remoto(
        raiz_biblioteca: &Path,
        manifest_url: Option<&str>,
    ) -> Result<CatalogoConectores, SourceError> {
        let url = manifest_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(URL_MANIFESTO_OFICIAL);

        let client = reqwest::Client::builder()
            .user_agent("MAM-Updater/0.1.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let resp_man = client.get(url).send().await?;
        if !resp_man.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao baixar manifesto de conectores",
                resp_man.status()
            )));
        }
        let manifesto: ManifestoConectores = resp_man.json().await?;

        let resp_sites = client.get(&manifesto.sites_url).send().await?;
        if !resp_sites.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao baixar catálogo de conectores",
                resp_sites.status()
            )));
        }
        let raw_sites = resp_sites.text().await?;
        Self::salvar_catalogo_atomico(raiz_biblioteca, &raw_sites, manifesto.sha256.as_deref())
    }

    /// Encontra todos os padrões de URL de capítulo configurados para um domínio (ou todos os conectores).
    pub fn padroes_capitulo_ativos(raiz_biblioteca: Option<&Path>) -> Vec<String> {
        let cat = Self::carregar(raiz_biblioteca);
        let mut out = Vec::new();
        for site in cat.sites {
            for p in site.padroes_url_capitulo {
                let norm = p.trim().to_lowercase();
                if !norm.is_empty() && !out.contains(&norm) {
                    out.push(norm);
                }
            }
        }
        out
    }

    /// Busca obras usando um conector web com `url_busca` via `ai-engine` (CDP).
    pub async fn buscar_obras_via_cdp(
        conector: &ConectorSiteConfig,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let template = conector.url_busca.as_deref().ok_or_else(|| {
            SourceError::Operacao(format!(
                "Conector '{}' não possui mecanismo de busca por título configurado",
                conector.nome
            ))
        })?;
        let query_enc: String = consulta
            .trim()
            .bytes()
            .flat_map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    vec![b as char]
                }
                b' ' => vec!['+'],
                _ => format!("%{:02X}", b).chars().collect(),
            })
            .collect();
        let url_busca = template.replace("{query}", &query_enc);

        let py = mam_ai_engine::AiEngineManager::detectar_executavel_python();
        let dir = mam_ai_engine::AiEngineManager::detectar_diretorio_padrao();
        let manager = mam_ai_engine::AiEngineManager::nova(py, dir)
            .com_timeout(std::time::Duration::from_secs(45));
        let val = manager
            .chamar(
                "smart_search_works",
                serde_json::json!({
                    "url_busca": url_busca,
                    "conector_id": conector.id,
                    "dominios": conector.dominios,
                }),
            )
            .await
            .map_err(|e| SourceError::Operacao(e.to_string()))?;

        let obras = serde_json::from_value::<Vec<ObraBuscaResultado>>(
            val.get("obras").cloned().unwrap_or(serde_json::json!([])),
        )
        .map_err(|e| SourceError::Operacao(e.to_string()))?;
        Ok(obras)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedencia_conectores_usuario_sobrepoe_padrao() {
        let base = std::env::temp_dir().join(format!("mam_conn_prec_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        let padrao = GerenciadorConectores::carregar(Some(&base));
        assert_eq!(padrao.version, "2026.11.21.1");

        let custom_json = r#"{
            "schema_version": 1,
            "version": "2026.11.22.9",
            "sites": [{
                "id": "novosite",
                "nome": "Novo Site Exemplo",
                "dominios": ["novosite.example"],
                "padroes_url_capitulo": ["/leitura/cap-"]
            }]
        }"#;
        let salvo = GerenciadorConectores::salvar_catalogo_atomico(&base, custom_json, None).unwrap();
        assert_eq!(salvo.version, "2026.11.22.9");

        let carregado = GerenciadorConectores::carregar(Some(&base));
        assert_eq!(carregado.version, "2026.11.22.9");
        assert_eq!(carregado.sites[0].id, "novosite");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn conector_dinamico_adiciona_suporte_a_site_novo_sem_recompilar() {
        let base = std::env::temp_dir().join(format!("mam_conn_dyn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        let custom_json = r#"{
            "schema_version": 1,
            "version": "2026.11.22.1",
            "sites": [{
                "id": "novosite",
                "nome": "Novo Site",
                "dominios": ["novosite.example"],
                "padroes_url_capitulo": ["/leitura/cap-"]
            }]
        }"#;
        GerenciadorConectores::salvar_catalogo_atomico(&base, custom_json, None).unwrap();

        let padroes = GerenciadorConectores::padroes_capitulo_ativos(Some(&base));
        assert!(padroes.contains(&"/leitura/cap-".to_string()));

        let html = r#"
            <html><head><title>Obra Nova</title></head><body>
              <a href="/obra/1/leitura/cap-05">Capítulo 5</a>
              <a href="/obra/1/leitura/cap-06">Capítulo 6</a>
            </body></html>
        "#;
        let res = crate::extractors::extrair_capitulos_de_html_com_padroes(
            "https://novosite.example/obra/1",
            html,
            &padroes,
        );
        assert_eq!(res.capitulos.len(), 2);
        assert_eq!(res.capitulos[0].numero, Some(5));
        assert_eq!(res.capitulos[1].numero, Some(6));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn atualizacao_conectores_preserva_biblioteca_e_faz_rollback_em_falha() {
        let base = std::env::temp_dir().join(format!("mam_conn_rb_{}", uuid::Uuid::new_v4()));
        let cbz = base.join("Manga/Obra Existente/Capitulo_0001.cbz");
        std::fs::create_dir_all(cbz.parent().unwrap()).unwrap();
        std::fs::write(&cbz, b"CBZ-ORIGINAL").unwrap();

        let v1 = r#"{
            "schema_version": 1,
            "version": "2026.11.21.1",
            "sites": [{"id": "site-v1", "nome": "Site V1", "dominios": ["v1.example"], "padroes_url_capitulo": ["/cap-"]}]
        }"#;
        GerenciadorConectores::salvar_catalogo_atomico(&base, v1, None).unwrap();

        // Tentativa de atualização com SHA-256 inválido deve falhar e preservar v1 + biblioteca
        let err = GerenciadorConectores::salvar_catalogo_atomico(
            &base,
            v1,
            Some("0000000000000000000000000000000000000000000000000000000000000000"),
        );
        assert!(err.is_err());

        // Tentativa com JSON inválido também deve preservar v1
        let err_json = GerenciadorConectores::salvar_catalogo_atomico(&base, "{broken", None);
        assert!(err_json.is_err());

        let atual = GerenciadorConectores::carregar(Some(&base));
        assert_eq!(atual.sites[0].id, "site-v1");
        assert_eq!(std::fs::read(&cbz).unwrap(), b"CBZ-ORIGINAL");

        let _ = std::fs::remove_dir_all(&base);
    }
}
