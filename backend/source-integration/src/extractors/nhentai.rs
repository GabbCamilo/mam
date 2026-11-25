//! Extrator modular para galerias/obras únicas do nHentai (`nhentai.net/g/<id>/`)
//! utilizando a API pública oficial v2 (`/api/v2/galleries/<id>`).

use async_trait::async_trait;
use serde::Deserialize;

use crate::extractors::{EntradaImportacao, ExtratorSite, ObraExtraidaPorLink};
use crate::{CapituloBuscaResultado, SourceError};

#[derive(Debug, Deserialize)]
struct NHentaiTitle {
    pretty: Option<String>,
    english: Option<String>,
    japanese: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NHentaiImageRef {
    path: String,
}

#[derive(Debug, Deserialize)]
struct NHentaiPage {
    number: usize,
    path: String,
}

#[derive(Debug, Deserialize)]
struct NHentaiTag {
    #[serde(rename = "type")]
    kind: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NHentaiGalleryResp {
    id: u64,
    title: NHentaiTitle,
    cover: Option<NHentaiImageRef>,
    #[serde(default)]
    pages: Vec<NHentaiPage>,
    #[serde(default)]
    tags: Vec<NHentaiTag>,
}

/// Extrai o ID numérico da galeria a partir de URLs como `https://nhentai.net/g/381205/` ou `.../g/381205/1/`.
pub fn extrair_gallery_id(url: &str) -> Option<String> {
    let lower = url.to_lowercase();
    let idx = lower.find("nhentai.net/g/")?;
    let rest = &url[idx + "nhentai.net/g/".len()..];
    let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if id.is_empty() { None } else { Some(id) }
}

fn detectar_idioma_tags(tags: &[NHentaiTag]) -> Option<String> {
    for tag in tags {
        if tag.kind.as_deref() == Some("language") {
            match tag.name.as_deref().unwrap_or("").to_lowercase().as_str() {
                "portuguese" => return Some("PT".to_string()),
                "english" => return Some("ENG".to_string()),
                "japanese" => return Some("JPN".to_string()),
                "spanish" => return Some("ESP".to_string()),
                _ => {}
            }
        }
    }
    None
}

/// Consulta a API v2 do nHentai e retorna os metadados da obra única + capítulo único.
pub async fn obter_obra_galeria(url_galeria: &str) -> Result<ObraExtraidaPorLink, SourceError> {
    let id = extrair_gallery_id(url_galeria)
        .ok_or_else(|| SourceError::Operacao("ID de galeria Manga Alternativo inválido na URL".to_string()))?;

    let api_url = format!("https://nhentai.net/api/v2/galleries/{}", id);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = client.get(&api_url).send().await?;
    if !resp.status().is_success() {
        return Err(SourceError::Operacao(format!(
            "HTTP {} ao consultar API v2 do Manga Alternativo ({})",
            resp.status(),
            api_url
        )));
    }
    let data: NHentaiGalleryResp = resp.json().await?;
    let titulo = data
        .title
        .pretty
        .filter(|s| !s.trim().is_empty())
        .or(data.title.english.filter(|s| !s.trim().is_empty()))
        .or(data.title.japanese.filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| format!("Manga Alternativo #{}", data.id));

    let capa_url = data
        .cover
        .as_ref()
        .map(|c| format!("https://t.nhentai.net/{}", c.path.trim_start_matches('/')))
        .or_else(|| {
            data.pages
                .first()
                .map(|p| format!("https://i.nhentai.net/{}", p.path.trim_start_matches('/')))
        });

    let idioma = detectar_idioma_tags(&data.tags);
    let url_canonica = format!("https://nhentai.net/g/{}/", data.id);

    Ok(ObraExtraidaPorLink {
        titulo: titulo.clone(),
        capa_url,
        url_obra: url_canonica.clone(),
        capitulos: vec![CapituloBuscaResultado {
            fonte_id: "extrator-link".to_string(),
            obra_id_fonte: url_canonica.clone(),
            capitulo_id_fonte: url_canonica.clone(),
            numero: Some(1),
            titulo: Some(format!("Obra Única — {} ({} págs)", titulo, data.pages.len())),
            url: Some(url_canonica),
            idioma,
        }],
    })
}

/// Extrator modular para galerias/capítulos únicos do `nhentai.net`.
pub struct NHentaiExtractor {
    client: reqwest::Client,
}

impl NHentaiExtractor {
    pub fn nova() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

/// Extrai todas as galerias (`/g/<id>/`) de uma página de listagem do nHentai
/// (autor `/artist/...`, tag `/tag/...`, paródia `/parody/...`, busca `/search/...`).
pub fn extrair_colecao_html(url_pagina: &str, html: &str) -> ObraExtraidaPorLink {
    let mut vistos = std::collections::HashSet::new();
    let mut capitulos = Vec::new();
    let lower = html.to_lowercase();
    let mut offset = 0usize;
    let mut primeira_capa: Option<String> = None;

    while let Some(pos) = lower[offset..].find("href=\"/g/") {
        let abs = offset + pos;
        let start_href = abs + 6;
        let Some(end_href_rel) = lower[start_href..].find('"') else {
            break;
        };
        let rel_href = &html[start_href..start_href + end_href_rel];
        offset = start_href + end_href_rel;

        let Some(id_str) = extrair_gallery_id(&format!("https://nhentai.net{}", rel_href)) else {
            continue;
        };
        let url_canonica = format!("https://nhentai.net/g/{}/", id_str);
        if !vistos.insert(id_str.clone()) {
            continue;
        }

        let trecho_fim = (abs + 900).min(html.len());
        let trecho = &html[abs..trecho_fim];
        let titulo_alt = extrair_attr(trecho, "alt=").or_else(|| extrair_caption(trecho));
        let capa_item = extrair_attr(trecho, "data-src=")
            .or_else(|| extrair_attr(trecho, "src="))
            .map(|u| {
                if u.starts_with("//") {
                    format!("https:{}", u)
                } else {
                    u
                }
            });
        if primeira_capa.is_none() {
            primeira_capa = capa_item;
        }

        let idioma = titulo_alt.as_deref().and_then(|t| {
            let tl = t.to_lowercase();
            if tl.contains("[portuguese]") || tl.contains("portugues") {
                Some("PT".to_string())
            } else if tl.contains("[english]") {
                Some("ENG".to_string())
            } else if tl.contains("[japanese]") {
                Some("JPN".to_string())
            } else {
                None
            }
        });

        let numero = id_str.parse::<i32>().ok();
        capitulos.push(CapituloBuscaResultado {
            fonte_id: "extrator-link".to_string(),
            obra_id_fonte: url_pagina.to_string(),
            capitulo_id_fonte: url_canonica.clone(),
            numero,
            titulo: Some(
                titulo_alt
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("Galeria #{}", id_str)),
            ),
            url: Some(url_canonica),
            idioma,
        });
    }

