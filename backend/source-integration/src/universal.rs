//! Orquestrador **Conector Universal** gerenciado pela IA interna portátil.
//!
//! Unifica todos os conectores declarativos (`sites.json` / `default_sites.json`) e a memória
//! progressiva (`ai_memory.json`) para correlacionar qualquer URL importada ao melhor caminho
//! de descoberta e captura de capítulos.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ai_memory::{GerenciadorMemoriaIA, TipoPadraoMemoria};
use crate::connectors::{ConectorSiteConfig, GerenciadorConectores};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotaCorrelacionada {
    pub conector_id: String,
    pub conector_nome: String,
    pub origem_correlacao: String,
    pub padroes_url: Vec<String>,
    pub seletores_imagem: Vec<String>,
}

pub struct ConectorUniversal;

impl ConectorUniversal {
    fn extrair_host(url: &str) -> String {
        url.split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or("")
            .trim_start_matches("www.")
            .to_lowercase()
    }

    /// Correlaciona uma URL importada com todos os conectores registrados e com a memória
    /// progressiva portátil (`ai_memory.json`), retornando padrões e seletores unificados.
    pub fn correlacionar_link(url: &str, raiz_biblioteca: Option<&Path>) -> RotaCorrelacionada {
        let host = Self::extrair_host(url);
        let catalogo = GerenciadorConectores::carregar(raiz_biblioteca);

        let mut conector_match: Option<ConectorSiteConfig> = None;
        if !host.is_empty() {
            for site in catalogo.sites {
                if site.dominios.iter().any(|d| {
                    let dom = d.trim().trim_start_matches("www.").to_lowercase();
                    !dom.is_empty() && (host == dom || host.ends_with(&format!(".{}", dom)))
                }) {
                    conector_match = Some(site);
                    break;
                }
            }
        }

        let mem_urls = GerenciadorMemoriaIA::padroes_ranqueados(
            raiz_biblioteca,
            TipoPadraoMemoria::UrlCapitulo,
            40,
        );
        let mem_imgs = GerenciadorMemoriaIA::padroes_ranqueados(
            raiz_biblioteca,
            TipoPadraoMemoria::SeletorImagem,
            20,
        );

        let mut padroes_url = Vec::new();
        let mut seletores_imagem = Vec::new();

        if let Some(ref c) = conector_match {
            for p in &c.padroes_url_capitulo {
                if !padroes_url.contains(p) {
                    padroes_url.push(p.clone());
                }
            }
            for s in &c.seletores_imagem {
                if !seletores_imagem.contains(s) {
                    seletores_imagem.push(s.clone());
                }
            }
        }

        // Complementa com padrões cross-site da memória progressiva
        for p in mem_urls {
            if !padroes_url.contains(&p) {
                padroes_url.push(p);
            }
        }
        for s in mem_imgs {
            if !seletores_imagem.contains(&s) {
                seletores_imagem.push(s);
            }
        }

        match conector_match {
            Some(c) => RotaCorrelacionada {
                conector_id: c.id,
                conector_nome: c.nome,
                origem_correlacao: "dominio-registrado".to_string(),
                padroes_url,
                seletores_imagem,
            },
            None if !padroes_url.is_empty() => RotaCorrelacionada {
                conector_id: "universal-memoria".to_string(),
                conector_nome: "Universal (Memória IA)".to_string(),
                origem_correlacao: "memoria-progressiva".to_string(),
                padroes_url,
                seletores_imagem,
            },
            None => RotaCorrelacionada {
                conector_id: "universal-heuristica".to_string(),
                conector_nome: "Universal (IA Estrutural)".to_string(),
                origem_correlacao: "ai-engine".to_string(),
                padroes_url: GerenciadorConectores::padroes_capitulo_ativos(raiz_biblioteca),
                seletores_imagem,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlaciona_dominio_conhecido_e_desconhecido_com_memoria() {
        let base = std::env::temp_dir().join(format!("mam_uni_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        GerenciadorMemoriaIA::registrar_sucesso(
            &base,
            TipoPadraoMemoria::UrlCapitulo,
            "/leitura/{num}",
            Some("site-novo.org"),
        )
        .unwrap();

        let rota_conhecida =
            ConectorUniversal::correlacionar_link("https://mangafire.to/manga/medaka", Some(&base));
        assert_eq!(rota_conhecida.conector_id, "mangafire");
        assert_eq!(rota_conhecida.origem_correlacao, "dominio-registrado");
        assert!(rota_conhecida.padroes_url.contains(&"/leitura/{num}".to_string()));

        let rota_inedita =
            ConectorUniversal::correlacionar_link("https://desconhecido.xyz/obra/1", Some(&base));
        assert_eq!(rota_inedita.origem_correlacao, "memoria-progressiva");
        assert!(rota_inedita.padroes_url.contains(&"/leitura/{num}".to_string()));

        let _ = std::fs::remove_dir_all(&base);
    }
}
