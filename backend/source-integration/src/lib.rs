//! # mam-source-integration
//! Busca e obtenção autorizada de obras e capítulos em fontes compatíveis.
//!
//! A integração com fontes é modular e desacoplada. Cada fonte implementa o
//! trait `Fonte`. Somente fontes autorizadas (eh_autorizada == true) permitem
//! arquivamento. Não há mecanismos para contornar DRM, paywalls ou controles
//! de acesso.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use mam_database::Database;


use serde::{Deserialize, Serialize};

pub mod extractors;
pub mod fonte;
pub mod mangadex;
mod busca;
mod capitulos;
mod operacao;
mod duplicatas;

pub use fonte::Fonte;

/// Erros do módulo de integração com fontes.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("Erro de rede: {0}")]
    Rede(#[from] reqwest::Error),
    #[error("Erro de serialização: {0}")]
    Serializacao(#[from] serde_json::Error),
    #[error("Erro de banco: {0}")]
    Banco(#[from] mam_database::DatabaseError),
    #[error("Fonte não encontrada: {0}")]
    FonteNaoEncontrada(String),
    #[error("Fonte não autorizada: {0}")]
    FonteNaoAutorizada(String),
    #[error("Erro de fonte: {0}")]
    Fonte(#[from] crate::fonte::FonteError),
    #[error("Obra não encontrada: {0}")]
    ObraNaoEncontrada(String),
    #[error("Capítulo não encontrado: {0}")]
    CapituloNaoEncontrado(String),
    #[error("Erro de operação: {0}")]
    Operacao(String),
}

/// Resultado de busca de obra em uma fonte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObraBuscaResultado {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub titulo: String,
    pub autor: Option<String>,
    pub descricao: Option<String>,
    #[serde(default)]
    pub capa_url: Option<String>,
}

/// Resultado de listagem de capítulos em uma fonte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapituloBuscaResultado {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub capitulo_id_fonte: String,
    pub numero: Option<i32>,
    pub titulo: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub idioma: Option<String>,
}

/// Gerenciador de integração com fontes.
/// Orquestra o registro de fontes, busca de obras, listagem de capítulos
/// e operações de arquivamento.
pub struct SourceIntegrationManager {
    fontes: Arc<RwLock<HashMap<String, Arc<dyn Fonte>>>>,
    db: Arc<Database>,
    operacoes: Arc<RwLock<HashMap<String, operacao::OperacaoArquivamento>>>,
}

impl SourceIntegrationManager {
    pub fn nova(db: Arc<Database>) -> Self {
        Self {
            fontes: Arc::new(RwLock::new(HashMap::new())),
            db,
            operacoes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registra uma nova fonte no gerenciador.
    pub async fn registrar_fonte(
        &self,
        fonte: Arc<dyn Fonte>,
    ) -> Result<(), SourceError> {
        let id = fonte.id().to_string();
        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(id.clone()));
        }
        self.fontes.write().await.insert(id.clone(), fonte);
        Ok(())
    }

    /// Lista todas as fontes registradas.
    pub async fn listar_fontes(&self) -> Vec<FonteInfo> {
        let fontes = self.fontes.read().await;
        fontes
            .values()
            .map(|f| FonteInfo {
                id: f.id().to_string(),
                nome: f.nome().to_string(),
                autorizada: f.eh_autorizada(),
            })
            .collect()
    }

    /// Busca obras por título em todas as fontes autorizadas.
    pub async fn buscar_obras(
        &self,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let mut resultados = Vec::new();

        for fonte in fontes.values() {
            if !fonte.eh_autorizada() {
                continue;
            }
            match fonte.buscar_obras(consulta).await {
                Ok(obras) => {
                    for obra in obras {
                        resultados.push(ObraBuscaResultado {
                            fonte_id: fonte.id().to_string(),
                            obra_id_fonte: obra.obra_id_fonte,
                            titulo: obra.titulo,
                            autor: obra.autor,
                            descricao: obra.descricao,
                            capa_url: obra.capa_url,
                        });
                    }
                }
                Err(_) => {
                    // Fonte falhou, continua com as outras
                    continue;
                }
            }
        }

        Ok(resultados)
    }

    /// Lista capítulos de uma obra em uma fonte específica.
    pub async fn listar_capitulos(
        &self,
        fonte_id: &str,
        obra_id_fonte: &str,
    ) -> Result<Vec<CapituloBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let fonte = fontes
            .get(fonte_id)
            .ok_or_else(|| SourceError::FonteNaoEncontrada(fonte_id.to_string()))?;

        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(fonte_id.to_string()));
        }

        let capitulos = fonte.listar_capitulos(obra_id_fonte).await?;

        Ok(capitulos
            .into_iter()
            .map(|c| CapituloBuscaResultado {
                fonte_id: fonte.id().to_string(),
                obra_id_fonte: obra_id_fonte.to_string(),
                capitulo_id_fonte: c.capitulo_id_fonte,
                numero: c.numero,
                titulo: c.titulo,
                url: c.url,
                idioma: c.idioma,
            })
            .collect())
    }

    /// Verifica se uma obra já está arquivada no banco (detecção de duplicata).
    pub async fn obra_ja_arquivada(&self, titulo: &str) -> Result<bool, SourceError> {
        Ok(self.db.buscar_obra_por_titulo(titulo).await?.is_some())
    }

    /// Verifica se um capítulo já está arquivado (detecção de duplicata por caminho).
    pub async fn capitulo_ja_arquivado(
        &self,
        caminho_relativo: &str,
    ) -> Result<bool, SourceError> {
        Ok(self.db.consultar_integridade(caminho_relativo).await?.is_some())
    }

    /// Inicia uma operação de arquivamento de capítulos.
    pub async fn iniciar_operacao_arquivamento(
        &self,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
    ) -> Result<String, SourceError> {
        let id_operacao = uuid::Uuid::new_v4().to_string();

        let operacao = operacao::OperacaoArquivamento::nova(
            id_operacao.clone(),
            obra_titulo,
            capitulos,
        );

        self.operacoes.write().await.insert(id_operacao.clone(), operacao);

        Ok(id_operacao)
    }

    /// Retorna o status de uma operação de arquivamento.
    pub async fn status_operacao(
        &self,
        id_operacao: &str,
    ) -> Result<Option<StatusOperacao>, SourceError> {
        let operacoes = self.operacoes.read().await;
        Ok(operacoes.get(id_operacao).map(|o| {
            let s = o.status();
            StatusOperacao {
                id: o.id.clone(),
                obra_titulo: o.obra_titulo.clone(),
                total_capitulos: s.total_capitulos,
                capitulos_processados: s.capitulos_processados,
                percentual: s.percentual,
                concluida: s.concluida,
            }
        }))
    }

    /// Lista todas as operações de arquivamento.
    pub async fn listar_operacoes(&self) -> Vec<StatusOperacao> {
        let operacoes = self.operacoes.read().await;
        operacoes
            .values()
            .map(|o| {
                let s = o.status();
                StatusOperacao {
                    id: o.id.clone(),
                    obra_titulo: o.obra_titulo.clone(),
                    total_capitulos: s.total_capitulos,
                    capitulos_processados: s.capitulos_processados,
                    percentual: s.percentual,
                    concluida: s.concluida,
                }
            })
            .collect()
    }

    /// Executa o arquivamento de um capítulo (atualiza progresso).
    pub async fn arquivar_capitulo(
        &self,
        id_operacao: &str,
        _fonte_id: &str,
        capitulo: &CapituloBuscaResultado,
    ) -> Result<(), SourceError> {
        let mut operacoes = self.operacoes.write().await;
        if let Some(op) = operacoes.get_mut(id_operacao) {
            op.atualizar_progresso(capitulo.capitulo_id_fonte.as_str());
        }
        Ok(())
    }

    /// Executa o arquivamento completo de uma lista de capítulos na biblioteca:
    /// obtém bytes da fonte autorizada, grava `.cbz` atomicamente (`temp` + `rename`),
    /// calcula SHA-256, salva capa local (quando informada) e registra obra/capítulo/log no SQLite.
    pub async fn arquivar_para_biblioteca(
        &self,
        raiz_biblioteca: &std::path::Path,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
    ) -> Result<String, SourceError> {
        self.arquivar_para_biblioteca_com_capa(raiz_biblioteca, obra_titulo, capitulos, None)
            .await
    }

    /// Variante de `arquivar_para_biblioteca` que também persiste `cover.jpg` localmente quando `capa_url` é fornecida.
    pub async fn arquivar_para_biblioteca_com_capa(
        &self,
        raiz_biblioteca: &std::path::Path,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
        capa_url: Option<String>,
    ) -> Result<String, SourceError> {
        let id_operacao = self
            .iniciar_operacao_arquivamento(obra_titulo.clone(), capitulos.clone())
            .await?;

        let nome_pasta_obra = sanitizar_nome(&obra_titulo);
        let caminho_rel_obra = format!("Manga/{}", nome_pasta_obra);
        let pasta_obra_abs = raiz_biblioteca.join(&caminho_rel_obra);
        let _ = std::fs::create_dir_all(&pasta_obra_abs);

        if let Some(ref url_capa) = capa_url {
            let _ = salvar_capa_local(url_capa, &pasta_obra_abs).await;
        }

        let obra = match self.db.buscar_obra_por_titulo(&obra_titulo).await? {
            Some(o) => o,
            None => {
                self.db
                    .criar_obra(
                        mam_core::ObraId::new(),
                        obra_titulo.clone(),
                        mam_core::CaminhoRelativo::from_str(caminho_rel_obra.clone()),
                        capitulos.first().map(|c| c.fonte_id.clone()),
                    )
                    .await?
            }
        };

        for cap in capitulos {
            let fonte = {
                let fontes = self.fontes.read().await;
                fontes
                    .get(&cap.fonte_id)
                    .cloned()
                    .ok_or_else(|| SourceError::FonteNaoEncontrada(cap.fonte_id.clone()))?
            };

            if !fonte.eh_autorizada() {
                return Err(SourceError::FonteNaoAutorizada(cap.fonte_id));
            }

            let cbz_bytes = fonte.obter_capitulo(&cap.capitulo_id_fonte).await?;
            let numero = cap.numero.unwrap_or(0);
            let nome_arquivo = format!("Capitulo_{:04}.cbz", numero);
            let idioma_limpo = cap
                .idioma
                .as_deref()
                .map(|s| sanitizar_nome(s))
                .filter(|s| !s.is_empty());
            let pasta_destino_rel = match &idioma_limpo {
                Some(lang) => format!("{}/{}", caminho_rel_obra, lang),
                None => caminho_rel_obra.clone(),
            };
            let caminho_rel_cap = format!("{}/{}", pasta_destino_rel, nome_arquivo);
            let caminho_abs = raiz_biblioteca.join(&caminho_rel_cap);

            if let Some(pai) = caminho_abs.parent() {
                std::fs::create_dir_all(pai).map_err(|e| SourceError::Operacao(e.to_string()))?;
            }

            let temp_path = caminho_abs.with_extension("cbz.mam_tmp");
            std::fs::write(&temp_path, &cbz_bytes)
                .map_err(|e| SourceError::Operacao(e.to_string()))?;
            if caminho_abs.exists() {
                std::fs::remove_file(&caminho_abs)
                    .map_err(|e| SourceError::Operacao(e.to_string()))?;
            }
            std::fs::rename(&temp_path, &caminho_abs)
                .map_err(|e| SourceError::Operacao(e.to_string()))?;

            use sha2::Digest;
            let hash = format!("{:x}", sha2::Sha256::digest(&cbz_bytes));
            self.db
                .registrar_integridade(&caminho_rel_cap, &hash, "valido")
                .await?;

            let qtd_paginas = contar_paginas_cbz_bytes(&cbz_bytes) as i32;
            self.db
                .criar_capitulo_com_idioma(
                    mam_core::CapituloId::new(),
                    None,
                    obra.id,
                    numero,
                    cap.titulo.clone(),
                    mam_core::CaminhoRelativo::from_str(caminho_rel_cap.clone()),
                    qtd_paginas,
                    cap.idioma.clone(),
                )
                .await?;

            let detalhes = serde_json::json!({
                "nome": match &cap.idioma {
                    Some(lang) if !lang.trim().is_empty() => format!("{} — Cap. {} [{}]", obra_titulo, numero, lang),
                    _ => format!("{} — Cap. {}", obra_titulo, numero),
                },
                "pasta": pasta_destino_rel,
                "arquivo": nome_arquivo,
                "paginas": qtd_paginas,
            })
            .to_string();
            let _ = self
                .db
                .registrar_operacao(
                    "download".to_string(),
                    Some(caminho_rel_cap),
                    Some(detalhes),
                    None,
                )
                .await;

            self.arquivar_capitulo(&id_operacao, &cap.fonte_id, &cap)
                .await?;
        }

        Ok(id_operacao)
    }
}

