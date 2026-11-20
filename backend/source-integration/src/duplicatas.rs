//! Detecção de conteúdos já existentes (duplicatas) na biblioteca.
//! Antes de arquivar um capítulo, o sistema verifica se ele já existe
//! no banco de dados local para evitar downloads desnecessários.

use super::SourceIntegrationManager;

impl SourceIntegrationManager {
    /// Verifica se uma obra já está arquivada por título.
    pub async fn verificar_duplicata_obra(
        &self,
        titulo: &str,
    ) -> Result<bool, super::SourceError> {
        Ok(self.db.buscar_obra_por_titulo(titulo).await?.is_some())
    }

    /// Verifica se um capítulo já está arquivado por caminho relativo.
    pub async fn verificar_duplicata_capitulo(
        &self,
        caminho_relativo: &str,
    ) -> Result<bool, super::SourceError> {
        Ok(self.db.consultar_integridade(caminho_relativo).await?.is_some())
    }

    /// Filtra capítulos já arquivados de uma lista, retornando apenas os que precisam ser baixados.
    pub async fn filtrar_nao_arquivados(
        &self,
        capitulos: Vec<super::CapituloBuscaResultado>,
    ) -> Result<Vec<super::CapituloBuscaResultado>, super::SourceError> {
        let mut nao_arquivados = Vec::new();

        for cap in capitulos {
            let ja_arquivado = if let Some(url) = &cap.url {
                self.verificar_duplicata_capitulo(url).await.unwrap_or(false)
            } else {
                false
            };

            if !ja_arquivado {
                nao_arquivados.push(cap);
            }
        }

        Ok(nao_arquivados)
    }

    /// Retorna estatísticas de duplicatas para um conjunto de capítulos.
    pub async fn estatisticas_duplicatas(
        &self,
        capitulos: &[super::CapituloBuscaResultado],
    ) -> Result<EstatisticasDuplicatas, super::SourceError> {
        let mut total = 0;
        let mut ja_arquivados = 0;
        let mut nao_arquivados = 0;

        for cap in capitulos {
            total += 1;
            let ja_arquivado = if let Some(url) = &cap.url {
                self.verificar_duplicata_capitulo(url).await.unwrap_or(false)
            } else {
                false
            };

            if ja_arquivado {
                ja_arquivados += 1;
            } else {
                nao_arquivados += 1;
            }
        }

        Ok(EstatisticasDuplicatas {
            total,
            ja_arquivados,
            nao_arquivados,
        })
    }
}

/// Estatísticas de detecção de duplicatas.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EstatisticasDuplicatas {
    pub total: usize,
    pub ja_arquivados: usize,
    pub nao_arquivados: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mam_core::CaminhoRelativo;
    use mam_database::Database;
    use std::sync::Arc;
    use std::env;

    async fn criar_gerenciador_com_obra() -> (SourceIntegrationManager, String) {
        let temp_dir = env::temp_dir().join(format!("mam_test_dup_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("database.db");
        let db = Database::abrir(db_path, temp_dir.clone()).await.expect("abrir banco");

        // Cria uma obra para testar duplicata
        let obra_id = mam_core::ObraId::new();
        let caminho = CaminhoRelativo::from_str("Manga/Obra Existente".to_string());
        db.criar_obra(
            obra_id,
            "Obra Existente".to_string(),
            caminho,
            None,
        ).await.expect("criar obra");

        let gm = SourceIntegrationManager::nova(Arc::new(db));
        (gm, "Obra Existente".to_string())
    }

    #[tokio::test]
    async fn verificar_duplicata_obra_existente() {
        let (gm, titulo) = criar_gerenciador_com_obra().await;
        let resultado = gm.verificar_duplicata_obra(&titulo).await.unwrap();
        assert!(resultado);
    }

    #[tokio::test]
    async fn verificar_duplicata_obra_inexistente() {
        let (gm, _) = criar_gerenciador_com_obra().await;
        let resultado = gm.verificar_duplicata_obra("Obra Inexistente").await.unwrap();
        assert!(!resultado);
    }

    #[tokio::test]
    async fn filtrar_nao_arquivados() {
        let (gm, _) = criar_gerenciador_com_obra().await;

        let capitulos = vec![
            crate::CapituloBuscaResultado {
                fonte_id: "f1".to_string(),
                obra_id_fonte: "o1".to_string(),
                capitulo_id_fonte: "cap-novo".to_string(),
                numero: Some(1),
                titulo: Some("Cap Novo".to_string()),
                url: Some("Manga/Obra Existente/Cap 1.cbz".to_string()),
            },
            crate::CapituloBuscaResultado {
                fonte_id: "f1".to_string(),
                obra_id_fonte: "o1".to_string(),
                capitulo_id_fonte: "cap-existente".to_string(),
                numero: Some(2),
                titulo: Some("Cap Existente".to_string()),
                url: Some("Manga/Obra Existente/Cap 2.cbz".to_string()),
            },
        ];

        let resultado = gm.filtrar_nao_arquivados(capitulos.clone()).await.unwrap();
        // Como não criamos integridade para esses caminhos, nenhum será detectado como duplicato
        // pelo consultar_integridade. O teste verifica que a função executa sem erro.
        assert_eq!(resultado.len(), 2);
    }

    #[test]
    fn estatisticas_duplicatas_cria() {
        let stats = EstatisticasDuplicatas {
            total: 10,
            ja_arquivados: 3,
            nao_arquivados: 7,
        };
        assert_eq!(stats.total, 10);
        assert_eq!(stats.ja_arquivados, 3);
        assert_eq!(stats.nao_arquivados, 7);
    }
}
