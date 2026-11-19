use std::sync::Arc;
use std::path::PathBuf;

use tauri::Manager;

use mam_database::Database;
use mam_chapter::ChapterManager;
use mam_interface::{db_health as if_db_health, db_list_obras as if_db_list_obras};
use mam_interface::{chapter_detectar as if_chapter_detectar, chapter_listar_por_obra as if_chapter_listar_por_obra};
use mam_interface::library_raiz as if_library_raiz;

#[tauri::command]
fn health_check() -> String {
    "ok".to_string()
}

#[tauri::command]
async fn db_health() -> String {
    if_db_health().await
}

#[tauri::command]
async fn db_list_obras(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<String>, String> {
    if_db_list_obras(db.inner().clone()).await
}

#[tauri::command]
async fn chapter_detectar(
    chapter_manager: tauri::State<'_, Arc<ChapterManager>>,
) -> Result<Vec<String>, String> {
    if_chapter_detectar(chapter_manager.inner().clone()).await
}

#[tauri::command]
async fn chapter_listar_por_obra(
    db: tauri::State<'_, Arc<Database>>,
    obra_titulo: String,
) -> Result<Vec<String>, String> {
    if_chapter_listar_por_obra(db.inner().clone(), obra_titulo).await
}

#[tauri::command]
async fn library_raiz() -> Result<String, String> {
    if_library_raiz().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      health_check,
      db_health,
      db_list_obras,
      chapter_detectar,
      chapter_listar_por_obra,
      library_raiz,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Inicializa o banco de dados SQLite como estado gerenciado pelo Tauri
      let app_handle = app.handle();
      let caminho_db = app_handle.path().app_data_dir()
        .unwrap_or(PathBuf::from("."))
        .join("database.db");

      let raiz_biblioteca = app_handle.path().app_data_dir()
        .unwrap_or(PathBuf::from("."));

      // Abre o banco de dados SQLite (runtime do Tauri executa setup de forma síncrona)
      let db = tokio::runtime::Runtime::new().unwrap().block_on(async {
        Database::abrir(caminho_db, raiz_biblioteca.clone()).await
          .expect("Falha ao inicializar banco de dados SQLite")
      });

      let db_arc = Arc::new(db);
      app.manage(db_arc.clone());

      // Inicializa o ChapterManager como estado
      let chapter_manager = ChapterManager::nova(db_arc, raiz_biblioteca);
      app.manage(Arc::new(chapter_manager));

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
