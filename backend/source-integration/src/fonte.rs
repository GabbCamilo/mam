//! Trait Fonte e tipos relacionados para integração com fontes autorizadas.
//!
//! Cada fonte de conteúdo implementa este trait. Somente fontes marcadas
//! como autorizadas permitem arquivamento. Não há mecanismos para contornar
//! DRM, paywalls ou controles de acesso.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Erros de fonte.
#[derive(Debug, thiserror::Error)]
pub enum FonteError {
    #[error("Erro de rede: {0}")]
    Rede(String),
    #[error("Obra não encontrada: {0}")]
    ObraNaoEncontrada(String),
    #[error("Capítulo não encontrado: {0}")]
    CapituloNaoEncontrado(String),
    #[error("Fonte não autorizada")]
    NaoAutorizada,
    #[error("Erro desconhecido: {0}")]
    Desconhecido(String),
}

/// Obra encontrada em uma fonte externa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObraEncontrada {
    pub obra_id_fonte: String,
    pub titulo: String,
    pub autor: Option<String>,
    pub descricao: Option<String>,
}

/// Capítulo encontrado em uma fonte externa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapituloEncontrado {
    pub capitulo_id_fonte: String,
    pub numero: Option<i32>,
    pub titulo: Option<String>,
    pub url: Option<String>,
}

/// Informação de uma fonte cadastrada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FonteRegistrada {
    pub id: String,
    pub nome: String,
    pub url_base: Option<String>,
    pub autorizada: bool,
    pub ultima_verificacao: Option<String>,
}

/// Trait que toda fonte de conteúdo deve implementar.
///
/// Implementações devem respeitar estritamente as autorizações de acesso
/// da fonte. Não implementar mecanismos para contornar DRM, paywalls,
/// autenticação ou controles de acesso.
#[async_trait]
pub trait Fonte: Send + Sync {
    /// Identificador único da fonte.
    fn id(&self) -> &str;

    /// Nome exibível da fonte.
    fn nome(&self) -> &str;

    /// Indica se a fonte está autorizada para arquivamento.
    fn eh_autorizada(&self) -> bool;

    /// Busca obras por consulta de texto.
    async fn buscar_obras(
        &self,
        consulta: &str,
    ) -> Result<Vec<ObraEncontrada>, FonteError>;

    /// Lista capítulos disponíveis para uma obra.
    async fn listar_capitulos(
        &self,
        obra_id_fonte: &str,
    ) -> Result<Vec<CapituloEncontrado>, FonteError>;

    /// Obtém o conteúdo bruto de um capítulo (para posterior criação de CBZ).
    async fn obter_capitulo(
        &self,
        capitulo_id_fonte: &str,
    ) -> Result<Vec<u8>, FonteError>;

    /// Verifica se a fonte ainda está acessível (healthcheck).
    async fn health_check(&self) -> bool {
        let _ = self.buscar_obras("").await;
        true
    }
}

/// Gerencia o registro de fontes no sistema.
pub struct GerenciadorFontes {
    fontes: Vec<Box<dyn Fonte>>,
}

impl GerenciadorFontes {
    pub fn nova() -> Self {
        Self { fontes: Vec::new() }
    }

    pub fn registrar(&mut self, fonte: Box<dyn Fonte>) {
        self.fontes.push(fonte);
    }

    pub fn listar(&self) -> Vec<FonteRegistrada> {
        self.fontes
            .iter()
            .map(|f| FonteRegistrada {
                id: f.id().to_string(),
                nome: f.nome().to_string(),
                url_base: None,
                autorizada: f.eh_autorizada(),
                ultima_verificacao: None,
            })
            .collect()
    }

    pub fn fontes_autorizadas(&self) -> Vec<&dyn Fonte> {
        self.fontes.iter().filter(|f| f.eh_autorizada()).map(|f| f.as_ref()).collect()
    }
}

