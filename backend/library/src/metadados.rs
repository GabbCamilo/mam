//! Validação e atualização de metadados complementares em JSON.

use serde_json::Value;

use mam_core::ObraId;
use mam_database::Database;

use crate::LibraryError;

/// Valida que a string fornecida é um JSON válido (objeto JSON).
pub fn validar_metadados_json(metadados_json: &str) -> Result<String, LibraryError> {
    let valor: Value = serde_json::from_str(metadados_json)
        .map_err(|e| LibraryError::MetadadosInvalidos(format!("JSON inválido: {}", e)))?;

    if !valor.is_object() {
        return Err(LibraryError::MetadadosInvalidos(
            "Os metadados complementares devem ser um objeto JSON".to_string(),
        ));
    }

    serde_json::to_string(&valor)
        .map_err(|e| LibraryError::MetadadosInvalidos(format!("Falha ao serializar JSON: {}", e)))
}

/// Atualiza os metadados complementares de uma obra sem alterar arquivos `.cbz`.
pub async fn atualizar_metadados_obra(
    db: &Database,
    obra_id: ObraId,
    metadados_json: &str,
) -> Result<(), LibraryError> {
    if db.buscar_obra_por_id(obra_id).await?.is_none() {
        return Err(LibraryError::ObraNaoEncontrada(obra_id.to_string()));
    }

    let json_normalizado = validar_metadados_json(metadados_json)?;
    db.atualizar_metadados_obra(obra_id, &json_normalizado)
        .await?;
    Ok(())
}
