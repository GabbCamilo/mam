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

use crate::{CapituloBuscaResultado, SourceError, sanitizar_nome};

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

/// Resultado da descoberta de uma obra e seus capítulos a partir de uma URL centralizadora.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObraExtraidaPorLink {
    pub titulo: String,
    pub capa_url: Option<String>,
    pub url_obra: String,
    pub capitulos: Vec<CapituloBuscaResultado>,
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

        let detalhes = serde_json::json!({
            "nome": format!("{} — Cap. {}", entrada.obra_titulo, entrada.numero_capitulo),
            "pasta": format!("Manga/{}", pasta_obra),
            "arquivo": format!("Capitulo_{:04}.cbz", entrada.numero_capitulo),
            "paginas": paginas_bytes.len(),
        })
        .to_string();
        let _ = db
            .registrar_operacao(
                "download".to_string(),
                Some(caminho_rel_cap.clone()),
                Some(detalhes),
                None,
            )
            .await;

        Ok(ResultadoImportacao {
            extrator_usado: extrator.id().to_string(),
            obra_titulo: entrada.obra_titulo.clone(),
            numero_capitulo: entrada.numero_capitulo,
            caminho_relativo: caminho_rel_cap,
            quantidade_paginas: paginas_bytes.len(),
            hash_sha256: hash,
        })
    }

    /// Baixa todas as páginas de um capítulo a partir da sua URL e retorna os bytes do `.cbz` em memória.
    pub async fn baixar_capitulo_cbz_por_url(
        &self,
        url_capitulo: &str,
    ) -> Result<Vec<u8>, SourceError> {
        let extrator = self
            .selecionar(Some(url_capitulo))
            .ok_or_else(|| SourceError::Operacao("Nenhum extrator registrado".to_string()))?;

        let entrada = EntradaImportacao {
            obra_titulo: "Importacao".to_string(),
            numero_capitulo: 0,
            url: Some(url_capitulo.to_string()),
            html: None,
            urls_ordenadas: Vec::new(),
        };

        let urls_paginas = extrator.extrair_paginas(&entrada).await?;
        if urls_paginas.is_empty() {
            return Err(SourceError::Operacao(
                "Nenhuma página encontrada para o capítulo informado".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .http1_only()
            .pool_max_idle_per_host(0)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut paginas_bytes: Vec<(&'static str, Vec<u8>)> = Vec::with_capacity(urls_paginas.len());
        for url in &urls_paginas {
            let resp = client
                .get(url)
                .header(reqwest::header::REFERER, url_capitulo)
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(SourceError::Operacao(format!(
                    "Falha ao baixar imagem {}: HTTP {}",
                    url,
                    resp.status()
                )));
            }
            let ext = extrair_extensao_url(url);
            let bytes = resp.bytes().await?.to_vec();
            paginas_bytes.push((ext, bytes));
        }

        empacotar_paginas_em_cbz(&paginas_bytes)
    }
}

/// Descobre capítulos relacionados a uma obra a partir da URL da página principal (ou HTML colado).
pub async fn descobrir_capitulos_por_link(
    url_obra: &str,
    html_opcional: Option<&str>,
) -> Result<ObraExtraidaPorLink, SourceError> {
    let url_limpa = url_obra.trim();
    if url_limpa.is_empty() {
        return Err(SourceError::Operacao("Informe a URL da obra".to_string()));
    }

    let html = if let Some(h) = html_opcional.filter(|s| !s.trim().is_empty()) {
        h.to_string()
    } else {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .http1_only()
            .pool_max_idle_per_host(0)
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let resp = client.get(url_limpa).send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao acessar a página da obra (cole o HTML da página se o site exigir navegador).",
                resp.status()
            )));
        }
        resp.text().await?
    };

    let extraida = extrair_capitulos_de_html(url_limpa, &html);
    if extraida.capitulos.is_empty() {
        return Err(SourceError::Operacao(
            "Nenhum link de capítulo reconhecido na página da obra. Se o site carregar capítulos dinamicamente, cole o HTML da página.".to_string(),
        ));
    }
    Ok(extraida)
}

