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
use std::path::Path;

use mam_database::Database;
use mam_chapter::ChapterManager;
use mam_archive::{ArchiveManager, PaginaExtraida};
use mam_source_integration::SourceIntegrationManager;
use mam_source_integration::{FonteInfo, ObraBuscaResultado, CapituloBuscaResultado, StatusOperacao};
use mam_source_integration::extractors::{EntradaImportacao, ExtratorInfo, RegistroExtratores, ResultadoImportacao};
use mam_storage::CacheL1Lru;

/// Cache L1 LRU em memória para páginas extraídas sob demanda.
pub type CachePaginas = tokio::sync::Mutex<CacheL1Lru<String, PaginaExtraida>>;
use mam_core::{CapituloId, ObraId};
use mam_database::models::{Obra, ResultadoIA, Volume};
use mam_library::{DetalhesObra, LibraryManager};
use mam_transfer::{DiferencaBibliotecas, RelatorioTransferencia, TransferManager};
use mam_ai_engine::{AiEngineManager, ModeloInfo};

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
pub async fn library_raiz(library_manager: Arc<LibraryManager>) -> Result<String, String> {
    Ok(library_manager.raiz().to_string_lossy().to_string())
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

/// Busca obras por título usando um conector específico (`mangadex`, `mangafire` ou conector web com `url_busca`).
pub async fn source_buscar_obras_por_conector(
    source_manager: Arc<SourceIntegrationManager>,
    library_manager: Arc<LibraryManager>,
    conector_id: String,
    consulta: String,
) -> Result<Vec<ObraBuscaResultado>, String> {
    if conector_id == "mangadex" {
        return source_buscar_obras(source_manager, consulta).await;
    }
    let cat = mam_source_integration::connectors::GerenciadorConectores::carregar(Some(
        library_manager.raiz(),
    ));
    let conector = cat
        .sites
        .into_iter()
        .find(|s| s.id == conector_id)
        .ok_or_else(|| format!("Conector '{}' não encontrado", conector_id))?;
    mam_source_integration::connectors::GerenciadorConectores::buscar_obras_via_cdp(
        &conector, &consulta,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Lista capítulos de uma obra a partir de um conector (`mangadex` via API ou conector web via CDP/HTML).
pub async fn source_listar_capitulos_por_conector(
    source_manager: Arc<SourceIntegrationManager>,
    fonte_id: String,
    obra_id_fonte: String,
) -> Result<Vec<CapituloBuscaResultado>, String> {
    if fonte_id == "mangadex" {
        return source_listar_capitulos(source_manager, fonte_id, obra_id_fonte).await;
    }
    let res = mam_source_integration::extractors::descobrir_capitulos_por_link_com_opcoes(
        &obra_id_fonte,
        None,
        Some("pt-br"),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(res.capitulos)
}

/// Lista os conectores declarativos ativos (`System/connectors/sites.json` > padrão).
pub async fn source_listar_conectores(
    library_manager: Arc<LibraryManager>,
) -> Result<mam_source_integration::connectors::CatalogoConectores, String> {
    Ok(mam_source_integration::connectors::GerenciadorConectores::carregar(Some(
        library_manager.raiz(),
    )))
}

/// Verifica se há atualização disponível para os conectores no manifesto oficial.
pub async fn source_verificar_atualizacao_conectores(
    library_manager: Arc<LibraryManager>,
    manifest_url: Option<String>,
) -> Result<mam_source_integration::connectors::StatusAtualizacaoConectores, String> {
    mam_source_integration::connectors::GerenciadorConectores::verificar_atualizacao_remota(
        Some(library_manager.raiz()),
        manifest_url.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Baixa e aplica atomicamente a atualização de conectores preservando toda a `MAM_LIBRARY`.
pub async fn source_atualizar_conectores_remoto(
    library_manager: Arc<LibraryManager>,
    manifest_url: Option<String>,
) -> Result<mam_source_integration::connectors::CatalogoConectores, String> {
    mam_source_integration::connectors::GerenciadorConectores::atualizar_remoto(
        library_manager.raiz(),
        manifest_url.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Importa manualmente um catálogo JSON de conectores (modo offline) preservando a biblioteca.
pub async fn source_importar_conector_json(
    library_manager: Arc<LibraryManager>,
    conteudo_json: String,
) -> Result<mam_source_integration::connectors::CatalogoConectores, String> {
    mam_source_integration::connectors::GerenciadorConectores::salvar_catalogo_atomico(
        library_manager.raiz(),
        &conteudo_json,
        None,
    )
    .map_err(|e| e.to_string())
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

/// Inicia uma operação de arquivamento real de capítulos para a biblioteca.
pub async fn source_arquivar_capitulos(
    source_manager: Arc<SourceIntegrationManager>,
    library_manager: Arc<LibraryManager>,
    obra_titulo: String,
    capitulos: Vec<CapituloBuscaResultado>,
    capa_url: Option<String>,
) -> Result<String, String> {
    source_manager
        .arquivar_para_biblioteca_com_capa(
            library_manager.raiz(),
            obra_titulo,
            capitulos,
            capa_url,
        )
        .await
        .map_err(|e| e.to_string())
}

/// Inicia uma operação de arquivamento em segundo plano e retorna imediatamente o `id_operacao`.
pub async fn source_iniciar_arquivamento(
    source_manager: Arc<SourceIntegrationManager>,
    library_manager: Arc<LibraryManager>,
    obra_titulo: String,
    capitulos: Vec<CapituloBuscaResultado>,
    capa_url: Option<String>,
) -> Result<String, String> {
    let id_operacao = source_manager
        .iniciar_operacao_arquivamento(obra_titulo.clone(), capitulos.clone())
        .await
        .map_err(|e| e.to_string())?;

    let id_clone = id_operacao.clone();
    let raiz = library_manager.raiz().to_path_buf();
    tokio::spawn(async move {
        let _ = source_manager
            .executar_arquivamento_operacao(&id_clone, &raiz, obra_titulo, capitulos, capa_url)
            .await;
    });

    Ok(id_operacao)
}

/// Lista todas as operações de arquivamento (em andamento, pausadas e concluídas).
pub async fn source_listar_operacoes(
    source_manager: Arc<SourceIntegrationManager>,
) -> Result<Vec<StatusOperacao>, String> {
    let _ = source_manager.carregar_operacoes_pausadas_do_banco().await;
    Ok(source_manager.listar_operacoes().await)
}

/// Pausa uma operação de arquivamento ativa.
pub async fn source_pausar_operacao(
    source_manager: Arc<SourceIntegrationManager>,
    id_operacao: String,
) -> Result<(), String> {
    source_manager
        .pausar_operacao(&id_operacao)
        .await
        .map_err(|e| e.to_string())
}

/// Retoma (despausa) uma operação de arquivamento pausada (inclusive após reinício do app).
pub async fn source_retomar_operacao(
    source_manager: Arc<SourceIntegrationManager>,
    library_manager: Arc<LibraryManager>,
    id_operacao: String,
) -> Result<(), String> {
    if let Some((obra_titulo, capitulos, capa_url)) = source_manager
        .preparar_retomada_operacao(&id_operacao)
        .await
        .map_err(|e| e.to_string())?
    {
        let raiz = library_manager.raiz().to_path_buf();
        tokio::spawn(async move {
            let _ = source_manager
                .executar_arquivamento_operacao(&id_operacao, &raiz, obra_titulo, capitulos, capa_url)
                .await;
        });
    }
    Ok(())
}

/// Cancela uma operação de arquivamento aplicando rollback completo (estilo Docker).
pub async fn source_cancelar_operacao(
    source_manager: Arc<SourceIntegrationManager>,
    library_manager: Arc<LibraryManager>,
    id_operacao: String,
) -> Result<(), String> {
    source_manager
        .cancelar_operacao(&id_operacao, library_manager.raiz())
        .await
        .map_err(|e| e.to_string())
}

/// Baixa uma imagem de capa pelo backend Rust (evitando bloqueio de Referer/CSP no WebView).
pub async fn source_obter_capa(url: String) -> Result<String, String> {
    mam_source_integration::baixar_imagem_data_url(&url)
        .await
        .map_err(|e| e.to_string())
}

/// Descobre a obra e seus capítulos a partir do link principal da obra (ex.: MangaFire / MangaLivre).
pub async fn source_descobrir_por_link(
    url_obra: String,
    html: Option<String>,
    idioma_preferido: Option<String>,
) -> Result<mam_source_integration::extractors::ObraExtraidaPorLink, String> {
    mam_source_integration::extractors::descobrir_capitulos_por_link_com_opcoes(
        &url_obra,
        html.as_deref(),
        idioma_preferido.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Resumo de uma obra na Biblioteca com contagem de capítulos e capa em destaque.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObraResumo {
    pub obra: Obra,
    pub total_capitulos: usize,
    pub capa_data_url: Option<String>,
}

/// Lista todas as obras da Biblioteca com total de capítulos e capa local em destaque.
pub async fn library_listar_obras_resumo(
    db: Arc<Database>,
    library_manager: Arc<LibraryManager>,
) -> Result<Vec<ObraResumo>, String> {
    let obras = db.listar_obras().await.map_err(|e| e.to_string())?;
    let contagens = db
        .contar_capitulos_por_obra()
        .await
        .map_err(|e| e.to_string())?;
    let raiz = library_manager.raiz();

    let mut resumos = Vec::with_capacity(obras.len());
    for obra in obras {
        let total_capitulos = contagens.get(&obra.id.to_string()).copied().unwrap_or(0);
        let cover_path = raiz.join(obra.caminho_relativo.as_str()).join("cover.jpg");
        let capa_data_url = if cover_path.is_file() {
            let path_clone = cover_path.clone();
            tokio::task::spawn_blocking(move || std::fs::read(&path_clone).ok())
                .await
                .ok()
                .flatten()
                .map(|bytes| {
                    format!(
                        "data:image/jpeg;base64,{}",
                        mam_source_integration::codificar_base64(&bytes)
                    )
                })
        } else if total_capitulos > 0 {
            let mut fallback_url = None;
            if let Ok(caps) = db.listar_capitulos_obra(obra.id).await {
                if let Some(primeiro_cap) = caps.first() {
                    let cbz_path = raiz.join(primeiro_cap.caminho_relativo.as_str());
                    if cbz_path.is_file() {
                        let arch = ArchiveManager::nova(db.clone());
                        if let Ok(pag) = arch.ler_pagina(&cbz_path, 0).await {
                            fallback_url = Some(format!(
                                "data:{};base64,{}",
                                pag.mime_type, pag.dados_base64
                            ));
                        }
                    }
                }
            }
            fallback_url
        } else {
            None
        };
        resumos.push(ObraResumo {
            obra,
            total_capitulos,
            capa_data_url,
        });
    }
    Ok(resumos)
}

/// Registro visual de um download concluído na biblioteca.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadLogItem {
    pub id: String,
    pub nome: String,
    pub pasta_destino: String,
    pub data_hora: String,
    pub operacao_id: Option<String>,
    pub obra_id: Option<String>,
    pub obra_titulo: Option<String>,
    pub capitulo_id: Option<String>,
    pub caminho_relativo: Option<String>,
}

/// Lista os downloads registrados no histórico do SQLite.
pub async fn source_listar_historico_downloads(
    db: Arc<Database>,
) -> Result<Vec<DownloadLogItem>, String> {
    let rows = db
        .listar_historico_downloads()
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(id, alvo, detalhes, criado_em)| {
            let parsed = detalhes
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
            let nome = parsed
                .as_ref()
                .and_then(|v| v["nome"].as_str())
                .map(|s| s.to_string())
                .or_else(|| alvo.clone())
                .unwrap_or_else(|| "Download".to_string());
            let pasta_destino = parsed
                .as_ref()
                .and_then(|v| v["pasta"].as_str())
                .map(|s| s.to_string())
                .or_else(|| alvo.clone())
                .unwrap_or_default();
            let operacao_id = parsed
                .as_ref()
                .and_then(|v| v["operacao_id"].as_str())
                .map(|s| s.to_string());
            let obra_id = parsed
                .as_ref()
                .and_then(|v| v["obra_id"].as_str())
                .map(|s| s.to_string());
            let obra_titulo = parsed
                .as_ref()
                .and_then(|v| v["obra_titulo"].as_str())
                .map(|s| s.to_string());
            let capitulo_id = parsed
                .as_ref()
                .and_then(|v| v["capitulo_id"].as_str())
                .map(|s| s.to_string());
            let caminho_relativo = parsed
                .as_ref()
                .and_then(|v| v["caminho_relativo"].as_str())
                .map(|s| s.to_string())
                .or(alvo);

            DownloadLogItem {
                id,
                nome,
                pasta_destino,
                data_hora: criado_em,
                operacao_id,
                obra_id,
                obra_titulo,
                capitulo_id,
                caminho_relativo,
            }
        })
        .collect())
}

/// Consulta o status de uma operação de arquivamento.
pub async fn source_status_operacao(
    source_manager: Arc<SourceIntegrationManager>,
    id_operacao: String,
) -> Result<Option<StatusOperacao>, String> {
    source_manager
        .status_operacao(&id_operacao)
        .await
        .map_err(|e| e.to_string())
}

/// Marcador salvo pelo leitor em uma página específica.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarcadorPagina {
    pub pagina: usize,
    pub nota: String,
}

/// Lê uma página específica de um arquivo CBZ sob demanda, utilizando Cache L1 LRU em memória.
pub async fn reader_obter_pagina(
    archive_manager: Arc<ArchiveManager>,
    library_manager: Arc<LibraryManager>,
    cache: Arc<CachePaginas>,
    caminho_cbz: String,
    indice: usize,
) -> Result<PaginaExtraida, String> {
    let chave_cache = format!("{}#{}", caminho_cbz, indice);
    {
        let mut guard = cache.lock().await;
        if let Some(pagina) = guard.obter(&chave_cache) {
            return Ok(pagina.clone());
        }
    }

    let path = Path::new(&caminho_cbz);
    let caminho_resolvido = if path.is_relative() {
        library_manager.raiz().join(path)
    } else {
        path.to_path_buf()
    };

    let extraida = archive_manager
        .ler_pagina(&caminho_resolvido, indice)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut guard = cache.lock().await;
        let peso_bytes = extraida.dados_base64.len();
        guard.inserir_com_peso(chave_cache, extraida.clone(), peso_bytes);
    }

    Ok(extraida)
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

/// Remove o marcador de uma página específica do capítulo.
pub async fn reader_remover_marcador(
    db: Arc<Database>,
    capitulo_id: String,
    pagina: usize,
) -> Result<(), String> {
    let chave = format!("reader:marcador:{}:{:06}", capitulo_id, pagina);
    db.remover_configuracao(&chave)
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

/// Marcador de uma obra agrupado por capítulo.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarcadorObra {
    pub capitulo_id: String,
    pub pagina: usize,
    pub nota: String,
}

/// Lista todos os marcadores de uma obra em uma única consulta SQL.
pub async fn reader_listar_marcadores_obra(
    db: Arc<Database>,
    obra_id: String,
) -> Result<Vec<MarcadorObra>, String> {
    let oid = ObraId::from_string(obra_id);
    let itens = db
        .listar_marcadores_por_obra(oid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(itens
        .into_iter()
        .map(|(capitulo_id, pagina, nota)| MarcadorObra {
            capitulo_id,
            pagina,
            nota,
        })
        .collect())
}

/// Comprime toda a biblioteca atual em um arquivo container `.zip` nomeável para distribuição.
pub async fn transfer_empacotar_container(
    db: Arc<Database>,
    library_manager: Arc<LibraryManager>,
    transfer_manager: Arc<TransferManager>,
    pasta_destino: Option<String>,
    nome_container: String,
) -> Result<mam_transfer::RelatorioContainer, String> {
    let _ = db.checkpoint_truncar().await;
    let raiz = library_manager.raiz();
    let destino = match pasta_destino {
        Some(p) if !p.trim().is_empty() => std::path::PathBuf::from(p.trim()),
        _ => raiz.join("Exports"),
    };
    transfer_manager
        .empacotar_container(raiz, &destino, &nome_container)
        .await
        .map_err(|e| e.to_string())
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

/// Verifica saúde e provedores do Manga AI Engine.
pub async fn ai_health(
    ai_manager: Arc<AiEngineManager>,
) -> Result<serde_json::Value, String> {
    ai_manager.ping().await.map_err(|e| e.to_string())
}

/// Executa processamento OCR local em uma página sem alterar o CBZ original.
pub async fn ai_processar_pagina(
    ai_manager: Arc<AiEngineManager>,
    caminho_imagem: String,
    idioma_origem: String,
) -> Result<serde_json::Value, String> {
    ai_manager
        .processar_ocr(Path::new(&caminho_imagem), &idioma_origem)
        .await
        .map_err(|e| e.to_string())
}

/// Lista modelos opcionais instalados localmente sem downloads automáticos.
pub async fn ai_listar_modelos(
    ai_manager: Arc<AiEngineManager>,
    diretorios: Vec<String>,
) -> Result<Vec<ModeloInfo>, String> {
    ai_manager
        .listar_modelos(diretorios)
        .await
        .map_err(|e| e.to_string())
}

/// Lista resultados de IA salvos para um capítulo (com estado `nao_revisado` ou `revisado`).
pub async fn ai_listar_resultados_capitulo(
    db: Arc<Database>,
    capitulo_id: String,
) -> Result<Vec<ResultadoIA>, String> {
    let id = CapituloId::from_string(capitulo_id);
    db.listar_resultados_ia_capitulo(id)
        .await
        .map_err(|e| e.to_string())
}

/// Salva ou revisa manualmente um resultado de IA sem alterar o CBZ original.
pub async fn ai_revisar_resultado(
    db: Arc<Database>,
    resultado_id: String,
    resultado_json_revisado: String,
) -> Result<(), String> {
    db.revisar_resultado_ia(&resultado_id, &resultado_json_revisado)
        .await
        .map_err(|e| e.to_string())
}

/// Calcula chave determinística de cache de IA (L3) com hash SHA-256 da imagem e parâmetros do pipeline.
pub fn calcular_chave_cache_ia(
    bytes_imagem: &[u8],
    modelo: &str,
    versao_pipeline: &str,
    idioma_origem: &str,
    idioma_destino: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes_imagem);
    hasher.update(b"|");
    hasher.update(modelo.as_bytes());
    hasher.update(b"|");
    hasher.update(versao_pipeline.as_bytes());
    hasher.update(b"|");
    hasher.update(idioma_origem.as_bytes());
    hasher.update(b"|");
    hasher.update(idioma_destino.as_bytes());
    format!("ai:cache:{:x}", hasher.finalize())
}

/// Processa uma página de um CBZ com o Manga AI Engine (com reuso de cache L3 por SHA-256)
/// e persiste em `resultados_ia` (`nao_revisado`), preservando o `.cbz` original intacto.
#[allow(clippy::too_many_arguments)]
pub async fn ai_processar_e_salvar_pagina(
    ai_manager: Arc<AiEngineManager>,
    archive_manager: Arc<ArchiveManager>,
    library_manager: Arc<LibraryManager>,
    cache: Arc<CachePaginas>,
    db: Arc<Database>,
    capitulo_id: String,
    caminho_cbz: String,
    indice: usize,
    idioma_origem: String,
    idioma_destino: String,
    caminho_modelo: Option<String>,
) -> Result<ResultadoIA, String> {
    let pagina = reader_obter_pagina(archive_manager, library_manager, cache, caminho_cbz, indice).await?;
    let bytes = decodificar_base64(&pagina.dados_base64);
    let modelo_chave = caminho_modelo.as_deref().unwrap_or("padrao");
    let chave_cache_ia = calcular_chave_cache_ia(
        &bytes,
        modelo_chave,
        "1.0",
        &idioma_origem,
        &idioma_destino,
    );

    let (provedor, resultado_json) = if let Some(cached_json) = db
        .obter_configuracao(&chave_cache_ia)
        .await
        .map_err(|e| e.to_string())?
    {
        let parsed: serde_json::Value =
            serde_json::from_str(&cached_json).unwrap_or(serde_json::Value::Null);
        let prov = parsed["provedor"]
            .as_str()
            .unwrap_or("cache-ia")
            .to_string();
        (prov, cached_json)
    } else {
        let ext = pagina.nome.rsplit('.').next().unwrap_or("jpg");
        let temp_img =
            std::env::temp_dir().join(format!("mam_ai_page_{}.{}", uuid::Uuid::new_v4(), ext));
        std::fs::write(&temp_img, &bytes).map_err(|e| e.to_string())?;

        let resultado_val = ai_manager
            .processar_pagina(
                &temp_img,
                &idioma_origem,
                &idioma_destino,
                caminho_modelo.as_deref(),
            )
            .await;

        let _ = std::fs::remove_file(&temp_img);
        let resultado = resultado_val.map_err(|e| e.to_string())?;
        let prov = resultado["provedor"]
            .as_str()
            .unwrap_or("cpu-fallback")
            .to_string();
        let json_str = serde_json::to_string(&resultado).map_err(|e| e.to_string())?;
        let _ = db.salvar_configuracao(&chave_cache_ia, &json_str).await;
        (prov, json_str)
    };

    let cap_id = CapituloId::from_string(capitulo_id);
    db.salvar_resultado_ia(
        cap_id,
        indice as i32,
        &provedor,
        "1.0",
        "process_page",
        &resultado_json,
        Some(&idioma_origem),
        Some(&idioma_destino),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Processa todas as páginas de um capítulo inteiro com o Manga AI Engine (reaproveitando cache L3 por SHA-256).
#[allow(clippy::too_many_arguments)]
pub async fn ai_processar_capitulo(
    ai_manager: Arc<AiEngineManager>,
    archive_manager: Arc<ArchiveManager>,
    library_manager: Arc<LibraryManager>,
    cache: Arc<CachePaginas>,
    db: Arc<Database>,
    capitulo_id: String,
    caminho_cbz: String,
    total_paginas: usize,
    idioma_origem: String,
    idioma_destino: String,
    caminho_modelo: Option<String>,
) -> Result<Vec<ResultadoIA>, String> {
    let mut resultados = Vec::with_capacity(total_paginas);
    for indice in 0..total_paginas {
        let res = ai_processar_e_salvar_pagina(
            ai_manager.clone(),
            archive_manager.clone(),
            library_manager.clone(),
            cache.clone(),
            db.clone(),
            capitulo_id.clone(),
            caminho_cbz.clone(),
            indice,
            idioma_origem.clone(),
            idioma_destino.clone(),
            caminho_modelo.clone(),
        )
        .await?;
        resultados.push(res);
    }
    Ok(resultados)
}

fn decodificar_base64(input: &str) -> Vec<u8> {
    let mut saida = Vec::with_capacity((input.len() * 3) / 4);
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            saida.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    saida
}

/// Lista os extratores modulares disponíveis (MangaDex, MangaFire, Genérico, etc.).
pub async fn source_listar_extratores() -> Result<Vec<ExtratorInfo>, String> {
    Ok(RegistroExtratores::padrao().listar())
}

/// Importa um capítulo usando o extrator modular adequado (URL, HTML ou lista ordenada de URLs).
pub async fn source_importar_extrator(
    db: Arc<Database>,
    library_manager: Arc<LibraryManager>,
    entrada: EntradaImportacao,
) -> Result<ResultadoImportacao, String> {
    RegistroExtratores::padrao()
        .importar_capitulo(&db, library_manager.raiz(), &entrada)
        .await
        .map_err(|e| e.to_string())
}

/// Exclui um capítulo arquivado do catálogo (e opcionalmente o arquivo `.cbz` se não houver outra referência).
pub async fn library_excluir_capitulo(
    db: Arc<Database>,
    library_manager: Arc<LibraryManager>,
    capitulo_id: String,
    remover_arquivo: bool,
) -> Result<(), String> {
    let id = CapituloId::from_string(capitulo_id);
    let cap_info = db.obter_capitulo_por_id(id).await.map_err(|e| e.to_string())?;
    let caminho_orfao = db.excluir_capitulo(id).await.map_err(|e| e.to_string())?;
    if remover_arquivo {
        if let Some(ref rel) = caminho_orfao {
            let abs = library_manager.raiz().join(rel);
            if abs.exists() {
                let _ = std::fs::remove_file(abs);
            }
        }
    }
    if let Some(cap) = cap_info {
        let pasta = std::path::Path::new(cap.caminho_relativo.as_str())
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| cap.caminho_relativo.as_str().to_string());
        let titulo_cap = cap
            .titulo_original
            .as_deref()
            .filter(|t| !t.trim().is_empty())
            .map(|t| format!("Cap. {} — {}", cap.numero, t))
            .unwrap_or_else(|| format!("Cap. {}", cap.numero));
        let detalhes = serde_json::json!({
            "nome": titulo_cap,
            "pasta": pasta,
            "caminho_relativo": cap.caminho_relativo.as_str(),
        })
        .to_string();
        let _ = db
            .registrar_operacao(
                "exclusao".to_string(),
                Some(cap.caminho_relativo.as_str().to_string()),
                Some(detalhes),
                None,
            )
            .await;
    }
    Ok(())
}

/// Exclui uma obra inteira da biblioteca (banco e pasta local) e registra no log de exclusões.
pub async fn library_excluir_obra(
    db: Arc<Database>,
    library_manager: Arc<LibraryManager>,
    obra_id: String,
    remover_arquivos: bool,
) -> Result<(), String> {
    let id = ObraId::from_string(obra_id);
    let info = db.excluir_obra(id).await.map_err(|e| e.to_string())?;
    if let Some((titulo, caminho_rel)) = info {
        if remover_arquivos && !caminho_rel.trim().is_empty() {
            let abs = library_manager.raiz().join(&caminho_rel);
            if abs.exists() {
                let _ = std::fs::remove_dir_all(abs);
            }
        }
        let detalhes = serde_json::json!({
            "nome": format!("Obra: {}", titulo),
            "pasta": caminho_rel,
            "caminho_relativo": caminho_rel,
        })
        .to_string();
        let _ = db
            .registrar_operacao("exclusao".to_string(), Some(caminho_rel), Some(detalhes), None)
            .await;
    }
    Ok(())
}

/// Cria um volume com número de ordem e nome, vinculando os capítulos selecionados a ele.
pub async fn library_criar_volume_com_capitulos(
    db: Arc<Database>,
    library_manager: Arc<LibraryManager>,
    obra_id: String,
    numero: i32,
    titulo: Option<String>,
    capitulo_ids: Vec<String>,
) -> Result<Volume, String> {
    let id_obra = ObraId::from_string(obra_id);
    let detalhes = library_manager
        .obter_detalhes_obra(id_obra)
        .await
        .map_err(|e| e.to_string())?;
    let caminho_rel = format!(
        "{}/Vol_{:03}",
        detalhes.obra.caminho_relativo.as_str().trim_end_matches('/'),
        numero.max(0)
    );
    let vol = library_manager
        .criar_volume(id_obra, numero, titulo, caminho_rel)
        .await
        .map_err(|e| e.to_string())?;
    let ids: Vec<CapituloId> = capitulo_ids
        .into_iter()
        .map(CapituloId::from_string)
        .collect();
    if !ids.is_empty() {
        db.vincular_capitulos_volume(Some(vol.id), &ids)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(vol)
}

/// Renomeia um volume (nome e número de ordem) e registra no log de renomeações.
pub async fn library_renomear_volume(
    db: Arc<Database>,
    volume_id: String,
    numero: i32,
    titulo: Option<String>,
) -> Result<(), String> {
    db.atualizar_volume(mam_core::VolumeId::from_string(volume_id), numero, titulo.clone())
        .await
        .map_err(|e| e.to_string())?;
    let nome = match titulo.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(t) => format!("Volume {} — {}", numero, t),
        None => format!("Volume {}", numero),
    };
    let detalhes = serde_json::json!({ "nome": nome, "pasta": "Volume" }).to_string();
    let _ = db
        .registrar_operacao("renomeacao".to_string(), None, Some(detalhes), None)
        .await;
    Ok(())
}

/// Exclui um volume da biblioteca (desvinculando seus capítulos sem remover os arquivos) e registra no log.
pub async fn library_excluir_volume(
    db: Arc<Database>,
    volume_id: String,
) -> Result<(), String> {
    let info = db
        .excluir_volume(mam_core::VolumeId::from_string(volume_id))
        .await
        .map_err(|e| e.to_string())?;
    if let Some((numero, titulo)) = info {
        let nome = match titulo.as_deref().filter(|t| !t.trim().is_empty()) {
            Some(t) => format!("Volume {} — {}", numero, t),
            None => format!("Volume {}", numero),
        };
        let detalhes = serde_json::json!({ "nome": nome, "pasta": "Volume" }).to_string();
        let _ = db
            .registrar_operacao("exclusao".to_string(), None, Some(detalhes), None)
            .await;
    }
    Ok(())
}

/// Renomeia o título de um capítulo baixado e registra no log de renomeações.
pub async fn library_renomear_capitulo(
    db: Arc<Database>,
    capitulo_id: String,
    novo_titulo: Option<String>,
) -> Result<(), String> {
    let id = CapituloId::from_string(capitulo_id);
    let cap_info = db.obter_capitulo_por_id(id).await.map_err(|e| e.to_string())?;
    db.renomear_capitulo(id, novo_titulo.clone())
        .await
        .map_err(|e| e.to_string())?;
    if let Some(cap) = cap_info {
        let pasta = std::path::Path::new(cap.caminho_relativo.as_str())
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| cap.caminho_relativo.as_str().to_string());
        let nome = match novo_titulo.as_deref().filter(|t| !t.trim().is_empty()) {
            Some(t) => format!("Cap. {} → {}", cap.numero, t),
            None => format!("Cap. {} (título removido)", cap.numero),
        };
        let detalhes = serde_json::json!({
            "nome": nome,
            "pasta": pasta,
            "caminho_relativo": cap.caminho_relativo.as_str(),
        })
        .to_string();
        let _ = db
            .registrar_operacao(
                "renomeacao".to_string(),
                Some(cap.caminho_relativo.as_str().to_string()),
                Some(detalhes),
                None,
            )
            .await;
    }
    Ok(())
}

fn map_historico_rows(
    rows: Vec<(String, Option<String>, Option<String>, String)>,
    fallback_nome: &str,
) -> Vec<DownloadLogItem> {
    rows.into_iter()
        .map(|(id, alvo, detalhes, criado_em)| {
            let parsed = detalhes
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
            let nome = parsed
                .as_ref()
                .and_then(|v| v["nome"].as_str())
                .map(|s| s.to_string())
                .or_else(|| alvo.clone())
                .unwrap_or_else(|| fallback_nome.to_string());
            let pasta_destino = parsed
                .as_ref()
                .and_then(|v| v["pasta"].as_str())
                .map(|s| s.to_string())
                .or_else(|| alvo.clone())
                .unwrap_or_default();
            DownloadLogItem {
                id,
                nome,
                pasta_destino,
                data_hora: criado_em,
                operacao_id: None,
                obra_id: None,
                obra_titulo: None,
                capitulo_id: None,
                caminho_relativo: alvo,
            }
        })
        .collect()
}

/// Lista os registros de exclusão no histórico do SQLite.
pub async fn library_listar_historico_exclusoes(
    db: Arc<Database>,
) -> Result<Vec<DownloadLogItem>, String> {
    let rows = db
        .listar_historico_exclusoes()
        .await
        .map_err(|e| e.to_string())?;
    Ok(map_historico_rows(rows, "Exclusão"))
}

/// Lista os registros de renomeação no histórico do SQLite.
pub async fn library_listar_historico_renomeacoes(
    db: Arc<Database>,
) -> Result<Vec<DownloadLogItem>, String> {
    let rows = db
        .listar_historico_renomeacoes()
        .await
        .map_err(|e| e.to_string())?;
    Ok(map_historico_rows(rows, "Renomeação"))
}

/// Limpa o histórico de operações no SQLite (de um tipo específico ou todos).
pub async fn library_limpar_historico_logs(
    db: Arc<Database>,
    tipo: Option<String>,
) -> Result<(), String> {
    db.limpar_historico(tipo.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Marca ou desmarca um capítulo como lido na biblioteca.
pub async fn reader_marcar_lido(
    db: Arc<Database>,
    capitulo_id: String,
    lido: bool,
) -> Result<(), String> {
    let chave = format!("reader:lido:{}", capitulo_id);
    db.salvar_configuracao(&chave, if lido { "true" } else { "false" })
        .await
        .map_err(|e| e.to_string())
}

/// Lista os IDs de capítulos marcados como lidos.
pub async fn reader_listar_lidos(db: Arc<Database>) -> Result<Vec<String>, String> {
    let itens = db
        .listar_configuracoes_por_prefixo("reader:lido:")
        .await
        .map_err(|e| e.to_string())?;
    Ok(itens
        .into_iter()
        .filter(|(_, v)| v == "true")
        .filter_map(|(k, _)| k.strip_prefix("reader:lido:").map(|s| s.to_string()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn portabilidade_entre_unidades_ou_diretorios_distintos() {
        let base = env::temp_dir().join(format!("mam_test_portabilidade_{}", Uuid::new_v4()));
        let ssd_pc_a = base.join("PC_A_Unidade_D");
        let ssd_pc_b = base.join("PC_B_Unidade_E_Outro_Caminho");
        let imgs_tmp = base.join("imgs_origem");

        fs::create_dir_all(ssd_pc_a.join("System")).unwrap();
        fs::create_dir_all(ssd_pc_a.join("Manga/Obra Portatil")).unwrap();
        fs::create_dir_all(&imgs_tmp).unwrap();
        fs::write(imgs_tmp.join("001.jpg"), b"FAKEJPGDATA").unwrap();

        // 1. Cria biblioteca, CBZ, obra, capítulo e progresso no "PC A"
        let db_a = Arc::new(
            Database::abrir(ssd_pc_a.join("System/database.db"), ssd_pc_a.clone())
                .await
                .expect("abrir db no PC A"),
        );
        let lib_a = Arc::new(LibraryManager::nova(db_a.clone(), ssd_pc_a.clone()));
        let arch_a = Arc::new(ArchiveManager::nova(db_a.clone()));

        let rel_obra = "Manga/Obra Portatil".to_string();
        let rel_cap = "Manga/Obra Portatil/Cap 001.cbz".to_string();

        arch_a
            .criar_cbz_de_imagens(&imgs_tmp, &ssd_pc_a.join(&rel_cap))
            .await
            .expect("criar cbz no PC A");

        let obra = lib_a
            .criar_obra("Obra Portátil".to_string(), rel_obra.clone(), None)
            .await
            .expect("criar obra");

        let cap = lib_a
            .registrar_capitulo(obra.id, None, 1, None, rel_cap.clone(), 1)
            .await
            .expect("registrar capitulo");

        reader_salvar_progresso(db_a.clone(), cap.id.to_string(), 0)
            .await
            .expect("salvar progresso");

        // Fecha instâncias do PC A antes de mover o SSD
        drop(arch_a);
        drop(lib_a);
        drop(db_a);

        // 2. Simula desconectar o SSD do PC A e montar em outro caminho no PC B
        fs::rename(&ssd_pc_a, &ssd_pc_b).expect("mover biblioteca para novo caminho");

        // 3. Abre tudo a partir do novo caminho no PC B
        let db_b = Arc::new(
            Database::abrir(ssd_pc_b.join("System/database.db"), ssd_pc_b.clone())
                .await
                .expect("abrir db no PC B"),
        );
        let lib_b = Arc::new(LibraryManager::nova(db_b.clone(), ssd_pc_b.clone()));
        let arch_b = Arc::new(ArchiveManager::nova(db_b.clone()));

        let detalhes = library_obter_detalhes_obra(lib_b.clone(), obra.id.to_string())
            .await
            .expect("ler detalhes da obra no PC B");
        assert_eq!(detalhes.obra.caminho_relativo.as_str(), rel_obra);
        assert_eq!(detalhes.capitulos_avulsos.len(), 1);

        let caminho_cap_rel = detalhes.capitulos_avulsos[0].caminho_relativo.as_str().to_string();
        assert_eq!(caminho_cap_rel, rel_cap);

        let cache_b = Arc::new(tokio::sync::Mutex::new(CacheL1Lru::nova(16)));
        let pagina = reader_obter_pagina(arch_b.clone(), lib_b.clone(), cache_b.clone(), caminho_cap_rel.clone(), 0)
            .await
            .expect("ler pagina do CBZ no PC B usando caminho relativo");
        assert_eq!(pagina.nome, "001.jpg");
        assert_eq!(pagina.total_paginas, 1);

        // Remove o arquivo do disco para provar que a 2ª leitura vem do Cache L1 LRU
        let _ = fs::remove_file(ssd_pc_b.join(&caminho_cap_rel));
        let pagina_cache = reader_obter_pagina(arch_b, lib_b, cache_b, caminho_cap_rel, 0)
            .await
            .expect("ler pagina do cache L1 LRU");
        assert_eq!(pagina_cache.nome, "001.jpg");

        let progresso = reader_obter_progresso(db_b.clone(), cap.id.to_string())
            .await
            .expect("recuperar progresso no PC B");
        assert_eq!(progresso, 0);

        let bytes_decodificados = decodificar_base64(&pagina_cache.dados_base64);
        assert_eq!(bytes_decodificados, b"FAKEJPGDATA");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cache_ia_deterministico_e_invalida_por_parametros() {
        let k1 = calcular_chave_cache_ia(b"img1", "modelo-a", "1.0", "ja", "pt-BR");
        let k2 = calcular_chave_cache_ia(b"img1", "modelo-a", "1.0", "ja", "pt-BR");
        let k_idioma = calcular_chave_cache_ia(b"img1", "modelo-a", "1.0", "ja", "en");
        let k_img = calcular_chave_cache_ia(b"img2", "modelo-a", "1.0", "ja", "pt-BR");
        assert_eq!(k1, k2);
        assert_ne!(k1, k_idioma);
        assert_ne!(k1, k_img);
    }
}
