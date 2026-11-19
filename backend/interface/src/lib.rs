//! # mam-interface
//! Comandos e eventos Tauri para comunicação com o frontend (React + TypeScript).
//!
//! Depende de `mam-core` para tipos de identificador e tarefa.
//! Depende de `mam-database` para operações de persistência.
//!
//! As funções aqui são lógica de negócio; o macro #[command] é aplicado
//! no src-tauri para evitar duplicação de identificadores do macro Tauri.

use std::sync::Arc;

use mam_database::Database;

/// Retorna o status do banco de dados.
pub async fn db_health() -> String {
    "ok".to_string()
}

/// Lista todas as obras cadastradas na biblioteca.
pub async fn db_list_obras(db: Arc<Database>) -> Result<Vec<String>, String> {
    let obras = db.listar_obras().await.map_err(|e| e.to_string())?;
    Ok(obras.into_iter().map(|o| o.titulo).collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
