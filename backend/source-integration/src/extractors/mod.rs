//! Arquitetura modular de extratores por site para importação de capítulos em `.cbz`.
//!
//! Novos sites podem ser adicionados implementando [`ExtratorSite`] e registrando-os
//! em [`RegistroExtratores`].

pub mod generico;
pub mod mangadex;
pub mod mangafire;
pub mod nhentai;

use std::future::Future;
use std::io::{Cursor, Write};
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};

/// Callback opcional registrado pelo app desktop para capturar páginas via Janela Assistida (`mam-assistido`).
pub type FallbackJanelaAssistidaFn = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Vec<String>, SourceError>> + Send>>
        + Send
        + Sync,
>;

static FALLBACK_JANELA_ASSISTIDA: OnceLock<FallbackJanelaAssistidaFn> = OnceLock::new();

/// Registra o fallback de captura via Janela Assistida do WebView (chamado pelo `src-tauri` no startup).
pub fn registrar_fallback_janela_assistida(cb: FallbackJanelaAssistidaFn) {
    let _ = FALLBACK_JANELA_ASSISTIDA.set(cb);
}

async fn extrair_paginas_com_fallback(
    extrator: &Arc<dyn ExtratorSite>,
    entrada: &EntradaImportacao,
) -> Result<Vec<String>, SourceError> {
    let resultado = extrator.extrair_paginas(entrada).await;
    if let Ok(ref urls) = resultado {
        if !urls.is_empty() {
            return Ok(urls.clone());
        }
    }

    if let Some(url) = entrada.url.as_deref().filter(|u| !u.trim().is_empty()) {
        if let Some(cb) = FALLBACK_JANELA_ASSISTIDA.get() {
            if let Ok(urls_assistidas) = cb(url.to_string()).await {
                if !urls_assistidas.is_empty() {
                    return Ok(urls_assistidas);
                }
            }
        }
    }

    resultado
}

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
    #[serde(default)]
    pub obra_id_destino: Option<String>,
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

/// Garante que apenas URLs HTTP/HTTPS sejam requisitadas pelo downloader (bloqueia `file://`, etc.).
fn validar_url_http_segura(url: &str) -> Result<(), SourceError> {
    let lower = url.trim().to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        Ok(())
    } else {
        Err(SourceError::Operacao(format!(
            "Esquema de URL bloqueado por segurança: {}",
            url
        )))
    }
}

/// Baixa e valida (URL HTTP/HTTPS, Content-Length <= 50 MB, magic bytes) uma lista ordenada de páginas.
async fn baixar_e_validar_paginas<F>(
    urls: &[String],
    referer_fallback: Option<&str>,
    usar_headers_conector: bool,
    mut on_progress: F,
) -> Result<Vec<(&'static str, Vec<u8>)>, SourceError>
where
    F: FnMut(usize, usize) + Send,
{
    const MAX_IMAGEM_BYTES: u64 = 50 * 1024 * 1024;
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .http1_only()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let total = urls.len();
    let mut paginas_bytes: Vec<(&'static str, Vec<u8>)> = Vec::with_capacity(total);
    for (idx, url_img) in urls.iter().enumerate() {
        validar_url_http_segura(url_img)?;
        let mut req = client.get(url_img);
        if usar_headers_conector {
            let headers_cfg = crate::connectors::GerenciadorConectores::headers_para_url(url_img, None);
            if headers_cfg.is_empty() {
                if let Some(ref_url) = referer_fallback {
                    req = req.header(reqwest::header::REFERER, ref_url);
                }
            } else {
                for (k, v) in headers_cfg {
                    req = req.header(&k, &v);
                }
            }
        } else if let Some(ref_url) = referer_fallback {
            req = req.header(reqwest::header::REFERER, ref_url);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "Falha ao baixar imagem '{}': HTTP {}",
                url_img,
                resp.status()
            )));
        }
        if let Some(len) = resp.content_length() {
            if len > MAX_IMAGEM_BYTES {
                return Err(SourceError::Operacao(
                    "Arquivo de imagem excede o limite de segurança (50 MB)".to_string(),
                ));
            }
        }
        let bytes = resp.bytes().await?.to_vec();
        let ext = validar_bytes_imagem(&bytes)?;
        paginas_bytes.push((ext, bytes));
        on_progress(idx + 1, total);
    }
    Ok(paginas_bytes)
}

/// Valida os magic bytes de uma imagem baixada para impedir gravação de executáveis/HTML maliciosos no CBZ.
pub fn validar_bytes_imagem(bytes: &[u8]) -> Result<&'static str, SourceError> {
    const MAX_IMAGEM_BYTES: usize = 50 * 1024 * 1024;
    if bytes.len() < 8 {
        return Err(SourceError::Operacao(
            "Arquivo de imagem inválido ou vazio".to_string(),
        ));
    }
    if bytes.len() > MAX_IMAGEM_BYTES {
        return Err(SourceError::Operacao(
            "Arquivo de imagem excede o limite de segurança (50 MB)".to_string(),
        ));
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok("jpg");
    }
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok("png");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Ok("gif");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok("webp");
    }
    if bytes.starts_with(b"BM") {
        return Ok("bmp");
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return Ok("avif");
    }
    Err(SourceError::Operacao(
        "Conteúdo baixado bloqueado: assinatura não corresponde a uma imagem válida".to_string(),
    ))
}

/// Baixa todas as páginas de um capítulo a partir da sua URL e retorna os bytes do `.cbz` em memória.
impl RegistroExtratores {
    pub fn novo() -> Self {
        Self {
            extratores: Vec::new(),
        }
    }

