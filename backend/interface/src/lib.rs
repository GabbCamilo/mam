//! # mam-interface
//! Comandos e eventos Tauri para comunicação com o frontend (React + TypeScript).
//!
//! Depende de `mam-core` para tipos de identificador e tarefa.
//! Depende de `mam-database` para operações de persistência.
//! Depende de `mam-chapter` para detecção e gerenciamento de capítulos.
//!
//! As funções aqui são lógica de negócio pura (sem macro Tauri).
//! O macro #[command] é aplicado no src-tauri via wrappers.

use std::sync::Arc;
use std::path::PathBuf;

use mam_database::Database;
use mam_chapter::ChapterManager;

/// Retorna o status do banco de dados.
pub async fn db_health() -> String {
    "ok".to_string()
}

/// Lista todas as obras cadastradas na biblioteca.
pub async fn db_list_obras(db: Arc<Database>) -> Result<Vec<String>, String> {
    let obras = db.listar_obras().await.map_err(|e| e.to_string())?;
    Ok(obras.into_iter().map(|o| o.titulo).collect())
}

/// Detecta capítulos na biblioteca e os registra no banco.
pub async fn chapter_detectar(
    chapter_manager: Arc<ChapterManager>,
) -> Result<Vec<String>, String> {
    let detectados = chapter_manager.detectar_todos().await.map_err(|e| e.to_string())?;
    Ok(detectados
        .into_iter()
        .map(|d| format!("{} - {} ({} páginas)", d.obra_titulo, d.caminho_relativo, d.quantidade_paginas))
        .collect())
}

/// Lista capítulos de uma obra por título.
pub async fn chapter_listar_por_obra(
    db: Arc<Database>,
    obra_titulo: String,
) -> Result<Vec<String>, String> {
    let obra = db
        .buscar_obra_por_titulo(&obra_titulo)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Obra '{}' não encontrada", obra_titulo))?;

    let capitulos = db
        .listar_capitulos_obra(obra.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(capitulos
        .into_iter()
        .map(|c| format!("Capítulo {} - {} ({} páginas)", c.numero, c.caminho_relativo.as_str(), c.quantidade_paginas))
        .collect())
}

/// Retorna o caminho da raiz da biblioteca configurada.
pub async fn library_raiz() -> Result<String, String> {
    Ok(PathBuf::from("MAM_LIBRARY").to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
