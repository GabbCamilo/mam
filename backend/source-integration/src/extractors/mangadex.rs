//! Extrator modular para links diretos do MangaDex (API pública oficial).

use async_trait::async_trait;
use serde::Deserialize;

use crate::SourceError;
use crate::extractors::{EntradaImportacao, ExtratorSite};

const API_BASE: &str = "https://api.mangadex.org";

/// Extrator modular para URLs `mangadex.org/chapter/<uuid>` e `mangadex.org/title/<uuid>`.
pub struct MangaDexExtractor {
    client: reqwest::Client,
}

impl MangaDexExtractor {
    pub fn nova() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MAM-MangaArchiveManager/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Extrai `(tipo, uuid)` de um link do MangaDex (`chapter` ou `title`).
    pub fn extrair_id_url(url: &str) -> Option<(&'static str, String)> {
        let lower = url.to_lowercase();
        for tipo in ["chapter", "title"] {
            let marcador = format!("mangadex.org/{}/", tipo);
            if let Some(pos) = lower.find(&marcador) {
                let resto = &url[pos + marcador.len()..];
                let id = resto.split(&['/', '?', '#'][..]).next().unwrap_or("").trim();
                if !id.is_empty() {
                    return Some((tipo, id.to_string()));
                }
            }
        }
        None
    }
}

impl Default for MangaDexExtractor {
    fn default() -> Self {
        Self::nova()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AtHomeResponse {
    base_url: String,
    chapter: AtHomeChapter,
}

#[derive(Deserialize)]
struct AtHomeChapter {
    hash: String,
    data: Vec<String>,
}

#[async_trait]
impl ExtratorSite for MangaDexExtractor {
    fn id(&self) -> &str {
        "mangadex"
    }

    fn descricao(&self) -> &str {
        "MangaDex (API Pública: links /chapter/<id>)"
    }

    fn suporta_url(&self, url: &str) -> bool {
        url.to_lowercase().contains("mangadex.org")
    }

    async fn extrair_paginas(&self, entrada: &EntradaImportacao) -> Result<Vec<String>, SourceError> {
        if !entrada.urls_ordenadas.is_empty() {
            return Ok(entrada.urls_ordenadas.clone());
        }

        let url = entrada.url.as_deref().ok_or_else(|| {
            SourceError::Operacao("Informe a URL de capítulo do MangaDex".to_string())
        })?;

        let (tipo, id) = Self::extrair_id_url(url).ok_or_else(|| {
            SourceError::Operacao(format!("URL do MangaDex não reconhecida: {}", url))
        })?;

        if tipo != "chapter" {
            return Err(SourceError::Operacao(
                "Para links /title/ do MangaDex, use a busca de capítulos da obra ou informe o link /chapter/<id>".to_string(),
            ));
        }

        let resp = self
            .client
            .get(format!("{}/at-home/server/{}", API_BASE, id))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "MangaDex retornou HTTP {}",
                resp.status()
            )));
        }

        let at_home: AtHomeResponse = resp.json().await?;
        Ok(at_home
            .chapter
            .data
            .into_iter()
            .map(|arquivo| format!("{}/data/{}/{}", at_home.base_url, at_home.chapter.hash, arquivo))
            .collect())
    }
}
