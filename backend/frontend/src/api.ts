import { invoke } from '@tauri-apps/api/core'

export interface Obra {
  id: string
  titulo: string
  caminho_relativo: string
  origem?: string | null
  data_arquivamento: string
  estado_integridade: string
  metadados_complementares?: string | null
}

export interface Volume {
  id: string
  obra_id: string
  numero: number
  caminho_relativo: string
  titulo?: string | null
}

export interface Capitulo {
  id: string
  volume_id?: string | null
  obra_id: string
  numero: number
  titulo_original?: string | null
  caminho_relativo: string
  quantidade_paginas: number
  idioma?: string | null
  data_arquivamento: string
  estado_integridade: string
}

export interface VolumeComCapitulos {
  volume: Volume
  capitulos: Capitulo[]
}

export interface DetalhesObra {
  obra: Obra
  volumes: VolumeComCapitulos[]
  capitulos_avulsos: Capitulo[]
}

export interface PaginaExtraida {
  nome: string
  mime_type: string
  dados_base64: string
  total_paginas: number
}

export interface MarcadorPagina {
  pagina: number
  nota: string
}

export interface MarcadorObra {
  capitulo_id: string
  pagina: number
  nota: string
}

export interface RelatorioTransferencia {
  operacao_id: string
  arquivos_copiados: string[]
  arquivos_ignorados: string[]
  bytes_transferidos: number
}

export interface RelatorioContainer {
  caminho_arquivo: string
  quantidade_arquivos: number
  bytes_totais: number
}

export interface DiferencaBibliotecas {
  ausentes_no_destino: string[]
  diferentes: string[]
  identicos: string[]
}

export interface ModeloInfo {
  id: string
  nome: string
  formato: string
  caminho: string
  tamanho_bytes: number
  disponivel: boolean
}

export interface FonteInfo {
  id: string
  nome: string
  autorizada: boolean
}

export interface ObraBuscaResultado {
  fonte_id: string
  obra_id_fonte: string
  titulo: string
  autor?: string | null
  descricao?: string | null
  capa_url?: string | null
}

export interface CapituloBuscaResultado {
  fonte_id: string
  obra_id_fonte: string
  capitulo_id_fonte: string
  numero?: number | null
  titulo?: string | null
  url?: string | null
  idioma?: string | null
}

export interface StatusOperacao {
  id: string
  obra_titulo: string
  total_capitulos: number
  capitulos_processados: number
  percentual: number
  concluida: boolean
  estado?: string
  pausada?: boolean
  cancelada?: boolean
}

export interface ConectorSiteConfig {
  id: string
  nome: string
  versao: string
  tipo: string
  dominios: string[]
  url_busca?: string | null
  padroes_url_capitulo: string[]
}

export interface CatalogoConectores {
  schema_version: number
  version: string
  sites: ConectorSiteConfig[]
}

export interface StatusAtualizacaoConectores {
  versao_atual: string
  versao_remota?: string | null
  atualizacao_disponivel: boolean
  notas?: string | null
}

export interface ResultadoIA {
  id: string
  capitulo_id: string
  pagina_numero: number
  modelo_id: string
  versao_modelo: string
  tipo_processamento: string
  resultado_json: string
  idioma_origem?: string | null
  idioma_destino?: string | null
  estado_revisao: string
}

export interface ExtratorInfo {
  id: string
  descricao: string
}

export interface EntradaImportacao {
  obra_titulo: string
  numero_capitulo: number
  url?: string | null
  html?: string | null
  urls_ordenadas: string[]
}

export interface ResultadoImportacao {
  extrator_usado: string
  obra_titulo: string
  numero_capitulo: number
  caminho_relativo: string
  quantidade_paginas: number
  hash_sha256: string
}

export interface ObraExtraidaPorLink {
  titulo: string
  capa_url?: string | null
  url_obra: string
  capitulos: CapituloBuscaResultado[]
}

