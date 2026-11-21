use std::sync::Arc;
use std::path::PathBuf;

use tauri::Manager;

use mam_database::Database;
use mam_chapter::ChapterManager;
use mam_archive::{ArchiveManager, PaginaExtraida};
use mam_interface::{db_health as if_db_health, db_list_obras as if_db_list_obras};
use mam_interface::{chapter_detectar as if_chapter_detectar, chapter_listar_por_obra as if_chapter_listar_por_obra};
use mam_interface::{library_raiz as if_library_raiz, library_criar_obra as if_library_criar_obra, library_listar_obras as if_library_listar_obras, library_obter_detalhes_obra as if_library_obter_detalhes_obra, library_criar_volume as if_library_criar_volume, library_atualizar_metadados as if_library_atualizar_metadados};
use mam_database::models::{Obra, ResultadoIA, Volume};
use mam_library::{DetalhesObra, LibraryManager};
use mam_interface::{archive_criar_cbz as if_archive_criar_cbz, archive_ler_cbz as if_archive_ler_cbz, archive_validar_cbz as if_archive_validar_cbz};
use mam_interface::{source_listar_fontes as if_source_listar_fontes, source_buscar_obras as if_source_buscar_obras};
use mam_interface::{source_listar_capitulos as if_source_listar_capitulos, source_verificar_duplicatas as if_source_verificar_duplicatas};
use mam_interface::{
    source_arquivar_capitulos as if_source_arquivar_capitulos,
    source_iniciar_arquivamento as if_source_iniciar_arquivamento,
    source_listar_operacoes as if_source_listar_operacoes,
    source_status_operacao as if_source_status_operacao,
};
use mam_interface::{
    MarcadorObra,
    MarcadorPagina,
    reader_obter_pagina as if_reader_obter_pagina,
    reader_salvar_progresso as if_reader_salvar_progresso,
    reader_obter_progresso as if_reader_obter_progresso,
    reader_alternar_favorito as if_reader_alternar_favorito,
    reader_adicionar_marcador as if_reader_adicionar_marcador,
    reader_listar_marcadores as if_reader_listar_marcadores,
    reader_listar_marcadores_obra as if_reader_listar_marcadores_obra,
    transfer_copia_integral as if_transfer_copia_integral,
    transfer_seletiva as if_transfer_seletiva,
    sync_comparar_bibliotecas as if_sync_comparar_bibliotecas,
    sync_sincronizar_bibliotecas as if_sync_sincronizar_bibliotecas,
    ai_health as if_ai_health,
    ai_processar_pagina as if_ai_processar_pagina,
    ai_listar_modelos as if_ai_listar_modelos,
    ai_listar_resultados_capitulo as if_ai_listar_resultados_capitulo,
    ai_revisar_resultado as if_ai_revisar_resultado,
    ai_processar_e_salvar_pagina as if_ai_processar_e_salvar_pagina,
    CachePaginas,
    source_listar_extratores as if_source_listar_extratores,
    source_importar_extrator as if_source_importar_extrator,
    library_excluir_capitulo as if_library_excluir_capitulo,
    reader_marcar_lido as if_reader_marcar_lido,
    reader_listar_lidos as if_reader_listar_lidos,
    DownloadLogItem,
    ObraResumo,
    library_listar_obras_resumo as if_library_listar_obras_resumo,
    source_listar_historico_downloads as if_source_listar_historico_downloads,
    source_obter_capa as if_source_obter_capa,
};
use mam_source_integration::extractors::{EntradaImportacao, ExtratorInfo, ResultadoImportacao};
use mam_storage::CacheL1Lru;
use mam_ai_engine::{AiEngineManager, ModeloInfo};
use mam_transfer::{DiferencaBibliotecas, RelatorioTransferencia, TransferManager};
use mam_source_integration::SourceIntegrationManager;
use mam_source_integration::mangadex::MangaDexFonte;
use mam_source_integration::{FonteInfo, ObraBuscaResultado, CapituloBuscaResultado, StatusOperacao};

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
async fn library_raiz(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
) -> Result<String, String> {
    if_library_raiz(library_manager.inner().clone()).await
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
    if fonte_id != "mangadex" {
        return mam_interface::source_listar_capitulos_por_conector(
            source_manager.inner().clone(),
            fonte_id,
            obra_id_fonte,
        )
        .await;
    }
    if_source_listar_capitulos(source_manager.inner().clone(), fonte_id, obra_id_fonte).await
}

