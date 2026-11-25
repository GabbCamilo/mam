//! Gerenciamento de Conectores Declarativos (`connectors/*.json`) atualizáveis sem reinstalar o app.
//!
//! Prioridade de carregamento:
//! 1. `<MAM_LIBRARY>/System/connectors/sites.json` (atualizações baixadas ou importadas offline)
//! 2. `<exe_dir>/connectors/default_sites.json` (embutido no pacote portátil)
//! 3. Fallback embutido no binário (`include_str!`)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ObraBuscaResultado, SourceError};

pub const URL_MANIFESTO_OFICIAL: &str =
    "https://raw.githubusercontent.com/GabbCamilo/mam/main/connectors/manifest.json";

const DEFAULT_SITES_EMBEDDED: &str = include_str!("../../../connectors/default_sites.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBuscaConfig {
    pub endpoint: String,
    #[serde(default = "metodo_get_padrao")]
    pub metodo: String,
    #[serde(default = "formato_json_padrao")]
    pub formato: String,
    #[serde(default)]
    pub caminho_itens: Option<String>,
    pub campo_id_ou_url: String,
    #[serde(default)]
    pub template_url_obra: Option<String>,
    pub campo_titulo: String,
    #[serde(default)]
    pub campo_capa: Option<String>,
    #[serde(default)]
    pub template_url_capa: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

fn metodo_get_padrao() -> String {
    "GET".to_string()
}
fn formato_json_padrao() -> String {
    "json".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConectorSiteConfig {
    pub id: String,
    pub nome: String,
    #[serde(default = "versao_padrao")]
    pub versao: String,
    #[serde(default = "tipo_padrao")]
    pub tipo: String, // "api" | "web-cdp" | "web"
    #[serde(default)]
    pub dominios: Vec<String>,
    #[serde(default)]
    pub url_busca: Option<String>,
    #[serde(default)]
    pub api_busca: Option<ApiBuscaConfig>,
    #[serde(default)]
    pub padroes_url_capitulo: Vec<String>,
    #[serde(default)]
    pub seletor_link: Option<String>,
    #[serde(default)]
    pub seletor_idioma_linha: Option<String>,
    #[serde(default)]
    pub seletor_paginacao: Option<String>,
    #[serde(default = "cdp_auto")]
    pub requer_cdp: String,
    #[serde(default)]
    pub seletores_imagem: Vec<String>,
    #[serde(default)]
    pub rejeitar_tokens_imagem: Vec<String>,
    #[serde(default)]
    pub headers_imagem: Option<HashMap<String, String>>,
    #[serde(default)]
    pub estrategia_descoberta: Option<String>,
}

fn versao_padrao() -> String {
    "1.0.0".to_string()
}
fn tipo_padrao() -> String {
    "web".to_string()
}
fn cdp_auto() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogoConectores {
    pub schema_version: u32,
    pub version: String,
    pub sites: Vec<ConectorSiteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestoConectores {
    pub schema_version: u32,
    pub version: String,
    pub sites_url: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub notas: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusAtualizacaoConectores {
    pub versao_atual: String,
    pub versao_remota: Option<String>,
    pub atualizacao_disponivel: bool,
    pub notas: Option<String>,
}

pub struct GerenciadorConectores;

/// Entrada completa para aprendizado profundo a partir de uma sessão na Janela Assistida.
pub struct EntradaAprendizadoSessao<'a> {
    pub url_obra: &'a str,
    pub html_obra: Option<&'a str>,
    pub url_capitulo: &'a str,
    pub html_capitulo: Option<&'a str>,
}

fn eh_segmento_numerico(seg: &str) -> bool {
    let s = seg.trim();
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        && s.chars().any(|c| c.is_ascii_digit())
}

fn extrair_seletor_link_do_html(html_obra: &str, path_cap: &str) -> Option<String> {
    if path_cap.trim().is_empty() {
        return None;
    }
    let lower = html_obra.to_lowercase();
    let path_lower = path_cap.trim_end_matches('/').to_lowercase();
    let mut offset = 0usize;
    while let Some(pos) = lower[offset..].find("<a") {
        let abs = offset + pos;
        let Some(end_rel) = lower[abs..].find('>') else {
            break;
        };
        let tag = &html_obra[abs..=abs + end_rel];
        offset = abs + end_rel + 1;

        let tag_lower = tag.to_lowercase();
        if !tag_lower.contains(&path_lower) {
            continue;
        }
        if let Some(idx_cls) = tag_lower.find("class=") {
            let resto = &tag[idx_cls + 6..];
            let quote = resto.chars().next()?;
            if quote == '"' || quote == '\'' {
                if let Some(fim) = resto[1..].find(quote) {
                    let classes = resto[1..1 + fim]
                        .split_whitespace()
                        .filter(|c| !matches!(*c, "btn" | "button" | "active" | "visited" | "link"))
                        .collect::<Vec<_>>();
                    if let Some(primeira) = classes.first() {
                        return Some(format!("a.{}", primeira));
                    }
                }
            }
        }
    }
    None
}

fn extrair_seletores_imagem_do_html(html_cap: &str) -> Vec<String> {
    let lower = html_cap.to_lowercase();
    let candidatos = [
        ("data-url", "img[data-url]"),
        ("data-original", "img[data-original]"),
        ("data-lazy-src", "img[data-lazy-src]"),
        ("data-src", "img[data-src]"),
        ("src", "img[src]"),
    ];
    let mut contagens: Vec<(usize, &str)> = candidatos
        .iter()
        .map(|(attr, sel)| {
            let needle = format!("{}=\"http", attr);
            let needle2 = format!("{}='http", attr);
            let count = lower.matches(&needle).count() + lower.matches(&needle2).count();
            (count, *sel)
        })
        .filter(|(c, _)| *c > 0)
        .collect();
    contagens.sort_by(|a, b| b.0.cmp(&a.0));
    contagens.into_iter().map(|(_, s)| s.to_string()).collect()
}

impl GerenciadorConectores {
    /// Caminho do arquivo de conectores atualizado pelo usuário dentro da biblioteca (`System/connectors/sites.json`).
    pub fn caminho_usuario(raiz_biblioteca: &Path) -> PathBuf {
        raiz_biblioteca
            .join("System")
            .join("connectors")
            .join("sites.json")
    }

    /// Carrega o catálogo ativo respeitando a precedência:
    /// `System/connectors/sites.json` > `<exe_dir>/connectors/default_sites.json` > embutido.
    pub fn carregar(raiz_biblioteca: Option<&Path>) -> CatalogoConectores {
        if let Some(raiz) = raiz_biblioteca {
            let user_path = Self::caminho_usuario(raiz);
            if let Ok(raw) = std::fs::read_to_string(&user_path) {
                if let Ok(cat) = Self::validar_catalogo_json(&raw) {
                    return cat;
                }
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let pkg_path = dir.join("connectors").join("default_sites.json");
                if let Ok(raw) = std::fs::read_to_string(&pkg_path) {
                    if let Ok(cat) = Self::validar_catalogo_json(&raw) {
                        return cat;
                    }
                }
            }
        }

        Self::validar_catalogo_json(DEFAULT_SITES_EMBEDDED).expect("default_sites.json embutido válido")
    }

    /// Valida estrutura JSON do catálogo de conectores.
    pub fn validar_catalogo_json(raw: &str) -> Result<CatalogoConectores, SourceError> {
        let cat: CatalogoConectores = serde_json::from_str(raw)
            .map_err(|e| SourceError::Operacao(format!("JSON de conectores inválido: {}", e)))?;
        if cat.schema_version != 1 {
            return Err(SourceError::Operacao(format!(
                "Versão de schema não suportada: {}",
                cat.schema_version
            )));
        }
        if cat.sites.is_empty() {
            return Err(SourceError::Operacao(
                "Catálogo de conectores não pode estar vazio".to_string(),
            ));
        }
        let mut ids_vistos = std::collections::HashSet::new();
        for s in &cat.sites {
            if s.id.trim().is_empty() || s.nome.trim().is_empty() {
                return Err(SourceError::Operacao(
                    "Cada conector precisa de `id` e `nome` válidos".to_string(),
                ));
            }
            let id_norm = s.id.trim().to_ascii_lowercase();
            if !ids_vistos.insert(id_norm) {
                return Err(SourceError::Operacao(format!(
                    "Conector duplicado no catálogo: `{}`",
                    s.id
                )));
            }
        }
        Ok(cat)
    }

    /// Grava atomicamente um catálogo validado em `<MAM_LIBRARY>/System/connectors/sites.json`.
    pub fn salvar_catalogo_atomico(
        raiz_biblioteca: &Path,
        raw_json: &str,
        sha256_esperado: Option<&str>,
    ) -> Result<CatalogoConectores, SourceError> {
        if let Some(esperado) = sha256_esperado.filter(|s| !s.trim().is_empty()) {
            let mut hasher = Sha256::new();
            hasher.update(raw_json.as_bytes());
            let calculado = format!("{:x}", hasher.finalize());
            if !calculado.eq_ignore_ascii_case(esperado.trim()) {
                return Err(SourceError::Operacao(format!(
                    "Hash SHA-256 divergente na atualização de conectores (esperado {}, obtido {})",
                    esperado, calculado
                )));
            }
        }

        let cat = Self::validar_catalogo_json(raw_json)?;
        let destino = Self::caminho_usuario(raiz_biblioteca);
        if let Some(pai) = destino.parent() {
            std::fs::create_dir_all(pai).map_err(|e| SourceError::Operacao(e.to_string()))?;
        }
        let temp = destino.with_extension("json.mam_tmp");
        std::fs::write(&temp, raw_json.as_bytes())
            .map_err(|e| SourceError::Operacao(e.to_string()))?;
        if destino.exists() {
            let _ = std::fs::remove_file(&destino);
        }
        std::fs::rename(&temp, &destino).map_err(|e| SourceError::Operacao(e.to_string()))?;
        Ok(cat)
    }

    /// Verifica no manifesto remoto se existe uma versão mais nova de conectores.
    pub async fn verificar_atualizacao_remota(
        raiz_biblioteca: Option<&Path>,
        manifest_url: Option<&str>,
    ) -> Result<StatusAtualizacaoConectores, SourceError> {
        let atual = Self::carregar(raiz_biblioteca);
        let url = manifest_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(URL_MANIFESTO_OFICIAL);

        let client = reqwest::Client::builder()
            .user_agent("MAM-Updater/0.1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao consultar manifesto de conectores",
                resp.status()
            )));
        }
        let manifesto: ManifestoConectores = resp.json().await?;
        let disponivel = manifesto.version.trim() != atual.version.trim();

        Ok(StatusAtualizacaoConectores {
            versao_atual: atual.version,
            versao_remota: Some(manifesto.version),
            atualizacao_disponivel: disponivel,
            notas: manifesto.notas,
        })
    }

    /// Baixa o catálogo do manifesto remoto, valida schema + SHA-256 e salva atomicamente.
    pub async fn atualizar_remoto(
        raiz_biblioteca: &Path,
        manifest_url: Option<&str>,
    ) -> Result<CatalogoConectores, SourceError> {
        let url = manifest_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(URL_MANIFESTO_OFICIAL);

        let client = reqwest::Client::builder()
            .user_agent("MAM-Updater/0.1.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let resp_man = client.get(url).send().await?;
        if !resp_man.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao baixar manifesto de conectores",
                resp_man.status()
            )));
        }
        let manifesto: ManifestoConectores = resp_man.json().await?;

        let resp_sites = client.get(&manifesto.sites_url).send().await?;
        if !resp_sites.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "HTTP {} ao baixar catálogo de conectores",
                resp_sites.status()
            )));
        }
        let raw_sites = resp_sites.text().await?;
        Self::salvar_catalogo_atomico(raiz_biblioteca, &raw_sites, manifesto.sha256.as_deref())
    }

    /// Encontra todos os padrões de URL de capítulo configurados para um domínio (ou todos os conectores).
    pub fn padroes_capitulo_ativos(raiz_biblioteca: Option<&Path>) -> Vec<String> {
        let cat = Self::carregar(raiz_biblioteca);
        let mut out = Vec::new();
        for site in cat.sites {
            for p in site.padroes_url_capitulo {
                let norm = p.trim().to_lowercase();
                if !norm.is_empty() && !out.contains(&norm) {
                    out.push(norm);
                }
            }
        }
        out
    }

    /// Busca obras usando um conector web com `url_busca` via `ai-engine` (CDP).
    pub async fn buscar_obras_via_cdp(
        conector: &ConectorSiteConfig,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let template = conector.url_busca.as_deref().ok_or_else(|| {
            SourceError::Operacao(format!(
                "Conector '{}' não possui mecanismo de busca por título configurado",
                conector.nome
            ))
        })?;
        let query_enc: String = consulta
            .trim()
            .bytes()
            .flat_map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    vec![b as char]
                }
                b' ' => vec!['+'],
                _ => format!("%{:02X}", b).chars().collect(),
            })
            .collect();
        let url_busca = template.replace("{query}", &query_enc);

        let py = mam_ai_engine::AiEngineManager::detectar_executavel_python();
        let dir = mam_ai_engine::AiEngineManager::detectar_diretorio_padrao();
        let manager = mam_ai_engine::AiEngineManager::nova(py, dir)
            .com_timeout(std::time::Duration::from_secs(45));
        let val = manager
            .chamar(
                "smart_search_works",
                serde_json::json!({
                    "url_busca": url_busca,
                    "conector_id": conector.id,
                    "dominios": conector.dominios,
                }),
            )
            .await
            .map_err(|e| SourceError::Operacao(e.to_string()))?;

        let obras = serde_json::from_value::<Vec<ObraBuscaResultado>>(
            val.get("obras").cloned().unwrap_or(serde_json::json!([])),
        )
        .map_err(|e| SourceError::Operacao(e.to_string()))?;
        Ok(obras)
    }

    /// Retorna os headers HTTP extras (ex.: `Referer`) configurados para o domínio da URL.
    pub fn headers_para_url(url: &str, raiz_biblioteca: Option<&Path>) -> HashMap<String, String> {
        let lower = url.to_lowercase();
        let catalogo = Self::carregar(raiz_biblioteca);
        for site in catalogo.sites {
            if site.dominios.iter().any(|d| !d.is_empty() && lower.contains(&d.to_lowercase())) {
                if let Some(h) = site.headers_imagem {
                    return h;
                }
            }
        }
        HashMap::new()
    }

    /// Aprende o padrão de URL de capítulo comparando a página da obra com a página de um capítulo
    /// navegada pelo usuário na Janela Assistida, persistindo em `System/connectors/sites.json`.
    pub fn aprender_padrao_url(
        raiz_biblioteca: &Path,
        url_obra: &str,
        url_capitulo: &str,
    ) -> Result<ConectorSiteConfig, SourceError> {
        Self::aprender_de_sessao(
            raiz_biblioteca,
            EntradaAprendizadoSessao {
                url_obra,
                html_obra: None,
                url_capitulo,
                html_capitulo: None,
            },
        )
    }

    /// Aprendizado profundo a partir de URLs e DOMs da obra e do capítulo na Janela Assistida.
    /// Atualiza `System/connectors/sites.json` e alimenta a memória progressiva (`ai_memory.json`).
    pub fn aprender_de_sessao(
        raiz_biblioteca: &Path,
        entrada: EntradaAprendizadoSessao<'_>,
    ) -> Result<ConectorSiteConfig, SourceError> {
        let url_obra = entrada.url_obra;
        let url_capitulo = entrada.url_capitulo;
        let extrair_host = |u: &str| -> String {
            u.split("://")
                .nth(1)
                .unwrap_or(u)
                .split('/')
                .next()
                .unwrap_or("")
                .trim_start_matches("www.")
                .to_lowercase()
        };
        let extrair_path = |u: &str| -> String {
            let sem_q = u.split('?').next().unwrap_or(u).split('#').next().unwrap_or(u);
            if let Some(after) = sem_q.split("://").nth(1) {
                if let Some(idx) = after.find('/') {
                    return after[idx..].to_string();
                }
            }
            String::new()
        };

        let host = extrair_host(url_capitulo);
        let host = if host.is_empty() { extrair_host(url_obra) } else { host };
        if host.is_empty() {
            return Err(SourceError::Operacao("Domínio inválido para aprendizado".to_string()));
        }

        let path_obra = extrair_path(url_obra);
        let path_cap = extrair_path(url_capitulo);
        let query_cap = url_capitulo.split('?').nth(1).unwrap_or("");
        let padrao_query = ["episode_no=", "chapter_no=", "chapter_id=", "chapter=", "chap=", "ep="]
            .iter()
            .find(|k| query_cap.to_lowercase().contains(*k))
            .map(|k| (*k).to_string());

        let padrao = if path_cap.trim_end_matches('/') != path_obra.trim_end_matches('/') && !path_cap.is_empty() {
            let segs_obra: Vec<&str> = path_obra.split('/').filter(|s| !s.is_empty()).collect();
            let segs_cap: Vec<&str> = path_cap.split('/').filter(|s| !s.is_empty()).collect();
            let mut comum = 0usize;
            while comum < segs_obra.len() && comum < segs_cap.len() && segs_obra[comum] == segs_cap[comum] {
                comum += 1;
            }
            if comum < segs_cap.len() && segs_cap.len() >= 2 {
                let seg_diff = segs_cap[comum];
                if eh_segmento_numerico(seg_diff) && comum > 0 {
                    format!("/{}/{{num}}", segs_cap[comum - 1])
                } else {
                    format!("/{}/", seg_diff)
                }
            } else if let Some(q) = padrao_query.clone() {
                q
            } else if let Some(primeiro) = segs_cap.first() {
                format!("/{}/", primeiro)
            } else {
                return Err(SourceError::Operacao("Não foi possível inferir padrão da URL do capítulo".to_string()));
            }
        } else if let Some(q) = padrao_query {
            q
        } else {
            return Err(SourceError::Operacao(
                "Navegue até um capítulo dentro da Janela Assistida antes de ensinar o padrão.".to_string(),
            ));
        };

        let seletor_link_aprendido = entrada
            .html_obra
            .and_then(|h| extrair_seletor_link_do_html(h, &path_cap));
        let seletores_img_aprendidos = entrada
            .html_capitulo
            .map(extrair_seletores_imagem_do_html)
            .unwrap_or_default();

        let mut catalogo = Self::carregar(Some(raiz_biblioteca));
        let mut atualizado: Option<ConectorSiteConfig> = None;
        for site in &mut catalogo.sites {
            if site.dominios.iter().any(|d| d.trim_start_matches("www.").eq_ignore_ascii_case(&host)) {
                if !site.padroes_url_capitulo.contains(&padrao) {
                    site.padroes_url_capitulo.insert(0, padrao.clone());
                }
                if site.seletor_link.is_none() {
                    site.seletor_link = seletor_link_aprendido.clone();
                }
                for sel in seletores_img_aprendidos.iter().rev() {
                    if !site.seletores_imagem.contains(sel) {
                        site.seletores_imagem.insert(0, sel.clone());
                    }
                }
                atualizado = Some(site.clone());
                break;
            }
        }

        let conector_final = match atualizado {
            Some(c) => c,
            None => {
                let id_limpo: String = host
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                    .collect();
                let mut seletores_img = seletores_img_aprendidos.clone();
                for pad in ["img[data-url]", "img[data-original]", "img[data-src]", "img[src]"] {
                    if !seletores_img.iter().any(|s| s == pad) {
                        seletores_img.push(pad.to_string());
                    }
                }
                let novo = ConectorSiteConfig {
                    id: format!("learned-{}", id_limpo.trim_matches('-')),
                    nome: format!("{} (Aprendido)", host),
                    versao: "1.0.0".to_string(),
                    tipo: "web-cdp".to_string(),
                    dominios: vec![host.clone()],
                    url_busca: None,
                    api_busca: None,
                    padroes_url_capitulo: vec![padrao.clone()],
                    seletor_link: seletor_link_aprendido.clone(),
                    seletor_idioma_linha: None,
                    seletor_paginacao: None,
                    requer_cdp: "auto".to_string(),
                    seletores_imagem: seletores_img,
                    rejeitar_tokens_imagem: vec!["logo".to_string(), "avatar".to_string(), "favicon".to_string()],
                    headers_imagem: Some(HashMap::from([("Referer".to_string(), format!("https://{}/", host))])),
                    estrategia_descoberta: Some("auto".to_string()),
                };
                catalogo.sites.push(novo.clone());
                novo
            }
        };

        let json = serde_json::to_string_pretty(&catalogo)
            .map_err(|e| SourceError::Operacao(e.to_string()))?;
        Self::salvar_catalogo_atomico(raiz_biblioteca, &json, None)?;

        // Reforça memória progressiva cross-site
        let _ = crate::ai_memory::GerenciadorMemoriaIA::registrar_sucesso(
            raiz_biblioteca,
            crate::ai_memory::TipoPadraoMemoria::UrlCapitulo,
            &padrao,
            Some(&host),
        );
        if let Some(ref sel) = seletor_link_aprendido {
            let _ = crate::ai_memory::GerenciadorMemoriaIA::registrar_sucesso(
                raiz_biblioteca,
                crate::ai_memory::TipoPadraoMemoria::SeletorLink,
                sel,
                Some(&host),
            );
        }
        for sel in &seletores_img_aprendidos {
            let _ = crate::ai_memory::GerenciadorMemoriaIA::registrar_sucesso(
                raiz_biblioteca,
                crate::ai_memory::TipoPadraoMemoria::SeletorImagem,
                sel,
                Some(&host),
            );
        }

        Ok(conector_final)
    }

    /// Persiste uma configuração `api_busca` sintetizada em um conector já existente em `sites.json`.
    pub fn salvar_api_busca_conector(
        raiz_biblioteca: &Path,
        conector_id: &str,
        api_busca: ApiBuscaConfig,
    ) -> Result<ConectorSiteConfig, SourceError> {
        let mut catalogo = Self::carregar(Some(raiz_biblioteca));
        let mut atualizado: Option<ConectorSiteConfig> = None;
        for site in &mut catalogo.sites {
            if site.id.eq_ignore_ascii_case(conector_id) {
                site.api_busca = Some(api_busca.clone());
                if site.url_busca.is_none() {
                    site.url_busca = Some(api_busca.endpoint.clone());
                }
                atualizado = Some(site.clone());
                break;
            }
        }
        let conector_final = atualizado.ok_or_else(|| {
            SourceError::Operacao(format!("Conector '{}' não encontrado para salvar API", conector_id))
        })?;
        let json = serde_json::to_string_pretty(&catalogo)
            .map_err(|e| SourceError::Operacao(e.to_string()))?;
        Self::salvar_catalogo_atomico(raiz_biblioteca, &json, None)?;
        Ok(conector_final)
    }

    /// Solicita ao `ai-engine` a síntese automática de `ApiBuscaConfig` para um conector.
    pub async fn sintetizar_api_busca_via_ai(
        raiz_biblioteca: &Path,
        conector: &ConectorSiteConfig,
        consulta: &str,
    ) -> Result<Option<ApiBuscaConfig>, SourceError> {
        let dominio = conector
            .dominios
            .first()
            .map(|d| d.trim())
            .filter(|d| !d.is_empty())
            .ok_or_else(|| SourceError::Operacao("Conector sem domínio para síntese".to_string()))?;
        let base_url = if dominio.starts_with("http") {
            dominio.to_string()
        } else {
            format!("https://{}/", dominio)
        };

        let py = mam_ai_engine::AiEngineManager::detectar_executavel_python();
        let dir = mam_ai_engine::AiEngineManager::detectar_diretorio_padrao();
        let manager = mam_ai_engine::AiEngineManager::nova(py, dir)
            .com_timeout(std::time::Duration::from_secs(40));
        let val = manager
            .chamar(
                "synthesize_search_api",
                serde_json::json!({
                    "base_url": base_url,
                    "query": consulta,
                }),
            )
            .await
            .map_err(|e| SourceError::Operacao(e.to_string()))?;

        let api_opt = val
            .get("api_busca")
            .and_then(|v| if v.is_null() { None } else { serde_json::from_value::<ApiBuscaConfig>(v.clone()).ok() });

        if let Some(ref api) = api_opt {
            let _ = Self::salvar_api_busca_conector(raiz_biblioteca, &conector.id, api.clone());
        }
        Ok(api_opt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedencia_conectores_usuario_sobrepoe_padrao() {
        let base = std::env::temp_dir().join(format!("mam_conn_prec_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        let padrao = GerenciadorConectores::carregar(Some(&base));
        assert_eq!(padrao.version, "2026.11.24.1");

        let custom_json = r#"{
            "schema_version": 1,
            "version": "2026.11.22.9",
            "sites": [{
                "id": "novosite",
                "nome": "Novo Site Exemplo",
                "dominios": ["novosite.example"],
                "padroes_url_capitulo": ["/leitura/cap-"]
            }]
        }"#;
        let salvo = GerenciadorConectores::salvar_catalogo_atomico(&base, custom_json, None).unwrap();
        assert_eq!(salvo.version, "2026.11.22.9");

        let carregado = GerenciadorConectores::carregar(Some(&base));
        assert_eq!(carregado.version, "2026.11.22.9");
        assert_eq!(carregado.sites[0].id, "novosite");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn conector_dinamico_adiciona_suporte_a_site_novo_sem_recompilar() {
        let base = std::env::temp_dir().join(format!("mam_conn_dyn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        let custom_json = r#"{
            "schema_version": 1,
            "version": "2026.11.22.1",
            "sites": [{
                "id": "novosite",
                "nome": "Novo Site",
                "dominios": ["novosite.example"],
                "padroes_url_capitulo": ["/leitura/cap-"]
            }]
        }"#;
        GerenciadorConectores::salvar_catalogo_atomico(&base, custom_json, None).unwrap();

        let padroes = GerenciadorConectores::padroes_capitulo_ativos(Some(&base));
        assert!(padroes.contains(&"/leitura/cap-".to_string()));

        let html = r#"
            <html><head><title>Obra Nova</title></head><body>
              <a href="/obra/1/leitura/cap-05">Capítulo 5</a>
              <a href="/obra/1/leitura/cap-06">Capítulo 6</a>
            </body></html>
        "#;
        let res = crate::extractors::extrair_capitulos_de_html_com_padroes(
            "https://novosite.example/obra/1",
            html,
            &padroes,
        );
        assert_eq!(res.capitulos.len(), 2);
        assert_eq!(res.capitulos[0].numero, Some(5));
        assert_eq!(res.capitulos[1].numero, Some(6));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn atualizacao_conectores_preserva_biblioteca_e_faz_rollback_em_falha() {
        let base = std::env::temp_dir().join(format!("mam_conn_rb_{}", uuid::Uuid::new_v4()));
        let cbz = base.join("Manga/Obra Existente/Capitulo_0001.cbz");
        std::fs::create_dir_all(cbz.parent().unwrap()).unwrap();
        std::fs::write(&cbz, b"CBZ-ORIGINAL").unwrap();

        let v1 = r#"{
            "schema_version": 1,
            "version": "2026.11.21.1",
            "sites": [{"id": "site-v1", "nome": "Site V1", "dominios": ["v1.example"], "padroes_url_capitulo": ["/cap-"]}]
        }"#;
        GerenciadorConectores::salvar_catalogo_atomico(&base, v1, None).unwrap();

        // Tentativa de atualização com SHA-256 inválido deve falhar e preservar v1 + biblioteca
        let err = GerenciadorConectores::salvar_catalogo_atomico(
            &base,
            v1,
            Some("0000000000000000000000000000000000000000000000000000000000000000"),
        );
        assert!(err.is_err());

        // Tentativa com JSON inválido também deve preservar v1
        let err_json = GerenciadorConectores::salvar_catalogo_atomico(&base, "{broken", None);
        assert!(err_json.is_err());

        let atual = GerenciadorConectores::carregar(Some(&base));
        assert_eq!(atual.sites[0].id, "site-v1");
        assert_eq!(std::fs::read(&cbz).unwrap(), b"CBZ-ORIGINAL");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn aprender_padrao_url_suporta_path_e_query() {
        let base = std::env::temp_dir().join(format!("mam_conn_learn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        let cfg_path = GerenciadorConectores::aprender_padrao_url(
            &base,
            "https://exemplo.com/manga/minha-obra",
            "https://exemplo.com/manga/minha-obra/leitura/12",
        )
        .unwrap();
        assert_eq!(cfg_path.padroes_url_capitulo[0], "/leitura/");

        let cfg_query = GerenciadorConectores::aprender_padrao_url(
            &base,
            "https://webtoons.example/series/viewer?title_no=10",
            "https://webtoons.example/series/viewer?title_no=10&episode_no=5",
        )
        .unwrap();
        assert_eq!(cfg_query.padroes_url_capitulo[0], "episode_no=");

        let _ = std::fs::remove_dir_all(&base);
    }
}