/// Extrai metadados e capítulos vinculados de um HTML de página principal de obra
/// (suportando padrões como `mangafire.to/title/.../chapter/...` e `mangalivre.to/manga/.../capitulo-.../`).
pub fn extrair_capitulos_de_html(url_obra: &str, html: &str) -> ObraExtraidaPorLink {
    let base_origin = extrair_origem(url_obra);
    let caminho_obra = extrair_caminho_url(url_obra);
    let titulo = extrair_titulo_obra(html, &caminho_obra);
    let capa_url = extrair_meta_og_image(html, &base_origin);

    let mut vistos = std::collections::HashSet::new();
    let mut capitulos: Vec<CapituloBuscaResultado> = Vec::new();
    let lower = html.to_lowercase();
    let mut offset = 0;

    while let Some(pos) = lower[offset..].find("<a ") {
        let abs_pos = offset + pos;
        let Some(tag_end_rel) = lower[abs_pos..].find('>') else {
            break;
        };
        let tag_end = abs_pos + tag_end_rel;
        let tag_html = &html[abs_pos..=tag_end];
        let close_pos = lower[tag_end + 1..]
            .find("</a>")
            .map(|p| tag_end + 1 + p)
            .unwrap_or(tag_end + 1);
        let anchor_text = limpar_tags_html(&html[tag_end + 1..close_pos]);
        offset = tag_end + 1;

        let Some(raw_href) = extrair_atributo_href(tag_html) else {
            continue;
        };
        let href_trim = raw_href.trim();
        if href_trim.is_empty()
            || href_trim.starts_with('#')
            || href_trim.starts_with("javascript:")
        {
            continue;
        }

        let url_completa = resolver_url(href_trim, &base_origin, url_obra);
        if !eh_link_capitulo(&url_completa, &caminho_obra) {
            continue;
        }
        let normalizada = url_completa.trim_end_matches('/').to_string();
        if !vistos.insert(normalizada) {
            continue;
        }

        let numero = extrair_numero_capitulo(&url_completa, &anchor_text)
            .or_else(|| extrair_data_number(tag_html));
        let idioma = detectar_idioma_link(&url_completa, &anchor_text);
        let titulo_cap = if anchor_text.is_empty() {
            None
        } else {
            Some(anchor_text)
        };

        capitulos.push(CapituloBuscaResultado {
            fonte_id: "extrator-link".to_string(),
            obra_id_fonte: url_obra.to_string(),
            capitulo_id_fonte: url_completa.clone(),
            numero,
            titulo: titulo_cap,
            url: Some(url_completa),
            idioma,
        });
    }

    // Camada 2: Varredura em JSON/scripts embutidos (ex.: __NEXT_DATA__ ou listas hidratadas)
    if capitulos.is_empty() && !caminho_obra.is_empty() {
        for cand in extrair_urls_scripts(html, &base_origin, url_obra) {
            if !eh_link_capitulo(&cand, &caminho_obra) {
                continue;
            }
            let normalizada = cand.trim_end_matches('/').to_string();
            if !vistos.insert(normalizada) {
                continue;
            }
            let numero = extrair_numero_capitulo(&cand, "");
            let idioma = detectar_idioma_link(&cand, "");
            capitulos.push(CapituloBuscaResultado {
                fonte_id: "extrator-link".to_string(),
                obra_id_fonte: url_obra.to_string(),
                capitulo_id_fonte: cand.clone(),
                numero,
                titulo: None,
                url: Some(cand),
                idioma,
            });
        }
    }

    // Camada 3: Heurística de agrupamento repetitivo sob o mesmo prefixo da obra
    if capitulos.is_empty() && !caminho_obra.is_empty() {
        let mut ag_offset = 0;
        while let Some(pos) = lower[ag_offset..].find("<a ") {
            let abs_pos = ag_offset + pos;
            let Some(tag_end_rel) = lower[abs_pos..].find('>') else {
                break;
            };
            let tag_end = abs_pos + tag_end_rel;
            let tag_html = &html[abs_pos..=tag_end];
            let close_pos = lower[tag_end + 1..]
                .find("</a>")
                .map(|p| tag_end + 1 + p)
                .unwrap_or(tag_end + 1);
            let anchor_text = limpar_tags_html(&html[tag_end + 1..close_pos]);
            ag_offset = tag_end + 1;

            let Some(raw_href) = extrair_atributo_href(tag_html) else {
                continue;
            };
            let url_completa = resolver_url(raw_href.trim(), &base_origin, url_obra);
            let caminho = extrair_caminho_url(&url_completa);
            if caminho.starts_with(&caminho_obra) && caminho.len() > caminho_obra.len() + 1 {
                let resto = &caminho[caminho_obra.len() + 1..];
                if resto.chars().any(|c| c.is_ascii_digit()) {
                    let normalizada = url_completa.trim_end_matches('/').to_string();
                    if vistos.insert(normalizada) {
                        let numero = extrair_numero_capitulo(&url_completa, &anchor_text)
                            .or_else(|| extrair_data_number(tag_html));
                        let idioma = detectar_idioma_link(&url_completa, &anchor_text);
                        capitulos.push(CapituloBuscaResultado {
                            fonte_id: "extrator-link".to_string(),
                            obra_id_fonte: url_obra.to_string(),
                            capitulo_id_fonte: url_completa.clone(),
                            numero,
                            titulo: if anchor_text.is_empty() { None } else { Some(anchor_text) },
                            url: Some(url_completa),
                            idioma,
                        });
                    }
                }
            }
        }
    }

    capitulos.sort_by_key(|c| c.numero.unwrap_or(i32::MAX));

    ObraExtraidaPorLink {
        titulo,
        capa_url,
        url_obra: url_obra.to_string(),
        capitulos,
    }
}

