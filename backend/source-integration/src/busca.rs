//! Busca de obras em fontes autorizadas.
//! Utiliza o SourceIntegrationManager para consultar múltiplas fontes.

use super::{SourceError, SourceIntegrationManager, ObraBuscaResultado};

impl SourceIntegrationManager {
    /// Busca obras por título em todas as fontes autorizadas.
    /// Retorna resultados agregados de todas as fontes.
    pub async fn buscar_em_todas_fontes(
        &self,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        self.buscar_obras(consulta).await
    }

    /// Busca obras apenas em uma fonte específica.
    pub async fn buscar_em_fonte(
        &self,
        fonte_id: &str,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let fonte = fontes
            .get(fonte_id)
            .ok_or_else(|| SourceError::FonteNaoEncontrada(fonte_id.to_string()))?;

        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(fonte_id.to_string()));
        }

        let obras = fonte.buscar_obras(consulta).await?;
        Ok(obras
            .into_iter()
            .map(|obra| ObraBuscaResultado {
                fonte_id: fonte.id().to_string(),
                obra_id_fonte: obra.obra_id_fonte,
                titulo: obra.titulo,
                autor: obra.autor,
                descricao: obra.descricao,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mam_database::Database;
    use std::sync::Arc;
    use std::env;

    async fn criar_gerenciador_teste() -> SourceIntegrationManager {
        let temp_dir = env::temp_dir().join(format!("mam_test_busca_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("database.db");
        let db = Database::abrir(db_path, temp_dir.clone()).await.expect("abrir banco");
        SourceIntegrationManager::nova(Arc::new(db))
    }

    #[tokio::test]
    async fn buscar_obras_vazio_retorna_lista_vazia() {
        let gm = criar_gerenciador_teste().await;
        let resultado = gm.buscar_obras("").await.unwrap();
        assert_eq!(resultado.len(), 0);
    }
}
