use std::sync::Arc;
use std::path::PathBuf;

use tauri::Manager;

use mam_database::Database;
use mam_chapter::ChapterManager;
use mam_archive::{ArchiveManager, PaginaExtraida};
use mam_interface::{db_health as if_db_health, db_list_obras as if_db_list_obras};
use mam_interface::{chapter_detectar as if_chapter_detectar, chapter_listar_por_obra as if_chapter_listar_por_obra};
use mam_interface::{library_raiz as if_library_raiz, library_criar_obra as if_library_criar_obra, library_listar_obras as if_library_listar_obras, library_obter_detalhes_obra as if_library_obter_detalhes_obra, library_criar_volume as if_library_criar_volume, library_atualizar_metadados as if_library_atualizar_metadados};
use mam_database::models::{Obra, Volume};
use mam_library::{DetalhesObra, LibraryManager};
use mam_interface::{archive_criar_cbz as if_archive_criar_cbz, archive_ler_cbz as if_archive_ler_cbz, archive_validar_cbz as if_archive_validar_cbz};
use mam_interface::{source_listar_fontes as if_source_listar_fontes, source_buscar_obras as if_source_buscar_obras};
use mam_interface::{source_listar_capitulos as if_source_listar_capitulos, source_verificar_duplicatas as if_source_verificar_duplicatas};
use mam_interface::{source_arquivar_capitulos as if_source_arquivar_capitulos};
use mam_interface::{
    MarcadorPagina,
    reader_obter_pagina as if_reader_obter_pagina,
    reader_salvar_progresso as if_reader_salvar_progresso,
    reader_obter_progresso as if_reader_obter_progresso,
    reader_alternar_favorito as if_reader_alternar_favorito,
    reader_adicionar_marcador as if_reader_adicionar_marcador,
    reader_listar_marcadores as if_reader_listar_marcadores,
    transfer_copia_integral as if_transfer_copia_integral,
    transfer_seletiva as if_transfer_seletiva,
    sync_comparar_bibliotecas as if_sync_comparar_bibliotecas,
    sync_sincronizar_bibliotecas as if_sync_sincronizar_bibliotecas,
};
use mam_transfer::{DiferencaBibliotecas, RelatorioTransferencia, TransferManager};
use mam_source_integration::SourceIntegrationManager;
use mam_source_integration::{FonteInfo, ObraBuscaResultado, CapituloBuscaResultado};

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

#[tauri::command]
async fn library_criar_obra(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    titulo: String,
    caminho_relativo: String,
    origem: Option<String>,
) -> Result<Obra, String> {
    if_library_criar_obra(library_manager.inner().clone(), titulo, caminho_relativo, origem).await
}

#[tauri::command]
async fn library_listar_obras(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
) -> Result<Vec<Obra>, String> {
    if_library_listar_obras(library_manager.inner().clone()).await
}

#[tauri::command]
async fn library_obter_detalhes_obra(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_id: String,
) -> Result<DetalhesObra, String> {
    if_library_obter_detalhes_obra(library_manager.inner().clone(), obra_id).await
}

#[tauri::command]
async fn library_criar_volume(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_id: String,
    numero: i32,
    titulo: Option<String>,
    caminho_relativo: String,
) -> Result<Volume, String> {
    if_library_criar_volume(library_manager.inner().clone(), obra_id, numero, titulo, caminho_relativo).await
}

#[tauri::command]
async fn library_atualizar_metadados(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_id: String,
    metadados_json: String,
) -> Result<(), String> {
    if_library_atualizar_metadados(library_manager.inner().clone(), obra_id, metadados_json).await
}

#[tauri::command]
async fn archive_criar_cbz(
    archive_manager: tauri::State<'_, Arc<ArchiveManager>>,
    imagens_dir: String,
    caminho_destino: String,
) -> Result<String, String> {
    if_archive_criar_cbz(archive_manager.inner().clone(), imagens_dir, caminho_destino).await
}

#[tauri::command]
async fn archive_ler_cbz(
    archive_manager: tauri::State<'_, Arc<ArchiveManager>>,
    caminho_cbz: String,
) -> Result<String, String> {
    if_archive_ler_cbz(archive_manager.inner().clone(), caminho_cbz).await
}

#[tauri::command]
async fn archive_validar_cbz(
    archive_manager: tauri::State<'_, Arc<ArchiveManager>>,
    caminho_cbz: String,
) -> Result<bool, String> {
    if_archive_validar_cbz(archive_manager.inner().clone(), caminho_cbz).await
}

#[tauri::command]
async fn source_listar_fontes(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
) -> Result<Vec<FonteInfo>, String> {
    if_source_listar_fontes(source_manager.inner().clone()).await
}

#[tauri::command]
async fn source_buscar_obras(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    consulta: String,
) -> Result<Vec<ObraBuscaResultado>, String> {
    if_source_buscar_obras(source_manager.inner().clone(), consulta).await
}

#[tauri::command]
async fn source_listar_capitulos(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    fonte_id: String,
    obra_id_fonte: String,
) -> Result<Vec<CapituloBuscaResultado>, String> {
    if_source_listar_capitulos(source_manager.inner().clone(), fonte_id, obra_id_fonte).await
}

