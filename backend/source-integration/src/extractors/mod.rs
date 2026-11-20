//! Arquitetura modular de extratores por site para importação de capítulos em `.cbz`.
//!
//! Novos sites podem ser adicionados implementando [`ExtratorSite`] e registrando-os
//! em [`RegistroExtratores`].

pub mod generico;
pub mod mangadex;
pub mod mangafire;

use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::write::{FileOptions, ZipWriter};

use mam_core::{CaminhoRelativo, CapituloId, ObraId};
use mam_database::Database;

use crate::{SourceError, sanitizar_nome};

/// Dados de entrada para importação modular de um capítulo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaImportacao {
    pub obra_titulo: String,
    pub numero_capitulo: i32,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub urls_ordenadas: Vec<String>,
}

/// Metadados de um extrator registrado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtratorInfo {
    pub id: String,
    pub descricao: String,
}

/// Resultado de uma importação modular concluída.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoImportacao {
    pub extrator_usado: String,
    pub obra_titulo: String,
    pub numero_capitulo: i32,
    pub caminho_relativo: String,
    pub quantidade_paginas: usize,
    pub hash_sha256: String,
}

/// Contrato modular que cada site de extração deve implementar.
#[async_trait]
pub trait ExtratorSite: Send + Sync {
    fn id(&self) -> &str;
    fn descricao(&self) -> &str;
    fn suporta_url(&self, url: &str) -> bool;
    async fn extrair_paginas(&self, entrada: &EntradaImportacao) -> Result<Vec<String>, SourceError>;
}

/// Registro plugável de extratores por site.
pub struct RegistroExtratores {
    extratores: Vec<Arc<dyn ExtratorSite>>,
}

impl RegistroExtratores {
    pub fn novo() -> Self {
        Self {
            extratores: Vec::new(),
        }
    }

    /// Cria o registro padrão com MangaDex, MangaFire e extrator Genérico.
    pub fn padrao() -> Self {
        let mut reg = Self::novo();
        reg.registrar(Arc::new(mangadex::MangaDexExtractor::nova()));
        reg.registrar(Arc::new(mangafire::MangaFireExtractor::nova()));
        reg.registrar(Arc::new(generico::GenericoExtractor::nova()));
        reg
    }

    /// Registra um novo extrator modular (permite expandir para outros sites).
    pub fn registrar(&mut self, extrator: Arc<dyn ExtratorSite>) {
        self.extratores.push(extrator);
    }

    pub fn listar(&self) -> Vec<ExtratorInfo> {
        self.extratores
            .iter()
            .map(|e| ExtratorInfo {
                id: e.id().to_string(),
                descricao: e.descricao().to_string(),
            })
            .collect()
    }

    /// Seleciona o extrator mais adequado para a URL informada (ou fallback genérico).
    pub fn selecionar(&self, url: Option<&str>) -> Option<Arc<dyn ExtratorSite>> {
        if let Some(u) = url.filter(|s| !s.trim().is_empty()) {
            if let Some(encontrado) = self.extratores.iter().find(|e| e.suporta_url(u)) {
                return Some(encontrado.clone());
            }
        }
        self.extratores.last().cloned()
    }

