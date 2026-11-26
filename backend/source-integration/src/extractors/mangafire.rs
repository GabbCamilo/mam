//! Extrator modular para páginas do MangaFire (`mangafire.to`) via HTML ou lista ordenada de URLs.
//! Não implementa contorno de anti-bot, DRM ou paywall.

use async_trait::async_trait;

use crate::SourceError;
use crate::extractors::{EntradaImportacao, ExtratorSite, extrair_urls_imagens_html};

/// Extrator modular para links `mangafire.to/...`.
pub struct MangaFireExtractor {
    client: reqwest::Client,
}

impl MangaFireExtractor {
    pub fn nova() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MAM-MangaArchiveManager/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

impl Default for MangaFireExtractor {
    fn default() -> Self {
        Self::nova()
    }
}

#[async_trait]
impl ExtratorSite for MangaFireExtractor {
    fn id(&self) -> &str {
        "mangafire"
    }

    fn descricao(&self) -> &str {
        "MangaFire (HTML do capítulo ou lista ordenada de URLs)"
    }

    fn suporta_url(&self, url: &str) -> bool {
        url.to_lowercase().contains("mangafire.to")
    }

    async fn extrair_paginas(&self, entrada: &EntradaImportacao) -> Result<Vec<String>, SourceError> {
        if !entrada.urls_ordenadas.is_empty() {
            return Ok(entrada.urls_ordenadas.clone());
        }

        if let Some(html) = entrada.html.as_deref().filter(|h| !h.trim().is_empty()) {
            let paginas = extrair_urls_imagens_html(html);
            if paginas.is_empty() {
                return Err(SourceError::Operacao(
                    "Nenhuma URL de imagem encontrada no HTML fornecido do MangaFire".to_string(),
                ));
            }
            return Ok(paginas);
        }

        if let Some(url) = entrada.url.as_deref().filter(|u| !u.trim().is_empty()) {
            let mut paginas_estaticas = Vec::new();
            if let Ok(resp) = self.client.get(url).send().await {
                if resp.status().is_success() {
                    if let Ok(html) = resp.text().await {
                        if let Some(paginas_container) =
                            crate::extractors::extrair_urls_imagens_de_container_leitor(&html)
                        {
                            return Ok(paginas_container);
                        }
                        paginas_estaticas = extrair_urls_imagens_html(&html);
                    }
                }
            }
            if let Ok(paginas_cdp) =
                crate::extractors::smart_extract_pages_via_ai_engine(url).await
            {
                if !paginas_cdp.is_empty() {
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
            "O MangaFire requer HTML da página do capítulo ou lista ordenada de URLs de imagem (sem contorno anti-bot)".to_string(),
        ))
    }
}
