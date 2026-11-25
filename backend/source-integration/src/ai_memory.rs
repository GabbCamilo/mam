//! Memória Progressiva Cross-Site da IA de Extração (`System/connectors/ai_memory.json`).
//!
//! Armazena padrões estruturais generalizados (URLs de capítulos, seletores/classes de links
//! e seletores/atributos de imagens de leitura) com pontuação baseada em acertos e falhas,
//! permitindo reutilizar aprendizado entre sites diferentes mesmo sem conector cadastrado.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::SourceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoPadraoMemoria {
    UrlCapitulo,
    SeletorLink,
    SeletorImagem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PadraoMemoria {
    pub valor: String,
    #[serde(default)]
    pub hits: u64,
    #[serde(default)]
    pub misses: u64,
    #[serde(default)]
    pub ultima_confirmacao_ts: u64,
    #[serde(default)]
    pub dominios_origem: Vec<String>,
}

impl PadraoMemoria {
    /// Score de confiança com suavização de Laplace e bônus logarítmico por volume de acertos.
    pub fn score(&self) -> f64 {
        let h = self.hits as f64;
        let m = self.misses as f64;
        let taxa = (h + 1.0) / (h + m + 2.0);
        let bonus_volume = 1.0 + (1.0 + h).ln();
        let bonus_cross_site = 1.0 + (self.dominios_origem.len().min(10) as f64) * 0.08;
        taxa * bonus_volume * bonus_cross_site
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoriaAprendizadoIA {
    #[serde(default = "schema_v1")]
    pub schema_version: u32,
    #[serde(default)]
    pub padroes_url: Vec<PadraoMemoria>,
    #[serde(default)]
    pub seletores_capitulo: Vec<PadraoMemoria>,
    #[serde(default)]
    pub seletores_imagem: Vec<PadraoMemoria>,
}

fn schema_v1() -> u32 {
    1
}

impl Default for MemoriaAprendizadoIA {
    fn default() -> Self {
        Self {
            schema_version: 1,
            padroes_url: Vec::new(),
            seletores_capitulo: Vec::new(),
            seletores_imagem: Vec::new(),
        }
    }
}

pub struct GerenciadorMemoriaIA;

impl GerenciadorMemoriaIA {
    pub fn caminho_arquivo(raiz_biblioteca: &Path) -> PathBuf {
        raiz_biblioteca
            .join("System")
            .join("connectors")
            .join("ai_memory.json")
    }

    pub fn carregar(raiz_biblioteca: Option<&Path>) -> MemoriaAprendizadoIA {
        if let Some(raiz) = raiz_biblioteca {
            let path = Self::caminho_arquivo(raiz);
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(mem) = serde_json::from_str::<MemoriaAprendizadoIA>(&raw) {
                    if mem.schema_version == 1 {
                        return mem;
                    }
                }
            }
        }
        MemoriaAprendizadoIA::default()
    }

    pub fn salvar_atomico(
        raiz_biblioteca: &Path,
        memoria: &MemoriaAprendizadoIA,
    ) -> Result<(), SourceError> {
        let dest = Self::caminho_arquivo(raiz_biblioteca);
        if let Some(pai) = dest.parent() {
            std::fs::create_dir_all(pai).map_err(|e| SourceError::Operacao(e.to_string()))?;
        }
        let json =
            serde_json::to_string_pretty(memoria).map_err(|e| SourceError::Operacao(e.to_string()))?;
        let tmp = dest.with_extension("json.mam_tmp");
        std::fs::write(&tmp, json.as_bytes()).map_err(|e| SourceError::Operacao(e.to_string()))?;
        if dest.exists() {
            let _ = std::fs::remove_file(&dest);
        }
        std::fs::rename(&tmp, &dest).map_err(|e| SourceError::Operacao(e.to_string()))?;
        Ok(())
    }

    fn lista_mut(mem: &mut MemoriaAprendizadoIA, tipo: TipoPadraoMemoria) -> &mut Vec<PadraoMemoria> {
        match tipo {
            TipoPadraoMemoria::UrlCapitulo => &mut mem.padroes_url,
            TipoPadraoMemoria::SeletorLink => &mut mem.seletores_capitulo,
            TipoPadraoMemoria::SeletorImagem => &mut mem.seletores_imagem,
        }
    }

    fn lista_ref(mem: &MemoriaAprendizadoIA, tipo: TipoPadraoMemoria) -> &[PadraoMemoria] {
        match tipo {
            TipoPadraoMemoria::UrlCapitulo => &mem.padroes_url,
            TipoPadraoMemoria::SeletorLink => &mem.seletores_capitulo,
            TipoPadraoMemoria::SeletorImagem => &mem.seletores_imagem,
        }
    }

    /// Registra uso bem-sucedido de um padrão (reforço positivo cross-site).
    pub fn registrar_sucesso(
        raiz_biblioteca: &Path,
        tipo: TipoPadraoMemoria,
        valor: &str,
        dominio: Option<&str>,
    ) -> Result<(), SourceError> {
        let val = valor.trim();
        if val.len() < 3 {
            return Ok(());
        }
        let agora = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut mem = Self::carregar(Some(raiz_biblioteca));
        let lista = Self::lista_mut(&mut mem, tipo);

        if let Some(item) = lista.iter_mut().find(|p| p.valor.eq_ignore_ascii_case(val)) {
            item.hits = item.hits.saturating_add(1);
            item.ultima_confirmacao_ts = agora;
            if let Some(dom) = dominio.map(|d| d.trim().to_lowercase()).filter(|d| !d.is_empty()) {
                if !item.dominios_origem.iter().any(|d| d == &dom) && item.dominios_origem.len() < 16 {
                    item.dominios_origem.push(dom);
                }
            }
        } else {
            let mut doms = Vec::new();
            if let Some(dom) = dominio.map(|d| d.trim().to_lowercase()).filter(|d| !d.is_empty()) {
                doms.push(dom);
            }
            lista.push(PadraoMemoria {
                valor: val.to_string(),
                hits: 1,
                misses: 0,
                ultima_confirmacao_ts: agora,
                dominios_origem: doms,
            });
        }

        // Mantém os 120 padrões mais relevantes por categoria para preservar performance
        lista.sort_by(|a, b| {
            b.score()
                .partial_cmp(&a.score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if lista.len() > 120 {
            lista.truncate(120);
        }

        Self::salvar_atomico(raiz_biblioteca, &mem)
    }

    /// Registra falha de um padrão para reduzir sua prioridade futura.
    pub fn registrar_falha(
        raiz_biblioteca: &Path,
        tipo: TipoPadraoMemoria,
        valor: &str,
    ) -> Result<(), SourceError> {
        let val = valor.trim();
        if val.is_empty() {
            return Ok(());
        }
        let mut mem = Self::carregar(Some(raiz_biblioteca));
        let lista = Self::lista_mut(&mut mem, tipo);
        if let Some(item) = lista.iter_mut().find(|p| p.valor.eq_ignore_ascii_case(val)) {
            item.misses = item.misses.saturating_add(1);
            Self::salvar_atomico(raiz_biblioteca, &mem)?;
        }
        Ok(())
    }

    /// Retorna os padrões ranqueados por confiança (score > 0.45).
    pub fn padroes_ranqueados(
        raiz_biblioteca: Option<&Path>,
        tipo: TipoPadraoMemoria,
        limite: usize,
    ) -> Vec<String> {
        let mem = Self::carregar(raiz_biblioteca);
        let mut itens: Vec<&PadraoMemoria> = Self::lista_ref(&mem, tipo)
            .iter()
            .filter(|p| p.score() >= 0.45)
            .collect();
        itens.sort_by(|a, b| {
            b.score()
                .partial_cmp(&a.score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        itens
            .into_iter()
            .take(limite)
            .map(|p| p.valor.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memoria_progressiva_reforca_e_ordena_cross_site() {
        let base = std::env::temp_dir().join(format!("mam_ai_mem_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        GerenciadorMemoriaIA::registrar_sucesso(
            &base,
            TipoPadraoMemoria::UrlCapitulo,
            "/leitura/{num}",
            Some("sitea.com"),
        )
        .unwrap();
        GerenciadorMemoriaIA::registrar_sucesso(
            &base,
            TipoPadraoMemoria::UrlCapitulo,
            "/leitura/{num}",
            Some("siteb.org"),
        )
        .unwrap();
        GerenciadorMemoriaIA::registrar_sucesso(
            &base,
            TipoPadraoMemoria::UrlCapitulo,
            "/ruim/",
            Some("sitec.net"),
        )
        .unwrap();
        GerenciadorMemoriaIA::registrar_falha(&base, TipoPadraoMemoria::UrlCapitulo, "/ruim/")
            .unwrap();
        GerenciadorMemoriaIA::registrar_falha(&base, TipoPadraoMemoria::UrlCapitulo, "/ruim/")
            .unwrap();

        let ranqueados = GerenciadorMemoriaIA::padroes_ranqueados(
            Some(&base),
            TipoPadraoMemoria::UrlCapitulo,
            10,
        );
        assert_eq!(ranqueados.first().map(String::as_str), Some("/leitura/{num}"));

        let _ = std::fs::remove_dir_all(&base);
    }
}