fn extrair_data_number(tag: &str) -> Option<i32> {
    let lower = tag.to_lowercase();
    for attr in ["data-number=", "data-chapter="] {
        if let Some(idx) = lower.find(attr) {
            let rest = tag[idx + attr.len()..].trim_start();
            let digitos: String = rest
                .trim_start_matches(['"', '\''])
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digitos.parse::<i32>() {
                return Some(n);
            }
        }
    }
    None
}

fn extrair_urls_scripts(html: &str, base_origin: &str, url_obra: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = html.to_lowercase();
    let mut offset = 0;
    while let Some(pos) = lower[offset..].find("<script") {
        let abs = offset + pos;
        let Some(end_tag) = lower[abs..].find('>') else {
            break;
        };
        let start_body = abs + end_tag + 1;
        let end_body = lower[start_body..]
            .find("</script>")
            .map(|p| start_body + p)
            .unwrap_or(html.len());
        let body = &html[start_body..end_body];
        offset = end_body;

        for token in body.split(['"', '\'']) {
            let t = token.trim();
            if t.contains("/chapter/") || t.contains("/capitulo-") || t.contains("/chapter-") {
                out.push(resolver_url(t, base_origin, url_obra));
            }
        }
    }
    out
}

fn extrair_origem(url: &str) -> String {
    if let Some(idx_scheme) = url.find("://") {
        let after = idx_scheme + 3;
        if let Some(slash) = url[after..].find('/') {
            return url[..after + slash].to_string();
        }
        return url.to_string();
    }
    String::new()
}

fn extrair_caminho_url(url: &str) -> String {
    let sem_query = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);
    if let Some(idx_scheme) = sem_query.find("://") {
        let after = idx_scheme + 3;
        if let Some(slash) = sem_query[after..].find('/') {
            return sem_query[after + slash..].trim_end_matches('/').to_lowercase();
        }
        return String::new();
    }
    sem_query.trim_end_matches('/').to_lowercase()
}

fn resolver_url(href: &str, base_origin: &str, url_obra: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{}", href)
    } else if href.starts_with('/') {
        format!("{}{}", base_origin, href)
    } else {
        let base = url_obra.trim_end_matches('/');
        format!("{}/{}", base, href)
    }
}

