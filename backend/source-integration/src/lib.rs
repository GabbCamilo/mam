//! # mam-source-integration
//! Busca e obtenção autorizada de obras e capítulos em fontes compatíveis.
//!
//! A integração com fontes é modular e desacoplada. Cada fonte implementa o
//! trait `Fonte`. Somente fontes autorizadas (eh_autorizada == true) permitem
//! arquivamento. Não há mecanismos para contornar DRM, paywalls ou controles
//! de acesso.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use mam_database::Database;

use crate::fonte::{Fonte, ObraEncontrada, CapituloEncontrado};
use crate::operacao::OperacaoArquivamento;
use crate::duplicatas::EstatisticasDuplicatas;

use serde::{Deserialize, Serialize};

mod fonte;
mod busca;
mod capitulos;
mod operacao;
mod duplicatas;

/// Erros do módulo de integração com fontes.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("Erro de rede: {0}")]
    Rede(#[from] reqwest::Error),
    #[error("Erro de serialização: {0}")]
    Serializacao(#[from] serde_json::Error),
    #[error("Erro de banco: {0}")]
    Banco(#[from] mam_database::DatabaseError),
    #[error("Fonte não encontrada: {0}")]
    FonteNaoEncontrada(String),
    #[error("Fonte não autorizada: {0}")]
    FonteNaoAutorizada(String),
    #[error("Erro de fonte: {0}")]
    Fonte(#[from] crate::fonte::FonteError),
    #[error("Obra não encontrada: {0}")]
    ObraNaoEncontrada(String),
    #[error("Capítulo não encontrado: {0}")]
    CapituloNaoEncontrado(String),
    #[error("Erro de operação: {0}")]
    Operacao(String),
}

/// Resultado de busca de obra em uma fonte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObraBuscaResultado {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub titulo: String,
    pub autor: Option<String>,
    pub descricao: Option<String>,
}

/// Resultado de listagem de capítulos em uma fonte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapituloBuscaResultado {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub capitulo_id_fonte: String,
    pub numero: Option<i32>,
    pub titulo: Option<String>,
    pub url: Option<String>,
}

/// Gerenciador de integração com fontes.
/// Orquestra o registro de fontes, busca de obras, listagem de capítulos
/// e operações de arquivamento.
pub struct SourceIntegrationManager {
    fontes: Arc<RwLock<HashMap<String, Arc<dyn Fonte>>>>,
    db: Arc<Database>,
    operacoes: Arc<RwLock<HashMap<String, operacao::OperacaoArquivamento>>>,
}

impl SourceIntegrationManager {
    pub fn nova(db: Arc<Database>) -> Self {
        Self {
            fontes: Arc::new(RwLock::new(HashMap::new())),
            db,
            operacoes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registra uma nova fonte no gerenciador.
    pub async fn registrar_fonte(
        &self,
        fonte: Arc<dyn Fonte>,
    ) -> Result<(), SourceError> {
        let id = fonte.id().to_string();
        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(id.clone()));
        }
        self.fontes.write().await.insert(id.clone(), fonte);
        Ok(())
    }

    /// Lista todas as fontes registradas.
    pub async fn listar_fontes(&self) -> Vec<FonteInfo> {
        let fontes = self.fontes.read().await;
        fontes
            .values()
            .map(|f| FonteInfo {
                id: f.id().to_string(),
                nome: f.nome().to_string(),
                autorizada: f.eh_autorizada(),
            })
            .collect()
    }

    /// Busca obras por título em todas as fontes autorizadas.
    pub async fn buscar_obras(
        &self,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let mut resultados = Vec::new();

        for fonte in fontes.values() {
            if !fonte.eh_autorizada() {
                continue;
            }
            match fonte.buscar_obras(consulta).await {
                Ok(obras) => {
                    for obra in obras {
                        resultados.push(ObraBuscaResultado {
                            fonte_id: fonte.id().to_string(),
                            obra_id_fonte: obra.obra_id_fonte,
                            titulo: obra.titulo,
                            autor: obra.autor,
                            descricao: obra.descricao,
                        });
                    }
                }
                Err(_) => {
                    // Fonte falhou, continua com as outras
                    continue;
                }
            }
        }

        Ok(resultados)
    }

    /// Lista capítulos de uma obra em uma fonte específica.
    pub async fn listar_capitulos(
        &self,
        fonte_id: &str,
        obra_id_fonte: &str,
    ) -> Result<Vec<CapituloBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let fonte = fontes
            .get(fonte_id)
            .ok_or_else(|| SourceError::FonteNaoEncontrada(fonte_id.to_string()))?;

        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(fonte_id.to_string()));
        }

        let capitulos = fonte.listar_capitulos(obra_id_fonte).await?;

        Ok(capitulos
            .into_iter()
            .map(|c| CapituloBuscaResultado {
                fonte_id: fonte.id().to_string(),
                obra_id_fonte: obra_id_fonte.to_string(),
                capitulo_id_fonte: c.capitulo_id_fonte,
                numero: c.numero,
                titulo: c.titulo,
                url: c.url,
            })
            .collect())
    }

    /// Verifica se uma obra já está arquivada no banco (detecção de duplicata).
    pub async fn obra_ja_arquivada(&self, titulo: &str) -> Result<bool, SourceError> {
        Ok(self.db.buscar_obra_por_titulo(titulo).await?.is_some())
    }

    /// Verifica se um capítulo já está arquivado (detecção de duplicata por caminho).
    pub async fn capitulo_ja_arquivado(
        &self,
        caminho_relativo: &str,
    ) -> Result<bool, SourceError> {
        Ok(self.db.consultar_integridade(caminho_relativo).await?.is_some())
    }

    /// Inicia uma operação de arquivamento de capítulos.
    pub async fn iniciar_operacao_arquivamento(
        &self,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
    ) -> Result<String, SourceError> {
        let id_operacao = uuid::Uuid::new_v4().to_string();

        let operacao = operacao::OperacaoArquivamento::nova(
            id_operacao.clone(),
            obra_titulo,
            capitulos,
        );

        self.operacoes.write().await.insert(id_operacao.clone(), operacao);

        Ok(id_operacao)
    }

    /// Retorna o status de uma operação de arquivamento.
    pub async fn status_operacao(
        &self,
        id_operacao: &str,
    ) -> Result<Option<StatusOperacao>, SourceError> {
        let operacoes = self.operacoes.read().await;
        Ok(operacoes.get(id_operacao).map(|o| StatusOperacao {
            id: o.id.clone(),
            obra_titulo: o.obra_titulo.clone(),
            total_capitulos: o.total_capitulos(),
            capitulos_processados: o.capitulos_processados(),
            percentual: o.percentual(),
            concluida: o.esta_concluida(),
        }))
    }

    /// Lista todas as operações de arquivamento.
    pub async fn listar_operacoes(&self) -> Vec<StatusOperacao> {
        let operacoes = self.operacoes.read().await;
        operacoes
            .values()
            .map(|o| StatusOperacao {
                id: o.id.clone(),
                obra_titulo: o.obra_titulo.clone(),
                total_capitulos: o.total_capitulos(),
                capitulos_processados: o.capitulos_processados(),
                percentual: o.percentual(),
                concluida: o.esta_concluida(),
            })
            .collect()
    }

    /// Executa o arquivamento de um capítulo (atualiza progresso).
    pub async fn arquivar_capitulo(
        &self,
        id_operacao: &str,
        fonte_id: &str,
        capitulo: &CapituloBuscaResultado,
    ) -> Result<(), SourceError> {
        let mut operacoes = self.operacoes.write().await;
        if let Some(op) = operacoes.get_mut(id_operacao) {
            op.atualizar_progresso(capitulo.capitulo_id_fonte.as_str());
        }
        Ok(())
    }
}

/// Informação de uma fonte registrada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FonteInfo {
    pub id: String,
    pub nome: String,
    pub autorizada: bool,
}

/// Status de uma operação de arquivamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusOperacao {
    pub id: String,
    pub obra_titulo: String,
    pub total_capitulos: usize,
    pub capitulos_processados: usize,
    pub percentual: f32,
    pub concluida: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonte_info_cria() {
        let info = FonteInfo {
            id: "fonte-teste".to_string(),
            nome: "Fonte Teste".to_string(),
            autorizada: true,
        };
        assert_eq!(info.id, "fonte-teste");
        assert!(info.autorizada);
    }

    #[test]
    fn status_operacao_calcula_percentual() {
        let status = StatusOperacao {
            id: "op-1".to_string(),
            obra_titulo: "Obra Teste".to_string(),
            total_capitulos: 10,
            capitulos_processados: 3,
            percentual: 30.0,
            concluida: false,
        };
        assert_eq!(status.percentual, 30.0);
        assert!(!status.concluida);
    }
}