#[tauri::command]
async fn source_verificar_duplicatas(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    caminho_relativo: String,
) -> Result<bool, String> {
    if_source_verificar_duplicatas(source_manager.inner().clone(), caminho_relativo).await
}

#[tauri::command]
async fn source_arquivar_capitulos(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    obra_titulo: String,
    capitulos: Vec<CapituloBuscaResultado>,
) -> Result<String, String> {
    if_source_arquivar_capitulos(source_manager.inner().clone(), obra_titulo, capitulos).await
}

#[tauri::command]
async fn reader_obter_pagina(
    archive_manager: tauri::State<'_, Arc<ArchiveManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    caminho_cbz: String,
    indice: usize,
) -> Result<PaginaExtraida, String> {
    if_reader_obter_pagina(
        archive_manager.inner().clone(),
        library_manager.inner().clone(),
        caminho_cbz,
        indice,
    )
    .await
}

#[tauri::command]
async fn reader_salvar_progresso(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
    pagina: usize,
) -> Result<(), String> {
    if_reader_salvar_progresso(db.inner().clone(), capitulo_id, pagina).await
}

#[tauri::command]
async fn reader_obter_progresso(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
) -> Result<usize, String> {
    if_reader_obter_progresso(db.inner().clone(), capitulo_id).await
}

#[tauri::command]
async fn reader_alternar_favorito(
    db: tauri::State<'_, Arc<Database>>,
    alvo_id: String,
) -> Result<bool, String> {
    if_reader_alternar_favorito(db.inner().clone(), alvo_id).await
}

#[tauri::command]
async fn reader_adicionar_marcador(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
    pagina: usize,
    nota: String,
) -> Result<(), String> {
    if_reader_adicionar_marcador(db.inner().clone(), capitulo_id, pagina, nota).await
}

#[tauri::command]
async fn reader_listar_marcadores(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
) -> Result<Vec<MarcadorPagina>, String> {
    if_reader_listar_marcadores(db.inner().clone(), capitulo_id).await
}

#[tauri::command]
async fn transfer_copia_integral(
    transfer_manager: tauri::State<'_, Arc<TransferManager>>,
    origem: String,
    destino: String,
) -> Result<RelatorioTransferencia, String> {
    if_transfer_copia_integral(transfer_manager.inner().clone(), origem, destino).await
}

#[tauri::command]
async fn transfer_seletiva(
    transfer_manager: tauri::State<'_, Arc<TransferManager>>,
    origem: String,
    destino: String,
    caminhos_relativos: Vec<String>,
) -> Result<RelatorioTransferencia, String> {
    if_transfer_seletiva(transfer_manager.inner().clone(), origem, destino, caminhos_relativos).await
}

#[tauri::command]
async fn sync_comparar_bibliotecas(
    transfer_manager: tauri::State<'_, Arc<TransferManager>>,
    origem: String,
    destino: String,
) -> Result<DiferencaBibliotecas, String> {
    if_sync_comparar_bibliotecas(transfer_manager.inner().clone(), origem, destino).await
}

#[tauri::command]
async fn sync_sincronizar_bibliotecas(
    transfer_manager: tauri::State<'_, Arc<TransferManager>>,
    origem: String,
    destino: String,
    permitir_sobrescrita: bool,
) -> Result<RelatorioTransferencia, String> {
    if_sync_sincronizar_bibliotecas(transfer_manager.inner().clone(), origem, destino, permitir_sobrescrita).await
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
      library_criar_obra,
      library_listar_obras,
      library_obter_detalhes_obra,
      library_criar_volume,
      library_atualizar_metadados,
      archive_criar_cbz,
      archive_ler_cbz,
      archive_validar_cbz,
      source_listar_fontes,
      source_buscar_obras,
      source_listar_capitulos,
      source_verificar_duplicatas,
      source_arquivar_capitulos,
      reader_obter_pagina,
      reader_salvar_progresso,
      reader_obter_progresso,
      reader_alternar_favorito,
      reader_adicionar_marcador,
      reader_listar_marcadores,
      transfer_copia_integral,
      transfer_seletiva,
      sync_comparar_bibliotecas,
      sync_sincronizar_bibliotecas,
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
      let chapter_manager = ChapterManager::nova(db_arc.clone(), raiz_biblioteca.clone());
      app.manage(Arc::new(chapter_manager));

      // Inicializa o LibraryManager como estado
      let library_manager = LibraryManager::nova(db_arc.clone(), raiz_biblioteca);
      app.manage(Arc::new(library_manager));

      // Inicializa o ArchiveManager como estado
      let archive_manager = ArchiveManager::nova(db_arc.clone());
      app.manage(Arc::new(archive_manager));

      // Inicializa o SourceIntegrationManager como estado
      let source_manager = SourceIntegrationManager::nova(db_arc);
      app.manage(Arc::new(source_manager));

      // Inicializa o TransferManager como estado
      let transfer_manager = TransferManager::nova();
      app.manage(Arc::new(transfer_manager));

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