    /// Cria o registro padrão com MangaDex, MangaFire, nHentai e extrator Genérico.
    pub fn padrao() -> Self {
        let mut reg = Self::novo();
        reg.registrar(Arc::new(mangadex::MangaDexExtractor::nova()));
        reg.registrar(Arc::new(mangafire::MangaFireExtractor::nova()));
        reg.registrar(Arc::new(nhentai::NHentaiExtractor::nova()));
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

        let urls = extrair_paginas_com_fallback(&extrator, entrada).await?;
        if urls.is_empty() {
            return Err(SourceError::Operacao(
                "Nenhuma URL de página obtida para o capítulo".to_string(),
            ));
        }

        let origem_ref = entrada
            .url
            .as_deref()
            .map(extrair_origem)
            .filter(|o| !o.is_empty())
            .map(|o| format!("{}/", o));

        let paginas_bytes = baixar_e_validar_paginas(&urls, origem_ref.as_deref(), false, |_, _| {}).await?;

        let cbz_bytes = empacotar_paginas_em_cbz(&paginas_bytes)?;
        let hash = format!("{:x}", Sha256::digest(&cbz_bytes));

        let obra_existente = if let Some(ref id_dest) = entrada.obra_id_destino {
            let id_trim = id_dest.trim();
            if !id_trim.is_empty() {
                db.buscar_obra_por_id(ObraId::from_string(id_trim.to_string())).await?
            } else {
                None
            }
        } else {
            None
        };

        let (titulo_efetivo, caminho_rel_obra, obra_para_vincular) = match obra_existente {
            Some(o) => (o.titulo.clone(), o.caminho_relativo.as_str().to_string(), Some(o)),
            None => {
                let pasta_obra = sanitizar_nome(&entrada.obra_titulo);
                (entrada.obra_titulo.clone(), format!("Manga/{}", pasta_obra), None)
            }
        };

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

        let obra = match obra_para_vincular {
            Some(o) => o,
            None => match db.buscar_obra_por_titulo(&titulo_efetivo).await? {
                Some(o) => o,
                None => {
                    db.criar_obra(
                        ObraId::new(),
                        titulo_efetivo.clone(),
                        CaminhoRelativo::from_str(caminho_rel_obra.clone()),
                        Some(extrator.id().to_string()),
                    )
                    .await?
                }
            },
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
            "nome": format!("{} — Cap. {}", titulo_efetivo, entrada.numero_capitulo),
            "pasta": caminho_rel_obra,
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
            obra_titulo: titulo_efetivo,
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
        self.baixar_capitulo_cbz_por_url_com_progresso(url_capitulo, |_, _| {})
            .await
    }

    /// Baixa todas as páginas de um capítulo notificando `(paginas_baixadas, total_paginas)`.
    pub async fn baixar_capitulo_cbz_por_url_com_progresso<F>(
        &self,
        url_capitulo: &str,
        mut on_progress: F,
    ) -> Result<Vec<u8>, SourceError>
    where
        F: FnMut(usize, usize) + Send,
    {
        let extrator = self
            .selecionar(Some(url_capitulo))
            .ok_or_else(|| SourceError::Operacao("Nenhum extrator registrado".to_string()))?;

        let entrada = EntradaImportacao {
            obra_titulo: "Importacao".to_string(),
            numero_capitulo: 0,
            url: Some(url_capitulo.to_string()),
            html: None,
            urls_ordenadas: Vec::new(),
            obra_id_destino: None,
        };

        let urls_paginas = extrair_paginas_com_fallback(&extrator, &entrada).await?;
        if urls_paginas.is_empty() {
            return Err(SourceError::Operacao(
                "Nenhuma página encontrada para o capítulo informado".to_string(),
            ));
        }

        let origem_capitulo = extrair_origem(url_capitulo);
        let referer_padrao = if origem_capitulo.is_empty() {
            url_capitulo.to_string()
        } else {
            format!("{}/", origem_capitulo)
        };

        let paginas_bytes = baixar_e_validar_paginas(
            &urls_paginas,
            Some(&referer_padrao),
            true,
            on_progress,
        )
        .await?;

        empacotar_paginas_em_cbz(&paginas_bytes)
    }
}

/// Descobre capítulos relacionados a uma obra a partir da URL da página principal (ou HTML colado),
/// usando por padrão idioma preferido `pt-br` e fallback automático para o Leitor Web Inteligente (CDP).
pub async fn descobrir_capitulos_por_link(
    url_obra: &str,
    html_opcional: Option<&str>,
) -> Result<ObraExtraidaPorLink, SourceError> {
    descobrir_capitulos_por_link_com_opcoes(url_obra, html_opcional, Some("pt-br")).await
}

/// Descobre capítulos com preferência explícita de idioma (`pt-br`, `en`, `ja`, `all`).
/// Para SPAs ou sites paginados, aciona o `ai-engine` (CDP multi-página/multi-idioma) e faz fallback para `--dump-dom`.
pub async fn descobrir_capitulos_por_link_com_opcoes(
    url_obra: &str,
    html_opcional: Option<&str>,
    idioma_preferido: Option<&str>,
) -> Result<ObraExtraidaPorLink, SourceError> {
    descobrir_capitulos_por_link_com_biblioteca(url_obra, html_opcional, idioma_preferido, None)
        .await
}

/// Descobre capítulos com preferência explícita de idioma e carregamento de conectores aprendidos da biblioteca.
pub async fn descobrir_capitulos_por_link_com_biblioteca(
    url_obra: &str,
    html_opcional: Option<&str>,
    idioma_preferido: Option<&str>,
    raiz_biblioteca: Option<&Path>,
) -> Result<ObraExtraidaPorLink, SourceError> {
    let url_limpa = url_obra.trim();
    if url_limpa.is_empty() {
        return Err(SourceError::Operacao("Informe a URL da obra".to_string()));
    }

    // Obras únicas com API direta (ex.: nhentai.net/g/<id>/)
    if nhentai::extrair_gallery_id(url_limpa).is_some() {
        return nhentai::obter_obra_galeria(url_limpa).await;
    }

    let rota = crate::universal::ConectorUniversal::correlacionar_link(url_limpa, raiz_biblioteca);
    let padroes_extra = rota.padroes_url.clone();

    // URL direta de capítulo unitário: extrai metadados e páginas do próprio capítulo sem varrer Prev/Next como lote.
    if eh_url_capitulo_direto(url_limpa, &padroes_extra) {
        let mut html_capitulo = html_opcional
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string());

        if html_capitulo.is_none() {
            let client_meta = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
                .http1_only()
                .pool_max_idle_per_host(0)
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            if let Ok(resp) = client_meta.get(url_limpa).send().await {
                if resp.status().is_success() {
                    if let Ok(texto) = resp.text().await {
                        html_capitulo = Some(texto);
                    }
                }
            }
        }

        let paginas = match RegistroExtratores::padrao().selecionar(Some(url_limpa)) {
            Some(extrator) => extrator
                .extrair_paginas(&EntradaImportacao {
                    obra_titulo: String::new(),
                    numero_capitulo: 1,
                    url: Some(url_limpa.to_string()),
                    html: html_capitulo.clone(),
                    urls_ordenadas: Vec::new(),
                    obra_id_destino: None,
                })
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        };

        if !paginas.is_empty() || html_capitulo.is_some() {
            return Ok(extrair_capitulo_unitario_de_url_e_html(
                url_limpa,
                html_capitulo.as_deref(),
                &paginas,
            ));
        }
    }

