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

export interface RelatorioTransferencia {
  operacao_id: string
  arquivos_copiados: string[]
  arquivos_ignorados: string[]
  bytes_transferidos: number
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
}

export interface CapituloBuscaResultado {
  fonte_id: string
  obra_id_fonte: string
  capitulo_id_fonte: string
  numero?: number | null
  titulo?: string | null
  url?: string | null
}

export interface StatusOperacao {
  id: string
  obra_titulo: string
  total_capitulos: number
  capitulos_processados: number
  percentual: number
  concluida: boolean
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
  listarMarcadores: (capituloId: string) =>
    invoke<MarcadorPagina[]>('reader_listar_marcadores', { capituloId }),
  transferCopiaIntegral: (origem: string, destino: string) =>
    invoke<RelatorioTransferencia>('transfer_copia_integral', { origem, destino }),
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
  sourceArquivarCapitulos: (obraTitulo: string, capitulos: CapituloBuscaResultado[]) =>
    invoke<string>('source_arquivar_capitulos', { obraTitulo, capitulos }),
  sourceStatusOperacao: (idOperacao: string) =>
    invoke<StatusOperacao | null>('source_status_operacao', { idOperacao }),
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
  sourceListarExtratores: () => invoke<ExtratorInfo[]>('source_listar_extratores'),
  sourceImportarExtrator: (entrada: EntradaImportacao) =>
    invoke<ResultadoImportacao>('source_importar_extrator', { entrada }),
}