impl Default for GerenciadorFontes {
    fn default() -> Self {
        Self::nova()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fonte fictícia para testes (autorizada).
    struct FonteTeste;

    impl Fonte for FonteTeste {
        fn id(&self) -> &str {
            "fonte-teste"
        }

        fn nome(&self) -> &str {
            "Fonte de Teste"
        }

        fn eh_autorizada(&self) -> bool {
            true
        }

        async fn buscar_obras(
            &self,
            consulta: &str,
        ) -> Result<Vec<ObraEncontrada>, FonteError> {
            if consulta.is_empty() {
                return Ok(vec![]);
            }
            Ok(vec![ObraEncontrada {
                obra_id_fonte: "obra-1".to_string(),
                titulo: consulta.to_string(),
                autor: Some("Autor Teste".to_string()),
                descricao: Some("Descrição de teste".to_string()),
            }])
        }

        async fn listar_capitulos(
            &self,
            obra_id_fonte: &str,
        ) -> Result<Vec<CapituloEncontrado>, FonteError> {
            if obra_id_fonte != "obra-1" {
                return Err(FonteError::ObraNaoEncontrada(obra_id_fonte.to_string()));
            }
            Ok(vec![
                CapituloEncontrado {
                    capitulo_id_fonte: "cap-1".to_string(),
                    numero: Some(1),
                    titulo: Some("Capítulo 1".to_string()),
                    url: Some("https://exemplo.com/cap1".to_string()),
                },
                CapituloEncontrado {
                    capitulo_id_fonte: "cap-2".to_string(),
                    numero: Some(2),
                    titulo: Some("Capítulo 2".to_string()),
                    url: Some("https://exemplo.com/cap2".to_string()),
                },
            ])
        }

        async fn obter_capitulo(
            &self,
            _capitulo_id_fonte: &str,
        ) -> Result<Vec<u8>, FonteError> {
            Err(FonteError::NaoAutorizada)
        }
    }

    #[test]
    fn fonte_teste_eh_autorizada() {
        let fonte = FonteTeste;
        assert!(fonte.eh_autorizada());
        assert_eq!(fonte.id(), "fonte-teste");
    }

    #[test]
    fn gerenciador_registra_e_lista() {
        let mut gm = GerenciadorFontes::nova();
        gm.registrar(Box::new(FonteTeste));

        let fontes = gm.listar();
        assert_eq!(fontes.len(), 1);
        assert_eq!(fontes[0].id, "fonte-teste");
        assert!(fontes[0].autorizada);

        let autorizadas = gm.fontes_autorizadas();
        assert_eq!(autorizadas.len(), 1);
    }

    #[tokio::test]
    async fn fonte_busca_obras() {
        let fonte = FonteTeste;
        let obras = fonte.buscar_obras("Naruto").await.unwrap();
        assert_eq!(obras.len(), 1);
        assert_eq!(obras[0].titulo, "Naruto");
    }

    #[test]
    fn fonte_nao_autorizada_nao_aparece_autorizadas() {
        struct FonteNaoAutorizada;
        impl Fonte for FonteNaoAutorizada {
            fn id(&self) -> &str {
                "nao-autorizada"
            }
            fn nome(&self) -> &str {
                "Fonte Não Autorizada"
            }
            fn eh_autorizada(&self) -> bool {
                false
            }
            async fn buscar_obras(
                &self,
                _consulta: &str,
            ) -> Result<Vec<ObraEncontrada>, FonteError> {
                Ok(vec![])
            }
            async fn listar_capitulos(
                &self,
                _obra_id_fonte: &str,
            ) -> Result<Vec<CapituloEncontrado>, FonteError> {
                Ok(vec![])
            }
            async fn obter_capitulo(
                &self,
                _capitulo_id_fonte: &str,
            ) -> Result<Vec<u8>, FonteError> {
                Err(FonteError::NaoAutorizada)
            }
        }

        let mut gm = GerenciadorFontes::nova();
        gm.registrar(Box::new(FonteNaoAutorizada));
        let autorizadas = gm.fontes_autorizadas();
        assert_eq!(autorizadas.len(), 0);
    }
}
