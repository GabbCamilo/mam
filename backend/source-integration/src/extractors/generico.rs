//! Extrator genérico extensível para qualquer outro site via lista ordenada de URLs ou HTML.

use async_trait::async_trait;

use crate::SourceError;
use crate::extractors::{EntradaImportacao, ExtratorSite, extrair_urls_imagens_html};

/// Extrator genérico para sites públicos ou listas ordenadas de URLs.
pub struct GenericoExtractor {
    client: reqwest::Client,
}

impl GenericoExtractor {
    pub fn nova() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MAM-MangaArchiveManager/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

impl Default for GenericoExtractor {
    fn default() -> Self {
        Self::nova()
    }
}

#[async_trait]
impl ExtratorSite for GenericoExtractor {
    fn id(&self) -> &str {
        "generico"
    }

    fn descricao(&self) -> &str {
        "Extrator Genérico (lista ordenada de URLs ou HTML público)"
    }

    fn suporta_url(&self, _url: &str) -> bool {
        true
    }

    async fn extrair_paginas(&self, entrada: &EntradaImportacao) -> Result<Vec<String>, SourceError> {
        if !entrada.urls_ordenadas.is_empty() {
            return Ok(entrada.urls_ordenadas.clone());
        }

        if let Some(html) = entrada.html.as_deref().filter(|h| !h.trim().is_empty()) {
            let paginas = extrair_urls_imagens_html(html);
            if !paginas.is_empty() {
                return Ok(paginas);
            }
        }

        if let Some(url) = entrada.url.as_deref().filter(|u| !u.trim().is_empty()) {
            let mut paginas_estaticas = Vec::new();
            if let Ok(resp) = self.client.get(url).send().await {
                if resp.status().is_success() {
                    if let Ok(html) = resp.text().await {
                        paginas_estaticas = extrair_urls_imagens_html(&html);
                        if paginas_estaticas.len() > 1 {
                            return Ok(paginas_estaticas);
                        }
                    }
                }
            }
            if let Ok(paginas_cdp) =
                crate::extractors::smart_extract_pages_via_ai_engine(url).await
            {
                if paginas_cdp.len() > paginas_estaticas.len() {
                    return Ok(paginas_cdp);
                }
            }
            if let Ok(dom) = crate::extractors::renderizar_html_com_navegador(url).await {
                let paginas = extrair_urls_imagens_html(&dom);
                if paginas.len() > paginas_estaticas.len() {
                    return Ok(paginas);
                }
            }
            if !paginas_estaticas.is_empty() {
                return Ok(paginas_estaticas);
            }
        }

        Err(SourceError::Operacao(
            "Nenhuma página de imagem encontrada na entrada informada".to_string(),
        ))
    }
}