#[tauri::command]
async fn source_buscar_obras_por_conector(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    conector_id: String,
    consulta: String,
) -> Result<Vec<ObraBuscaResultado>, String> {
    mam_interface::source_buscar_obras_por_conector(
        source_manager.inner().clone(),
        library_manager.inner().clone(),
        conector_id,
        consulta,
    )
    .await
}

#[tauri::command]
async fn source_listar_conectores(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
) -> Result<mam_source_integration::connectors::CatalogoConectores, String> {
    mam_interface::source_listar_conectores(library_manager.inner().clone()).await
}

#[tauri::command]
async fn source_verificar_atualizacao_conectores(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    manifest_url: Option<String>,
) -> Result<mam_source_integration::connectors::StatusAtualizacaoConectores, String> {
    mam_interface::source_verificar_atualizacao_conectores(
        library_manager.inner().clone(),
        manifest_url,
    )
    .await
}

#[tauri::command]
async fn source_atualizar_conectores_remoto(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    manifest_url: Option<String>,
) -> Result<mam_source_integration::connectors::CatalogoConectores, String> {
    mam_interface::source_atualizar_conectores_remoto(library_manager.inner().clone(), manifest_url)
        .await
}

#[tauri::command]
async fn source_importar_conector_json(
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    conteudo_json: String,
) -> Result<mam_source_integration::connectors::CatalogoConectores, String> {
    mam_interface::source_importar_conector_json(library_manager.inner().clone(), conteudo_json)
        .await
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
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_titulo: String,
    capitulos: Vec<CapituloBuscaResultado>,
    capa_url: Option<String>,
) -> Result<String, String> {
    if_source_arquivar_capitulos(
        source_manager.inner().clone(),
        library_manager.inner().clone(),
        obra_titulo,
        capitulos,
        capa_url,
    )
    .await
}

#[tauri::command]
async fn source_iniciar_arquivamento(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_titulo: String,
    capitulos: Vec<CapituloBuscaResultado>,
    capa_url: Option<String>,
) -> Result<String, String> {
    if_source_iniciar_arquivamento(
        source_manager.inner().clone(),
        library_manager.inner().clone(),
        obra_titulo,
        capitulos,
        capa_url,
    )
    .await
}

#[tauri::command]
async fn source_listar_operacoes(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
) -> Result<Vec<StatusOperacao>, String> {
    if_source_listar_operacoes(source_manager.inner().clone()).await
}

#[tauri::command]
async fn source_pausar_operacao(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    id_operacao: String,
) -> Result<(), String> {
    mam_interface::source_pausar_operacao(source_manager.inner().clone(), id_operacao).await
}

#[tauri::command]
async fn source_retomar_operacao(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    id_operacao: String,
) -> Result<(), String> {
    mam_interface::source_retomar_operacao(
        source_manager.inner().clone(),
        library_manager.inner().clone(),
        id_operacao,
    )
    .await
}

#[tauri::command]
async fn source_cancelar_operacao(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    id_operacao: String,
) -> Result<(), String> {
    mam_interface::source_cancelar_operacao(
        source_manager.inner().clone(),
        library_manager.inner().clone(),
        id_operacao,
    )
    .await
}

#[tauri::command]
async fn source_obter_capa(url: String) -> Result<String, String> {
    if_source_obter_capa(url).await
}

#[tauri::command]
async fn source_descobrir_por_link(
    url_obra: String,
    html: Option<String>,
    idioma_preferido: Option<String>,
) -> Result<mam_source_integration::extractors::ObraExtraidaPorLink, String> {
    mam_interface::source_descobrir_por_link(url_obra, html, idioma_preferido).await
}

#[tauri::command]
async fn library_listar_obras_resumo(
    db: tauri::State<'_, Arc<Database>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
) -> Result<Vec<ObraResumo>, String> {
    if_library_listar_obras_resumo(db.inner().clone(), library_manager.inner().clone()).await
}

