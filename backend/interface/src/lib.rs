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
use std::path::{Path, PathBuf};

use mam_database::Database;
use mam_chapter::ChapterManager;
use mam_archive::{ArchiveManager, PaginaExtraida};
use mam_source_integration::SourceIntegrationManager;
use mam_source_integration::{FonteInfo, ObraBuscaResultado, CapituloBuscaResultado};
use mam_core::ObraId;
use mam_database::models::{Obra, Volume};
use mam_library::{DetalhesObra, LibraryManager};
use mam_transfer::{DiferencaBibliotecas, RelatorioTransferencia, TransferManager};

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

/// Cadastra uma nova obra na biblioteca.
pub async fn library_criar_obra(
    library_manager: Arc<LibraryManager>,
    titulo: String,
    caminho_relativo: String,
    origem: Option<String>,
) -> Result<Obra, String> {
    library_manager
        .criar_obra(titulo, caminho_relativo, origem)
        .await
        .map_err(|e| e.to_string())
}

/// Lista todas as obras cadastradas no catálogo da biblioteca.
pub async fn library_listar_obras(
    library_manager: Arc<LibraryManager>,
) -> Result<Vec<Obra>, String> {
    library_manager
        .listar_obras()
        .await
        .map_err(|e| e.to_string())
}

/// Obtém os detalhes completos de uma obra (volumes + capítulos híbridos).
pub async fn library_obter_detalhes_obra(
    library_manager: Arc<LibraryManager>,
    obra_id: String,
) -> Result<DetalhesObra, String> {
    let id = ObraId::from_string(obra_id);
    library_manager
        .obter_detalhes_obra(id)
        .await
        .map_err(|e| e.to_string())
}

/// Cria um volume vinculado a uma obra existente.
pub async fn library_criar_volume(
    library_manager: Arc<LibraryManager>,
    obra_id: String,
    numero: i32,
    titulo: Option<String>,
    caminho_relativo: String,
) -> Result<Volume, String> {
    let id = ObraId::from_string(obra_id);
    library_manager
        .criar_volume(id, numero, titulo, caminho_relativo)
        .await
        .map_err(|e| e.to_string())
}

/// Atualiza os metadados complementares JSON de uma obra.
pub async fn library_atualizar_metadados(
    library_manager: Arc<LibraryManager>,
    obra_id: String,
    metadados_json: String,
) -> Result<(), String> {
    let id = ObraId::from_string(obra_id);
    library_manager
        .atualizar_metadados_obra(id, &metadados_json)
        .await
        .map_err(|e| e.to_string())
}

/// Cria um arquivo CBZ a partir de um diretório de imagens.
pub async fn archive_criar_cbz(
    archive_manager: Arc<ArchiveManager>,
    imagens_dir: String,
    caminho_destino: String,
) -> Result<String, String> {
    let imagens_path = Path::new(&imagens_dir);
    let destino_path = Path::new(&caminho_destino);

    let resultado = archive_manager
        .criar_cbz_de_imagens(imagens_path, destino_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "CBZ criado: {} ({} páginas, hash: {})",
        resultado.caminho_destino,
        resultado.quantidade_paginas,
        &resultado.hash_sha256[..8]
    ))
}

/// Lê um arquivo CBZ existente e retorna informações das páginas.
pub async fn archive_ler_cbz(
    archive_manager: Arc<ArchiveManager>,
    caminho_cbz: String,
) -> Result<String, String> {
    let cbz = archive_manager
        .ler_cbz(Path::new(&caminho_cbz))
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "CBZ: {} - {} páginas, {} ausentes",
        cbz.caminho,
        cbz.quantidade_paginas,
        cbz.paginas_ausentes.len()
    ))
}

/// Valida a integridade de um arquivo CBZ.
pub async fn archive_validar_cbz(
    archive_manager: Arc<ArchiveManager>,
    caminho_cbz: String,
) -> Result<bool, String> {
    archive_manager
        .validar_cbz(Path::new(&caminho_cbz))
        .await
        .map_err(|e| e.to_string())
}

/// Lista fontes registradas no gerenciador de integração.
pub async fn source_listar_fontes(
    source_manager: Arc<SourceIntegrationManager>,
) -> Result<Vec<FonteInfo>, String> {
    Ok(source_manager.listar_fontes().await)
}

/// Busca obras por título em fontes autorizadas.
pub async fn source_buscar_obras(
    source_manager: Arc<SourceIntegrationManager>,
    consulta: String,
) -> Result<Vec<ObraBuscaResultado>, String> {
    source_manager
        .buscar_obras(&consulta)
        .await
        .map_err(|e| e.to_string())
}

/// Lista capítulos de uma obra em uma fonte específica.
pub async fn source_listar_capitulos(
    source_manager: Arc<SourceIntegrationManager>,
    fonte_id: String,
    obra_id_fonte: String,
) -> Result<Vec<CapituloBuscaResultado>, String> {
    source_manager
        .listar_capitulos(&fonte_id, &obra_id_fonte)
        .await
        .map_err(|e| e.to_string())
}

/// Verifica se um capítulo já está arquivado (detecção de duplicata).
pub async fn source_verificar_duplicatas(
    source_manager: Arc<SourceIntegrationManager>,
    caminho_relativo: String,
) -> Result<bool, String> {
    source_manager
        .capitulo_ja_arquivado(&caminho_relativo)
        .await
        .map_err(|e| e.to_string())
}