    let titulo_pagina = url_pagina
        .trim_end_matches('/')
        .split('/')
        .last()
        .filter(|s| !s.is_empty())
        .unwrap_or("Coleção Manga Alternativo")
        .replace('-', " ");

    ObraExtraidaPorLink {
        titulo: format!("Manga Alternativo — {}", titulo_pagina),
        capa_url: primeira_capa,
        url_obra: url_pagina.to_string(),
        capitulos,
    }
}

fn extrair_attr(trecho: &str, attr: &str) -> Option<String> {
    let lower = trecho.to_lowercase();
    let idx = lower.find(attr)?;
    let rest = trecho[idx + attr.len()..].trim_start();
    let q = rest.chars().next()?;
    if q == '"' || q == '\'' {
        let end = rest[1..].find(q)?;
        let val = rest[1..1 + end].trim();
        if val.is_empty() { None } else { Some(val.to_string()) }
    } else {
        None
    }
}

fn extrair_caption(trecho: &str) -> Option<String> {
    let lower = trecho.to_lowercase();
    let idx = lower.find("class=\"caption\"")?;
    let after = &trecho[idx..];
    let gt = after.find('>')?;
    let end = after[gt + 1..].find("</div>")?;
    let txt = after[gt + 1..gt + 1 + end].trim();
    if txt.is_empty() { None } else { Some(txt.to_string()) }
}

impl Default for NHentaiExtractor {
    fn default() -> Self {
        Self::nova()
    }
}

#[async_trait]
impl ExtratorSite for NHentaiExtractor {
    fn id(&self) -> &str {
        "nhentai"
    }