fn extrair_atributo_href(tag: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let idx = lower.find("href=")?;
    let rest = tag[idx + 5..].trim_start();
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)?;
        Some(rest[1..1 + end].to_string())
    } else {
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

fn eh_link_capitulo(url_completa: &str, caminho_obra: &str) -> bool {
    let caminho = extrair_caminho_url(url_completa);
    if caminho.is_empty() || caminho == caminho_obra {
        return false;
    }
    let prefixo_obra = if !caminho_obra.is_empty() && caminho.starts_with(caminho_obra) {
        let resto = &caminho[caminho_obra.len()..];
        resto.starts_with('/')
    } else {
        false
    };

    let tem_padrao_capitulo = caminho.contains("/chapter/")
        || caminho.contains("/capitulo-")
        || caminho.contains("/chapter-")
        || caminho.contains("/cap-")
        || caminho.contains("/ch-");

    if !caminho_obra.is_empty() && caminho.contains("/title/") {
        return prefixo_obra && tem_padrao_capitulo;
    }
    if !caminho_obra.is_empty() && caminho.contains("/manga/") {
        return prefixo_obra && tem_padrao_capitulo;
    }
    tem_padrao_capitulo
}

fn extrair_numero_capitulo(url: &str, texto: &str) -> Option<i32> {
    let caminho = extrair_caminho_url(url);
    for seg in caminho.split('/').rev() {
        for pref in ["capitulo-", "chapter-", "cap-", "ch-"] {
            if let Some(rest) = seg.strip_prefix(pref) {
                let digitos: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(n) = digitos.parse::<i32>() {
                    return Some(n);
                }
            }
        }
    }
    let lower_txt = texto.to_lowercase();
    for palavra in ["capítulo", "capitulo", "chapter", "cap.", "cap ", "ch.", "ch "] {
        if let Some(pos) = lower_txt.find(palavra) {
            let after = lower_txt[pos + palavra.len()..].trim_start();
            let digitos: String = after
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digitos.parse::<i32>() {
                return Some(n);
            }
        }
    }
    None
}

fn detectar_idioma_link(url: &str, texto: &str) -> Option<String> {
    let comb = format!("{} {}", url, texto).to_lowercase();
    if comb.contains("pt-br") || comb.contains("/pt/") || comb.contains("portugues") {
        Some("PT".to_string())
    } else if comb.contains("/en/") || comb.contains("-en-") || comb.contains("english") {
        Some("ENG".to_string())
    } else if comb.contains("/ja/") || comb.contains("/jp/") || comb.contains("japanese") {
        Some("JPN".to_string())
    } else {
        None
    }
}

fn limpar_tags_html(s: &str) -> String {
    let mut out = String::new();
    let mut dentro = false;
    for ch in s.chars() {
        match ch {
            '<' => dentro = true,
            '>' => {
                dentro = false;
                out.push(' ');
            }
            _ if !dentro => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extrair_titulo_obra(html: &str, caminho_obra: &str) -> String {
    let lower = html.to_lowercase();
    if let (Some(a), Some(b)) = (lower.find("<h1"), lower.find("</h1>")) {
        if let Some(gt) = html[a..b].find('>') {
            let txt = limpar_tags_html(&html[a + gt + 1..b]);
            if !txt.is_empty() {
                return txt;
            }
        }
    }
    if let (Some(a), Some(b)) = (lower.find("<title>"), lower.find("</title>")) {
        let txt = limpar_tags_html(&html[a + 7..b]);
        let limpo = txt
            .split(" - ")
            .next()
            .unwrap_or(&txt)
            .split(" | ")
            .next()
            .unwrap_or(&txt)
            .trim()
            .to_string();
        if !limpo.is_empty() {
            return limpo;
        }
    }
    let slug = caminho_obra
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("Obra Importada");
    slug.replace('-', " ")
}

fn extrair_meta_og_image(html: &str, base_origin: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let pos = lower.find("og:image")?;
    let trecho_start = pos.saturating_sub(120);
    let trecho_end = (pos + 240).min(html.len());
    let trecho = &html[trecho_start..trecho_end];
    let lower_trecho = trecho.to_lowercase();
    let idx = lower_trecho.find("content=")?;
    let rest = trecho[idx + 8..].trim_start();
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)?;
        let raw = rest[1..1 + end].trim();
        if raw.starts_with("http") {
            Some(raw.to_string())
        } else if raw.starts_with('/') && !base_origin.is_empty() {
            Some(format!("{}{}", base_origin, raw))
        } else {
            None
        }
    } else {
        None
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

    #[test]
    fn extrair_capitulos_de_html_suporta_mangafire_mangalivre_e_scripts() {
        let html_mf = r#"
            <html>
              <head>
                <title>Medaka Box - MangaFire</title>
                <meta property="og:image" content="https://cdn.exemplo.com/medaka.jpg" />
              </head>
              <body>
                <a href="/title/jw2-medaka-boxx/chapter/4570384" data-number="192">Ch. 192</a>
                <a href="/title/jw2-medaka-boxx/chapter/4570383" data-number="191">Ch. 191</a>
              </body>
            </html>
        "#;
        let mf = extrair_capitulos_de_html("https://mangafire.to/title/jw2-medaka-boxx", html_mf);
        assert_eq!(mf.titulo, "Medaka Box");
        assert_eq!(mf.capa_url.as_deref(), Some("https://cdn.exemplo.com/medaka.jpg"));
        assert_eq!(mf.capitulos.len(), 2);
        assert_eq!(mf.capitulos[0].numero, Some(191));
        assert_eq!(
            mf.capitulos[1].capitulo_id_fonte,
            "https://mangafire.to/title/jw2-medaka-boxx/chapter/4570384"
        );

        let html_ml = r#"
            <html>
              <head><title>Chainsaw Man - MangaLivre</title></head>
              <body>
                <a href="https://mangalivre.to/manga/chainsaw-man-pt-br/capitulo-208/">Capítulo 208</a>
                <a href="/manga/chainsaw-man-pt-br/capitulo-207/">Capítulo 207</a>
              </body>
            </html>
        "#;
        let ml = extrair_capitulos_de_html(
            "https://mangalivre.to/manga/chainsaw-man-pt-br/",
            html_ml,
        );
        assert_eq!(ml.titulo, "Chainsaw Man");
        assert_eq!(ml.capitulos.len(), 2);
        assert_eq!(ml.capitulos[0].numero, Some(207));
        assert_eq!(ml.capitulos[0].idioma.as_deref(), Some("PT"));
        assert_eq!(ml.capitulos[1].numero, Some(208));

        let html_script = r#"
            <html><head><title>Obra Script</title></head>
            <body><script>window.__DATA__ = ["/title/jw2-medaka-boxx/chapter/4570384"];</script></body></html>
        "#;
        let sc = extrair_capitulos_de_html("https://mangafire.to/title/jw2-medaka-boxx", html_script);
        assert_eq!(sc.capitulos.len(), 1);
    }
}