/// Inicia uma operação de arquivamento de capítulos.
pub async fn source_arquivar_capitulos(
    source_manager: Arc<SourceIntegrationManager>,
    obra_titulo: String,
    capitulos: Vec<CapituloBuscaResultado>,
) -> Result<String, String> {
    source_manager
        .iniciar_operacao_arquivamento(obra_titulo, capitulos)
        .await
        .map_err(|e| e.to_string())
}

/// Marcador salvo pelo leitor em uma página específica.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarcadorPagina {
    pub pagina: usize,
    pub nota: String,
}

/// Lê uma página específica de um arquivo CBZ sob demanda (resolvendo caminho relativo à biblioteca).
pub async fn reader_obter_pagina(
    archive_manager: Arc<ArchiveManager>,
    library_manager: Arc<LibraryManager>,
    caminho_cbz: String,
    indice: usize,
) -> Result<PaginaExtraida, String> {
    let path = Path::new(&caminho_cbz);
    let caminho_resolvido = if path.is_relative() {
        library_manager.raiz().join(path)
    } else {
        path.to_path_buf()
    };

    archive_manager
        .ler_pagina(&caminho_resolvido, indice)
        .await
        .map_err(|e| e.to_string())
}

/// Salva a última página lida de um capítulo no banco local.
pub async fn reader_salvar_progresso(
    db: Arc<Database>,
    capitulo_id: String,
    pagina: usize,
) -> Result<(), String> {
    let chave = format!("reader:progresso:{}", capitulo_id);
    db.salvar_configuracao(&chave, &pagina.to_string())
        .await
        .map_err(|e| e.to_string())
}

/// Recupera a última página lida de um capítulo (ou 0 se ainda não houver registro).
pub async fn reader_obter_progresso(
    db: Arc<Database>,
    capitulo_id: String,
) -> Result<usize, String> {
    let chave = format!("reader:progresso:{}", capitulo_id);
    let valor = db
        .obter_configuracao(&chave)
        .await
        .map_err(|e| e.to_string())?;
    Ok(valor.and_then(|v| v.parse::<usize>().ok()).unwrap_or(0))
}

/// Alterna o estado de favorito de uma obra ou capítulo e retorna o novo estado.
pub async fn reader_alternar_favorito(
    db: Arc<Database>,
    alvo_id: String,
) -> Result<bool, String> {
    let chave = format!("reader:favorito:{}", alvo_id);
    let atual = db
        .obter_configuracao(&chave)
        .await
        .map_err(|e| e.to_string())?
        .map(|v| v == "true")
        .unwrap_or(false);
    let novo = !atual;
    db.salvar_configuracao(&chave, if novo { "true" } else { "false" })
        .await
        .map_err(|e| e.to_string())?;
    Ok(novo)
}

/// Adiciona um marcador em uma página específica de um capítulo.
pub async fn reader_adicionar_marcador(
    db: Arc<Database>,
    capitulo_id: String,
    pagina: usize,
    nota: String,
) -> Result<(), String> {
    let chave = format!("reader:marcador:{}:{:06}", capitulo_id, pagina);
    db.salvar_configuracao(&chave, &nota)
        .await
        .map_err(|e| e.to_string())
}

/// Lista todos os marcadores salvos para um capítulo.
pub async fn reader_listar_marcadores(
    db: Arc<Database>,
    capitulo_id: String,
) -> Result<Vec<MarcadorPagina>, String> {
    let prefixo = format!("reader:marcador:{}:", capitulo_id);
    let itens = db
        .listar_configuracoes_por_prefixo(&prefixo)
        .await
        .map_err(|e| e.to_string())?;

    Ok(itens
        .into_iter()
        .filter_map(|(chave, nota)| {
            chave
                .rsplit(':')
                .next()
                .and_then(|p| p.parse::<usize>().ok())
                .map(|pagina| MarcadorPagina { pagina, nota })
        })
        .collect())
}

/// Executa cópia integral de uma biblioteca origem para um destino.
pub async fn transfer_copia_integral(
    transfer_manager: Arc<TransferManager>,
    origem: String,
    destino: String,
) -> Result<RelatorioTransferencia, String> {
    transfer_manager
        .copia_integral(Path::new(&origem), Path::new(&destino))
        .await
        .map_err(|e| e.to_string())
}

/// Executa transferência seletiva de caminhos relativos para um destino.
pub async fn transfer_seletiva(
    transfer_manager: Arc<TransferManager>,
    origem: String,
    destino: String,
    caminhos_relativos: Vec<String>,
) -> Result<RelatorioTransferencia, String> {
    transfer_manager
        .transferencia_seletiva(Path::new(&origem), Path::new(&destino), &caminhos_relativos)
        .await
        .map_err(|e| e.to_string())
}

/// Compara duas bibliotecas identificando arquivos ausentes, diferentes e idênticos.
pub async fn sync_comparar_bibliotecas(
    transfer_manager: Arc<TransferManager>,
    origem: String,
    destino: String,
) -> Result<DiferencaBibliotecas, String> {
    transfer_manager
        .comparar_bibliotecas(Path::new(&origem), Path::new(&destino))
        .await
        .map_err(|e| e.to_string())
}

/// Sincroniza duas bibliotecas com controle explícito de sobrescrita.
pub async fn sync_sincronizar_bibliotecas(
    transfer_manager: Arc<TransferManager>,
    origem: String,
    destino: String,
    permitir_sobrescrita: bool,
) -> Result<RelatorioTransferencia, String> {
    transfer_manager
        .sincronizar_bibliotecas(Path::new(&origem), Path::new(&destino), permitir_sobrescrita)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