/// Baixa uma imagem externa via HTTP (sem Referer de WebView) e retorna Data URL Base64.
pub async fn baixar_imagem_data_url(url: &str) -> Result<String, SourceError> {
    let client = reqwest::Client::builder()
        .user_agent("MAM-MangaArchiveManager/0.1.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(SourceError::Operacao(format!(
            "Falha ao obter imagem: HTTP {}",
            resp.status()
        )));
    }
    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .split(';')
        .next()
        .unwrap_or("image/jpeg")
        .to_string();
    let bytes = resp.bytes().await?;
    Ok(format!("data:{};base64,{}", mime, codificar_base64(&bytes)))
}

/// Baixa e salva `cover.jpg` na pasta da obra para exibição offline na Biblioteca.
pub async fn salvar_capa_local(url: &str, pasta_obra: &std::path::Path) -> Result<(), SourceError> {
    let client = reqwest::Client::builder()
        .user_agent("MAM-MangaArchiveManager/0.1.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(SourceError::Operacao(format!(
            "Falha ao baixar capa: HTTP {}",
            resp.status()
        )));
    }
    let bytes = resp.bytes().await?;
    std::fs::create_dir_all(pasta_obra).map_err(|e| SourceError::Operacao(e.to_string()))?;
    let destino = pasta_obra.join("cover.jpg");
    let temp = pasta_obra.join(".cover.jpg.mam_tmp");
    std::fs::write(&temp, &bytes).map_err(|e| SourceError::Operacao(e.to_string()))?;
    if destino.exists() {
        let _ = std::fs::remove_file(&destino);
    }
    std::fs::rename(&temp, &destino).map_err(|e| SourceError::Operacao(e.to_string()))?;
    Ok(())
}

