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
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
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
            // Suporte nativo ao WeebCentral (/chapters/<id>/images?reading_style=long_strip)
            if url.to_lowercase().contains("weebcentral.com/chapters/") {
                let base = url.trim_end_matches('/').split('?').next().unwrap_or(url);
                let images_url = if base.ends_with("/images") {
                    format!(
                        "{}?is_prev=False&current_page=1&reading_style=long_strip",
                        base
                    )
                } else {
                    format!(
                        "{}/images?is_prev=False&current_page=1&reading_style=long_strip",
                        base
                    )
                };
                if let Ok(resp) = self
                    .client
                    .get(&images_url)
                    .header("HX-Request", "true")
                    .send()
                    .await
                {
                    if resp.status().is_success() {
                        if let Ok(html) = resp.text().await {
                            let paginas = extrair_urls_imagens_html(&html);
                            if !paginas.is_empty() {
                                return Ok(paginas);
                            }
                        }
                    }
                }
            }

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
            let mut erro_cdp: Option<SourceError> = None;
            match crate::extractors::smart_extract_pages_via_ai_engine(url).await {
                Ok(paginas_cdp) => {
                    if !paginas_cdp.is_empty() {
                        return Ok(paginas_cdp);
                    }
                }
                Err(e) => {
                    erro_cdp = Some(e);
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
            if let Some(e) = erro_cdp {
                if e.to_string().contains("CLOUDFLARE_CHALLENGE") {
                    return Err(e);
                }
            }
        }

        Err(SourceError::Operacao(
            "Nenhuma página de imagem encontrada na entrada informada".to_string(),
        ))
    }
}