    fn descricao(&self) -> &str {
        "Manga Alternativo (API v2 oficial para obras/capítulos únicos)"
    }

    fn suporta_url(&self, url: &str) -> bool {
        extrair_gallery_id(url).is_some()
    }

    async fn extrair_paginas(&self, entrada: &EntradaImportacao) -> Result<Vec<String>, SourceError> {
        if !entrada.urls_ordenadas.is_empty() {
            return Ok(entrada.urls_ordenadas.clone());
        }

        let url = entrada
            .url
            .as_deref()
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| SourceError::Operacao("Informe o link da galeria Manga Alternativo".to_string()))?;

        let id = extrair_gallery_id(url)
            .ok_or_else(|| SourceError::Operacao("URL do Manga Alternativo sem ID de galeria (/g/<id>/)".to_string()))?;

        let api_url = format!("https://nhentai.net/api/v2/galleries/{}", id);
        let resp = self.client.get(&api_url).send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao consultar API v2 do Manga Alternativo",
                resp.status()
            )));
        }

        let mut data: NHentaiGalleryResp = resp.json().await?;
        data.pages.sort_by_key(|p| p.number);
        let urls: Vec<String> = data
            .pages
            .into_iter()
            .map(|p| format!("https://i.nhentai.net/{}", p.path.trim_start_matches('/')))
            .collect();

        if urls.is_empty() {
            return Err(SourceError::Operacao(
                "Nenhuma página retornada pela API v2 do Manga Alternativo".to_string(),
            ));
        }

        Ok(urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrai_id_galeria_nhentai() {
        assert_eq!(
            extrair_gallery_id("https://nhentai.net/g/381205/"),
            Some("381205".to_string())
        );
        assert_eq!(
            extrair_gallery_id("https://nhentai.net/g/381205/1/"),
            Some("381205".to_string())
        );
        assert_eq!(extrair_gallery_id("https://mangadex.org/title/123"), None);
    }

    #[test]
    fn parse_api_v2_json_nhentai() {
        let json = r#"{
            "id": 381205,
            "title": { "pretty": "Hey, Only Look At Ririmu", "english": null, "japanese": null },
            "cover": { "path": "galleries/2067220/cover.png" },
            "pages": [
                { "number": 2, "path": "galleries/2067220/2.png" },
                { "number": 1, "path": "galleries/2067220/1.png" }
            ],
            "tags": [{ "type": "language", "name": "english" }]
        }"#;
        let mut parsed: NHentaiGalleryResp = serde_json::from_str(json).unwrap();
        parsed.pages.sort_by_key(|p| p.number);
        assert_eq!(parsed.pages[0].path, "galleries/2067220/1.png");
        assert_eq!(parsed.pages[1].path, "galleries/2067220/2.png");
        assert_eq!(detectar_idioma_tags(&parsed.tags), Some("ENG".to_string()));
    }

    #[test]
    fn extrai_colecao_de_autor_nhentai() {
        let html = r#"
            <div class="gallery"><a href="/g/660870/" class="cover"><img alt="[Ura no Kyuu] Obra Teste [English]" src="https://t3.nhentai.net/galleries/1/thumb.webp"/></a></div>
            <div class="gallery"><a href="/g/623417/" class="cover"><img alt="[Ura no Kyuu] Obra 2 [Portuguese]" src="https://t3.nhentai.net/galleries/2/thumb.webp"/></a></div>
        "#;
        let col = extrair_colecao_html("https://nhentai.net/artist/uranokyuu/", html);
        assert_eq!(col.capitulos.len(), 2);
        assert_eq!(col.capitulos[0].numero, Some(660870));
        assert_eq!(col.capitulos[0].idioma.as_deref(), Some("ENG"));
        assert_eq!(col.capitulos[1].idioma.as_deref(), Some("PT"));
    }
}
