//! Adaptador autorizado para a API pública do MangaDex (sem DRM ou paywall).

use std::collections::HashMap;
use std::io::{Cursor, Write};

use async_trait::async_trait;
use serde::Deserialize;
use zip::write::{FileOptions, ZipWriter};

use crate::fonte::{CapituloEncontrado, Fonte, FonteError, ObraEncontrada};

const API_BASE: &str = "https://api.mangadex.org";

/// Fonte pública autorizada do MangaDex.
pub struct MangaDexFonte {
    client: reqwest::Client,
}

impl MangaDexFonte {
    pub fn nova() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MAM-MangaArchiveManager/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

impl Default for MangaDexFonte {
    fn default() -> Self {
        Self::nova()
    }
}

#[derive(Deserialize)]
struct ListaResponse<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
struct MangaItem {
    id: String,
    attributes: MangaAttrs,
    #[serde(default)]
    relationships: Vec<RelationshipItem>,
}

#[derive(Deserialize)]
struct RelationshipItem {
    #[serde(rename = "type")]
    rel_type: String,
    #[serde(default)]
    attributes: Option<CoverAttrs>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoverAttrs {
    file_name: Option<String>,
}

#[derive(Deserialize)]
struct MangaAttrs {
    title: HashMap<String, String>,
    #[serde(default)]
    description: HashMap<String, String>,
}

#[derive(Deserialize)]
struct FeedItem {
    id: String,
    attributes: FeedAttrs,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FeedAttrs {
    chapter: Option<String>,
    title: Option<String>,
    translated_language: Option<String>,
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

fn escolher_texto(mapa: &HashMap<String, String>) -> Option<String> {
    mapa.get("pt-br")
        .or_else(|| mapa.get("en"))
        .or_else(|| mapa.values().next())
        .cloned()
}

#[async_trait]
impl Fonte for MangaDexFonte {
    fn id(&self) -> &str {
        "mangadex"
    }

    fn nome(&self) -> &str {
        "MangaDex (API Pública)"
    }

    fn eh_autorizada(&self) -> bool {
        true
    }

    async fn buscar_obras(&self, consulta: &str) -> Result<Vec<ObraEncontrada>, FonteError> {
        let resp = self
            .client
            .get(format!("{}/manga", API_BASE))
            .query(&[("title", consulta), ("limit", "15"), ("includes[]", "cover_art")])
            .send()
            .await
            .map_err(|e| FonteError::Rede(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(FonteError::Rede(format!("HTTP {}", resp.status())));
        }

        let body: ListaResponse<MangaItem> = resp
            .json()
            .await
            .map_err(|e| FonteError::Desconhecido(e.to_string()))?;

        Ok(body
            .data
            .into_iter()
            .map(|m| {
                let capa_url = m
                    .relationships
                    .iter()
                    .find(|r| r.rel_type == "cover_art")
                    .and_then(|r| r.attributes.as_ref())
                    .and_then(|a| a.file_name.as_ref())
                    .map(|fname| {
                        format!("https://uploads.mangadex.org/covers/{}/{}.256.jpg", m.id, fname)
                    });
                ObraEncontrada {
                    obra_id_fonte: m.id,
                    titulo: escolher_texto(&m.attributes.title).unwrap_or_else(|| "Sem título".to_string()),
                    autor: None,
                    descricao: escolher_texto(&m.attributes.description),
                    capa_url,
                }
            })
            .collect())
    }

    async fn listar_capitulos(
        &self,
        obra_id_fonte: &str,
    ) -> Result<Vec<CapituloEncontrado>, FonteError> {
        let resp = self
            .client
            .get(format!("{}/manga/{}/feed", API_BASE, obra_id_fonte))
            .query(&[
                ("limit", "100"),
                ("order[chapter]", "asc"),
                ("includeExternalUrl", "0"),
                ("translatedLanguage[]", "pt-br"),
                ("translatedLanguage[]", "en"),
            ])
            .send()
            .await
            .map_err(|e| FonteError::Rede(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(FonteError::Rede(format!("HTTP {}", resp.status())));
        }

        let body: ListaResponse<FeedItem> = resp
            .json()
            .await
            .map_err(|e| FonteError::Desconhecido(e.to_string()))?;

        Ok(body
            .data
            .into_iter()
            .map(|c| {
                let num = c
                    .attributes
                    .chapter
                    .as_deref()
                    .and_then(|s| s.parse::<f32>().ok())
                    .map(|n| n.round() as i32);
                let idioma = c.attributes.translated_language.as_deref().map(|lang| {
                    match lang.to_lowercase().as_str() {
                        "en" => "ENG".to_string(),
                        "pt-br" | "pt" => "PT".to_string(),
                        "ja" | "jp" => "JPN".to_string(),
                        outro => outro.to_uppercase(),
                    }
                });
                CapituloEncontrado {
                    capitulo_id_fonte: c.id.clone(),
                    numero: num,
                    titulo: c.attributes.title,
                    url: Some(format!("https://mangadex.org/chapter/{}", c.id)),
                    idioma,
                }
            })
            .collect())
    }

    async fn obter_capitulo(&self, capitulo_id_fonte: &str) -> Result<Vec<u8>, FonteError> {
        let resp = self
            .client
            .get(format!("{}/at-home/server/{}", API_BASE, capitulo_id_fonte))
            .send()
            .await
            .map_err(|e| FonteError::Rede(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(FonteError::Rede(format!("HTTP {}", resp.status())));
        }

        let at_home: AtHomeResponse = resp
            .json()
            .await
            .map_err(|e| FonteError::Desconhecido(e.to_string()))?;

        if at_home.chapter.data.is_empty() {
            return Err(FonteError::Desconhecido(
                "Capítulo sem páginas disponíveis na fonte (link externo ou indisponível)."
                    .to_string(),
            ));
        }

        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);
            let options = FileOptions::default();

            for (idx, arquivo) in at_home.chapter.data.iter().enumerate() {
                let url_img = format!(
                    "{}/data/{}/{}",
                    at_home.base_url, at_home.chapter.hash, arquivo
                );
                let img_resp = self
                    .client
                    .get(&url_img)
                    .send()
                    .await
                    .map_err(|e| FonteError::Rede(e.to_string()))?;
                if !img_resp.status().is_success() {
                    return Err(FonteError::Rede(format!(
                        "Falha ao baixar imagem {}: HTTP {}",
                        arquivo,
                        img_resp.status()
                    )));
                }
                let bytes = img_resp
                    .bytes()
                    .await
                    .map_err(|e| FonteError::Rede(e.to_string()))?;

                let ext = arquivo.rsplit('.').next().unwrap_or("jpg");
                let nome_entrada = format!("{:04}.{}", idx + 1, ext);
                zip.start_file(nome_entrada, options)
                    .map_err(|e| FonteError::Desconhecido(e.to_string()))?;
                zip.write_all(&bytes)
                    .map_err(|e| FonteError::Desconhecido(e.to_string()))?;
            }

            zip.finish().map_err(|e| FonteError::Desconhecido(e.to_string()))?;
        }

        Ok(buffer.into_inner())
    }
}