    /// Executa extração das URLs em ordem, baixa as imagens, gera o `.cbz` atomicamente e registra no SQLite.
    pub async fn importar_capitulo(
        &self,
        db: &Database,
        raiz_biblioteca: &Path,
        entrada: &EntradaImportacao,
    ) -> Result<ResultadoImportacao, SourceError> {
        let extrator = self
            .selecionar(entrada.url.as_deref())
            .ok_or_else(|| SourceError::Operacao("Nenhum extrator registrado".to_string()))?;

        let urls = extrator.extrair_paginas(entrada).await?;
        if urls.is_empty() {
            return Err(SourceError::Operacao(
                "Nenhuma URL de página obtida para o capítulo".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .user_agent("MAM-MangaArchiveManager/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut paginas_bytes = Vec::with_capacity(urls.len());
        for url_img in &urls {
            let resp = client.get(url_img).send().await?;
            if !resp.status().is_success() {
                return Err(SourceError::Operacao(format!(
                    "Falha ao baixar imagem '{}': HTTP {}",
                    url_img,
                    resp.status()
                )));
            }
            let ext = extrair_extensao_url(url_img);
            let bytes = resp.bytes().await?.to_vec();
            paginas_bytes.push((ext, bytes));
        }

        let cbz_bytes = empacotar_paginas_em_cbz(&paginas_bytes)?;
        let hash = format!("{:x}", Sha256::digest(&cbz_bytes));

        let pasta_obra = sanitizar_nome(&entrada.obra_titulo);
        let caminho_rel_obra = format!("Manga/{}", pasta_obra);
        let caminho_rel_cap = format!(
            "{}/Capitulo_{:04}.cbz",
            caminho_rel_obra, entrada.numero_capitulo
        );
        let caminho_abs = raiz_biblioteca.join(&caminho_rel_cap);

        if let Some(pai) = caminho_abs.parent() {
            std::fs::create_dir_all(pai).map_err(|e| SourceError::Operacao(e.to_string()))?;
        }
        let temp = caminho_abs.with_extension("cbz.mam_tmp");
        std::fs::write(&temp, &cbz_bytes).map_err(|e| SourceError::Operacao(e.to_string()))?;
        if caminho_abs.exists() {
            std::fs::remove_file(&caminho_abs).map_err(|e| SourceError::Operacao(e.to_string()))?;
        }
        std::fs::rename(&temp, &caminho_abs).map_err(|e| SourceError::Operacao(e.to_string()))?;

        let obra = match db.buscar_obra_por_titulo(&entrada.obra_titulo).await? {
            Some(o) => o,
            None => {
                db.criar_obra(
                    ObraId::new(),
                    entrada.obra_titulo.clone(),
                    CaminhoRelativo::from_str(caminho_rel_obra),
                    Some(extrator.id().to_string()),
                )
                .await?
            }
        };

        db.registrar_integridade(&caminho_rel_cap, &hash, "valido")
            .await?;

        db.criar_capitulo(
            CapituloId::new(),
            None,
            obra.id,
            entrada.numero_capitulo,
            None,
            CaminhoRelativo::from_str(caminho_rel_cap.clone()),
            paginas_bytes.len() as i32,
        )
        .await?;

        Ok(ResultadoImportacao {
            extrator_usado: extrator.id().to_string(),
            obra_titulo: entrada.obra_titulo.clone(),
            numero_capitulo: entrada.numero_capitulo,
            caminho_relativo: caminho_rel_cap,
            quantidade_paginas: paginas_bytes.len(),
            hash_sha256: hash,
        })
    }
}

/// Empacota bytes de imagens em um arquivo CBZ (ZIP) preservando a ordem sequencial exata.
pub fn empacotar_paginas_em_cbz(paginas: &[(&str, Vec<u8>)]) -> Result<Vec<u8>, SourceError> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options = FileOptions::default();
        for (idx, (ext, bytes)) in paginas.iter().enumerate() {
            let nome = format!("{:04}.{}", idx + 1, ext);
            zip.start_file(nome, options)
                .map_err(|e| SourceError::Operacao(e.to_string()))?;
            zip.write_all(bytes)
                .map_err(|e| SourceError::Operacao(e.to_string()))?;
        }
        zip.finish()
            .map_err(|e| SourceError::Operacao(e.to_string()))?;
    }
    Ok(cursor.into_inner())
}

/// Extrai URLs de imagens (`data-src` ou `src`) de um HTML preservando a ordem de aparição e deduplicando.
pub fn extrair_urls_imagens_html(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let lower = html.to_lowercase();

    for attr in ["data-src=", "src="] {
        let mut offset = 0;
        while let Some(pos) = lower[offset..].find(attr) {
            let start = offset + pos + attr.len();
            let resto = &html[start..];
            let quote = resto.chars().next();
            if let Some(q @ ('"' | '\'')) = quote {
                let conteudo = &resto[1..];
                if let Some(end) = conteudo.find(q) {
                    let candidato = conteudo[..end].trim();
                    if eh_url_imagem_valida(candidato) && !urls.contains(&candidato.to_string()) {
                        urls.push(candidato.to_string());
                    }
                    offset = start + 1 + end + 1;
                    continue;
                }
            }
            offset = start;
        }
    }

    urls
}

fn eh_url_imagem_valida(url: &str) -> bool {
    let lower = url.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return false;
    }
    let sem_query = lower.split('?').next().unwrap_or(&lower);
    sem_query.ends_with(".jpg")
        || sem_query.ends_with(".jpeg")
        || sem_query.ends_with(".png")
        || sem_query.ends_with(".webp")
        || sem_query.ends_with(".avif")
        || lower.contains("/page")
        || lower.contains("cdn")
}

fn extrair_extensao_url(url: &str) -> &'static str {
    let sem_query = url.split('?').next().unwrap_or(url).to_lowercase();
    if sem_query.ends_with(".png") {
        "png"
    } else if sem_query.ends_with(".webp") {
        "webp"
    } else if sem_query.ends_with(".avif") {
        "avif"
    } else {
        "jpg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn registro_seleciona_extrator_mangafire_e_extrai_html_ordenado() {
        let reg = RegistroExtratores::padrao();
        let ext = reg
            .selecionar(Some("https://mangafire.to/title/jw2-medaka-boxx"))
            .unwrap();
        assert_eq!(ext.id(), "mangafire");

        let html = r#"
            <div class="pages">
                <img data-src="https://cdn.exemplo.com/p2.jpg" />
                <img data-src="https://cdn.exemplo.com/p1.png" />
            </div>
        "#;
        let entrada = EntradaImportacao {
            obra_titulo: "Medaka Box".to_string(),
            numero_capitulo: 1,
            url: Some("https://mangafire.to/read/jw2-medaka-boxx/en/chapter-1".to_string()),
            html: Some(html.to_string()),
            urls_ordenadas: vec![],
        };

        let paginas = ext.extrair_paginas(&entrada).await.unwrap();
        assert_eq!(
            paginas,
            vec![
                "https://cdn.exemplo.com/p2.jpg".to_string(),
                "https://cdn.exemplo.com/p1.png".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn extrator_mangafire_preserva_lista_ordenada_de_urls() {
        let reg = RegistroExtratores::padrao();
        let ext = reg.selecionar(Some("https://mangafire.to/title/jw2-medaka-boxx")).unwrap();
        let entrada = EntradaImportacao {
            obra_titulo: "Medaka Box".to_string(),
            numero_capitulo: 2,
            url: Some("https://mangafire.to/title/jw2-medaka-boxx".to_string()),
            html: None,
            urls_ordenadas: vec![
                "https://cdn.exemplo.com/01.jpg".to_string(),
                "https://cdn.exemplo.com/02.jpg".to_string(),
            ],
        };
        let paginas = ext.extrair_paginas(&entrada).await.unwrap();
        assert_eq!(paginas.len(), 2);
        assert_eq!(paginas[0], "https://cdn.exemplo.com/01.jpg");
        assert_eq!(paginas[1], "https://cdn.exemplo.com/02.jpg");
    }

    #[test]
    fn extrator_mangadex_identifica_uuid_da_url() {
        let (tipo, id) =
            mangadex::MangaDexExtractor::extrair_id_url("https://mangadex.org/chapter/abc-123-def")
                .unwrap();
        assert_eq!(tipo, "chapter");
        assert_eq!(id, "abc-123-def");
    }

    #[test]
    fn empacotar_paginas_em_cbz_mantem_sequencia() {
        let cbz = empacotar_paginas_em_cbz(&[
            ("jpg", b"p1".to_vec()),
            ("png", b"p2".to_vec()),
        ])
        .unwrap();
        let archive = zip::ZipArchive::new(Cursor::new(cbz)).unwrap();
        assert_eq!(archive.len(), 2);
    }
}
