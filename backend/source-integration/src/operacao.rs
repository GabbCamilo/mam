//! Gerenciamento de operações de arquivamento com progresso e retomada.
//! Operações são rastreadas em memória e podem ser consultadas para
//! acompanhar o progresso do arquivamento de capítulos.

use std::collections::HashMap;

use super::CapituloBuscaResultado;

/// Operação de arquivamento de capítulos de uma obra.
#[derive(Debug, Clone)]
pub struct OperacaoArquivamento {
    pub id: String,
    pub obra_titulo: String,
    pub capitulos: Vec<CapituloParaArquivar>,
    pub capitulos_processados: HashMap<String, bool>, // capitulo_id -> processado
    pub data_inicio: chrono::DateTime<chrono::Utc>,
    pub data_conclusao: Option<chrono::DateTime<chrono::Utc>>,
}

/// Capítulo a ser arquivado em uma operação.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapituloParaArquivar {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub capitulo_id_fonte: String,
    pub numero: Option<i32>,
    pub titulo: Option<String>,
    pub url: Option<String>,
}

/// Status simplificado de uma operação (para exposição via API).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusOperacao {
    pub id: String,
    pub obra_titulo: String,
    pub total_capitulos: usize,
    pub capitulos_processados: usize,
    pub percentual: f32,
    pub concluida: bool,
}

impl OperacaoArquivamento {
    /// Cria uma nova operação de arquivamento.
    pub fn nova(
        id: String,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
    ) -> Self {
        let mut capitulos_para_arquivar = Vec::new();
        let mut processados = HashMap::new();

        for cap in capitulos {
            capitulos_para_arquivar.push(CapituloParaArquivar {
                fonte_id: cap.fonte_id,
                obra_id_fonte: cap.obra_id_fonte,
                capitulo_id_fonte: cap.capitulo_id_fonte.clone(),
                numero: cap.numero,
                titulo: cap.titulo,
                url: cap.url,
            });
            processados.insert(cap.capitulo_id_fonte, false);
        }

        Self {
            id,
            obra_titulo,
            capitulos: capitulos_para_arquivar,
            capitulos_processados: processados,
            data_inicio: chrono::Utc::now(),
            data_conclusao: None,
        }
    }

    /// Atualiza o progresso de um capítulo (marca como processado).
    pub fn atualizar_progresso(&mut self, capitulo_id: &str) {
        if let Some(processado) = self.capitulos_processados.get_mut(capitulo_id) {
            *processado = true;
        }
        if self.capitulos_processados.values().all(|v| *v) {
            self.data_conclusao = Some(chrono::Utc::now());
        }
    }

    /// Retorna o status simplificado da operação.
    pub fn status(&self) -> StatusOperacao {
        let processados = self.capitulos_processados.values().filter(|v| **v).count();
        let total = self.capitulos.len();
        let percentual = if total > 0 {
            (processados as f32 / total as f32) * 100.0
        } else {
            100.0
        };
        let concluida = self.data_conclusao.is_some() || (total > 0 && processados == total);

        StatusOperacao {
            id: self.id.clone(),
            obra_titulo: self.obra_titulo.clone(),
            total_capitulos: total,
            capitulos_processados: processados,
            percentual,
            concluida,
        }
    }

    /// Verifica se todos os capítulos foram processados.
    pub fn esta_concluida(&self) -> bool {
        !self.capitulos.is_empty()
            && self.capitulos_processados.values().all(|v| *v)
    }
}

impl Default for OperacaoArquivamento {
    fn default() -> Self {
        Self {
            id: String::new(),
            obra_titulo: String::new(),
            capitulos: Vec::new(),
            capitulos_processados: HashMap::new(),
            data_inicio: chrono::Utc::now(),
            data_conclusao: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operacao_calcula_progresso() {
        let mut op = OperacaoArquivamento::nova(
            "op-1".to_string(),
            "Obra Teste".to_string(),
            vec![
                crate::CapituloBuscaResultado {
                    fonte_id: "f1".to_string(),
                    obra_id_fonte: "o1".to_string(),
                    capitulo_id_fonte: "c1".to_string(),
                    numero: Some(1),
                    titulo: Some("Cap 1".to_string()),
                    url: None,
                },
                crate::CapituloBuscaResultado {
                    fonte_id: "f1".to_string(),
                    obra_id_fonte: "o1".to_string(),
                    capitulo_id_fonte: "c2".to_string(),
                    numero: Some(2),
                    titulo: Some("Cap 2".to_string()),
                    url: None,
                },
            ],
        );

        let status = op.status();
        assert_eq!(status.total_capitulos, 2);
        assert_eq!(status.capitulos_processados, 0);
        assert_eq!(status.percentual, 0.0);
        assert!(!status.concluida);

        op.atualizar_progresso("c1");
        let status = op.status();
        assert_eq!(status.capitulos_processados, 1);
        assert_eq!(status.percentual, 50.0);
        assert!(!status.concluida);

        op.atualizar_progresso("c2");
        let status = op.status();
        assert_eq!(status.capitulos_processados, 2);
        assert_eq!(status.percentual, 100.0);
        assert!(status.concluida);
        assert!(op.esta_concluida());
    }

    #[test]
    fn operacao_sem_capitulos_esta_concluida() {
        let op = OperacaoArquivamento::nova(
            "op-vazio".to_string(),
            "Obra Vazia".to_string(),
            vec![],
        );
        assert!(!op.esta_concluida());
        assert_eq!(op.status().percentual, 100.0);
    }
}