    if let Some(h) = html_opcional.filter(|s| !s.trim().is_empty()) {
        let extraida = extrair_capitulos_de_html_com_biblioteca(url_limpa, h, raiz_biblioteca);
        if extraida.capitulos.is_empty() {
            return Err(SourceError::Operacao(
                "Nenhum link de capítulo reconhecido no HTML fornecido.".to_string(),
            ));
        }
        return Ok(extraida);
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .pool_max_idle_per_host(0)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut html_obtido: Option<String> = None;
    let mut bloqueado_cloudflare = false;
    if let Ok(resp) = client
        .get(url_limpa)
        .header(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .header(reqwest::header::ACCEPT_LANGUAGE, "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7")
        .send()
        .await
    {
        let status = resp.status();
        let cf_mitigated = resp
            .headers()
            .get("cf-mitigated")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.eq_ignore_ascii_case("challenge"));
        if status.as_u16() == 403 || cf_mitigated {
            bloqueado_cloudflare = true;
        }
        if status.is_success() {
            if let Ok(html) = resp.text().await {
                if html.contains("<title>Just a moment...</title>")
                    || html.contains("/cdn-cgi/challenge-platform/")
                {
                    bloqueado_cloudflare = true;
                } else if !html.trim().is_empty() {
                    html_obtido = Some(html);
                }
            }
        }
    }

    if html_obtido.is_none() {
        html_obtido = buscar_html_via_curl(url_limpa).await;
    }

    let mut melhor_resultado: Option<ObraExtraidaPorLink> = None;
    if let Some(html) = html_obtido {
        if url_limpa.to_lowercase().contains("nhentai.net/") {
            let colecao = nhentai::extrair_colecao_html(url_limpa, &html);
            if !colecao.capitulos.is_empty() {
                return Ok(colecao);
            }
        }
        let mut extraida =
            extrair_capitulos_de_html_com_biblioteca(url_limpa, &html, raiz_biblioteca);

                // Suporte nativo ao WeebCentral (HTMX /series/<id>/full-chapter-list)
                if let Some(wc_list_url) = weebcentral_full_chapter_list_url(url_limpa) {
                    if let Ok(resp_wc) = client
                        .get(&wc_list_url)
                        .header("HX-Request", "true")
                        .send()
                        .await
                    {
                        if resp_wc.status().is_success() {
                            if let Ok(wc_html) = resp_wc.text().await {
                                let via_wc = extrair_capitulos_de_html(url_limpa, &wc_html);
                                if via_wc.capitulos.len() > extraida.capitulos.len() {
                                    extraida.capitulos = via_wc.capitulos;
                                }
                            }
                        }
                    }
                }

                // Suporte automático a temas WordPress Madara (lista completa via POST .../ajax/chapters/)
                if extraida.capitulos.len() <= 2
                    || html.contains("wp-manga")
                    || html.contains("ajax/chapters")
                {
                    let ajax_url = format!("{}/ajax/chapters/", url_limpa.trim_end_matches('/'));
                    if let Ok(resp_ajax) = client
                        .post(&ajax_url)
                        .header("X-Requested-With", "XMLHttpRequest")
                        .send()
                        .await
                    {
                        if resp_ajax.status().is_success() {
                            if let Ok(ajax_html) = resp_ajax.text().await {
                                let via_ajax = extrair_capitulos_de_html(url_limpa, &ajax_html);
                                if via_ajax.capitulos.len() > extraida.capitulos.len() {
                                    extraida.capitulos = via_ajax.capitulos;
                                }
                            }
                        }
                    }
                }

                if extraida.capitulos.len() > 2 {
                    return Ok(extraida);
                }
                if !extraida.capitulos.is_empty() {
                    melhor_resultado = Some(extraida);
                }
    }

    // Fallback 1: Leitor Web Inteligente (CDP multi-página + seletor de idioma) via ai-engine
    if let Ok(extraida_cdp) =
        smart_discover_via_ai_engine(url_limpa, idioma_preferido.unwrap_or("pt-br")).await
    {
        if extraida_cdp.capitulos.len()
            > melhor_resultado.as_ref().map(|r| r.capitulos.len()).unwrap_or(0)
        {
            return Ok(extraida_cdp);
        }
    }

    // Fallback 2: Renderização simples --dump-dom caso ai-engine Python não esteja acessível
    if let Ok(dom_renderizado) = renderizar_html_com_navegador(url_limpa).await {
        let extraida = extrair_capitulos_de_html(url_limpa, &dom_renderizado);
        if extraida.capitulos.len()
            > melhor_resultado.as_ref().map(|r| r.capitulos.len()).unwrap_or(0)
        {
            return Ok(extraida);
        }
    }

    if let Some(res) = melhor_resultado {
        return Ok(res);
    }

    if bloqueado_cloudflare {
        return Err(SourceError::Operacao(
            "Site protegido por verificação humana/Cloudflare (HTTP 403). Abra 'Opções Avançadas' -> 'Janela Assistida', conclua a verificação na janela e clique em 'Capturar da Janela'.".to_string(),
        ));
    }

    Err(SourceError::Operacao(
        "Nenhum link de capítulo reconhecido na página da obra (nem após Leitor Web CDP). Se necessário, cole o HTML da página.".to_string(),
    ))
}

async fn buscar_html_via_curl(url: &str) -> Option<String> {
    let url_owned = url.to_string();
    tokio::task::spawn_blocking(move || {
        for bin in ["curl.exe", "curl"] {
            let mut cmd = std::process::Command::new(bin);
            cmd.args([
                "-sL",
                "--max-time",
                "15",
                "-A",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
                "-H",
                "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                "-H",
                "Accept-Language: pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7",
                &url_owned,
            ]);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000);
            }
            if let Ok(out) = cmd.output() {
                if out.status.success() && !out.stdout.is_empty() {
                    return Some(String::from_utf8_lossy(&out.stdout).to_string());
                }
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

async fn smart_discover_via_ai_engine(
    url_obra: &str,
    idioma_preferido: &str,
) -> Result<ObraExtraidaPorLink, SourceError> {
    let py = mam_ai_engine::AiEngineManager::detectar_executavel_python();
    let dir = mam_ai_engine::AiEngineManager::detectar_diretorio_padrao();
    let manager = mam_ai_engine::AiEngineManager::nova(py, dir)
        .com_timeout(std::time::Duration::from_secs(90));
    let val = manager
        .chamar(
            "smart_discover_chapters",
            serde_json::json!({
                "url_obra": url_obra,
                "idioma_preferido": idioma_preferido,
            }),
        )
        .await
        .map_err(|e| SourceError::Operacao(e.to_string()))?;
    serde_json::from_value::<ObraExtraidaPorLink>(val)
        .map_err(|e| SourceError::Operacao(e.to_string()))
}

/// Aciona o Leitor Web Inteligente (CDP) via `ai-engine` para extrair todas as páginas de um capítulo.
pub async fn smart_extract_pages_via_ai_engine(
    url_capitulo: &str,
) -> Result<Vec<String>, SourceError> {
    let py = mam_ai_engine::AiEngineManager::detectar_executavel_python();
    let dir = mam_ai_engine::AiEngineManager::detectar_diretorio_padrao();
    let manager = mam_ai_engine::AiEngineManager::nova(py, dir)
        .com_timeout(std::time::Duration::from_secs(90));
    let val = manager
        .chamar(
            "smart_extract_chapter_pages",
            serde_json::json!({ "url_capitulo": url_capitulo }),
        )
        .await
        .map_err(|e| SourceError::Operacao(e.to_string()))?;
    let paginas = val
        .get("urls_imagens")
        .or_else(|| val.get("paginas"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(paginas)
}

/// Renderiza uma página SPA/JS utilizando Microsoft Edge ou Google Chrome em modo headless (`--dump-dom`).
pub async fn renderizar_html_com_navegador(url: &str) -> Result<String, SourceError> {
    let candidatos = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        "msedge",
        "chrome",
        "google-chrome",
        "chromium",
    ];

    let executavel = candidatos
        .iter()
        .find(|p| !p.contains('\\') || Path::new(p).exists())
        .copied()
        .unwrap_or("msedge");

    let mut cmd = tokio::process::Command::new(executavel);
    cmd.args([
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--dump-dom",
        "--virtual-time-budget=8000",
        url,
    ]);
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = tokio::time::timeout(std::time::Duration::from_secs(25), cmd.output())
        .await
        .map_err(|_| SourceError::Operacao("Tempo esgotado ao renderizar página no navegador headless".to_string()))?
        .map_err(|e| SourceError::Operacao(format!("Falha ao executar navegador headless ({}): {}", executavel, e)))?;

    let dom = String::from_utf8_lossy(&output.stdout).to_string();
    if dom.trim().is_empty() {
        return Err(SourceError::Operacao("Navegador headless retornou DOM vazio".to_string()));
    }
    Ok(dom)
}

/// Extrai metadados e capítulos vinculados de um HTML de página principal de obra
/// usando padrões padrão + padrões dinâmicos dos conectores carregados.
pub fn extrair_capitulos_de_html(url_obra: &str, html: &str) -> ObraExtraidaPorLink {
    extrair_capitulos_de_html_com_biblioteca(url_obra, html, None)
}

/// Extrai metadados e capítulos vinculados usando conectores aprendidos na biblioteca informada.
pub fn extrair_capitulos_de_html_com_biblioteca(
    url_obra: &str,
    html: &str,
    raiz_biblioteca: Option<&Path>,
) -> ObraExtraidaPorLink {
    let padroes = crate::universal::ConectorUniversal::correlacionar_link(url_obra, raiz_biblioteca)
        .padroes_url;
    extrair_capitulos_de_html_com_padroes(url_obra, html, &padroes)
}

/// Extrai metadados e capítulos vinculados de um HTML usando uma lista explícita de padrões de URL.
pub fn extrair_capitulos_de_html_com_padroes(
    url_obra: &str,
    html: &str,
    padroes_extra: &[String],
) -> ObraExtraidaPorLink {
    let base_origin = extrair_origem(url_obra);
    let caminho_obra = extrair_caminho_url(url_obra);
    let titulo = extrair_titulo_obra(html, &caminho_obra);
    let capa_url = extrair_meta_og_image(html, &base_origin);

    let mut vistos = std::collections::HashSet::new();
    let mut capitulos: Vec<CapituloBuscaResultado> = Vec::new();
    let lower = html.to_lowercase();
    let mut offset = 0;

    while let Some(pos) = lower[offset..].find("<a") {
        let abs_pos = offset + pos;
        let next_ch = lower[abs_pos + 2..].chars().next();
        if !matches!(next_ch, Some(c) if c.is_whitespace()) {
            offset = abs_pos + 2;
            continue;
        }
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
        if !eh_link_capitulo(&url_completa, &caminho_obra, padroes_extra) {
            continue;
        }
        let normalizada = url_completa.trim_end_matches('/').to_string();
        if !vistos.insert(normalizada) {
            continue;
        }

        let numero = extrair_numero_capitulo(&url_completa, &anchor_text)
            .or_else(|| extrair_data_number(tag_html));
        let ctx_start = abs_pos.saturating_sub(350);
        let idioma = detectar_idioma_link(&url_completa, &anchor_text)
            .or_else(|| detectar_idioma_contexto(&html[ctx_start..abs_pos]));
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
            if !eh_link_capitulo(&cand, &caminho_obra, padroes_extra) {
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
        let body = html[start_body..end_body]
            .replace("\\/", "/")
            .replace("\\u002F", "/")
            .replace("\\u002f", "/");
        offset = end_body;

        for token in body.split(['"', '\'']) {
            let t = token.trim();
            if t.contains("/chapter/")
                || t.contains("/chapters/")
                || t.contains("/chapter-detail/")
                || t.contains("/capitulo-")
                || t.contains("/chapter-")
                || t.contains("/read/")
                || t.contains("/reader/")
            {
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

fn weebcentral_full_chapter_list_url(url: &str) -> Option<String> {
    let lower = url.to_lowercase();
    let idx = lower.find("weebcentral.com/series/")?;
    let after = &url[idx + "weebcentral.com/series/".len()..];
    let series_id = after.split('/').next()?.trim();
    if series_id.is_empty() {
        None
    } else {
        Some(format!(
            "https://weebcentral.com/series/{}/full-chapter-list",
            series_id
        ))
    }
}

fn eh_url_capitulo_direto(url: &str, padroes_extra: &[String]) -> bool {
    let caminho = extrair_caminho_url(url);
    if caminho.is_empty()
        || caminho.contains("/series/")
        || caminho.contains("/full-chapter-list")
        || caminho.ends_with("/list")
    {
        return false;
    }
    eh_link_capitulo(url, "", padroes_extra)
}

fn limpar_titulo_obra_de_capitulo(raw_title: &str) -> String {
    let sem_site = raw_title.split(" | ").next().unwrap_or(raw_title).trim();
    for parte in sem_site.split(" - ") {
        let p = parte.trim();
        if !p.is_empty() && !eh_titulo_de_capitulo(p) {
            return p.to_string();
        }
    }
    if sem_site.is_empty() {
        "Capítulo Importado".to_string()
    } else {
        sem_site.to_string()
    }
}

pub fn extrair_capitulo_unitario_de_url_e_html(
    url_capitulo: &str,
    html_opcional: Option<&str>,
    paginas: &[String],
) -> ObraExtraidaPorLink {
    let html = html_opcional.unwrap_or("");
    let lower = html.to_lowercase();
    let base_origin = extrair_origem(url_capitulo);

    let raw_title = if let (Some(a), Some(b)) = (lower.find("<title>"), lower.find("</title>")) {
        limpar_tags_html(&html[a + 7..b])
    } else {
        String::new()
    };

    let h1_text = if let (Some(a), Some(b)) = (lower.find("<h1"), lower.find("</h1>")) {
        html[a..b]
            .find('>')
            .map(|gt| limpar_tags_html(&html[a + gt + 1..b]))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let texto_busca_num = format!("{} {}", raw_title, h1_text);
    let numero = extrair_numero_capitulo(url_capitulo, &texto_busca_num);
    let idioma = detectar_idioma_link(url_capitulo, &texto_busca_num);

    let titulo_obra = if !raw_title.is_empty() {
        limpar_titulo_obra_de_capitulo(&raw_title)
    } else if !h1_text.is_empty() && !eh_titulo_de_capitulo(&h1_text) {
        h1_text
    } else {
        "Capítulo Importado".to_string()
    };

    let capa_url = extrair_meta_og_image(html, &base_origin).or_else(|| paginas.first().cloned());
    let titulo_cap = if paginas.is_empty() {
        "Capítulo único".to_string()
    } else {
        format!("{} páginas", paginas.len())
    };

    ObraExtraidaPorLink {
        titulo: titulo_obra,
        capa_url,
        url_obra: url_capitulo.to_string(),
        capitulos: vec![CapituloBuscaResultado {
            fonte_id: "extrator-link".to_string(),
            obra_id_fonte: url_capitulo.to_string(),
            capitulo_id_fonte: url_capitulo.to_string(),
            numero,
            titulo: Some(titulo_cap),
            url: Some(url_capitulo.to_string()),
            idioma,
        }],
    }
}

fn eh_link_capitulo(url_completa: &str, caminho_obra: &str, padroes_extra: &[String]) -> bool {
    let caminho = extrair_caminho_url(url_completa);
    if caminho.is_empty() || (caminho == caminho_obra && !url_completa.contains('?')) {
        return false;
    }
    if [
        "/comments/",
        "/contact",
        "/about",
        "/privacy",
        "/terms",
        "/dmca",
        "/faq",
        "/login",
        "/register",
        "/user/",
        "/tag/",
        "/genre/",
        "/author/",
        "/search",
        "/random",
    ]
    .iter()
    .any(|ruido| caminho.contains(ruido))
    {
        return false;
    }

    let prefixo_obra = if !caminho_obra.is_empty() && caminho.starts_with(caminho_obra) {
        let resto = &caminho[caminho_obra.len()..];
        resto.starts_with('/')
    } else {
        false
    };

    let url_lower = url_completa.to_lowercase();
    let tem_padrao_capitulo = caminho.contains("/chapter/")
        || caminho.contains("/chapters/")
        || caminho.contains("/capitulo-")
        || caminho.contains("/chapter-")
        || caminho.contains("/cap-")
        || caminho.contains("/ch-")
        || caminho.contains("/read/")
        || caminho.contains("/reader/")
        || caminho.contains("/viewer")
        || caminho.contains("/episode/")
        || caminho.contains("/episode-")
        || caminho.contains("/ep/")
        || caminho.contains("/ep-")
        || caminho.contains("/volume-")
        || caminho.contains("/vol-")
        || caminho.contains("/issue-")
        || caminho.contains("-chapter-")
        || caminho.contains("-ch-")
        || caminho.contains("-cap-")
        || url_lower.contains("episode_no=")
        || url_lower.contains("chapter_id=")
        || padroes_extra.iter().any(|p| {
            if p.len() < 3 {
                return false;
            }
            if let Some(prefixo) = p.strip_suffix("{num}") {
                if let Some(pos) = caminho.find(prefixo) {
                    let resto = &caminho[pos + prefixo.len()..];
                    let seg = resto.split('/').next().unwrap_or("");
                    return !seg.is_empty() && seg.chars().any(|c| c.is_ascii_digit());
                }
                return false;
            }
            caminho.contains(p.as_str()) || url_lower.contains(p.as_str())
        });

    if !caminho_obra.is_empty() && caminho.contains("/title/") {
        return prefixo_obra && tem_padrao_capitulo;
    }
    tem_padrao_capitulo
}

fn extrair_numero_capitulo(url: &str, texto: &str) -> Option<i32> {
    let url_lower = url.to_lowercase();
    if let Some(q_idx) = url_lower.find('?') {
        let query = &url_lower[q_idx + 1..];
        for key in ["episode_no=", "chapter_no=", "chapter=", "chap=", "ep="] {
            if let Some(pos) = query.find(key) {
                let rest = &query[pos + key.len()..];
                let digitos: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = digitos.parse::<i32>() {
                    return Some(n);
                }
            }
        }
    }

    let caminho = extrair_caminho_url(url);
    for seg in caminho.split('/').rev() {
        for pref in [
            "capitulo-",
            "chapter-",
            "cap-",
            "ch-",
            "episode-",
            "ep-",
            "volume-",
            "vol-",
            "issue-",
        ] {
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
    let mut melhor_match: Option<(usize, i32)> = None;
    for palavra in [
        "capítulo",
        "capitulo",
        "chapter",
        "cap.",
        "cap ",
        "ch.",
        "ch ",
        "volume",
        "vol.",
        "vol ",
        "episode",
        "ep.",
        "ep ",
        "#",
    ] {
        if let Some(pos) = lower_txt.find(palavra) {
            let after = lower_txt[pos + palavra.len()..].trim_start();
            let digitos: String = after
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digitos.parse::<i32>() {
                if melhor_match.map_or(true, |(best_pos, _)| pos < best_pos) {
                    melhor_match = Some((pos, n));
                }
            }
        }
    }
    if melhor_match.is_some() {
        return melhor_match.map(|(_, n)| n);
    }

    // Fallback: último segmento puramente numérico na URL (ex.: /read/obra/123)
    caminho
        .split('/')
        .rev()
        .find(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        .and_then(|s| s.parse::<i32>().ok())
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

fn detectar_idioma_contexto(trecho_anterior: &str) -> Option<String> {
    let ultimo_row = trecho_anterior
        .rfind("title-detail__row\"")
        .or_else(|| trecho_anterior.rfind("title-detail__row "))
        .or_else(|| trecho_anterior.rfind("<li"))
        .or_else(|| trecho_anterior.rfind("<tr"))
        .unwrap_or(0);
    let recorte = &trecho_anterior[ultimo_row..].to_lowercase();
    if recorte.contains("title=\"pt-br\"") || recorte.contains("title=\"pt\"") {
        Some("PT".to_string())
    } else if recorte.contains("title=\"en\"") || recorte.contains("title=\"gb\"") {
        Some("ENG".to_string())
    } else if recorte.contains("title=\"ja\"") || recorte.contains("title=\"jp\"") {
        Some("JPN".to_string())
    } else if recorte.contains("title=\"es\"") {
        Some("ESP".to_string())
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

fn eh_titulo_de_capitulo(txt: &str) -> bool {
    let lower = txt.trim().to_lowercase();
    for pref in ["episode ", "ep. ", "ep ", "chapter ", "capítulo ", "capitulo ", "cap. ", "cap "] {
        if let Some(rest) = lower.strip_prefix(pref) {
            if rest.trim_start().chars().next().is_some_and(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    false
}

fn eh_titulo_ruido_ui(txt: &str) -> bool {
    let lower = txt.trim().to_lowercase();
    matches!(
        lower.as_str(),
        "status"
            | "type"
            | "author"
            | "authors"
            | "artist"
            | "artists"
            | "release"
            | "released"
            | "year"
            | "summary"
            | "description"
            | "alternative"
            | "alternative name"
            | "rating"
            | "views"
            | "latest"
            | "popular"
            | "trending"
            | "bookmark"
            | "bookmarks"
            | "filter"
            | "filters"
            | "menu"
            | "search"
            | "genres"
            | "genre"
            | "chapters"
            | "chapter list"
            | "comments"
            | "home"
            | "login"
            | "sign in"
            | "register"
            | "loading"
            | "loading..."
            | "404"
            | "error"
    )
}

fn extrair_titulo_obra(html: &str, caminho_obra: &str) -> String {
    let lower = html.to_lowercase();
    let mut offset = 0;
    while let Some(pos) = lower[offset..].find("<h1") {
        let abs = offset + pos;
        let Some(end_rel) = lower[abs..].find("</h1>") else {
            break;
        };
        let end = abs + end_rel;
        offset = end + 5;
        if let Some(gt) = html[abs..end].find('>') {
            let txt = limpar_tags_html(&html[abs + gt + 1..end]);
            if !txt.is_empty() && !eh_titulo_de_capitulo(&txt) && !eh_titulo_ruido_ui(&txt) {
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
        if !limpo.is_empty() && !eh_titulo_ruido_ui(&limpo) {
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
    let mut offset = 0;
    while let Some(pos) = lower[offset..].find("<meta") {
        let abs = offset + pos;
        let Some(end_rel) = lower[abs..].find('>') else {
            break;
        };
        let tag = &html[abs..=abs + end_rel];
        let tag_lower = &lower[abs..=abs + end_rel];
        offset = abs + end_rel + 1;

        let eh_og_image = tag_lower.contains("property=\"og:image\"")
            || tag_lower.contains("property='og:image'")
            || tag_lower.contains("name=\"og:image\"")
            || tag_lower.contains("name='og:image'")
            || tag_lower.contains("name=\"twitter:image\"")
            || tag_lower.contains("name='twitter:image'");
        if !eh_og_image {
            continue;
        }

        if let Some(idx) = tag_lower.find("content=") {
            let rest = tag[idx + 8..].trim_start();
            if let Some(quote @ ('"' | '\'')) = rest.chars().next() {
                if let Some(end) = rest[1..].find(quote) {
                    let raw = rest[1..1 + end].trim();
                    if raw.starts_with("http://") || raw.starts_with("https://") {
                        return Some(raw.to_string());
                    }
                    if raw.starts_with('/') && !base_origin.is_empty() {
                        return Some(format!("{}{}", base_origin, raw));
                    }
                }
            }
        }
    }

    // Fallback: primeira <img> com "cover" no src ou alt (ex.: WeebCentral / mangás sem og:image)
    let mut img_offset = 0;
    while let Some(pos) = lower[img_offset..].find("<img") {
        let abs = img_offset + pos;
        let Some(end_rel) = lower[abs..].find('>') else {
            break;
        };
        let tag = &html[abs..=abs + end_rel];
        let tag_lower = &lower[abs..=abs + end_rel];
        img_offset = abs + end_rel + 1;

        if !tag_lower.contains("cover") {
            continue;
        }
        if let Some(idx) = tag_lower.find("src=") {
            let rest = tag[idx + 4..].trim_start();
            if let Some(quote @ ('"' | '\'')) = rest.chars().next() {
                if let Some(end) = rest[1..].find(quote) {
                    let raw = rest[1..1 + end].trim();
                    if raw.starts_with("http://") || raw.starts_with("https://") {
                        return Some(raw.to_string());
                    }
                    if raw.starts_with('/') && !base_origin.is_empty() {
                        return Some(format!("{}{}", base_origin, raw));
                    }
                }
            }
        }
    }

    None
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

/// Extrai URLs de imagens (`data-url`, `data-original`, `data-lazy-src`, `data-src` ou `src`)
/// de um HTML preservando a ordem de aparição e deduplicando.
/// Prioriza contêineres de leitura quando presentes para evitar baixar capas de recomendações/sidebar.
/// Extrai imagens apenas se o HTML contiver um container explícito de leitor de páginas.
/// Retorna `None` para páginas SPA/JS onde o conteúdo do leitor não está no HTML estático.
pub fn extrair_urls_imagens_de_container_leitor(html: &str) -> Option<Vec<String>> {
    let lower = html.to_lowercase();
    for marker in [
        "id=\"_imagelist\"",
        "id='_imagelist'",
        "class=\"viewer_img\"",
        "class='viewer_img'",
        "viewer-img",
        "comic-images",
        "class=\"reading-content\"",
        "class='reading-content'",
        "chapter-content",
        "reader-area",
        "pages-list",
        "id=\"readerarea\"",
        "class=\"container-chapter-reader\"",
    ] {
        if let Some(start) = lower.find(marker) {
            let max_end = (start + 250_000).min(html.len());
            let tail_lower = &lower[start..max_end];
            let stop_rel = [
                "id=\"recommend",
                "class=\"recommend",
                "episode_lst",
                "class=\"footer",
                "id=\"footer",
            ]
            .iter()
            .filter_map(|stop| tail_lower.find(stop))
            .filter(|&pos| pos > 100)
            .min();
            let end = stop_rel.map_or(max_end, |rel| start + rel);
            let recorte = extrair_urls_imagens_trecho(&html[start..end]);
            if recorte.len() >= 2 {
                return Some(recorte);
            }
        }
    }
    None
}

pub fn extrair_urls_imagens_html(html: &str) -> Vec<String> {
    if let Some(recorte) = extrair_urls_imagens_de_container_leitor(html) {
        return recorte;
    }
    extrair_urls_imagens_trecho(html)
}

fn extrair_attr_tag(tag: &str, tag_lower: &str, attr: &str) -> Option<String> {
    let mut pos = 0;
    while let Some(idx) = tag_lower[pos..].find(attr) {
        let abs = pos + idx;
        if abs > 0 {
            let prev = tag_lower.as_bytes()[abs - 1];
            if prev.is_ascii_alphanumeric() || prev == b'-' || prev == b'_' {
                pos = abs + attr.len();
                continue;
            }
        }
        let rest = tag[abs + attr.len()..].trim_start();
        if let Some(q @ ('"' | '\'')) = rest.chars().next() {
            if let Some(end) = rest[1..].find(q) {
                return Some(rest[1..1 + end].trim().to_string());
            }
        }
        pos = abs + attr.len();
    }
    None
}

fn extrair_urls_imagens_trecho(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let lower = html.to_lowercase();
    let attrs_prioridade = ["data-url=", "data-original=", "data-lazy-src=", "data-src=", "src="];

    // Passo 1: varre <img ...> em ordem exata do DOM, escolhendo o atributo de maior prioridade válido.
    let mut img_offset = 0;
    while let Some(pos) = lower[img_offset..].find("<img") {
        let abs = img_offset + pos;
        let Some(end_rel) = lower[abs..].find('>') else {
            break;
        };
        let tag = &html[abs..=abs + end_rel];
        let tag_lower = &lower[abs..=abs + end_rel];
        img_offset = abs + end_rel + 1;

        for attr in attrs_prioridade {
            if let Some(candidato) = extrair_attr_tag(tag, tag_lower, attr) {
                if eh_url_imagem_valida(&candidato) && !urls.contains(&candidato) {
                    urls.push(candidato);
                    break;
                }
            }
        }
    }

    if urls.len() >= 2 {
        return urls;
    }

    // Fallback: varredura direta por atributos caso imagens estejam em wrappers não-<img>.
    for attr in attrs_prioridade {
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
    let lower = url.trim().to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return false;
    }
    let sem_query = lower.split('?').next().unwrap_or(&lower).split('#').next().unwrap_or(&lower);
    if [".js", ".mjs", ".css", ".html", ".htm", ".svg", ".ico", ".json", ".woff", ".woff2", ".ttf"]
        .iter()
        .any(|ext| sem_query.ends_with(ext))
    {
        return false;
    }
    if [
        "favicon",
        "logo",
        "avatar",
        "banner",
        "badge",
        "flag",
        "broken_image",
        "chapter-badge",
        "spinner",
        "placeholder",
        "bg_transparency",
        "blank.gif",
        "1x1.",
        "pixel.",
        "spacer.",
        "transparent.png",
        "loading.gif",
        "/cover",
        "/covers/",
        "/thumb",
        "/thumbnails/",
        "recommend",
        "discord",
    ]
    .iter()
    .any(|tok| sem_query.contains(tok))
    {
        return false;
    }
    sem_query.ends_with(".jpg")
        || sem_query.ends_with(".jpeg")
        || sem_query.ends_with(".png")
        || sem_query.ends_with(".webp")
        || sem_query.ends_with(".avif")
        || lower.contains("/page")
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
            obra_id_destino: None,
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
            obra_id_destino: None,
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

        let html_spa = r#"
            <div id="app-root">
              <div class="title-detail">
                <h1>Medaka Box</h1>
                <div class="title-detail__list scroll-area">
                  <div class="title-detail__row">
                    <span class="title-detail__row-flag" title="en">GB</span>
                    <div class="title-detail__row-name">
                      <a class="title-detail__row-link" href="/title/jw2-medaka-boxx/chapter/6229076" data-discover="true">
                        <span class="title-detail__row-num">Ch. 192.6</span>
                        <span class="title-detail__row-sub">Good Loser Kumagawa Chapter 2</span>
                      </a>
                    </div>
                  </div>
                </div>
              </div>
            </div>
        "#;
        let spa = extrair_capitulos_de_html("https://mangafire.to/title/jw2-medaka-boxx", html_spa);
        assert_eq!(spa.titulo, "Medaka Box");
        assert_eq!(spa.capitulos.len(), 1);
        assert_eq!(spa.capitulos[0].numero, Some(192));
        assert_eq!(spa.capitulos[0].idioma.as_deref(), Some("ENG"));

        assert!(!eh_url_imagem_valida("https://s.mfcdn.nl/build/mf/assets/main-B7h3oPSn.js"));
        assert!(!eh_url_imagem_valida("https://cdn.exemplo.com/assets/logo.png"));
        assert!(!eh_url_imagem_valida("https://weebcentral.com/static/images/broken_image.jpg"));
        assert!(eh_url_imagem_valida("https://pw6.mfcdn2.xyz/mf/abc123/h/p.jpg"));
    }

    #[test]
    fn extrai_weebcentral_volumes_e_padroes_tbcpl() {
        let html_wc = r#"
            <div class="flex items-center">
              <a href="/chapters/01J76XYY7Q5DA1HQMDD9D7GQTC">
                <img src="/static/images/chapter-badge-official.svg"/>
                <span>Volume 6</span>
              </a>
            </div>
            <div class="flex items-center">
              <a href="/chapters/01J76XYY7QVSM10TSM7XBV0PS9">
                <span>Chapter 12</span>
              </a>
            </div>
        "#;
        let wc = extrair_capitulos_de_html(
            "https://weebcentral.com/series/01J76XY7E2VCSR0ZCC21KGXS1K/Kobato",
            html_wc,
        );
        assert_eq!(wc.capitulos.len(), 2);
        assert_eq!(wc.capitulos[0].numero, Some(6));
        assert_eq!(
            wc.capitulos[0].url.as_deref(),
            Some("https://weebcentral.com/chapters/01J76XYY7Q5DA1HQMDD9D7GQTC")
        );
        assert_eq!(wc.capitulos[1].numero, Some(12));
        assert_eq!(
            weebcentral_full_chapter_list_url(
                "https://weebcentral.com/series/01J76XY7E2VCSR0ZCC21KGXS1K/Kobato"
            )
            .as_deref(),
            Some("https://weebcentral.com/series/01J76XY7E2VCSR0ZCC21KGXS1K/full-chapter-list")
        );
    }

    #[test]
    fn extrai_paginas_webtoon_com_data_url_ignorando_recomendacoes() {
        let html = r#"
            <html><head><title>The Greatest Estate Developer</title></head><body>
              <h1>Episode 1</h1>
              <div id="_imageList" class="viewer_img">
                <img src="https://webtoon-phinf.pstatic.net/bg_transparency.png" data-url="https://webtoon-phinf.pstatic.net/20210101_1/page_01.jpg" />
                <img src="https://webtoon-phinf.pstatic.net/bg_transparency.png" data-url="https://webtoon-phinf.pstatic.net/20210101_1/page_02.jpg" />
              </div>
              <div id="recommendList" class="recommend">
                <img src="https://webtoon-phinf.pstatic.net/other/rec_cover_1.jpg" />
                <img src="https://webtoon-phinf.pstatic.net/other/rec_cover_2.jpg" />
              </div>
            </body></html>
        "#;

        let imgs = extrair_urls_imagens_html(html);
        assert_eq!(
            imgs,
            vec![
                "https://webtoon-phinf.pstatic.net/20210101_1/page_01.jpg",
                "https://webtoon-phinf.pstatic.net/20210101_1/page_02.jpg",
            ]
        );
        assert_eq!(
            extrair_titulo_obra(html, "/en/fantasy/the-greatest-estate-developer/list?title_no=3596"),
            "The Greatest Estate Developer"
        );
    }

    #[test]
    fn descobre_capitulos_com_query_episode_no() {
        let html = r#"
            <html><head><title>The Greatest Estate Developer</title></head><body>
              <a href="/en/fantasy/the-greatest-estate-developer/episode-1/viewer?title_no=3596&episode_no=1">Episode 1</a>
              <a href="/en/fantasy/the-greatest-estate-developer/episode-2/viewer?title_no=3596&episode_no=2">Episode 2</a>
            </body></html>
        "#;
        let res = extrair_capitulos_de_html(
            "https://www.webtoons.com/en/fantasy/the-greatest-estate-developer/list?title_no=3596",
            html,
        );
        assert_eq!(res.capitulos.len(), 2);
        assert_eq!(res.capitulos[0].numero, Some(1));
        assert_eq!(res.capitulos[1].numero, Some(2));
    }

    #[test]
    fn url_direta_de_capitulo_weebcentral_retorna_capitulo_unico_sem_prev_next() {
        let url_cap = "https://weebcentral.com/chapters/01M84F1T0178JSKSEK8M3H99JS";
        let url_serie = "https://weebcentral.com/series/01J76XY7E2VCSR0ZCC21KGXS1K/Kobato";
        assert!(eh_url_capitulo_direto(url_cap, &[]));
        assert!(!eh_url_capitulo_direto(url_serie, &[]));

        let html = r#"
            <html>
              <head><title>Dan Da Dan - Chapter 175 | Weeb Central</title></head>
              <body>
                <a href="/chapters/01PREV">Prev</a>
                <a href="/chapters/01NEXT">Next</a>
                <a href="/chapters/01OTHER">List</a>
              </body>
            </html>
        "#;
        let paginas = vec![
            "https://cdn.example.com/001.png".to_string(),
            "https://cdn.example.com/002.png".to_string(),
        ];
        let res = extrair_capitulo_unitario_de_url_e_html(url_cap, Some(html), &paginas);
        assert_eq!(res.titulo, "Dan Da Dan");
        assert_eq!(res.capitulos.len(), 1);
        assert_eq!(res.capitulos[0].numero, Some(175));
        assert_eq!(res.capitulos[0].titulo.as_deref(), Some("2 páginas"));
        assert_eq!(res.capitulos[0].capitulo_id_fonte, url_cap);
        assert_eq!(res.capa_url.as_deref(), Some("https://cdn.example.com/001.png"));
    }

    #[test]
    fn validar_bytes_imagem_aceita_imagens_e_bloqueia_executavel_e_html() {
        assert_eq!(
            validar_bytes_imagem(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 1, 2, 3]).unwrap(),
            "jpg"
        );
        assert_eq!(
            validar_bytes_imagem(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap(),
            "png"
        );
        assert!(validar_bytes_imagem(b"MZ\x90\x00\x03\x00\x00\x00").is_err());
        assert!(validar_bytes_imagem(b"<!DOCTYPE html><html></html>").is_err());
        assert!(validar_url_http_segura("https://cdn.exemplo.com/1.jpg").is_ok());
        assert!(validar_url_http_segura("file:///C:/Windows/System32/cmd.exe").is_err());
    }

    #[test]
    fn extrair_titulo_obra_ignora_h1_ruido_ui_como_status() {
        let html = r#"
            <html>
              <head><title>Mahou Shoujo Madoka Magica: Majuu-hen - MangaBuddy</title></head>
              <body>
                <div class="sidebar"><h1>Status</h1><h1>Type</h1><h1>Genres</h1></div>
                <div class="main"><h1>Mahou Shoujo Madoka Magica: Majuu-hen</h1></div>
              </body>
            </html>
        "#;
        assert_eq!(
            extrair_titulo_obra(html, "/series/mahou-shoujo-madokamagica-majuu-hen.ZDKd5Q"),
            "Mahou Shoujo Madoka Magica: Majuu-hen"
        );
    }
}