pub fn codificar_base64(bytes: &[u8]) -> String {
    const TABELA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut saida = String::with_capacity(((bytes.len() + 2) / 3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triplo = (b0 << 16) | (b1 << 8) | b2;
        saida.push(TABELA[((triplo >> 18) & 0x3F) as usize] as char);
        saida.push(TABELA[((triplo >> 12) & 0x3F) as usize] as char);
        saida.push(if chunk.len() > 1 { TABELA[((triplo >> 6) & 0x3F) as usize] as char } else { '=' });
        saida.push(if chunk.len() > 2 { TABELA[(triplo & 0x3F) as usize] as char } else { '=' });
    }
    saida
}

pub fn sanitizar_nome(s: &str) -> String {
    let limpo: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    let trimmed = limpo.trim();
    if trimmed.is_empty() {
        "Obra".to_string()
    } else {
        trimmed.to_string()
    }
}

fn contar_paginas_cbz_bytes(bytes: &[u8]) -> usize {
    let cursor = std::io::Cursor::new(bytes);
    zip::ZipArchive::new(cursor).map(|z| z.len()).unwrap_or(0)
}

/// Informação de uma fonte registrada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FonteInfo {
    pub id: String,
    pub nome: String,
    pub autorizada: bool,
}

/// Status de uma operação de arquivamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusOperacao {
    pub id: String,
    pub obra_titulo: String,
    pub total_capitulos: usize,
    pub capitulos_processados: usize,
    pub percentual: f32,
    pub concluida: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonte_info_cria() {
        let info = FonteInfo {
            id: "fonte-teste".to_string(),
            nome: "Fonte Teste".to_string(),
            autorizada: true,
        };
        assert_eq!(info.id, "fonte-teste");
        assert!(info.autorizada);
    }

    #[test]
    fn status_operacao_calcula_percentual() {
        let status = StatusOperacao {
            id: "op-1".to_string(),
            obra_titulo: "Obra Teste".to_string(),
            total_capitulos: 10,
            capitulos_processados: 3,
            percentual: 30.0,
            concluida: false,
        };
        assert_eq!(status.percentual, 30.0);
        assert!(!status.concluida);
    }

    #[tokio::test]
    async fn arquivar_para_biblioteca_grava_cbz_e_sha256() {
        use std::io::Write;
        use crate::fonte::{CapituloEncontrado, ObraEncontrada};
        struct FonteMock;
        #[async_trait::async_trait]
        impl Fonte for FonteMock {
            fn id(&self) -> &str { "mock" }
            fn nome(&self) -> &str { "Mock Autorizada" }
            fn eh_autorizada(&self) -> bool { true }
            async fn buscar_obras(&self, _: &str) -> Result<Vec<ObraEncontrada>, crate::fonte::FonteError> { Ok(vec![]) }
            async fn listar_capitulos(&self, _: &str) -> Result<Vec<CapituloEncontrado>, crate::fonte::FonteError> { Ok(vec![]) }
            async fn obter_capitulo(&self, _: &str) -> Result<Vec<u8>, crate::fonte::FonteError> {
                let mut cur = std::io::Cursor::new(Vec::new());
                {
                    let mut zw = zip::ZipWriter::new(&mut cur);
                    zw.start_file("0001.jpg", zip::write::FileOptions::default()).unwrap();
                    zw.write_all(b"JPG").unwrap();
                    zw.finish().unwrap();
                }
                Ok(cur.into_inner())
            }
        }

        let base = std::env::temp_dir().join(format!("mam_test_src_arch_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();
        let db = Arc::new(Database::abrir(base.join("db.sqlite"), base.clone()).await.unwrap());
        let sm = SourceIntegrationManager::nova(db.clone());
        sm.registrar_fonte(Arc::new(FonteMock)).await.unwrap();

        let caps = vec![
            CapituloBuscaResultado {
                fonte_id: "mock".to_string(),
                obra_id_fonte: "obra-1".to_string(),
                capitulo_id_fonte: "cap-1-pt".to_string(),
                numero: Some(1),
                titulo: Some("Capítulo 1".to_string()),
                url: None,
                idioma: Some("PT".to_string()),
            },
            CapituloBuscaResultado {
                fonte_id: "mock".to_string(),
                obra_id_fonte: "obra-1".to_string(),
                capitulo_id_fonte: "cap-1-eng".to_string(),
                numero: Some(1),
                titulo: Some("Chapter 1".to_string()),
                url: None,
                idioma: Some("ENG".to_string()),
            },
        ];

        let op_id = sm
            .arquivar_para_biblioteca(&base, "Obra Web".to_string(), caps)
            .await
            .expect("arquivar para biblioteca");

        let status = sm.status_operacao(&op_id).await.unwrap().unwrap();
        assert!(status.concluida);
        assert_eq!(status.percentual, 100.0);
        assert!(base.join("Manga/Obra Web/PT/Capitulo_0001.cbz").exists());
        assert!(base.join("Manga/Obra Web/ENG/Capitulo_0001.cbz").exists());
        assert!(sm.capitulo_ja_arquivado("Manga/Obra Web/PT/Capitulo_0001.cbz").await.unwrap());
        assert!(sm.capitulo_ja_arquivado("Manga/Obra Web/ENG/Capitulo_0001.cbz").await.unwrap());

        let obra = db.buscar_obra_por_titulo("Obra Web").await.unwrap().unwrap();
        let caps_db = db.listar_capitulos_obra(obra.id).await.unwrap();
        assert_eq!(caps_db.len(), 2);
        assert!(caps_db.iter().any(|c| c.idioma.as_deref() == Some("PT")));
        assert!(caps_db.iter().any(|c| c.idioma.as_deref() == Some("ENG")));

        let _ = std::fs::remove_dir_all(&base);
    }
}