export interface CapturaJanelaAssistida {
  url: string
  html: string
  urls_imagens: string[]
}

export interface ObraResumo {
  obra: Obra
  total_capitulos: number
  capa_data_url?: string | null
}

export interface DownloadLogItem {
  id: string
  nome: string
  pasta_destino: string
  data_hora: string
  operacao_id?: string | null
  obra_id?: string | null
  obra_titulo?: string | null
  capitulo_id?: string | null
  caminho_relativo?: string | null
}

export interface EntradaDiretorio {
  nome: string
  caminho: string
  eh_diretorio: boolean
  tamanho_bytes: number
}

export interface ListagemDiretorio {
  caminho_atual: string
  pai?: string | null
  atalhos: EntradaDiretorio[]
  itens: EntradaDiretorio[]
}

export const mamApi = {
  healthCheck: () => invoke<string>('health_check'),
  libraryRaiz: () => invoke<string>('library_raiz'),
  chapterDetectar: () => invoke<string[]>('chapter_detectar'),
  listarObras: () => invoke<Obra[]>('library_listar_obras'),
  criarObra: (titulo: string, caminhoRelativo: string, origem?: string | null) =>
    invoke<Obra>('library_criar_obra', {
      titulo,
      caminhoRelativo,
      origem: origem ?? null,
    }),
  obterDetalhesObra: (obraId: string) =>
    invoke<DetalhesObra>('library_obter_detalhes_obra', { obraId }),
  criarVolume: (obraId: string, numero: number, titulo: string | null, caminhoRelativo: string) =>
    invoke<Volume>('library_criar_volume', {
      obraId,
      numero,
      titulo,
      caminhoRelativo,
    }),
  atualizarMetadados: (obraId: string, metadadosJson: string) =>
    invoke<void>('library_atualizar_metadados', { obraId, metadadosJson }),
  obterPagina: (caminhoCbz: string, indice: number) =>
    invoke<PaginaExtraida>('reader_obter_pagina', { caminhoCbz, indice }),
  salvarProgresso: (capituloId: string, pagina: number) =>
    invoke<void>('reader_salvar_progresso', { capituloId, pagina }),
  obterProgresso: (capituloId: string) =>
    invoke<number>('reader_obter_progresso', { capituloId }),
  alternarFavorito: (alvoId: string) =>
    invoke<boolean>('reader_alternar_favorito', { alvoId }),
  adicionarMarcador: (capituloId: string, pagina: number, nota: string) =>
    invoke<void>('reader_adicionar_marcador', { capituloId, pagina, nota }),
  removerMarcador: (capituloId: string, pagina: number) =>
    invoke<void>('reader_remover_marcador', { capituloId, pagina }),
  listarMarcadores: (capituloId: string) =>
    invoke<MarcadorPagina[]>('reader_listar_marcadores', { capituloId }),
  listarMarcadoresObra: (obraId: string) =>
    invoke<MarcadorObra[]>('reader_listar_marcadores_obra', { obraId }),
  transferCopiaIntegral: (origem: string, destino: string) =>
    invoke<RelatorioTransferencia>('transfer_copia_integral', { origem, destino }),
  transferEmpacotarContainer: (nomeContainer: string, pastaDestino?: string | null) =>
    invoke<RelatorioContainer>('transfer_empacotar_container', {
      nomeContainer,
      pastaDestino: pastaDestino ?? null,
    }),
  syncCompararBibliotecas: (origem: string, destino: string) =>
    invoke<DiferencaBibliotecas>('sync_comparar_bibliotecas', { origem, destino }),
  syncSincronizarBibliotecas: (origem: string, destino: string, permitirSobrescrita: boolean) =>
    invoke<RelatorioTransferencia>('sync_sincronizar_bibliotecas', {
      origem,
      destino,
      permitirSobrescrita,
    }),
  aiHealth: () => invoke<Record<string, unknown>>('ai_health'),
  aiListarModelos: (diretorios: string[]) =>
    invoke<ModeloInfo[]>('ai_listar_modelos', { diretorios }),
  sourceListarFontes: () => invoke<FonteInfo[]>('source_listar_fontes'),
  sourceBuscarObras: (consulta: string) =>
    invoke<ObraBuscaResultado[]>('source_buscar_obras', { consulta }),
  sourceListarCapitulos: (fonteId: string, obraIdFonte: string) =>
    invoke<CapituloBuscaResultado[]>('source_listar_capitulos', { fonteId, obraIdFonte }),
  sourceArquivarCapitulos: (
    obraTitulo: string,
    capitulos: CapituloBuscaResultado[],
    capaUrl?: string | null,
  ) =>
    invoke<string>('source_arquivar_capitulos', {
      obraTitulo,
      capitulos,
      capaUrl: capaUrl ?? null,
    }),
  sourceIniciarArquivamento: (
    obraTitulo: string,
    capitulos: CapituloBuscaResultado[],
    capaUrl?: string | null,
  ) =>
    invoke<string>('source_iniciar_arquivamento', {
      obraTitulo,
      capitulos,
      capaUrl: capaUrl ?? null,
    }),
  sourceListarOperacoes: () => invoke<StatusOperacao[]>('source_listar_operacoes'),
  sourceObterCapa: (url: string) => invoke<string>('source_obter_capa', { url }),
  libraryListarObrasResumo: () => invoke<ObraResumo[]>('library_listar_obras_resumo'),
  sourceListarHistoricoDownloads: () =>
    invoke<DownloadLogItem[]>('source_listar_historico_downloads'),
  sourceStatusOperacao: (idOperacao: string) =>
    invoke<StatusOperacao | null>('source_status_operacao', { idOperacao }),
  sourcePausarOperacao: (idOperacao: string) =>
    invoke<void>('source_pausar_operacao', { idOperacao }),
  sourceRetomarOperacao: (idOperacao: string) =>
    invoke<void>('source_retomar_operacao', { idOperacao }),
  sourceCancelarOperacao: (idOperacao: string) =>
    invoke<void>('source_cancelar_operacao', { idOperacao }),
  aiListarResultadosCapitulo: (capituloId: string) =>
    invoke<ResultadoIA[]>('ai_listar_resultados_capitulo', { capituloId }),
  aiRevisarResultado: (resultadoId: string, resultadoJsonRevisado: string) =>
    invoke<void>('ai_revisar_resultado', { resultadoId, resultadoJsonRevisado }),
  aiProcessarESalvarPagina: (params: {
    capituloId: string
    caminhoCbz: string
    indice: number
    idiomaOrigem: string
    idiomaDestino: string
    caminhoModelo?: string | null
  }) =>
    invoke<ResultadoIA>('ai_processar_e_salvar_pagina', {
      capituloId: params.capituloId,
      caminhoCbz: params.caminhoCbz,
      indice: params.indice,
      idiomaOrigem: params.idiomaOrigem,
      idiomaDestino: params.idiomaDestino,
      caminhoModelo: params.caminhoModelo ?? null,
    }),
  aiProcessarCapitulo: (params: {
    capituloId: string
    caminhoCbz: string
    totalPaginas: number
    idiomaOrigem: string
    idiomaDestino: string
    caminhoModelo?: string | null
  }) =>
    invoke<ResultadoIA[]>('ai_processar_capitulo', {
      capituloId: params.capituloId,
      caminhoCbz: params.caminhoCbz,
      totalPaginas: params.totalPaginas,
      idiomaOrigem: params.idiomaOrigem,
      idiomaDestino: params.idiomaDestino,
      caminhoModelo: params.caminhoModelo ?? null,
    }),
  sourceListarExtratores: () => invoke<ExtratorInfo[]>('source_listar_extratores'),
  sourceDescobrirPorLink: (
    urlObra: string,
    html?: string | null,
    idiomaPreferido?: string | null,
  ) =>
    invoke<ObraExtraidaPorLink>('source_descobrir_por_link', {
      urlObra,
      html: html ?? null,
      idiomaPreferido: idiomaPreferido ?? null,
    }),
  browserAbrirJanelaAssistida: (url: string) =>
    invoke<void>('browser_abrir_janela_assistida', { url }),
  browserCapturarJanelaAssistida: () =>
    invoke<CapturaJanelaAssistida>('browser_capturar_janela_assistida'),
  browserEnsinarPadraoCapitulo: (urlObra: string) =>
    invoke<ConectorSiteConfig>('browser_ensinar_padrao_capitulo', { urlObra }),
  sourceBuscarObrasPorConector: (conectorId: string, consulta: string) =>
    invoke<ObraBuscaResultado[]>('source_buscar_obras_por_conector', {
      conectorId,
      consulta,
    }),
  sourceListarConectores: () => invoke<CatalogoConectores>('source_listar_conectores'),
  sourceVerificarAtualizacaoConectores: (manifestUrl?: string | null) =>
    invoke<StatusAtualizacaoConectores>('source_verificar_atualizacao_conectores', {
      manifestUrl: manifestUrl ?? null,
    }),
  sourceAtualizarConectoresRemoto: (manifestUrl?: string | null) =>
    invoke<CatalogoConectores>('source_atualizar_conectores_remoto', {
      manifestUrl: manifestUrl ?? null,
    }),
  sourceImportarConectorJson: (conteudoJson: string) =>
    invoke<CatalogoConectores>('source_importar_conector_json', { conteudoJson }),
  sourceImportarExtrator: (entrada: EntradaImportacao) =>
    invoke<ResultadoImportacao>('source_importar_extrator', { entrada }),
  excluirCapitulo: (capituloId: string, removerArquivo: boolean) =>
    invoke<void>('library_excluir_capitulo', { capituloId, removerArquivo }),
  excluirObra: (obraId: string, removerArquivos: boolean) =>
    invoke<void>('library_excluir_obra', { obraId, removerArquivos }),
  criarVolumeComCapitulos: (
    obraId: string,
    numero: number,
    titulo: string | null,
    capituloIds: string[],
  ) =>
    invoke<Volume>('library_criar_volume_com_capitulos', {
      obraId,
      numero,
      titulo,
      capituloIds,
    }),
  renomearVolume: (volumeId: string, numero: number, titulo: string | null) =>
    invoke<void>('library_renomear_volume', { volumeId, numero, titulo }),
  renomearCapitulo: (capituloId: string, novoTitulo: string | null) =>
    invoke<void>('library_renomear_capitulo', { capituloId, novoTitulo }),
  listarHistoricoExclusoes: () =>
    invoke<DownloadLogItem[]>('library_listar_historico_exclusoes'),
  excluirVolume: (volumeId: string) =>
    invoke<void>('library_excluir_volume', { volumeId }),
  listarHistoricoRenomeacoes: () =>
    invoke<DownloadLogItem[]>('library_listar_historico_renomeacoes'),
  limparHistoricoLogs: (tipo?: string | null) =>
    invoke<void>('library_limpar_historico_logs', { tipo: tipo ?? null }),
  marcarLido: (capituloId: string, lido: boolean) =>
    invoke<void>('reader_marcar_lido', { capituloId, lido }),
  listarLidos: () => invoke<string[]>('reader_listar_lidos'),
  importarArquivoLocal: (
    caminhoArquivo: string,
    obraTitulo?: string | null,
    numeroCapitulo?: number | null,
  ) =>
    invoke<string>('library_importar_arquivo_local', {
      caminhoArquivo,
      obraTitulo: obraTitulo ?? null,
      numeroCapitulo: numeroCapitulo ?? null,
    }),
  listarDiretorio: (caminho?: string | null) =>
    invoke<ListagemDiretorio>('system_listar_diretorio', {
      caminho: caminho ?? null,
    }),
}
