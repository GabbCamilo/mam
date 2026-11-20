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
}
