//! Operações de registro de capítulos no banco de dados.
//! Utiliza as operações do crate `mam-database` com transações curtas.

use crate::{ChapterError, CapituloDetectado};
use mam_core::{CaminhoRelativo, EstadoIntegridade};
use mam_database::Database;

use std::sync::Arc;

/// Registra um capítulo detectado no banco de dados.
/// Se a obra já existe, não cria duplicata.
pub async fn registrar_capitulo_detectado(
    db: Arc<Database>,
    detectado: &CapituloDetectado,
) -> Result<(), ChapterError> {
    // Verificar se a obra já existe por título
    let _obra_id = match db.buscar_obra_por_titulo(&detectado.obra_titulo).await? {
        Some(obra) => obra.id,
        None => {
            let id = mam_core::ObraId::new();
            let caminho = CaminhoRelativo::from_str(
                detectado.caminho_relativo.rsplitn(2, '/').nth(1).unwrap_or("").to_string(),
            );
            db.criar_obra(
                id,
                detectado.obra_titulo.clone(),
                caminho,
                None,
            )
            .await?
            .id
        }
    };

    // Registrar integridade do capítulo
    db.registrar_integridade(
        &detectado.caminho_relativo,
        &detectado.hash_sha256,
        EstadoIntegridade::Valido.as_str(),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_error_imprime() {
        let err = ChapterError::CbzInvalido("teste".to_string());
        assert_eq!(err.to_string(), "CBZ inválido: teste");
    }
}
