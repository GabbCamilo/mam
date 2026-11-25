//! Executor nativo em Rust para APIs declarativas (`api_busca`) sintetizadas ou configuradas
//! nos conectores (`System/connectors/sites.json`).

use std::collections::HashMap;

use serde_json::Value;

use crate::connectors::ApiBuscaConfig;
use crate::{ObraBuscaResultado, SourceError};

pub struct ExecutorApiConector;

fn codificar_query(q: &str) -> String {
    q.trim()
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b as char],
            b' ' => vec!['+'],
            _ => format!("%{:02X}", b).chars().collect(),
        })
        .collect()
}

fn navegar_caminho<'a>(val: &'a Value, caminho: Option<&str>) -> Option<&'a Value> {
    let path = caminho.map(str::trim).unwrap_or("");
    if path.is_empty() {
        return Some(val);
    }
    let mut cur = val;
    for seg in path.split('.') {
        if let Ok(idx) = seg.parse::<usize>() {
            cur = cur.get(idx)?;
        } else {
            cur = cur.get(seg)?;
        }
    }
    Some(cur)
}

fn valor_para_string(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t.to_string()) }
        }
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Mapeia um payload JSON já carregado usando a configuração declarativa `ApiBuscaConfig`.
pub fn mapear_json_para_obras(
    conector_id: &str,
    cfg: &ApiBuscaConfig,
    json: &Value,
) -> Result<Vec<ObraBuscaResultado>, SourceError> {
    let lista_val = navegar_caminho(json, cfg.caminho_itens.as_deref()).ok_or_else(|| {
        SourceError::Operacao("Caminho de itens não encontrado na resposta da API".to_string())
    })?;
    let arr = lista_val.as_array().ok_or_else(|| {
        SourceError::Operacao("Resposta da API não contém um array de obras".to_string())
    })?;

    let mut saida = Vec::new();
    for item in arr.iter().take(25) {
        let Some(titulo) = navegar_caminho(item, Some(&cfg.campo_titulo)).and_then(valor_para_string)
        else {
            continue;
        };
        let Some(id_ou_url) =
            navegar_caminho(item, Some(&cfg.campo_id_ou_url)).and_then(valor_para_string)
        else {
            continue;
        };

        let obra_id_fonte = if let Some(ref tpl) = cfg.template_url_obra {
            tpl.replace("{id}", &id_ou_url)
                .replace("{slug}", &id_ou_url)
                .replace("{val}", &id_ou_url)
        } else {
            id_ou_url
        };

        let capa_url = cfg
            .campo_capa
            .as_deref()
            .and_then(|c| navegar_caminho(item, Some(c)))
            .and_then(valor_para_string)
            .map(|raw| {
                if raw.starts_with("http") {
                    raw
                } else if let Some(ref tpl) = cfg.template_url_capa {
                    tpl.replace("{val}", raw.trim_start_matches('/'))
                        .replace("{path}", raw.trim_start_matches('/'))
                } else {
                    raw
                }
            });

        saida.push(ObraBuscaResultado {
            fonte_id: conector_id.to_string(),
            obra_id_fonte,
            titulo,
            autor: None,
            descricao: None,
            capa_url,
        });
    }
    Ok(saida)
}

impl ExecutorApiConector {
    /// Executa uma busca diretamente contra o endpoint JSON configurado em `ApiBuscaConfig`.
    pub async fn buscar(
        conector_id: &str,
        cfg: &ApiBuscaConfig,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let q = consulta.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let q_enc = codificar_query(q);
        let url = cfg.endpoint.replace("{query}", &q_enc);

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let metodo = cfg.metodo.trim().to_uppercase();
        let mut req = if metodo == "POST" {
            client.post(&url)
        } else {
            client.get(&url)
        };

        let headers: &HashMap<String, String> = cfg.headers.as_ref().unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(HashMap::new)
        });
        for (k, v) in headers {
            req = req.header(k.as_str(), v.as_str());
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Operacao(format!(
                "API do conector '{}' retornou HTTP {}",
                conector_id,
                resp.status()
            )));
        }

        let json: Value = resp.json().await?;
        mapear_json_para_obras(conector_id, cfg, &json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapeia_payload_json_aninhado_com_templates() {
        let cfg = ApiBuscaConfig {
            endpoint: "https://exemplo.org/api/search?q={query}".to_string(),
            metodo: "GET".to_string(),
            formato: "json".to_string(),
            caminho_itens: Some("data.items".to_string()),
            campo_id_ou_url: "slug".to_string(),
            template_url_obra: Some("https://exemplo.org/manga/{slug}".to_string()),
            campo_titulo: "meta.title".to_string(),
            campo_capa: Some("cover".to_string()),
            template_url_capa: Some("https://cdn.exemplo.org/{path}".to_string()),
            headers: None,
        };

        let payload = serde_json::json!({
            "data": {
                "items": [
                    {
                        "slug": "medaka-box",
                        "meta": { "title": "Medaka Box" },
                        "cover": "/covers/mb.webp"
                    }
                ]
            }
        });

        let obras = mapear_json_para_obras("exemplo", &cfg, &payload).unwrap();
        assert_eq!(obras.len(), 1);
        assert_eq!(obras[0].fonte_id, "exemplo");
        assert_eq!(obras[0].titulo, "Medaka Box");
        assert_eq!(obras[0].obra_id_fonte, "https://exemplo.org/manga/medaka-box");
        assert_eq!(
            obras[0].capa_url.as_deref(),
            Some("https://cdn.exemplo.org/covers/mb.webp")
        );
    }
}