#[tauri::command]
async fn source_listar_historico_downloads(
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<DownloadLogItem>, String> {
    if_source_listar_historico_downloads(db.inner().clone()).await
}

#[tauri::command]
async fn source_status_operacao(
    source_manager: tauri::State<'_, Arc<SourceIntegrationManager>>,
    id_operacao: String,
) -> Result<Option<StatusOperacao>, String> {
    if_source_status_operacao(source_manager.inner().clone(), id_operacao).await
}

#[tauri::command]
async fn reader_obter_pagina(
    archive_manager: tauri::State<'_, Arc<ArchiveManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    cache: tauri::State<'_, Arc<CachePaginas>>,
    caminho_cbz: String,
    indice: usize,
) -> Result<PaginaExtraida, String> {
    if_reader_obter_pagina(
        archive_manager.inner().clone(),
        library_manager.inner().clone(),
        cache.inner().clone(),
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
async fn reader_listar_marcadores_obra(
    db: tauri::State<'_, Arc<Database>>,
    obra_id: String,
) -> Result<Vec<MarcadorObra>, String> {
    if_reader_listar_marcadores_obra(db.inner().clone(), obra_id).await
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

#[tauri::command]
async fn ai_health(
    ai_manager: tauri::State<'_, Arc<AiEngineManager>>,
) -> Result<serde_json::Value, String> {
    if_ai_health(ai_manager.inner().clone()).await
}

#[tauri::command]
async fn ai_processar_pagina(
    ai_manager: tauri::State<'_, Arc<AiEngineManager>>,
    caminho_imagem: String,
    idioma_origem: String,
) -> Result<serde_json::Value, String> {
    if_ai_processar_pagina(ai_manager.inner().clone(), caminho_imagem, idioma_origem).await
}

#[tauri::command]
async fn ai_listar_modelos(
    ai_manager: tauri::State<'_, Arc<AiEngineManager>>,
    diretorios: Vec<String>,
) -> Result<Vec<ModeloInfo>, String> {
    if_ai_listar_modelos(ai_manager.inner().clone(), diretorios).await
}

#[tauri::command]
async fn ai_listar_resultados_capitulo(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
) -> Result<Vec<ResultadoIA>, String> {
    if_ai_listar_resultados_capitulo(db.inner().clone(), capitulo_id).await
}

#[tauri::command]
async fn ai_revisar_resultado(
    db: tauri::State<'_, Arc<Database>>,
    resultado_id: String,
    resultado_json_revisado: String,
) -> Result<(), String> {
    if_ai_revisar_resultado(db.inner().clone(), resultado_id, resultado_json_revisado).await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn ai_processar_e_salvar_pagina(
    ai_manager: tauri::State<'_, Arc<AiEngineManager>>,
    archive_manager: tauri::State<'_, Arc<ArchiveManager>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    cache: tauri::State<'_, Arc<CachePaginas>>,
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
    caminho_cbz: String,
    indice: usize,
    idioma_origem: String,
    idioma_destino: String,
    caminho_modelo: Option<String>,
) -> Result<ResultadoIA, String> {
    if_ai_processar_e_salvar_pagina(
        ai_manager.inner().clone(),
        archive_manager.inner().clone(),
        library_manager.inner().clone(),
        cache.inner().clone(),
        db.inner().clone(),
        capitulo_id,
        caminho_cbz,
        indice,
        idioma_origem,
        idioma_destino,
        caminho_modelo,
    )
    .await
}

#[tauri::command]
async fn source_listar_extratores() -> Result<Vec<ExtratorInfo>, String> {
    if_source_listar_extratores().await
}

#[tauri::command]
async fn source_importar_extrator(
    db: tauri::State<'_, Arc<Database>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    entrada: EntradaImportacao,
) -> Result<ResultadoImportacao, String> {
    if_source_importar_extrator(db.inner().clone(), library_manager.inner().clone(), entrada).await
}

#[tauri::command]
async fn library_excluir_obra(
    db: tauri::State<'_, Arc<Database>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_id: String,
    remover_arquivos: bool,
) -> Result<(), String> {
    mam_interface::library_excluir_obra(
        db.inner().clone(),
        library_manager.inner().clone(),
        obra_id,
        remover_arquivos,
    )
    .await
}

#[tauri::command]
async fn library_criar_volume_com_capitulos(
    db: tauri::State<'_, Arc<Database>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    obra_id: String,
    numero: i32,
    titulo: Option<String>,
    capitulo_ids: Vec<String>,
) -> Result<Volume, String> {
    mam_interface::library_criar_volume_com_capitulos(
        db.inner().clone(),
        library_manager.inner().clone(),
        obra_id,
        numero,
        titulo,
        capitulo_ids,
    )
    .await
}

#[tauri::command]
async fn library_renomear_volume(
    db: tauri::State<'_, Arc<Database>>,
    volume_id: String,
    numero: i32,
    titulo: Option<String>,
) -> Result<(), String> {
    mam_interface::library_renomear_volume(db.inner().clone(), volume_id, numero, titulo).await
}

#[tauri::command]
async fn library_renomear_capitulo(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
    novo_titulo: Option<String>,
) -> Result<(), String> {
    mam_interface::library_renomear_capitulo(db.inner().clone(), capitulo_id, novo_titulo).await
}

#[tauri::command]
async fn library_listar_historico_exclusoes(
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<DownloadLogItem>, String> {
    mam_interface::library_listar_historico_exclusoes(db.inner().clone()).await
}

#[tauri::command]
async fn library_excluir_volume(
    db: tauri::State<'_, Arc<Database>>,
    volume_id: String,
) -> Result<(), String> {
    mam_interface::library_excluir_volume(db.inner().clone(), volume_id).await
}

#[tauri::command]
async fn library_listar_historico_renomeacoes(
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<DownloadLogItem>, String> {
    mam_interface::library_listar_historico_renomeacoes(db.inner().clone()).await
}

#[tauri::command]
async fn library_limpar_historico_logs(
    db: tauri::State<'_, Arc<Database>>,
    tipo: Option<String>,
) -> Result<(), String> {
    mam_interface::library_limpar_historico_logs(db.inner().clone(), tipo).await
}

#[tauri::command]
async fn reader_remover_marcador(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
    pagina: usize,
) -> Result<(), String> {
    mam_interface::reader_remover_marcador(db.inner().clone(), capitulo_id, pagina).await
}

#[tauri::command]
async fn library_excluir_capitulo(
    db: tauri::State<'_, Arc<Database>>,
    library_manager: tauri::State<'_, Arc<LibraryManager>>,
    capitulo_id: String,
    remover_arquivo: bool,
) -> Result<(), String> {
    if_library_excluir_capitulo(
        db.inner().clone(),
        library_manager.inner().clone(),
        capitulo_id,
        remover_arquivo,
    )
    .await
}

#[tauri::command]
async fn reader_marcar_lido(
    db: tauri::State<'_, Arc<Database>>,
    capitulo_id: String,
    lido: bool,
) -> Result<(), String> {
    if_reader_marcar_lido(db.inner().clone(), capitulo_id, lido).await
}

#[tauri::command]
async fn reader_listar_lidos(
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<String>, String> {
    if_reader_listar_lidos(db.inner().clone()).await
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
      source_iniciar_arquivamento,
      source_listar_operacoes,
      source_status_operacao,
      reader_obter_pagina,
      reader_salvar_progresso,
      reader_obter_progresso,
      reader_alternar_favorito,
      reader_adicionar_marcador,
      reader_remover_marcador,
      reader_listar_marcadores,
      reader_listar_marcadores_obra,
      transfer_copia_integral,
      transfer_seletiva,
      sync_comparar_bibliotecas,
      sync_sincronizar_bibliotecas,
      ai_health,
      ai_processar_pagina,
      ai_listar_modelos,
      ai_listar_resultados_capitulo,
      ai_revisar_resultado,
      ai_processar_e_salvar_pagina,
      source_listar_extratores,
      source_importar_extrator,
      library_excluir_capitulo,
      library_excluir_obra,
      library_criar_volume_com_capitulos,
      library_renomear_volume,
      library_renomear_capitulo,
      library_listar_historico_exclusoes,
      library_excluir_volume,
      library_listar_historico_renomeacoes,
      library_limpar_historico_logs,
      reader_marcar_lido,
      reader_listar_lidos,
      source_obter_capa,
      source_descobrir_por_link,
      source_pausar_operacao,
      source_retomar_operacao,
      source_cancelar_operacao,
      source_buscar_obras_por_conector,
      source_listar_conectores,
      source_verificar_atualizacao_conectores,
      source_atualizar_conectores_remoto,
      source_importar_conector_json,
      library_listar_obras_resumo,
      source_listar_historico_downloads,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Resolve a raiz portátil da biblioteca:
      // 1. MAM_LIBRARY_DIR (variável de ambiente explícita)
      // 2. Pasta portátil ao lado do executável (se existir System/database.db ou MAM_LIBRARY)
      // 3. Fallback para app_data_dir() do sistema
      let app_handle = app.handle();
      let raiz_biblioteca = resolver_raiz_biblioteca(app_handle);
      let sistema_dir = raiz_biblioteca.join("System");
      let _ = std::fs::create_dir_all(&sistema_dir);
      let caminho_db = sistema_dir.join("database.db");

      // Abre o banco de dados SQLite (runtime do Tauri executa setup de forma síncrona)
      let db = tokio::runtime::Runtime::new().unwrap().block_on(async {
        Database::abrir(caminho_db, raiz_biblioteca.clone()).await
          .expect("Falha ao inicializar banco de dados SQLite")
      });

      let db_arc = Arc::new(db);
      app.manage(db_arc.clone());

      // Inicializa o Cache L1 LRU de páginas em memória (máximo 64 páginas ou 64 MiB)
      let cache_paginas: Arc<CachePaginas> = Arc::new(tokio::sync::Mutex::new(
        CacheL1Lru::nova_com_limite_bytes(64, 64 * 1024 * 1024),
      ));
      app.manage(cache_paginas);

      // Inicializa o ChapterManager como estado
      let chapter_manager = ChapterManager::nova(db_arc.clone(), raiz_biblioteca.clone());
      app.manage(Arc::new(chapter_manager));

      // Inicializa o LibraryManager como estado
      let library_manager = LibraryManager::nova(db_arc.clone(), raiz_biblioteca);
      app.manage(Arc::new(library_manager));

      // Inicializa o ArchiveManager como estado
      let archive_manager = ArchiveManager::nova(db_arc.clone());
      app.manage(Arc::new(archive_manager));

      // Inicializa o SourceIntegrationManager com fonte pública autorizada
      let source_manager = SourceIntegrationManager::nova(db_arc);
      let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
        source_manager
          .registrar_fonte(Arc::new(MangaDexFonte::nova()))
          .await
      });
      app.manage(Arc::new(source_manager));

      // Inicializa o TransferManager como estado
      let transfer_manager = TransferManager::nova();
      app.manage(Arc::new(transfer_manager));

      // Inicializa o AiEngineManager como estado (prioriza runtime Python embutido)
      let ai_manager = AiEngineManager::nova(
        AiEngineManager::detectar_executavel_python(),
        AiEngineManager::detectar_diretorio_padrao(),
      );
      app.manage(Arc::new(ai_manager));

      Ok(())
    })
    .on_window_event(|window, event| {
      if let tauri::WindowEvent::CloseRequested { .. } = event {
        if let Some(source_manager) = window.try_state::<Arc<SourceIntegrationManager>>() {
          let sm = source_manager.inner().clone();
          let _ = tauri::async_runtime::block_on(async move {
            sm.pausar_todas_operacoes_ativas().await;
          });
        }
        if let Some(ai_manager) = window.try_state::<Arc<AiEngineManager>>() {
          let ai = ai_manager.inner().clone();
          let _ = tauri::async_runtime::block_on(async move {
            let _ = ai.chamar("shutdown", serde_json::json!({})).await;
          });
        }
        if let Some(db) = window.try_state::<Arc<Database>>() {
          let db_ref = db.inner().clone();
          let _ = tauri::async_runtime::block_on(async move {
            let _ = db_ref.checkpoint_truncar().await;
          });
        }
        #[cfg(windows)]
        {
          use std::os::windows::process::CommandExt;
          let _ = std::process::Command::new("powershell")
            .args([
              "-NoProfile",
              "-WindowStyle",
              "Hidden",
              "-Command",
              "Get-CimInstance Win32_Process -Filter \"Name='msedge.exe' OR Name='chrome.exe'\" | Where-Object { $_.CommandLine -like '*mam_cdp_*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }",
            ])
            .creation_flags(0x08000000)
            .spawn();
        }
      }
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}

/// Resolve a raiz da biblioteca priorizando portabilidade em SSD/mídia externa.
fn resolver_raiz_biblioteca(app_handle: &tauri::AppHandle) -> PathBuf {
    if let Ok(env_dir) = std::env::var("MAM_LIBRARY_DIR") {
        let trimmed = env_dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if exe_dir.join("System/database.db").exists() {
                return exe_dir.to_path_buf();
            }
            let pasta_portatil = exe_dir.join("MAM_LIBRARY");
            if pasta_portatil.exists() {
                return pasta_portatil;
            }
        }
    }

    app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}
