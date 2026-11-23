import { useEffect, useState } from 'react'
import {
  Capitulo,
  CapituloBuscaResultado,
  DetalhesObra,
  DiferencaBibliotecas,
  DownloadLogItem,
  ExtratorInfo,
  FonteInfo,
  ModeloInfo,
  ObraBuscaResultado,
  ObraResumo,
  PaginaExtraida,
  RelatorioContainer,
  RelatorioTransferencia,
  ResultadoImportacao,
  StatusOperacao,
  CatalogoConectores,
  StatusAtualizacaoConectores,
  mamApi,
} from './api'
import { CbzReader } from './components/CbzReader'

type AbaAtiva = 'biblioteca' | 'fontes' | 'downloads' | 'transferencia' | 'ai'
type SubAbaObra = 'capitulos' | 'volumes' | 'marcados'
type OrdemBiblioteca = 'nome-asc' | 'nome-desc' | 'caps-desc' | 'caps-asc'

function obterIdiomaCapitulo(cap: Capitulo): string | null {
  if (cap.idioma && cap.idioma.trim().length > 0) {
    return cap.idioma.trim()
  }
  const partes = cap.caminho_relativo.split('/').filter(Boolean)
  if (partes.length >= 4) {
    return partes[2]
  }
  return null
}

function obterDiretorioCapitulo(cap: Capitulo): string {
  const idx = cap.caminho_relativo.lastIndexOf('/')
  return idx > 0 ? cap.caminho_relativo.slice(0, idx) : cap.caminho_relativo
}

function App() {
  const [tema, setTema] = useState<'dark' | 'light'>('dark')
  const [abaAtiva, setAbaAtiva] = useState<AbaAtiva>('biblioteca')
  const [subAbaObra, setSubAbaObra] = useState<SubAbaObra>('capitulos')
  const [raizBiblioteca, setRaizBiblioteca] = useState<string>('')
  const [obrasResumo, setObrasResumo] = useState<ObraResumo[]>([])
  const [buscaObra, setBuscaObra] = useState('')
  const [ordemBiblioteca, setOrdemBiblioteca] = useState<OrdemBiblioteca>('nome-asc')
  const [buscaCapitulo, setBuscaCapitulo] = useState('')
  const [filtroIdiomaBiblioteca, setFiltroIdiomaBiblioteca] = useState<string>('TODOS')
  const [ordemCaps, setOrdemCaps] = useState<'desc' | 'asc'>('desc')
  const [detalhes, setDetalhes] = useState<DetalhesObra | null>(null)
  const [capaBiblioteca, setCapaBiblioteca] = useState<PaginaExtraida | null>(null)
  const [capitulosLidos, setCapitulosLidos] = useState<string[]>([])
  const [capituloAtivo, setCapituloAtivo] = useState<Capitulo | null>(null)
  const [paginaInicialAtiva, setPaginaInicialAtiva] = useState<number | null>(null)
  const [marcadoresObra, setMarcadoresObra] = useState<
    Array<{ capitulo: Capitulo; pagina: number; nota: string }>
  >([])
  const [tituloNovaObra, setTituloNovaObra] = useState('')
  const [caminhoNovaObra, setCaminhoNovaObra] = useState('')
  const [mensagem, setMensagem] = useState<string | null>(null)

  const [origemTransfer, setOrigemTransfer] = useState('')
  const [destinoTransfer, setDestinoTransfer] = useState('')
  const [diferenca, setDiferenca] = useState<DiferencaBibliotecas | null>(null)
  const [relatorioTransfer, setRelatorioTransfer] = useState<RelatorioTransferencia | null>(null)
  const [nomeContainer, setNomeContainer] = useState('MAM_Biblioteca_Container')
  const [pastaSaidaContainer, setPastaSaidaContainer] = useState('')
  const [relatorioContainer, setRelatorioContainer] = useState<RelatorioContainer | null>(null)
  const [empacotandoContainer, setEmpacotandoContainer] = useState(false)

  const [aiStatus, setAiStatus] = useState<Record<string, unknown> | null>(null)
  const [diretorioModelos, setDiretorioModelos] = useState('')
  const [modelos, setModelos] = useState<ModeloInfo[]>([])

  const [fontes, setFontes] = useState<FonteInfo[]>([])
  const [consultaFonte, setConsultaFonte] = useState('')
  const [obrasFonte, setObrasFonte] = useState<ObraBuscaResultado[]>([])
  const [capasCache, setCapasCache] = useState<Record<string, string>>({})
  const [obraFonteSelecionada, setObraFonteSelecionada] = useState<ObraBuscaResultado | null>(null)
  const [capitulosFonte, setCapitulosFonte] = useState<CapituloBuscaResultado[]>([])
  const [carregandoCapitulosFonte, setCarregandoCapitulosFonte] = useState(false)
  const [filtroIdiomaFonte, setFiltroIdiomaFonte] = useState<string>('TODOS')
  const [selecionadosIds, setSelecionadosIds] = useState<string[]>([])
  const [operacoesDownload, setOperacoesDownload] = useState<StatusOperacao[]>([])
  const [popupDownload, setPopupDownload] = useState<{ obra: string; total: number } | null>(null)
  const [downloadsLog, setDownloadsLog] = useState<DownloadLogItem[]>([])
  const [exclusoesLog, setExclusoesLog] = useState<DownloadLogItem[]>([])
  const [renomeacoesLog, setRenomeacoesLog] = useState<DownloadLogItem[]>([])
  const [subAbaLogs, setSubAbaLogs] = useState<'downloads' | 'exclusoes' | 'renomeacoes'>('downloads')
  const [gruposLogAbertos, setGruposLogAbertos] = useState<Record<string, boolean>>({})
  const [volNumero, setVolNumero] = useState('1')
  const [volTitulo, setVolTitulo] = useState('')
  const [volCapitulosIds, setVolCapitulosIds] = useState<string[]>([])

  const [modalImportarUrlAberto, setModalImportarUrlAberto] = useState(false)
  const [modalInfoImportacaoAberto, setModalInfoImportacaoAberto] = useState(false)
  const [modalConfirmacao, setModalConfirmacao] = useState<{
    titulo: string
    mensagem: string
    textoConfirmar: string
    perigo?: boolean
    onConfirm: () => Promise<void> | void
  } | null>(null)
  const [conectoresCatalogo, setConectoresCatalogo] = useState<CatalogoConectores | null>(null)
  const [conectorBuscaAtivo, setConectorBuscaAtivo] = useState('mangadex')
  const [statusAtualizacaoConectores, setStatusAtualizacaoConectores] =
    useState<StatusAtualizacaoConectores | null>(null)
  const [conectorJsonManual, setConectorJsonManual] = useState('')
  const [modoImportacao, setModoImportacao] = useState<'obra' | 'capitulo'>('obra')
  const [extratores, setExtratores] = useState<ExtratorInfo[]>([])
  const [extObraTitulo, setExtObraTitulo] = useState('')
  const [extNumeroCap, setExtNumeroCap] = useState('1')
  const [extUrl, setExtUrl] = useState('')
  const [extHtml, setExtHtml] = useState('')
  const [extUrlsTexto, setExtUrlsTexto] = useState('')
  const [extIdiomaPreferido, setExtIdiomaPreferido] = useState<'pt-br' | 'en' | 'ja' | 'all'>('pt-br')
  const [resultadoImportacao, setResultadoImportacao] = useState<ResultadoImportacao | null>(null)

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', tema)
  }, [tema])

  async function carregarObras() {
    try {
      const [lista, raiz, lidos, logs, exclusoes, renomeacoes] = await Promise.all([
        mamApi.libraryListarObrasResumo(),
        mamApi.libraryRaiz(),
        mamApi.listarLidos(),
        mamApi.sourceListarHistoricoDownloads(),
        mamApi.listarHistoricoExclusoes(),
        mamApi.listarHistoricoRenomeacoes(),
      ])
      setObrasResumo(lista)
      setRaizBiblioteca(raiz)
      setCapitulosLidos(lidos)
      setDownloadsLog(logs)
      setExclusoesLog(exclusoes)
      setRenomeacoesLog(renomeacoes)
    } catch (e) {
      setMensagem(`Erro ao listar obras: ${String(e)}`)
    }
  }

  useEffect(() => {
    carregarObras()
    mamApi
      .sourceListarConectores()
      .then(setConectoresCatalogo)
      .catch(() => {})
    mamApi
      .sourceVerificarAtualizacaoConectores()
      .then(setStatusAtualizacaoConectores)
      .catch(() => {})
  }, [])

  async function handleCriarObra(e: React.FormEvent) {
    e.preventDefault()
    if (!tituloNovaObra.trim() || !caminhoNovaObra.trim()) return
    try {
      await mamApi.criarObra(tituloNovaObra.trim(), caminhoNovaObra.trim())
      setTituloNovaObra('')
      setCaminhoNovaObra('')
      setMensagem('Obra cadastrada com sucesso.')
      await carregarObras()
    } catch (err) {
      setMensagem(`Erro ao cadastrar obra: ${String(err)}`)
    }
  }

  async function handleDetectarCapitulos() {
    try {
      const detectados = await mamApi.chapterDetectar()
      setMensagem(`Varredura concluída: ${detectados.length} capítulo(s) detectado(s).`)
      await carregarObras()
    } catch (err) {
      setMensagem(`Erro ao detectar capítulos: ${String(err)}`)
    }
  }

  async function handleSelecionarObra(obraId: string) {
    setFiltroIdiomaBiblioteca('TODOS')
    if (detalhes?.obra.id === obraId) {
      setDetalhes(null)
      setCapaBiblioteca(null)
      return
    }
    try {
      const d = await mamApi.obterDetalhesObra(obraId)
      setDetalhes(d)
      setCapituloAtivo(null)
      const primeiroCap = d.volumes[0]?.capitulos[0] ?? d.capitulos_avulsos[0] ?? null
      if (primeiroCap) {
        try {
          const capa = await mamApi.obterPagina(primeiroCap.caminho_relativo, 0)
          setCapaBiblioteca(capa)
        } catch {
          setCapaBiblioteca(null)
        }
      } else {
        setCapaBiblioteca(null)
      }
    } catch (err) {
      setMensagem(`Erro ao carregar detalhes: ${String(err)}`)
    }
  }

  async function handleExcluirCapitulo(e: React.MouseEvent, capId: string) {
    e.stopPropagation()
    try {
      await mamApi.excluirCapitulo(capId, true)
      setMensagem('Capítulo excluído da biblioteca.')
      if (detalhes) {
        const d = await mamApi.obterDetalhesObra(detalhes.obra.id)
        setDetalhes(d)
      }
      await carregarObras()
    } catch (err) {
      setMensagem(`Erro ao excluir capítulo: ${String(err)}`)
    }
  }

  async function handleExcluirObra(e: React.MouseEvent, obraId: string, titulo: string) {
    e.stopPropagation()
    setModalConfirmacao({
      titulo: 'Excluir Obra da Biblioteca',
      mensagem: `Excluir a obra "${titulo}" e todos os seus arquivos da biblioteca?`,
      textoConfirmar: 'Excluir Obra',
      perigo: true,
      onConfirm: async () => {
        try {
          await mamApi.excluirObra(obraId, true)
          if (detalhes?.obra.id === obraId) {
            setDetalhes(null)
            setCapaBiblioteca(null)
          }
          setMensagem(`Obra "${titulo}" excluída da biblioteca.`)
          await carregarObras()
        } catch (err) {
          setMensagem(`Erro ao excluir obra: ${String(err)}`)
        }
      },
    })
  }

  async function handleRenomearCapitulo(e: React.MouseEvent, cap: Capitulo) {
    e.stopPropagation()
    const novo = window.prompt('Novo título do capítulo:', cap.titulo_original ?? '')
    if (novo === null) return
    try {
      await mamApi.renomearCapitulo(cap.id, novo.trim() || null)
      if (detalhes) {
        const d = await mamApi.obterDetalhesObra(detalhes.obra.id)
        setDetalhes(d)
      }
      setMensagem('Título do capítulo atualizado.')
    } catch (err) {
      setMensagem(`Erro ao renomear capítulo: ${String(err)}`)
    }
  }

  async function handleCriarVolumeComCapitulos(e: React.FormEvent) {
    e.preventDefault()
    if (!detalhes) return
    const num = Number.parseInt(volNumero, 10)
    if (Number.isNaN(num)) {
      setMensagem('Informe um número de ordem válido para o volume.')
      return
    }
    try {
      await mamApi.criarVolumeComCapitulos(
        detalhes.obra.id,
        num,
        volTitulo.trim() || null,
        volCapitulosIds,
      )
      setVolTitulo('')
      setVolCapitulosIds([])
      const d = await mamApi.obterDetalhesObra(detalhes.obra.id)
      setDetalhes(d)
      setMensagem(`Volume ${num} criado com sucesso.`)
    } catch (err) {
      setMensagem(`Erro ao formar volume: ${String(err)}`)
    }
  }

  async function handleRenomearVolume(vol: { id: string; numero: number; titulo?: string | null }) {
    const novoTitulo = window.prompt('Nome do volume:', vol.titulo ?? '')
    if (novoTitulo === null) return
    const novoNumStr = window.prompt('Número de ordem do volume:', String(vol.numero))
    if (novoNumStr === null) return
    const novoNum = Number.parseInt(novoNumStr, 10)
    if (Number.isNaN(novoNum)) {
      setMensagem('Número de ordem inválido.')
      return
    }
    try {
      await mamApi.renomearVolume(vol.id, novoNum, novoTitulo.trim() || null)
      if (detalhes) {
        const d = await mamApi.obterDetalhesObra(detalhes.obra.id)
        setDetalhes(d)
      }
      await carregarObras()
      setMensagem('Volume renomeado com sucesso.')
    } catch (err) {
      setMensagem(`Erro ao renomear volume: ${String(err)}`)
    }
  }

  async function handleExcluirVolume(volId: string, label: string) {
    setModalConfirmacao({
      titulo: 'Excluir Volume',
      mensagem: `Excluir o "${label}"? (Os capítulos continuarão salvos na obra.)`,
      textoConfirmar: 'Excluir Volume',
      perigo: true,
      onConfirm: async () => {
        try {
          await mamApi.excluirVolume(volId)
          if (detalhes) {
            const d = await mamApi.obterDetalhesObra(detalhes.obra.id)
            setDetalhes(d)
          }
          await carregarObras()
          setMensagem('Volume excluído (capítulos preservados na obra).')
        } catch (err) {
          setMensagem(`Erro ao excluir volume: ${String(err)}`)
        }
      },
    })
  }

  async function handleLimparLog() {
    const tipo =
      subAbaLogs === 'downloads'
        ? 'download'
        : subAbaLogs === 'exclusoes'
          ? 'exclusao'
          : 'renomeacao'
    setModalConfirmacao({
      titulo: 'Limpar Histórico',
      mensagem: `Limpar o histórico de ${subAbaLogs}? (Downloads em andamento ou pausados serão preservados automaticamente.)`,
      textoConfirmar: 'Limpar Log',
      perigo: false,
      onConfirm: async () => {
        try {
          await mamApi.limparHistoricoLogs(tipo)
          if (subAbaLogs === 'downloads') {
            setOperacoesDownload([])
          }
          await carregarObras()
          setMensagem('Log limpo com sucesso.')
        } catch (err) {
          setMensagem(`Erro ao limpar log: ${String(err)}`)
        }
      },
    })
  }

  async function handleAlternarLido(e: React.MouseEvent, capId: string) {
    e.stopPropagation()
    const jaLido = capitulosLidos.includes(capId)
    try {
      await mamApi.marcarLido(capId, !jaLido)
      const atualizados = await mamApi.listarLidos()
      setCapitulosLidos(atualizados)
    } catch (err) {
      setMensagem(`Erro ao atualizar leitura: ${String(err)}`)
    }
  }

  async function handleCompararBibliotecas() {
    try {
      const diff = await mamApi.syncCompararBibliotecas(origemTransfer, destinoTransfer)
      setDiferenca(diff)
      setMensagem('Comparação concluída.')
    } catch (err) {
      setMensagem(`Erro ao comparar bibliotecas: ${String(err)}`)
    }
  }

  async function handleSincronizar(permitirSobrescrita: boolean) {
    try {
      const rel = await mamApi.syncSincronizarBibliotecas(
        origemTransfer,
        destinoTransfer,
        permitirSobrescrita,
      )
      setRelatorioTransfer(rel)
      setMensagem(`Sincronização concluída: ${rel.arquivos_copiados.length} copiado(s).`)
    } catch (err) {
      setMensagem(`Erro ao sincronizar: ${String(err)}`)
    }
  }

  async function handleCopiaIntegral() {
    try {
      const rel = await mamApi.transferCopiaIntegral(origemTransfer, destinoTransfer)
      setRelatorioTransfer(rel)
      setMensagem(`Cópia integral concluída: ${rel.arquivos_copiados.length} arquivo(s).`)
    } catch (err) {
      setMensagem(`Erro na cópia integral: ${String(err)}`)
    }
  }

  async function handleEmpacotarContainer() {
    setEmpacotandoContainer(true)
    try {
      const rel = await mamApi.transferEmpacotarContainer(
        nomeContainer.trim() || 'MAM_Biblioteca_Container',
        pastaSaidaContainer.trim() || null,
      )
      setRelatorioContainer(rel)
      setMensagem(
        `Container criado com ${rel.quantidade_arquivos} arquivo(s) em: ${rel.caminho_arquivo}`,
      )
    } catch (err) {
      setMensagem(`Erro ao comprimir biblioteca em container: ${String(err)}`)
    } finally {
      setEmpacotandoContainer(false)
    }
  }

  async function handleVerificarAi() {
    try {
      const status = await mamApi.aiHealth()
      setAiStatus(status)
      setMensagem('AI Engine respondeu via JSON-RPC stdio.')
    } catch (err) {
      setMensagem(`Erro ao consultar AI Engine: ${String(err)}`)
    }
  }

  async function handleListarModelos() {
    try {
      const dirs = diretorioModelos.trim() ? [diretorioModelos.trim()] : []
      const lista = await mamApi.aiListarModelos(dirs)
      setModelos(lista)
    } catch (err) {
      setMensagem(`Erro ao listar modelos: ${String(err)}`)
    }
  }

  async function handleCarregarFontes() {
    try {
      const [lista, listaExt] = await Promise.all([
        mamApi.sourceListarFontes(),
        mamApi.sourceListarExtratores(),
      ])
      setFontes(lista)
      setExtratores(listaExt)
    } catch (err) {
      setMensagem(`Erro ao listar fontes: ${String(err)}`)
    }
  }

  async function handleDescobrirCapitulosPorLink() {
    if (!extUrl.trim()) {
      setMensagem('Informe a URL principal da obra para descobrir os capítulos.')
      return
    }
    try {
      const res = await mamApi.sourceDescobrirPorLink(
        extUrl.trim(),
        extHtml.trim() || null,
        extIdiomaPreferido,
      )
      const obraResult: ObraBuscaResultado = {
        fonte_id: 'extrator-link',
        obra_id_fonte: res.url_obra,
        titulo: extObraTitulo.trim() || res.titulo,
        autor: null,
        descricao: null,
        capa_url: res.capa_url ?? null,
      }
      setObrasFonte([obraResult])
      setObraFonteSelecionada(obraResult)
      setCapitulosFonte(res.capitulos)
      setFiltroIdiomaFonte('TODOS')
      setSelecionadosIds([])
      if (res.capa_url && !capasCache[res.capa_url]) {
        mamApi
          .sourceObterCapa(res.capa_url)
          .then((dataUrl) => {
            setCapasCache((prev) => ({ ...prev, [res.capa_url!]: dataUrl }))
          })
          .catch(() => {})
      }
      setModalImportarUrlAberto(false)
      setAbaAtiva('fontes')
      setMensagem(`${res.capitulos.length} capítulo(s) encontrado(s) em "${obraResult.titulo}".`)
    } catch (err) {
      setMensagem(`Erro ao descobrir capítulos pelo link: ${String(err)}`)
    }
  }

  async function handleImportarExtrator(e: React.FormEvent) {
    e.preventDefault()
    const urls_ordenadas = extUrlsTexto
      .split(/\r?\n/)
      .map((u) => u.trim())
      .filter((u) => u.length > 0)

    let tituloFinal = extObraTitulo.trim()
    let numeroFinal = Number.parseInt(extNumeroCap, 10)

    const tituloVazio = !tituloFinal
    if ((tituloVazio || !Number.isFinite(numeroFinal)) && extUrl.trim()) {
      try {
        const info = await mamApi.sourceDescobrirPorLink(
          extUrl.trim(),
          extHtml.trim() || null,
          extIdiomaPreferido,
        )
        if (tituloVazio) {
          tituloFinal = info.titulo
          setExtObraTitulo(info.titulo)
        }
        if ((!Number.isFinite(numeroFinal) || (tituloVazio && extNumeroCap === '1')) && info.capitulos[0]?.numero != null) {
          numeroFinal = info.capitulos[0].numero
          setExtNumeroCap(String(numeroFinal))
        }
      } catch {
        // Fallback para valores informados manualmente caso a pré-análise falhe
      }
    }

    if (!tituloFinal) {
      setMensagem('Informe o título da obra ou uma URL válida do capítulo.')
      return
    }

    try {
      const res = await mamApi.sourceImportarExtrator({
        obra_titulo: tituloFinal,
        numero_capitulo: Number.isFinite(numeroFinal) && numeroFinal > 0 ? numeroFinal : 1,
        url: extUrl.trim() || null,
        html: extHtml.trim() || null,
        urls_ordenadas,
      })
      setResultadoImportacao(res)
      setMensagem(
        `Importado via '${res.extrator_usado}': ${res.caminho_relativo} (${res.quantidade_paginas} págs).`,
      )
      await carregarObras()
    } catch (err) {
      setMensagem(`Erro na importação modular: ${String(err)}`)
    }
  }

  async function handleBuscarNaFonte(e: React.FormEvent) {
    e.preventDefault()
    if (!consultaFonte.trim()) {
      setObrasFonte([])
      setObraFonteSelecionada(null)
      setCapitulosFonte([])
      setSelecionadosIds([])
      return
    }
    try {
      const res = await mamApi.sourceBuscarObrasPorConector(
        conectorBuscaAtivo,
        consultaFonte.trim(),
      )
      setObrasFonte(res)
      setObraFonteSelecionada(null)
      setCapitulosFonte([])
      setSelecionadosIds([])

      // Carrega capas em segundo plano pelo backend Rust (evita bloqueio de Referer/CSP)
      for (const item of res) {
        if (item.capa_url && !capasCache[item.capa_url]) {
          mamApi
            .sourceObterCapa(item.capa_url)
            .then((dataUrl) => {
              setCapasCache((prev) => ({ ...prev, [item.capa_url!]: dataUrl }))
            })
            .catch(() => {})
        }
      }
    } catch (err) {
      setMensagem(`Erro na busca de fontes: ${String(err)}`)
    }
  }

  async function handleSelecionarObraFonte(obra: ObraBuscaResultado) {
    if (obraFonteSelecionada?.obra_id_fonte === obra.obra_id_fonte) {
      setObraFonteSelecionada(null)
      setCapitulosFonte([])
      setFiltroIdiomaFonte('TODOS')
      setSelecionadosIds([])
      return
    }
    setObraFonteSelecionada(obra)
    setCapitulosFonte([])
    setCarregandoCapitulosFonte(true)
    setFiltroIdiomaFonte('TODOS')
    setSelecionadosIds([])
    try {
      const caps = await mamApi.sourceListarCapitulos(obra.fonte_id, obra.obra_id_fonte)
      setCapitulosFonte(caps)
    } catch (err) {
      setMensagem(`Erro ao listar capítulos da fonte: ${String(err)}`)
    } finally {
      setCarregandoCapitulosFonte(false)
    }
  }

  const idiomasFonteDisponiveis = Array.from(
    new Set(
      capitulosFonte
        .map((c) => c.idioma?.trim())
        .filter((lang): lang is string => Boolean(lang)),
    ),
  ).sort()

  const capitulosFonteFiltrados =
    filtroIdiomaFonte === 'TODOS'
      ? capitulosFonte
      : capitulosFonte.filter((c) => c.idioma === filtroIdiomaFonte)

  function alternarCapituloSelecionado(id: string) {
    setSelecionadosIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id],
    )
  }

  function alternarSelecionarTodosCapitulos() {
    const idsVisiveis = capitulosFonteFiltrados.map((c) => c.capitulo_id_fonte)
    const todosVisiveisSelecionados =
      idsVisiveis.length > 0 && idsVisiveis.every((id) => selecionadosIds.includes(id))
    if (todosVisiveisSelecionados) {
      setSelecionadosIds((prev) => prev.filter((id) => !idsVisiveis.includes(id)))
    } else {
      setSelecionadosIds((prev) => Array.from(new Set([...prev, ...idsVisiveis])))
    }
  }

  useEffect(() => {
    if (!popupDownload) return
    const timer = setTimeout(() => setPopupDownload(null), 6500)
    return () => clearTimeout(timer)
  }, [popupDownload])

  useEffect(() => {
    if (!mensagem) return
    const timer = setTimeout(() => setMensagem(null), 5000)
    return () => clearTimeout(timer)
  }, [mensagem])

  const temDownloadAtivo = operacoesDownload.some((op) => !op.concluida)

  useEffect(() => {
    if (!temDownloadAtivo) return
    let ultimoTotalProcessado = -1
    const interval = setInterval(async () => {
      try {
        const ops = await mamApi.sourceListarOperacoes()
        setOperacoesDownload(ops)
        const totalProcessado = ops.reduce((acc, op) => acc + op.capitulos_processados, 0)
        const todasConcluidas = ops.length > 0 && ops.every((op) => op.concluida)
        if (totalProcessado !== ultimoTotalProcessado || todasConcluidas) {
          ultimoTotalProcessado = totalProcessado
          const logs = await mamApi.sourceListarHistoricoDownloads()
          setDownloadsLog(logs)
        }
        if (todasConcluidas) {
          await carregarObras()
        }
      } catch {
        // Silencioso durante polling de progresso
      }
    }, 1500)
    return () => clearInterval(interval)
  }, [temDownloadAtivo])

  function abrirCapituloParaLeitura(cap: Capitulo, paginaInicial: number | null = null) {
    try {
      localStorage.setItem(`mam:ultimo_cap:${cap.obra_id}`, cap.id)
    } catch {
      // Ignora indisponibilidade de storage
    }
    setPaginaInicialAtiva(paginaInicial)
    setCapituloAtivo(cap)
  }

  async function carregarMarcadoresObra(d: DetalhesObra | null) {
    if (!d) {
      setMarcadoresObra([])
      return
    }
    try {
      const caps = [...d.volumes.flatMap((v) => v.capitulos), ...d.capitulos_avulsos]
      const mapaCaps = new Map(caps.map((c) => [c.id, c]))
      const lista = await mamApi.listarMarcadoresObra(d.obra.id)
      const resultado: Array<{ capitulo: Capitulo; pagina: number; nota: string }> = []
      for (const m of lista) {
        const cap = mapaCaps.get(m.capitulo_id)
        if (cap) {
          resultado.push({ capitulo: cap, pagina: m.pagina, nota: m.nota })
        }
      }
      setMarcadoresObra(resultado)
    } catch {
      setMarcadoresObra([])
    }
  }

  useEffect(() => {
    carregarMarcadoresObra(detalhes)
  }, [detalhes])

  async function handleAbrirCapituloDoLog(item: DownloadLogItem) {
    try {
      let obraId = item.obra_id ?? null
      if (!obraId && item.pasta_destino) {
        const encontrada = obrasResumo.find((r) =>
          item.pasta_destino.startsWith(r.obra.caminho_relativo),
        )
        obraId = encontrada?.obra.id ?? null
      }
      if (!obraId) {
        setMensagem('Não foi possível localizar a obra deste capítulo na biblioteca.')
        return
      }
      const d = await mamApi.obterDetalhesObra(obraId)
      setDetalhes(d)
      const caps = [...d.volumes.flatMap((v) => v.capitulos), ...d.capitulos_avulsos]
      const cap =
        caps.find((c) => item.capitulo_id && c.id === item.capitulo_id) ??
        caps.find((c) => item.caminho_relativo && c.caminho_relativo === item.caminho_relativo)
      if (!cap) {
        setMensagem('Capítulo não encontrado na biblioteca (pode ter sido excluído).')
        return
      }
      setCapituloAtivo(cap)
      try {
        localStorage.setItem(`mam:ultimo_cap:${cap.obra_id}`, cap.id)
      } catch {
        // Ignora
      }
    } catch (err) {
      setMensagem(`Erro ao abrir capítulo pelo log: ${String(err)}`)
    }
  }

  async function handleArquivarSelecionados() {
    if (!obraFonteSelecionada || selecionadosIds.length === 0) return
    const escolhidos = capitulosFonte.filter((c) => selecionadosIds.includes(c.capitulo_id_fonte))
    try {
      const capaResolvida =
        obraFonteSelecionada.capa_url && capasCache[obraFonteSelecionada.capa_url]
          ? capasCache[obraFonteSelecionada.capa_url]
          : obraFonteSelecionada.capa_url ?? null
      await mamApi.sourceIniciarArquivamento(
        obraFonteSelecionada.titulo,
        escolhidos,
        capaResolvida,
      )
      setPopupDownload({
        obra: obraFonteSelecionada.titulo,
        total: escolhidos.length,
      })
      const ops = await mamApi.sourceListarOperacoes()
      setOperacoesDownload(ops)
    } catch (err) {
      setMensagem(`Erro ao arquivar capítulos: ${String(err)}`)
    }
  }

  const todosCapitulosObra: Capitulo[] = detalhes
    ? [...detalhes.volumes.flatMap((v) => v.capitulos), ...detalhes.capitulos_avulsos].sort(
        (a, b) => a.numero - b.numero,
      )
    : []

  const idiomasBibliotecaDisponiveis = Array.from(
    new Set(
      todosCapitulosObra
        .map((c) => obterIdiomaCapitulo(c))
        .filter((lang): lang is string => Boolean(lang)),
    ),
  ).sort()

  const capitulosFiltrados = [...todosCapitulosObra]
    .filter((c) => {
      const bateBusca = buscaCapitulo.trim()
        ? String(c.numero).includes(buscaCapitulo.trim()) ||
          (c.titulo_original ?? '').toLowerCase().includes(buscaCapitulo.trim().toLowerCase())
        : true
      const bateIdioma =
        filtroIdiomaBiblioteca === 'TODOS' ||
        obterIdiomaCapitulo(c) === filtroIdiomaBiblioteca
      return bateBusca && bateIdioma
    })
    .sort((a, b) => (ordemCaps === 'desc' ? b.numero - a.numero : a.numero - b.numero))

  const gruposDiretoriosCapitulos = (() => {
    const grupos = new Map<string, { chave: string; idioma: string | null; capitulos: Capitulo[] }>()
    for (const cap of capitulosFiltrados) {
      const dir = obterDiretorioCapitulo(cap)
      const idioma = obterIdiomaCapitulo(cap)
      const existente = grupos.get(dir)
      if (existente) {
        existente.capitulos.push(cap)
      } else {
        grupos.set(dir, { chave: dir, idioma, capitulos: [cap] })
      }
    }
    return Array.from(grupos.values())
  })()

  const obrasFiltradas = [...obrasResumo]
    .filter((item) =>
      buscaObra.trim()
        ? item.obra.titulo.toLowerCase().includes(buscaObra.trim().toLowerCase())
        : true,
    )
    .sort((a, b) => {
      if (ordemBiblioteca === 'nome-asc') return a.obra.titulo.localeCompare(b.obra.titulo)
      if (ordemBiblioteca === 'nome-desc') return b.obra.titulo.localeCompare(a.obra.titulo)
      if (ordemBiblioteca === 'caps-desc') return b.total_capitulos - a.total_capitulos
      return a.total_capitulos - b.total_capitulos
    })

  const resumoObraSelecionada = detalhes
    ? obrasResumo.find((r) => r.obra.id === detalhes.obra.id)
    : undefined

  const capaDestaqueBiblioteca =
    resumoObraSelecionada?.capa_data_url ??
    (capaBiblioteca
      ? `data:${capaBiblioteca.mime_type};base64,${capaBiblioteca.dados_base64}`
      : null)

  if (capituloAtivo) {
    const idiomaAtivo = obterIdiomaCapitulo(capituloAtivo)
    const capitulosMesmoIdioma = todosCapitulosObra.filter(
      (c) => obterIdiomaCapitulo(c) === idiomaAtivo,
    )
    return (
      <CbzReader
        capitulo={capituloAtivo}
        capitulosDisponiveis={
          capitulosMesmoIdioma.length > 0 ? capitulosMesmoIdioma : todosCapitulosObra
        }
        volumesDisponiveis={detalhes?.volumes ?? []}
        paginaInicial={paginaInicialAtiva}
        onSelecionarCapitulo={(cap) => abrirCapituloParaLeitura(cap, null)}
        onFechar={async () => {
          setCapituloAtivo(null)
          setPaginaInicialAtiva(null)
          await carregarObras()
          await carregarMarcadoresObra(detalhes)
        }}
      />
    )
  }

  return (
    <div className="mam-shell">
      <header className="mam-topbar">
        <div className="mam-brand">
          <div className="mam-logo">MAM</div>
          <div>
            <div className="mam-brand-title">MANGA ARCHIVE MANAGER</div>
            <div className="mam-brand-sub">Biblioteca Portátil • Raiz: {raizBiblioteca || '...'}</div>
          </div>
          <span className="mam-pill-badge">{obrasResumo.length} obra(s)</span>
        </div>

        <button
          className="mam-btn"
          onClick={() => setTema((t) => (t === 'dark' ? 'light' : 'dark'))}
        >
          {tema === 'dark' ? '🌙 Dark' : '☀️ Light'}
        </button>
      </header>

      <nav className="mam-nav-bar">
        <button
          className="mam-btn mam-btn-primary"
          onClick={() => {
            handleCarregarFontes()
            setModalImportarUrlAberto(true)
          }}
        >
          + Importar URL
        </button>
        <button
          className={`mam-btn ${abaAtiva === 'biblioteca' ? 'mam-btn-active' : ''}`}
          onClick={() => {
            setAbaAtiva('biblioteca')
            carregarObras()
          }}
        >
          Biblioteca
        </button>
        <button
          className={`mam-btn ${abaAtiva === 'fontes' ? 'mam-btn-active' : ''}`}
          onClick={() => {
            setAbaAtiva('fontes')
            handleCarregarFontes()
          }}
        >
          Fontes & Importação
        </button>
        <button
          className={`mam-btn ${abaAtiva === 'downloads' ? 'mam-btn-active' : ''}`}
          onClick={() => {
            setAbaAtiva('downloads')
            carregarObras()
          }}
        >
          Log de Downloads ({downloadsLog.length})
        </button>
        <button
          className={`mam-btn ${abaAtiva === 'transferencia' ? 'mam-btn-active' : ''}`}
          onClick={() => setAbaAtiva('transferencia')}
        >
          Transferência & Sync
        </button>
        <button
          className={`mam-btn ${abaAtiva === 'ai' ? 'mam-btn-active' : ''}`}
          onClick={() => setAbaAtiva('ai')}
        >
          AI Engine
        </button>
      </nav>

      <main className="mam-main">
        {abaAtiva === 'biblioteca' && (
          <>
            <div className="mam-section-header">
              <div>
                <h1 className="mam-section-title">BIBLIOTECA</h1>
                <p className="mam-section-desc">
                  Obras carregadas automaticamente — ordene por nome ou quantidade de capítulos.
                </p>
              </div>
              <div className="mam-toolbar-row">
                <button className="mam-btn" onClick={handleDetectarCapitulos}>
                  Detectar Capítulos CBZ
                </button>
              </div>
            </div>

            <form onSubmit={handleCriarObra} className="mam-panel">
              <div className="mam-toolbar-row">
                <input
                  className="mam-input mam-search-input"
                  placeholder="Título da nova obra..."
                  value={tituloNovaObra}
                  onChange={(e) => setTituloNovaObra(e.target.value)}
                />
                <input
                  className="mam-input mam-search-input"
                  placeholder="Caminho relativo (ex: Manga/Medaka Box)"
                  value={caminhoNovaObra}
                  onChange={(e) => setCaminhoNovaObra(e.target.value)}
                />
                <button className="mam-btn mam-btn-primary" type="submit">
                  + Cadastrar Obra
                </button>
              </div>
            </form>

            <div className="mam-grid-2">
              <section className="mam-panel">
                <input
                  className="mam-input"
                  placeholder="🔍 Buscar obra na biblioteca..."
                  value={buscaObra}
                  onChange={(e) => setBuscaObra(e.target.value)}
                />
                <select
                  className="mam-select"
                  value={ordemBiblioteca}
                  onChange={(e) => setOrdemBiblioteca(e.target.value as OrdemBiblioteca)}
                >
                  <option value="nome-asc">Ordenar: Nome (A → Z)</option>
                  <option value="nome-desc">Ordenar: Nome (Z → A)</option>
                  <option value="caps-desc">Ordenar: Mais capítulos</option>
                  <option value="caps-asc">Ordenar: Menos capítulos</option>
                </select>

                {obrasFiltradas.length === 0 ? (
                  <p className="mam-section-desc">Nenhuma obra encontrada.</p>
                ) : (
                  obrasFiltradas.map((item) => (
                    <div
                      key={item.obra.id}
                      onClick={() => handleSelecionarObra(item.obra.id)}
                      className={`mam-row-card ${detalhes?.obra.id === item.obra.id ? 'mam-row-card-active' : ''}`}
                      style={{ flexWrap: 'nowrap', gap: '0.6rem' }}
                    >
                      <div style={{ display: 'flex', gap: '0.65rem', alignItems: 'center', minWidth: 0, flex: 1 }}>
                        {item.capa_data_url && (
                          <img
                            src={item.capa_data_url}
                            alt={item.obra.titulo}
                            style={{ width: 38, height: 54, objectFit: 'cover', borderRadius: 6, flexShrink: 0 }}
                          />
                        )}
                        <div style={{ minWidth: 0, flex: 1 }}>
                          <div style={{ fontWeight: 700, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                            {item.obra.titulo}
                          </div>
                          <div className="mam-brand-sub" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                            {item.obra.caminho_relativo}
                          </div>
                        </div>
                      </div>
                      <div style={{ display: 'flex', gap: '0.4rem', alignItems: 'center', flexShrink: 0, flexWrap: 'nowrap' }}>
                        <span className="mam-pill-badge" style={{ whiteSpace: 'nowrap' }}>
                          {item.total_capitulos} cap(s)
                        </span>
                      </div>
                    </div>
                  ))
                )}
              </section>

              <section className="mam-panel">
                {detalhes ? (
                  <>
                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '0.75rem',
                        marginBottom: '1rem',
                      }}
                    >
                      <div style={{ display: 'flex', gap: '0.85rem', alignItems: 'center' }}>
                        {capaDestaqueBiblioteca && (
                          <img
                            src={capaDestaqueBiblioteca}
                            alt={detalhes.obra.titulo}
                            style={{
                              width: 68,
                              height: 96,
                              objectFit: 'cover',
                              borderRadius: 8,
                              border: '1px solid var(--border-subtle)',
                              flexShrink: 0,
                            }}
                          />
                        )}
                        <div style={{ minWidth: 0, flex: 1 }}>
                          <h2 className="mam-section-title">{detalhes.obra.titulo}</h2>
                          <p className="mam-section-desc">{detalhes.obra.caminho_relativo}</p>
                        </div>
                      </div>

                      <div className="mam-toolbar-row" style={{ flexWrap: 'wrap' }}>
                        {(() => {
                          let ultimoCap: Capitulo | undefined
                          try {
                            const salvoId = localStorage.getItem(`mam:ultimo_cap:${detalhes.obra.id}`)
                            if (salvoId) {
                              ultimoCap = todosCapitulosObra.find((c) => c.id === salvoId)
                            }
                          } catch {
                            // Ignora
                          }
                          if (!ultimoCap) {
                            const lidosDaObra = todosCapitulosObra.filter((c) =>
                              capitulosLidos.includes(c.id),
                            )
                            ultimoCap =
                              lidosDaObra[lidosDaObra.length - 1] ?? todosCapitulosObra[0]
                          }
                          return ultimoCap ? (
                            <button
                              className="mam-btn mam-btn-primary"
                              onClick={() => abrirCapituloParaLeitura(ultimoCap!)}
                            >
                              ▶ Continuar Leitura (Cap. {ultimoCap.numero})
                            </button>
                          ) : null
                        })()}
                        <div className="mam-segmented">
                          <button
                            className={`mam-segmented-btn ${subAbaObra === 'capitulos' ? 'active' : ''}`}
                            onClick={() => setSubAbaObra('capitulos')}
                          >
                            Capítulos ({capitulosFiltrados.length})
                          </button>
                          <button
                            className={`mam-segmented-btn ${subAbaObra === 'volumes' ? 'active' : ''}`}
                            onClick={() => setSubAbaObra('volumes')}
                          >
                            Volumes ({detalhes.volumes.length})
                          </button>
                          <button
                            className={`mam-segmented-btn ${subAbaObra === 'marcados' ? 'active' : ''}`}
                            onClick={() => setSubAbaObra('marcados')}
                          >
                            Marcados (
                            {
                              marcadoresObra.filter(
                                (m) =>
                                  filtroIdiomaBiblioteca === 'TODOS' ||
                                  obterIdiomaCapitulo(m.capitulo) === filtroIdiomaBiblioteca,
                              ).length
                            }
                            )
                          </button>
                        </div>
                        {idiomasBibliotecaDisponiveis.length > 0 && (
                          <select
                            className="mam-select"
                            aria-label="Filtrar capítulos por idioma"
                            value={filtroIdiomaBiblioteca}
                            onChange={(e) => setFiltroIdiomaBiblioteca(e.target.value)}
                          >
                            <option value="TODOS">LANG: Todos</option>
                            {idiomasBibliotecaDisponiveis.map((lang) => (
                              <option key={lang} value={lang}>
                                LANG: {lang}
                              </option>
                            ))}
                          </select>
                        )}
                      </div>

                      <div>
                        <button
                          className="mam-btn"
                          onClick={(e) => handleExcluirObra(e, detalhes.obra.id, detalhes.obra.titulo)}
                          title="Excluir obra inteira da biblioteca"
                        >
                          🗑 Excluir Obra
                        </button>
                      </div>
                    </div>

                    {subAbaObra === 'capitulos' ? (
                      <>
                        <div className="mam-toolbar-row">
                          <input
                            className="mam-input mam-search-input"
                            placeholder="🔍 Buscar número ou título do capítulo..."
                            value={buscaCapitulo}
                            onChange={(e) => setBuscaCapitulo(e.target.value)}
                          />
                          <button
                            className="mam-btn"
                            onClick={() => setOrdemCaps((o) => (o === 'desc' ? 'asc' : 'desc'))}
                          >
                            Capítulo {ordemCaps === 'desc' ? '↓' : '↑'}
                          </button>
                        </div>

                        {capitulosFiltrados.length === 0 ? (
                          <p className="mam-section-desc">Nenhum capítulo cadastrado nesta obra.</p>
                        ) : (
                          (filtroIdiomaBiblioteca === 'TODOS' && gruposDiretoriosCapitulos.length > 1
                            ? gruposDiretoriosCapitulos
                            : [{ chave: 'unico', idioma: null, capitulos: capitulosFiltrados }]
                          ).map((grupo, idxGrupo) => (
                            <div
                              key={grupo.chave}
                              style={{
                                marginTop: idxGrupo > 0 ? '1.4rem' : '0.4rem',
                                paddingTop: idxGrupo > 0 ? '1.1rem' : 0,
                                borderTop: idxGrupo > 0 ? '1px solid var(--border-subtle)' : 'none',
                              }}
                            >
                              {gruposDiretoriosCapitulos.length > 1 &&
                                filtroIdiomaBiblioteca === 'TODOS' && (
                                  <div
                                    style={{
                                      display: 'flex',
                                      justifyContent: 'space-between',
                                      alignItems: 'center',
                                      marginBottom: '0.5rem',
                                    }}
                                  >
                                    <span className="mam-pill-badge">
                                      📁 {grupo.idioma ?? 'Sem idioma'} • {grupo.chave}
                                    </span>
                                    <span className="mam-brand-sub">
                                      {grupo.capitulos.length} capítulo(s)
                                    </span>
                                  </div>
                                )}
                              {grupo.capitulos.map((cap) => {
                                const lido = capitulosLidos.includes(cap.id)
                                const idiomaCap = obterIdiomaCapitulo(cap)
                                return (
                                  <div
                                    key={cap.id}
                                    className="mam-row-card"
                                    onClick={() => abrirCapituloParaLeitura(cap)}
                                  >
                                    <div className="mam-chapter-row-main">
                                      <div className="mam-chapter-header-line">
                                        {idiomaCap && <span className="mam-pill-badge">{idiomaCap}</span>}
                                        <strong style={{ whiteSpace: 'nowrap' }}>Cap. {cap.numero}</strong>
                                        {cap.titulo_original && (
                                          <span
                                            className="mam-chapter-title"
                                            title={cap.titulo_original}
                                          >
                                            • {cap.titulo_original}
                                          </span>
                                        )}
                                        <button
                                          className="mam-pill-badge"
                                          style={{
                                            cursor: 'pointer',
                                            whiteSpace: 'nowrap',
                                            background: lido ? 'rgba(34, 197, 94, 0.16)' : 'rgba(148, 163, 184, 0.12)',
                                            color: lido ? 'var(--success)' : 'var(--text-secondary)',
                                            borderColor: lido ? 'rgba(34, 197, 94, 0.35)' : 'var(--border-subtle)',
                                          }}
                                          onClick={(e) => handleAlternarLido(e, cap.id)}
                                        >
                                          {lido ? '✓ Lido' : '○ Não lido'}
                                        </button>
                                      </div>
                                      <div className="mam-brand-sub" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                                        {cap.caminho_relativo}
                                      </div>
                                    </div>
                                    <div className="mam-chapter-actions">
                                      <span className="mam-pill-badge">{cap.quantidade_paginas} págs</span>
                                      <button
                                        className="mam-btn"
                                        onClick={(e) => handleRenomearCapitulo(e, cap)}
                                        title="Renomear título do capítulo"
                                      >
                                        ✏️
                                      </button>
                                      <button
                                        className="mam-btn"
                                        onClick={(e) => handleExcluirCapitulo(e, cap.id)}
                                        title="Excluir capítulo arquivado"
                                      >
                                        🗑
                                      </button>
                                      <button className="mam-btn mam-btn-primary">Ler</button>
                                    </div>
                                  </div>
                                )
                              })}
                            </div>
                          ))
                        )}
                      </>
                    ) : subAbaObra === 'volumes' ? (
                      <>
                        <form onSubmit={handleCriarVolumeComCapitulos} className="mam-panel" style={{ marginBottom: '1rem' }}>
                          <div className="mam-section-header" style={{ marginBottom: '0.5rem' }}>
                            <strong>Formar Volume a partir dos capítulos</strong>
                            {idiomasBibliotecaDisponiveis.length > 0 && (
                              <select
                                className="mam-select"
                                aria-label="Filtrar volumes e capítulos por idioma"
                                value={filtroIdiomaBiblioteca}
                                onChange={(e) => setFiltroIdiomaBiblioteca(e.target.value)}
                              >
                                <option value="TODOS">LANG: Todos</option>
                                {idiomasBibliotecaDisponiveis.map((lang) => (
                                  <option key={lang} value={lang}>
                                    LANG: {lang}
                                  </option>
                                ))}
                              </select>
                            )}
                          </div>
                          <div className="mam-toolbar-row" style={{ marginBottom: '0.6rem' }}>
                            <input
                              className="mam-input"
                              type="number"
                              placeholder="Nº ordem"
                              value={volNumero}
                              onChange={(e) => setVolNumero(e.target.value)}
                              style={{ width: 110 }}
                              required
                            />
                            <input
                              className="mam-input mam-search-input"
                              placeholder="Nome do volume (ex.: Arco Inicial)"
                              value={volTitulo}
                              onChange={(e) => setVolTitulo(e.target.value)}
                            />
                            <button className="mam-btn mam-btn-primary" type="submit">
                              Formar Volume ({volCapitulosIds.length})
                            </button>
                          </div>
                          {capitulosFiltrados.length > 0 && (
                            <div
                              style={{
                                display: 'grid',
                                gap: '0.35rem',
                                maxHeight: 190,
                                overflowY: 'auto',
                                padding: '0.4rem',
                                border: '1px solid var(--border-subtle)',
                                borderRadius: 8,
                              }}
                            >
                              {capitulosFiltrados.map((cap) => {
                                const lang = obterIdiomaCapitulo(cap)
                                return (
                                  <label
                                    key={cap.id}
                                    style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', cursor: 'pointer' }}
                                  >
                                    <input
                                      type="checkbox"
                                      checked={volCapitulosIds.includes(cap.id)}
                                      onChange={() =>
                                        setVolCapitulosIds((prev) =>
                                          prev.includes(cap.id)
                                            ? prev.filter((id) => id !== cap.id)
                                            : [...prev, cap.id],
                                        )
                                      }
                                    />
                                    {lang && <span className="mam-pill-badge">{lang}</span>}
                                    <span>
                                      Cap. {cap.numero}
                                      {cap.titulo_original ? ` — ${cap.titulo_original}` : ''}
                                    </span>
                                  </label>
                                )
                              })}
                            </div>
                          )}
                        </form>

                        {detalhes.volumes.map((vc) => {
                          const capsVolumeFiltrados = vc.capitulos.filter(
                            (cap) =>
                              filtroIdiomaBiblioteca === 'TODOS' ||
                              obterIdiomaCapitulo(cap) === filtroIdiomaBiblioteca,
                          )
                          if (filtroIdiomaBiblioteca !== 'TODOS' && capsVolumeFiltrados.length === 0) {
                            return null
                          }
                          const labelVol = `Volume ${vc.volume.numero}${vc.volume.titulo ? ` — ${vc.volume.titulo}` : ''}`
                          return (
                            <div key={vc.volume.id} className="mam-panel">
                              <div className="mam-section-header" style={{ marginBottom: '0.5rem' }}>
                                <strong>{labelVol}</strong>
                                <div className="mam-toolbar-row">
                                  <button
                                    className="mam-btn"
                                    onClick={() => handleRenomearVolume(vc.volume)}
                                  >
                                    ✏️ Renomear
                                  </button>
                                  <button
                                    className="mam-btn"
                                    onClick={() => handleExcluirVolume(vc.volume.id, labelVol)}
                                    title="Excluir volume"
                                  >
                                    🗑
                                  </button>
                                </div>
                              </div>
                              {capsVolumeFiltrados.map((cap) => {
                                const langCap = obterIdiomaCapitulo(cap)
                                return (
                                  <div
                                    key={cap.id}
                                    className="mam-row-card"
                                    onClick={() => abrirCapituloParaLeitura(cap)}
                                  >
                                    <div className="mam-chapter-row-main">
                                      <div className="mam-chapter-header-line">
                                        {langCap && <span className="mam-pill-badge">{langCap}</span>}
                                        <strong style={{ whiteSpace: 'nowrap' }}>Cap. {cap.numero}</strong>
                                        {cap.titulo_original && (
                                          <span
                                            className="mam-chapter-title"
                                            title={cap.titulo_original}
                                          >
                                            — {cap.titulo_original}
                                          </span>
                                        )}
                                      </div>
                                    </div>
                                    <div className="mam-chapter-actions">
                                      <span className="mam-pill-badge">{cap.quantidade_paginas} págs</span>
                                      <button
                                        className="mam-btn"
                                        onClick={(e) => handleRenomearCapitulo(e, cap)}
                                        title="Renomear título do capítulo"
                                      >
                                        ✏️
                                      </button>
                                      <button
                                        className="mam-btn"
                                        onClick={(e) => handleExcluirCapitulo(e, cap.id)}
                                      >
                                        🗑
                                      </button>
                                      <button className="mam-btn mam-btn-primary">Ler</button>
                                    </div>
                                  </div>
                                )
                              })}
                            </div>
                          )
                        })}
                        {detalhes.volumes.length === 0 && (
                          <p className="mam-section-desc">
                            Nenhum volume agrupado; selecione os capítulos acima para formar um volume.
                          </p>
                        )}
                      </>
                    ) : null}

                    {subAbaObra === 'marcados' && (
                      <>
                        {idiomasBibliotecaDisponiveis.length > 0 && (
                          <div className="mam-toolbar-row" style={{ marginBottom: '0.6rem' }}>
                            <select
                              className="mam-select"
                              aria-label="Filtrar páginas marcadas por idioma"
                              value={filtroIdiomaBiblioteca}
                              onChange={(e) => setFiltroIdiomaBiblioteca(e.target.value)}
                            >
                              <option value="TODOS">LANG: Todos</option>
                              {idiomasBibliotecaDisponiveis.map((lang) => (
                                <option key={lang} value={lang}>
                                  LANG: {lang}
                                </option>
                              ))}
                            </select>
                          </div>
                        )}
                        {(() => {
                          const filtrados = marcadoresObra.filter(
                            (m) =>
                              filtroIdiomaBiblioteca === 'TODOS' ||
                              obterIdiomaCapitulo(m.capitulo) === filtroIdiomaBiblioteca,
                          )
                          if (filtrados.length === 0) {
                            return (
                              <p className="mam-section-desc">
                                Nenhuma página marcada nesta obra para o idioma selecionado.
                              </p>
                            )
                          }
                          return (
                            <div style={{ display: 'grid', gap: '0.5rem' }}>
                              {filtrados.map((m) => {
                                const lang = obterIdiomaCapitulo(m.capitulo)
                                return (
                                  <div
                                    key={`${m.capitulo.id}-${m.pagina}`}
                                    className="mam-row-card"
                                    onClick={() => abrirCapituloParaLeitura(m.capitulo, m.pagina)}
                                  >
                                    <div>
                                      <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', flexWrap: 'wrap' }}>
                                        {lang && <span className="mam-pill-badge">{lang}</span>}
                                        <strong>
                                          Cap. {m.capitulo.numero} • {m.nota}
                                        </strong>
                                      </div>
                                      <div className="mam-brand-sub">{m.capitulo.caminho_relativo}</div>
                                    </div>
                                    <div className="mam-toolbar-row">
                                      <button
                                        className="mam-btn"
                                        onClick={async (e) => {
                                          e.stopPropagation()
                                          await mamApi.removerMarcador(m.capitulo.id, m.pagina)
                                          await carregarMarcadoresObra(detalhes)
                                          setMensagem('Marcador removido.')
                                        }}
                                        title="Desmarcar página"
                                      >
                                        🔖 Desmarcar
                                      </button>
                                      <button className="mam-btn mam-btn-primary">Ler</button>
                                    </div>
                                  </div>
                                )
                              })}
                            </div>
                          )
                        })()}
                      </>
                    )}
                  </>
                ) : (
                  <p className="mam-section-desc">
                    Selecione uma obra à esquerda para visualizar capítulos, capa em destaque e volumes.
                  </p>
                )}
              </section>
            </div>
          </>
        )}

        {abaAtiva === 'fontes' && (
          <>
            <div className="mam-section-header">
              <div>
                <h1 className="mam-section-title">FONTES & IMPORTAÇÃO</h1>
                <p className="mam-section-desc">
                  Fontes ativas: {fontes.map((f) => f.nome).join(', ') || 'nenhuma'} • Extratores:{' '}
                  {extratores.map((e) => e.id).join(', ') || '...'}
                </p>
              </div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', flexWrap: 'wrap' }}>
                {statusAtualizacaoConectores?.atualizacao_disponivel && (
                  <button
                    type="button"
                    className="mam-pill-badge"
                    style={{ cursor: 'pointer', border: '1px solid var(--accent-primary)' }}
                    title={statusAtualizacaoConectores.notas ?? 'Nova versão de conectores disponível'}
                    onClick={async () => {
                      try {
                        const cat = await mamApi.sourceAtualizarConectoresRemoto()
                        setConectoresCatalogo(cat)
                        setStatusAtualizacaoConectores({
                          versao_atual: cat.version,
                          versao_remota: cat.version,
                          atualizacao_disponivel: false,
                          notas: null,
                        })
                        setMensagem(`Conectores atualizados para v${cat.version} com segurança!`)
                      } catch (err) {
                        setMensagem(`Erro ao atualizar conectores: ${String(err)}`)
                      }
                    }}
                  >
                    🔄 Atualização de sites disponível (v{statusAtualizacaoConectores.versao_remota})
                  </button>
                )}
                <button
                  type="button"
                  className="mam-btn"
                  title="Como usar o Importar por Link, conectores ativos e atualização segura"
                  onClick={() => setModalInfoImportacaoAberto(true)}
                >
                  ⓘ Guia / Conectores
                </button>
                <button
                  className="mam-btn mam-btn-primary"
                  onClick={() => setModalImportarUrlAberto(true)}
                >
                  + Importar por Link / HTML
                </button>
              </div>
            </div>

            <form onSubmit={handleBuscarNaFonte} className="mam-panel">
              <div className="mam-toolbar-row">
                <select
                  className="mam-select"
                  aria-label="Mecanismo de busca nativo"
                  value={conectorBuscaAtivo}
                  onChange={(e) => setConectorBuscaAtivo(e.target.value)}
                >
                  {(conectoresCatalogo?.sites ?? [])
                    .filter((s) => s.id === 'mangadex' || Boolean(s.url_busca))
                    .map((s) => (
                      <option key={s.id} value={s.id}>
                        {s.nome}
                      </option>
                    ))}
                  {!conectoresCatalogo && (
                    <>
                      <option value="mangadex">MangaDex (API Pública)</option>
                      <option value="mangafire">MangaFire (Web Inteligente)</option>
                    </>
                  )}
                </select>
                <input
                  className="mam-input mam-search-input"
                  placeholder={`🔍 Pesquisar obra por título em ${
                    conectoresCatalogo?.sites.find((s) => s.id === conectorBuscaAtivo)?.nome ??
                    conectorBuscaAtivo
                  }...`}
                  value={consultaFonte}
                  onChange={(e) => {
                    const v = e.target.value
                    setConsultaFonte(v)
                    if (!v.trim()) {
                      setObrasFonte([])
                      setObraFonteSelecionada(null)
                      setCapitulosFonte([])
                      setSelecionadosIds([])
                    }
                  }}
                />
                <button className="mam-btn mam-btn-primary" type="submit">
                  Pesquisar
                </button>
              </div>
            </form>

            <div className="mam-grid-2">
              <div className="mam-panel">
                <strong>Resultados ({obrasFonte.length})</strong>
                {obrasFonte.map((o) => {
                  const capaSrc = o.capa_url ? capasCache[o.capa_url] : undefined
                  return (
                    <div
                      key={`${o.fonte_id}-${o.obra_id_fonte}`}
                      className={`mam-row-card ${obraFonteSelecionada?.obra_id_fonte === o.obra_id_fonte ? 'mam-row-card-active' : ''}`}
                      onClick={() => handleSelecionarObraFonte(o)}
                    >
                      <div style={{ display: 'flex', gap: '0.65rem', alignItems: 'center', minWidth: 0 }}>
                        {capaSrc && (
                          <img
                            src={capaSrc}
                            alt={o.titulo}
                            style={{ width: 40, height: 56, objectFit: 'cover', borderRadius: 6, flexShrink: 0 }}
                          />
                        )}
                        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{o.titulo}</span>
                      </div>
                    </div>
                  )
                })}
              </div>

              <div className="mam-panel">
                {obraFonteSelecionada ? (
                  <>
                    <div className="mam-section-header">
                      <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center', minWidth: 0 }}>
                        {obraFonteSelecionada.capa_url && capasCache[obraFonteSelecionada.capa_url] && (
                          <img
                            src={capasCache[obraFonteSelecionada.capa_url]}
                            alt={obraFonteSelecionada.titulo}
                            style={{ width: 52, height: 74, objectFit: 'cover', borderRadius: 8, flexShrink: 0 }}
                          />
                        )}
                        <strong>{obraFonteSelecionada.titulo}</strong>
                      </div>
                      <div className="mam-toolbar-row">
                        {idiomasFonteDisponiveis.length > 0 && (
                          <select
                            className="mam-select"
                            aria-label="Filtrar capítulos da fonte por idioma"
                            value={filtroIdiomaFonte}
                            onChange={(e) => {
                              const novoIdioma = e.target.value
                              setFiltroIdiomaFonte(novoIdioma)
                              if (novoIdioma !== 'TODOS') {
                                const permitidos = new Set(
                                  capitulosFonte
                                    .filter((c) => c.idioma === novoIdioma)
                                    .map((c) => c.capitulo_id_fonte),
                                )
                                setSelecionadosIds((prev) =>
                                  prev.filter((id) => permitidos.has(id)),
                                )
                              }
                            }}
                          >
                            <option value="TODOS">LANG: Todos ({capitulosFonte.length})</option>
                            {idiomasFonteDisponiveis.map((lang) => (
                              <option key={lang} value={lang}>
                                LANG: {lang} (
                                {capitulosFonte.filter((c) => c.idioma === lang).length})
                              </option>
                            ))}
                          </select>
                        )}
                        <button className="mam-btn" onClick={alternarSelecionarTodosCapitulos}>
                          {capitulosFonteFiltrados.length > 0 &&
                          capitulosFonteFiltrados.every((c) =>
                            selecionadosIds.includes(c.capitulo_id_fonte),
                          )
                            ? 'Desmarcar Todas'
                            : 'Selecionar Todas'}
                        </button>
                        <button
                          className="mam-btn mam-btn-primary"
                          onClick={handleArquivarSelecionados}
                          disabled={selecionadosIds.length === 0}
                        >
                          Arquivar Selecionados ({selecionadosIds.length})
                        </button>
                      </div>
                    </div>
                    {carregandoCapitulosFonte ? (
                      <p className="mam-section-desc">⏳ Carregando capítulos da obra...</p>
                    ) : capitulosFonteFiltrados.length === 0 ? (
                      <p className="mam-section-desc">
                        Nenhum capítulo hospedado diretamente nesta fonte para os idiomas PT/ENG/JPN (esta obra pode possuir apenas links externos de editoras oficiais como MangaPlus/Viz). Use "+ Importar por Link / HTML" se desejar importar manualmente.
                      </p>
                    ) : (
                      capitulosFonteFiltrados.map((c) => (
                      <label key={c.capitulo_id_fonte} className="mam-row-card">
                        <span style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                          <input
                            type="checkbox"
                            checked={selecionadosIds.includes(c.capitulo_id_fonte)}
                            onChange={() => alternarCapituloSelecionado(c.capitulo_id_fonte)}
                          />
                          {c.idioma && <span className="mam-pill-badge">{c.idioma}</span>}
                          <span>
                            Cap. {c.numero ?? '?'} {c.titulo ? `— ${c.titulo}` : ''}
                          </span>
                        </span>
                      </label>
                    ))
                    )}
                  </>
                ) : (
                  <p className="mam-section-desc">
                    Pesquise e selecione uma obra para listar os capítulos disponíveis (clique novamente na obra para recolher).
                  </p>
                )}
              </div>
            </div>
          </>
        )}

        {abaAtiva === 'downloads' && (
          <section className="mam-panel">
            <div className="mam-section-header">
              <div>
                <h1 className="mam-section-title">
                  {subAbaLogs === 'downloads'
                    ? 'LOG DE DOWNLOADS'
                    : subAbaLogs === 'exclusoes'
                      ? 'LOG DE EXCLUSÕES'
                      : 'LOG DE RENOMEAÇÕES'}
                </h1>
                <p className="mam-section-desc">
                  {subAbaLogs === 'downloads'
                    ? 'Clique em qualquer conjunto para expandir ou recolher os capítulos baixados e abrir a leitura diretamente.'
                    : subAbaLogs === 'exclusoes'
                      ? 'Histórico de capítulos, volumes e obras removidos da biblioteca com nome, pasta e data/hora.'
                      : 'Histórico de renomeações de capítulos e volumes realizadas na biblioteca.'}
                </p>
              </div>
              <div className="mam-toolbar-row">
                <div className="mam-segmented">
                  <button
                    className={`mam-segmented-btn ${subAbaLogs === 'downloads' ? 'active' : ''}`}
                    onClick={() => setSubAbaLogs('downloads')}
                  >
                    Downloads ({downloadsLog.length})
                  </button>
                  <button
                    className={`mam-segmented-btn ${subAbaLogs === 'exclusoes' ? 'active' : ''}`}
                    onClick={() => setSubAbaLogs('exclusoes')}
                  >
                    Exclusões ({exclusoesLog.length})
                  </button>
                  <button
                    className={`mam-segmented-btn ${subAbaLogs === 'renomeacoes' ? 'active' : ''}`}
                    onClick={() => setSubAbaLogs('renomeacoes')}
                  >
                    Renomeações ({renomeacoesLog.length})
                  </button>
                </div>
                <button
                  className="mam-btn"
                  onClick={async () => {
                    const [ops] = await Promise.all([
                      mamApi.sourceListarOperacoes(),
                      carregarObras(),
                    ])
                    setOperacoesDownload(ops)
                  }}
                >
                  Atualizar Log
                </button>
                <button className="mam-btn" onClick={handleLimparLog}>
                  🗑 Limpar Log
                </button>
              </div>
            </div>

            {subAbaLogs === 'renomeacoes' ? (
              renomeacoesLog.length === 0 ? (
                <p className="mam-section-desc">Nenhuma renomeação registrada até o momento.</p>
              ) : (
                <div style={{ display: 'grid', gap: '0.5rem' }}>
                  {renomeacoesLog.map((item) => (
                    <div key={item.id} className="mam-row-card">
                      <div>
                        <strong>✏️ {item.nome}</strong>
                        <div className="mam-brand-sub">Caminho: {item.pasta_destino}</div>
                      </div>
                      <span className="mam-pill-badge">{item.data_hora}</span>
                    </div>
                  ))}
                </div>
              )
            ) : subAbaLogs === 'exclusoes' ? (
              exclusoesLog.length === 0 ? (
                <p className="mam-section-desc">Nenhuma exclusão registrada até o momento.</p>
              ) : (
                <div style={{ display: 'grid', gap: '0.5rem' }}>
                  {exclusoesLog.map((item) => (
                    <div key={item.id} className="mam-row-card">
                      <div>
                        <strong>🗑 {item.nome}</strong>
                        <div className="mam-brand-sub">Caminho: {item.pasta_destino}</div>
                      </div>
                      <span className="mam-pill-badge">{item.data_hora}</span>
                    </div>
                  ))}
                </div>
              )
            ) : (() => {
              type GrupoDownload = {
                id: string
                titulo: string
                processados: number
                total: number
                percentual: number
                concluida: boolean
                pausada: boolean
                cancelada: boolean
                estado: string
                itens: DownloadLogItem[]
              }

              const grupos: GrupoDownload[] = []
              const idsOperacoesVistas = new Set<string>()

              for (const op of [...operacoesDownload].reverse()) {
                idsOperacoesVistas.add(op.id)
                grupos.push({
                  id: op.id,
                  titulo: op.obra_titulo,
                  processados: op.capitulos_processados,
                  total: op.total_capitulos,
                  percentual: op.percentual,
                  concluida: op.concluida,
                  pausada: Boolean(op.pausada || op.estado === 'pausado'),
                  cancelada: Boolean(op.cancelada || op.estado === 'cancelado'),
                  estado: op.estado ?? (op.concluida ? 'concluido' : 'executando'),
                  itens: downloadsLog.filter((item) => item.operacao_id === op.id),
                })
              }

              const gruposHistoricos = new Map<string, DownloadLogItem[]>()
              for (const item of downloadsLog) {
                if (item.operacao_id && idsOperacoesVistas.has(item.operacao_id)) continue
                const chave = item.operacao_id ?? 'historico-anterior'
                const lista = gruposHistoricos.get(chave) ?? []
                lista.push(item)
                gruposHistoricos.set(chave, lista)
              }

              for (const [chave, itens] of gruposHistoricos.entries()) {
                const tituloGrupo =
                  chave === 'historico-anterior'
                    ? 'Downloads anteriores'
                    : (itens[0]?.obra_titulo ?? itens[0]?.pasta_destino ?? 'Conjunto baixado')
                grupos.push({
                  id: chave,
                  titulo: tituloGrupo,
                  processados: itens.length,
                  total: itens.length,
                  percentual: 100,
                  concluida: true,
                  pausada: false,
                  cancelada: false,
                  estado: 'concluido',
                  itens,
                })
              }

              if (grupos.length === 0) {
                return <p className="mam-section-desc">Nenhum download registrado até o momento.</p>
              }

              return (
                <div style={{ display: 'grid', gap: '0.75rem' }}>
                  {grupos.map((grupo) => {
                    const aberto = Boolean(gruposLogAbertos[grupo.id])
                    return (
                      <div
                        key={grupo.id}
                        style={{
                          border: '1px solid var(--border-subtle)',
                          borderRadius: 12,
                          padding: '0.85rem 1rem',
                          background: 'rgba(15, 23, 42, 0.35)',
                        }}
                      >
                        <div
                          role="button"
                          tabIndex={0}
                          onClick={() =>
                            setGruposLogAbertos((prev) => ({
                              ...prev,
                              [grupo.id]: !prev[grupo.id],
                            }))
                          }
                          onKeyDown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                              setGruposLogAbertos((prev) => ({
                                ...prev,
                                [grupo.id]: !prev[grupo.id],
                              }))
                            }
                          }}
                          style={{
                            cursor: 'pointer',
                            display: 'flex',
                            flexDirection: 'column',
                            gap: '0.5rem',
                          }}
                        >
                          <div
                            style={{
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'center',
                              gap: '0.75rem',
                              flexWrap: 'wrap',
                            }}
                          >
                            <strong>
                              {aberto ? '▾' : '▸'}{' '}
                              {grupo.cancelada
                                ? '✖ Cancelado (removido)'
                                : grupo.pausada
                                  ? '⏸ Pausado'
                                  : grupo.concluida
                                    ? '✓ Concluído'
                                    : '⬇️ Baixando conjunto'}
                              : {grupo.titulo}
                            </strong>
                            <div
                              style={{ display: 'flex', gap: '0.45rem', alignItems: 'center' }}
                              onClick={(e) => e.stopPropagation()}
                            >
                              {!grupo.concluida && !grupo.cancelada && (
                                <>
                                  {grupo.pausada ? (
                                    <button
                                      type="button"
                                      className="mam-btn mam-btn-primary"
                                      onClick={async () => {
                                        await mamApi.sourceRetomarOperacao(grupo.id)
                                        const ops = await mamApi.sourceListarOperacoes()
                                        setOperacoesDownload(ops)
                                      }}
                                    >
                                      ▶ Despausar
                                    </button>
                                  ) : (
                                    <button
                                      type="button"
                                      className="mam-btn"
                                      onClick={async () => {
                                        await mamApi.sourcePausarOperacao(grupo.id)
                                        const ops = await mamApi.sourceListarOperacoes()
                                        setOperacoesDownload(ops)
                                      }}
                                    >
                                      ⏸ Pausar
                                    </button>
                                  )}
                                  <button
                                    type="button"
                                    className="mam-btn mam-btn-danger"
                                    onClick={() => {
                                      setModalConfirmacao({
                                        titulo: 'Cancelar Download',
                                        mensagem: `Cancelar o download de "${grupo.titulo}" e excluir todos os capítulos já baixados deste conjunto?`,
                                        textoConfirmar: 'Cancelar e Excluir',
                                        perigo: true,
                                        onConfirm: async () => {
                                          await mamApi.sourceCancelarOperacao(grupo.id)
                                          const [ops, hist] = await Promise.all([
                                            mamApi.sourceListarOperacoes(),
                                            mamApi.sourceListarHistoricoDownloads(),
                                          ])
                                          setOperacoesDownload(ops)
                                          setDownloadsLog(hist)
                                          await carregarObras()
                                        },
                                      })
                                    }}
                                  >
                                    ✖ Cancelar
                                  </button>
                                </>
                              )}
                              <span className="mam-pill-badge">
                                {grupo.processados}/{grupo.total} ({Math.round(grupo.percentual)}%)
                              </span>
                            </div>
                          </div>
                          <div
                            style={{
                              height: 6,
                              borderRadius: 999,
                              background: 'rgba(148, 163, 184, 0.18)',
                              overflow: 'hidden',
                            }}
                          >
                            <div
                              style={{
                                width: `${Math.min(100, Math.max(0, grupo.percentual))}%`,
                                height: '100%',
                                background: grupo.concluida ? 'var(--success)' : 'var(--accent)',
                                transition: 'width 250ms ease',
                              }}
                            />
                          </div>
                        </div>

                        {aberto && (
                          <div
                            style={{
                              marginTop: '0.75rem',
                              paddingTop: '0.75rem',
                              borderTop: '1px solid var(--border-subtle)',
                              display: 'grid',
                              gap: '0.45rem',
                            }}
                          >
                            {grupo.itens.length === 0 ? (
                              <p className="mam-section-desc">
                                Aguardando conclusão do primeiro capítulo deste conjunto...
                              </p>
                            ) : (
                              grupo.itens.map((item) => (
                                  <div
                                    key={item.id}
                                    className="mam-row-card"
                                    onClick={() => {
                                      if (!item.nome.startsWith('❌')) {
                                        handleAbrirCapituloDoLog(item)
                                      }
                                    }}
                                  >
                                    <div className="mam-chapter-row-main">
                                      <strong style={{ wordBreak: 'break-word', display: 'block' }}>
                                        {item.nome}
                                      </strong>
                                      <div
                                        className="mam-brand-sub"
                                        style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}
                                      >
                                        Pasta: {item.pasta_destino}
                                      </div>
                                    </div>
                                    <div className="mam-chapter-actions">
                                      <span className="mam-pill-badge" style={{ whiteSpace: 'nowrap' }}>
                                        {item.data_hora}
                                      </span>
                                      {!item.nome.startsWith('❌') && (
                                        <button
                                          className="mam-btn mam-btn-primary"
                                          onClick={(e) => {
                                            e.stopPropagation()
                                            handleAbrirCapituloDoLog(item)
                                          }}
                                        >
                                          Ler
                                        </button>
                                      )}
                                    </div>
                                  </div>
                              ))
                            )}
                          </div>
                        )}
                      </div>
                    )
                  })}
                </div>
              )
            })()}
          </section>
        )}

        {abaAtiva === 'transferencia' && (
          <section className="mam-panel">
            <h1 className="mam-section-title">TRANSFERÊNCIA & SINCRONIZAÇÃO</h1>
            <p className="mam-section-desc">
              Compare bibliotecas (ou containers <code>.zip</code>), sincronize sem sobrescritas inesperadas e valide integridade SHA-256.
            </p>

            <div
              style={{
                padding: '0.9rem 1rem',
                borderRadius: 10,
                border: '1px solid var(--border-subtle)',
                background: 'var(--bg-card)',
                display: 'grid',
                gap: '0.65rem',
                marginBottom: '0.5rem',
              }}
            >
              <strong>📦 Container Portátil de Distribuição (.zip)</strong>
              <span className="mam-brand-sub">
                Comprime toda a biblioteca atual (<code>Manga/</code> + <code>System/</code>) em um único arquivo <code>.zip</code> nomeável para envio ou importação em outro computador.
                Destino padrão: <code>{raizBiblioteca ? `${raizBiblioteca}/Exports` : 'Exports'}</code>
              </span>
              <div className="mam-toolbar-row">
                <input
                  className="mam-input mam-search-input"
                  placeholder="Nome do container (ex: MinhaColecao_2026)"
                  value={nomeContainer}
                  onChange={(e) => setNomeContainer(e.target.value)}
                />
                <input
                  className="mam-input mam-search-input"
                  placeholder="Pasta de saída (opcional; vazio usa Exports)"
                  value={pastaSaidaContainer}
                  onChange={(e) => setPastaSaidaContainer(e.target.value)}
                />
                <button
                  className="mam-btn mam-btn-primary"
                  disabled={empacotandoContainer}
                  onClick={handleEmpacotarContainer}
                >
                  {empacotandoContainer ? 'Comprimindo...' : 'Comprimir Biblioteca em Container'}
                </button>
              </div>
              {relatorioContainer && (
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    gap: '0.75rem',
                    flexWrap: 'wrap',
                    padding: '0.55rem 0.75rem',
                    borderRadius: 8,
                    background: 'rgba(59, 130, 246, 0.12)',
                    border: '1px solid rgba(59, 130, 246, 0.35)',
                  }}
                >
                  <div style={{ minWidth: 0, flex: 1 }}>
                    <div style={{ fontSize: '0.84rem' }}>
                      <strong>Container gerado:</strong>{' '}
                      <code style={{ wordBreak: 'break-all' }}>{relatorioContainer.caminho_arquivo}</code>
                    </div>
                    <div className="mam-brand-sub">
                      {relatorioContainer.quantidade_arquivos} arquivo(s) •{' '}
                      {(relatorioContainer.bytes_totais / (1024 * 1024)).toFixed(2)} MB
                    </div>
                  </div>
                  <button
                    className="mam-btn"
                    onClick={() => setOrigemTransfer(relatorioContainer.caminho_arquivo)}
                  >
                    Usar como Origem
                  </button>
                </div>
              )}
            </div>

            <div className="mam-toolbar-row">
              <input
                className="mam-input mam-search-input"
                placeholder="Caminho da biblioteca origem ou container (.zip)"
                value={origemTransfer}
                onChange={(e) => setOrigemTransfer(e.target.value)}
              />
              <input
                className="mam-input mam-search-input"
                placeholder={`Caminho da biblioteca destino${raizBiblioteca ? ` (ex: ${raizBiblioteca})` : ''}`}
                value={destinoTransfer}
                onChange={(e) => setDestinoTransfer(e.target.value)}
              />
            </div>
            <div className="mam-toolbar-row">
              <button className="mam-btn" onClick={handleCompararBibliotecas}>
                Comparar Bibliotecas
              </button>
              <button className="mam-btn" onClick={() => handleSincronizar(false)}>
                Sincronizar (Sem Sobrescrita)
              </button>
              <button className="mam-btn mam-btn-primary" onClick={handleCopiaIntegral}>
                Cópia Integral (SHA-256)
              </button>
            </div>
            {diferenca && (
              <div style={{ marginTop: '1rem', display: 'grid', gap: '0.85rem' }}>
                <div
                  style={{
                    padding: '0.85rem 1rem',
                    borderRadius: 10,
                    border: '1px solid rgba(34, 197, 94, 0.45)',
                    background: 'rgba(34, 197, 94, 0.1)',
                  }}
                >
                  <strong style={{ color: 'var(--success, #22c55e)' }}>
                    🟢 Ausentes no Destino — prontos para adicionar sem sobrescrever (
                    {diferenca.ausentes_no_destino.length})
                  </strong>
                  {diferenca.ausentes_no_destino.length === 0 ? (
                    <p className="mam-section-desc" style={{ marginTop: '0.35rem' }}>
                      Nenhum arquivo/pasta ausente no destino.
                    </p>
                  ) : (
                    <div style={{ display: 'grid', gap: '0.3rem', marginTop: '0.5rem' }}>
                      {diferenca.ausentes_no_destino.map((caminho) => (
                        <div
                          key={caminho}
                          style={{
                            padding: '0.4rem 0.65rem',
                            borderRadius: 6,
                            background: 'rgba(34, 197, 94, 0.14)',
                            color: '#bbf7d0',
                            fontFamily: 'monospace',
                            fontSize: '0.85rem',
                          }}
                        >
                          + {caminho}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {diferenca.diferentes.length > 0 && (
                  <div
                    style={{
                      padding: '0.85rem 1rem',
                      borderRadius: 10,
                      border: '1px solid rgba(234, 179, 8, 0.4)',
                      background: 'rgba(234, 179, 8, 0.08)',
                    }}
                  >
                    <strong style={{ color: '#facc15' }}>
                      🟡 Diferentes ({diferenca.diferentes.length})
                    </strong>
                    <div style={{ display: 'grid', gap: '0.3rem', marginTop: '0.5rem' }}>
                      {diferenca.diferentes.map((caminho) => (
                        <div
                          key={caminho}
                          style={{ fontFamily: 'monospace', fontSize: '0.85rem', color: '#fef08a' }}
                        >
                          ≠ {caminho}
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className="mam-panel" style={{ padding: '0.75rem 1rem' }}>
                  <strong>⚪ Idênticos ({diferenca.identicos.length})</strong>
                  {diferenca.identicos.length > 0 && (
                    <div style={{ display: 'grid', gap: '0.2rem', marginTop: '0.4rem' }}>
                      {diferenca.identicos.map((caminho) => (
                        <div
                          key={caminho}
                          className="mam-brand-sub"
                          style={{ fontFamily: 'monospace' }}
                        >
                          = {caminho}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}

            {relatorioTransfer && (
              <div className="mam-panel" style={{ marginTop: '1rem' }}>
                <strong>
                  Relatório de Transferência ({relatorioTransfer.operacao_id})
                </strong>
                <div className="mam-toolbar-row" style={{ marginTop: '0.5rem' }}>
                  <span className="mam-pill-badge">
                    ✓ Copiados: {relatorioTransfer.arquivos_copiados.length}
                  </span>
                  <span className="mam-pill-badge">
                    Ignorados: {relatorioTransfer.arquivos_ignorados.length}
                  </span>
                  <span className="mam-pill-badge">
                    {relatorioTransfer.bytes_transferidos} bytes
                  </span>
                </div>
              </div>
            )}
          </section>
        )}

        {abaAtiva === 'ai' && (
          <section className="mam-panel">
            <h1 className="mam-section-title">MANGA AI ENGINE</h1>
            <p className="mam-section-desc">
              Processo auxiliar Python local (JSON-RPC 2.0 stdio) — sem downloads automáticos.
            </p>
            <div className="mam-toolbar-row">
              <button className="mam-btn mam-btn-primary" onClick={handleVerificarAi}>
                Verificar Status do AI Engine
              </button>
              <input
                className="mam-input mam-search-input"
                placeholder="Diretório de modelos locais (.onnx / .pt)"
                value={diretorioModelos}
                onChange={(e) => setDiretorioModelos(e.target.value)}
              />
              <button className="mam-btn" onClick={handleListarModelos}>
                Listar Modelos Locais
              </button>
            </div>
            {aiStatus && <pre className="mam-pre">{JSON.stringify(aiStatus, null, 2)}</pre>}
            {modelos.map((m) => (
              <div key={m.caminho} className="mam-row-card">
                <span>{m.nome}</span>
                <span className="mam-pill-badge">
                  {m.formato} • {m.tamanho_bytes} bytes
                </span>
              </div>
            ))}
          </section>
        )}
      </main>

      {/* Modal de confirmação estilizado (substitui notificações nativas do WebView) */}
      {modalConfirmacao && (
        <div
          className="mam-modal-backdrop"
          onClick={() => setModalConfirmacao(null)}
        >
          <div
            className="mam-modal"
            style={{ maxWidth: 460 }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="mam-modal-header">
              <strong>{modalConfirmacao.titulo}</strong>
              <button
                type="button"
                className="mam-btn"
                onClick={() => setModalConfirmacao(null)}
              >
                ✕
              </button>
            </div>
            <p className="mam-section-desc" style={{ margin: 0 }}>
              {modalConfirmacao.mensagem}
            </p>
            <div className="mam-toolbar-row" style={{ justifyContent: 'flex-end' }}>
              <button
                type="button"
                className="mam-btn"
                onClick={() => setModalConfirmacao(null)}
              >
                Cancelar
              </button>
              <button
                type="button"
                className={`mam-btn ${modalConfirmacao.perigo ? 'mam-btn-danger' : 'mam-btn-primary'}`}
                onClick={async () => {
                  const fn = modalConfirmacao.onConfirm
                  setModalConfirmacao(null)
                  await fn()
                }}
              >
                {modalConfirmacao.textoConfirmar}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal informativo de Como Usar o Importar por Link e Sites Suportados */}
      {modalInfoImportacaoAberto && (
        <div
          className="mam-modal-backdrop"
          onClick={() => setModalInfoImportacaoAberto(false)}
        >
          <div
            className="mam-modal"
            style={{ maxWidth: 620 }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="mam-modal-header">
              <strong>ⓘ Como usar o Importar por Link & Sites Suportados</strong>
              <button
                type="button"
                className="mam-btn"
                onClick={() => setModalInfoImportacaoAberto(false)}
              >
                ✕
              </button>
            </div>
            <div style={{ display: 'grid', gap: '0.75rem', fontSize: '0.92rem' }}>
              <div>
                <strong>1. Página Central da Obra (Lote):</strong> cole o link principal do mangá
                (ex.: <code>https://mangafire.to/title/...</code> ou{' '}
                <code>https://mangalivre.to/manga/...</code>), escolha o idioma preferido (padrão{' '}
                <strong>PT-BR</strong> com sequência automática em <strong>ENG</strong> caso haja
                capítulos pendentes) e clique em <strong>Descobrir Capítulos</strong>. Depois marque
                os capítulos e clique em <strong>Arquivar Selecionados</strong>.
              </div>
              <div>
                <strong>2. Capítulo Único / URLs Manuais:</strong> ideal para baixar um capítulo
                específico pelo link direto, lista ordenada de URLs de imagens ou HTML copiado.
              </div>
              <div>
                <strong>3. Conectores ativos (v{conectoresCatalogo?.version ?? 'padrão'}):</strong>
                <ul style={{ margin: '0.35rem 0 0 1.2rem', padding: 0 }}>
                  {(conectoresCatalogo?.sites ?? []).map((s) => (
                    <li key={s.id}>
                      <strong>{s.nome}</strong> (<code>{s.dominios.join(', ')}</code> — modo{' '}
                      <code>{s.tipo}</code>)
                    </li>
                  ))}
                  <li>
                    <strong>Importar por Link / URL externa</strong> — conector adaptativo com
                    Leitor Web Inteligente (CDP + IA)
                  </li>
                </ul>
              </div>
              <div style={{ display: 'grid', gap: '0.4rem' }}>
                <strong>Atualizar ou importar conector JSON (sem reinstalar o app):</strong>
                <textarea
                  className="mam-textarea"
                  rows={3}
                  placeholder="Opcional: cole aqui um catálogo JSON de conectores para importar offline..."
                  value={conectorJsonManual}
                  onChange={(e) => setConectorJsonManual(e.target.value)}
                />
                <div className="mam-toolbar-row" style={{ justifyContent: 'space-between' }}>
                  <button
                    type="button"
                    className="mam-btn"
                    onClick={async () => {
                      try {
                        const cat = await mamApi.sourceAtualizarConectoresRemoto()
                        setConectoresCatalogo(cat)
                        setStatusAtualizacaoConectores({
                          versao_atual: cat.version,
                          versao_remota: cat.version,
                          atualizacao_disponivel: false,
                          notas: null,
                        })
                        setMensagem(`Conectores atualizados para v${cat.version}!`)
                      } catch (err) {
                        setMensagem(`Erro ao atualizar conectores: ${String(err)}`)
                      }
                    }}
                  >
                    🔄 Atualizar Conectores Online
                  </button>
                  {conectorJsonManual.trim() && (
                    <button
                      type="button"
                      className="mam-btn mam-btn-primary"
                      onClick={async () => {
                        try {
                          const cat = await mamApi.sourceImportarConectorJson(conectorJsonManual)
                          setConectoresCatalogo(cat)
                          setConectorJsonManual('')
                          setMensagem(`Conectores importados (v${cat.version})!`)
                        } catch (err) {
                          setMensagem(`JSON de conector inválido: ${String(err)}`)
                        }
                      }}
                    >
                      Importar JSON Offline
                    </button>
                  )}
                </div>
              </div>
            </div>
            <div className="mam-toolbar-row" style={{ justifyContent: 'flex-end' }}>
              <button
                type="button"
                className="mam-btn mam-btn-primary"
                onClick={() => setModalInfoImportacaoAberto(false)}
              >
                Entendi
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal flutuante de Importação por Link Direto / HTML / Lista Ordenada */}
      {modalImportarUrlAberto && (
        <div className="mam-modal-backdrop">
          <form
            className="mam-modal"
            style={{ maxWidth: 620 }}
            onClick={(e) => e.stopPropagation()}
            onSubmit={(e) => {
              if (modoImportacao === 'obra') {
                e.preventDefault()
                handleDescobrirCapitulosPorLink()
              } else {
                handleImportarExtrator(e)
              }
            }}
          >
            <div className="mam-modal-header">
              <strong>Importar por Link / Página da Obra</strong>
              <button
                type="button"
                className="mam-btn"
                onClick={() => setModalImportarUrlAberto(false)}
              >
                ✕
              </button>
            </div>
            <div className="mam-segmented" style={{ alignSelf: 'flex-start' }}>
              <button
                type="button"
                className={`mam-segmented-btn ${modoImportacao === 'obra' ? 'active' : ''}`}
                onClick={() => setModoImportacao('obra')}
              >
                Página Central da Obra (Lote)
              </button>
              <button
                type="button"
                className={`mam-segmented-btn ${modoImportacao === 'capitulo' ? 'active' : ''}`}
                onClick={() => setModoImportacao('capitulo')}
              >
                Capítulo Único / URLs Manuais
              </button>
            </div>
            <p className="mam-section-desc">
              {modoImportacao === 'obra'
                ? 'Cole o link principal da obra (ex.: mangafire.to/title/... ou mangalivre.to/manga/...) para listar e selecionar todos os capítulos.'
                : `Extratores ativos: ${extratores.map((e) => e.id).join(', ') || 'mangadex, mangafire, generico'}`}
            </p>

            {modoImportacao === 'obra' ? (
              <>
                <div className="mam-toolbar-row">
                  <input
                    className="mam-input mam-search-input"
                    placeholder="URL central da obra (ex.: weebcentral.com/series/... ou mangafire.to/title/...)"
                    value={extUrl}
                    onChange={(e) => setExtUrl(e.target.value)}
                  />
                  {extUrl.trim().startsWith('http') && (
                    <>
                      <button
                        type="button"
                        className="mam-btn"
                        onClick={async () => {
                          try {
                            await mamApi.browserAbrirJanelaAssistida(extUrl.trim())
                            setMensagem('Janela Assistida aberta. Carregue a página/verificação e clique em "Capturar da Janela".')
                          } catch (err) {
                            setMensagem(`Erro ao abrir janela assistida: ${String(err)}`)
                          }
                        }}
                        title="Abrir página na janela assistida do MAM para sites com verificação humana"
                      >
                        Janela Assistida
                      </button>
                      <button
                        type="button"
                        className="mam-btn mam-btn-primary"
                        onClick={async () => {
                          try {
                            const cap = await mamApi.browserCapturarJanelaAssistida()
                            if (!extUrl.trim()) {
                              setExtUrl(cap.url)
                            }
                            setExtHtml(cap.html)
                            if (modoImportacao === 'capitulo' && cap.urls_imagens.length > 0) {
                              setExtUrlsTexto(cap.urls_imagens.join('\n'))
                            }
                            setMensagem(`HTML capturado da janela assistida (${cap.urls_imagens.length} imagem(ns)). Clique em "Descobrir Capítulos".`)
                          } catch (err) {
                            setMensagem(String(err))
                          }
                        }}
                        title="Capturar HTML já carregado na Janela Assistida"
                      >
                        Capturar da Janela
                      </button>
                      <button
                        type="button"
                        className="mam-btn"
                        onClick={async () => {
                          try {
                            const cfg = await mamApi.browserEnsinarPadraoCapitulo(extUrl.trim())
                            const atualizados = await mamApi.sourceListarConectores()
                            setConectoresCatalogo(atualizados)
                            setMensagem(
                              `Padrão aprendido para "${cfg.nome}": ${cfg.padroes_url_capitulo[0] ?? ''}. Volte à página da obra (ou use o HTML já capturado) e clique em "Descobrir Capítulos".`,
                            )
                          } catch (err) {
                            setMensagem(String(err))
                          }
                        }}
                        title="Entre em um capítulo na Janela Assistida e clique aqui para ensinar o padrão de URL à IA"
                      >
                        🎓 Ensinar Capítulo
                      </button>
                    </>
                  )}
                </div>
                <input
                  className="mam-input"
                  placeholder="Título personalizado (opcional — detectado automaticamente)"
                  value={extObraTitulo}
                  onChange={(e) => setExtObraTitulo(e.target.value)}
                />
                <div className="mam-toolbar-row" style={{ alignItems: 'center' }}>
                  <span className="mam-section-desc" style={{ margin: 0 }}>
                    Idioma preferido no site:
                  </span>
                  <select
                    className="mam-select"
                    value={extIdiomaPreferido}
                    onChange={(e) =>
                      setExtIdiomaPreferido(e.target.value as 'pt-br' | 'en' | 'ja' | 'all')
                    }
                  >
                    <option value="pt-br">PT-BR (Preferido)</option>
                    <option value="en">ENG (Inglês)</option>
                    <option value="ja">JPN (Japonês)</option>
                    <option value="all">Todos / Padrão do site</option>
                  </select>
                </div>
                <textarea
                  className="mam-textarea"
                  placeholder="HTML da página principal da obra (opcional — útil se o site carregar capítulos via JS no navegador)"
                  value={extHtml}
                  onChange={(e) => setExtHtml(e.target.value)}
                  rows={4}
                />
                <div className="mam-toolbar-row" style={{ justifyContent: 'flex-end' }}>
                  <button
                    type="button"
                    className="mam-btn"
                    onClick={() => {
                      setExtUrl('')
                      setExtObraTitulo('')
                      setExtHtml('')
                      setExtNumeroCap('1')
                      setExtUrlsTexto('')
                      setResultadoImportacao(null)
                    }}
                  >
                    Limpar
                  </button>
                  <button
                    type="button"
                    className="mam-btn"
                    onClick={() => setModalImportarUrlAberto(false)}
                  >
                    Fechar
                  </button>
                  <button className="mam-btn mam-btn-primary" type="submit">
                    🔎 Descobrir Capítulos
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="mam-toolbar-row">
                  <input
                    className="mam-input mam-search-input"
                    placeholder="Título da obra (opcional se informar o link do capítulo)"
                    value={extObraTitulo}
                    onChange={(e) => setExtObraTitulo(e.target.value)}
                  />
                  <input
                    className="mam-input"
                    type="number"
                    placeholder="Cap."
                    value={extNumeroCap}
                    onChange={(e) => setExtNumeroCap(e.target.value)}
                    style={{ width: 90 }}
                  />
                </div>
                <div className="mam-toolbar-row">
                  <input
                    className="mam-input mam-search-input"
                    placeholder="Link do capítulo (weebcentral.com/chapters/..., mangadex.org/chapter/..., mangafire.to/...)"
                    value={extUrl}
                    onChange={(e) => setExtUrl(e.target.value)}
                  />
                  {extUrl.trim().startsWith('http') && (
                    <>
                      <button
                        type="button"
                        className="mam-btn"
                        onClick={handleDescobrirCapitulosPorLink}
                        title="Inspecionar capítulo único e exibir suas páginas/metadados na aba Fontes"
                      >
                        🔎 Inspecionar
                      </button>
                      <button
                        type="button"
                        className="mam-btn"
                        onClick={async () => {
                          try {
                            await mamApi.browserAbrirJanelaAssistida(extUrl.trim())
                            setMensagem('Janela Assistida aberta. Carregue o capítulo e clique em "Capturar".')
                          } catch (err) {
                            setMensagem(`Erro ao abrir janela assistida: ${String(err)}`)
                          }
                        }}
                      >
                        Janela Assistida
                      </button>
                      <button
                        type="button"
                        className="mam-btn"
                        onClick={async () => {
                          try {
                            const cap = await mamApi.browserCapturarJanelaAssistida()
                            if (!extUrl.trim()) setExtUrl(cap.url)
                            setExtHtml(cap.html)
                            if (cap.urls_imagens.length > 0) {
                              setExtUrlsTexto(cap.urls_imagens.join('\n'))
                            }
                            setMensagem(`Capturadas ${cap.urls_imagens.length} página(s) da Janela Assistida.`)
                          } catch (err) {
                            setMensagem(String(err))
                          }
                        }}
                      >
                        Capturar
                      </button>
                    </>
                  )}
                </div>
                <textarea
                  className="mam-textarea"
                  placeholder="Lista ordenada de URLs de páginas (uma por linha, opcional)"
                  value={extUrlsTexto}
                  onChange={(e) => setExtUrlsTexto(e.target.value)}
                  rows={3}
                />
                <textarea
                  className="mam-textarea"
                  placeholder="HTML da página do capítulo (opcional)"
                  value={extHtml}
                  onChange={(e) => setExtHtml(e.target.value)}
                  rows={3}
                />
                <div className="mam-toolbar-row" style={{ justifyContent: 'flex-end' }}>
                  <button
                    type="button"
                    className="mam-btn"
                    onClick={() => {
                      setExtUrl('')
                      setExtObraTitulo('')
                      setExtHtml('')
                      setExtNumeroCap('1')
                      setExtUrlsTexto('')
                      setResultadoImportacao(null)
                    }}
                  >
                    Limpar
                  </button>
                  <button
                    type="button"
                    className="mam-btn"
                    onClick={() => setModalImportarUrlAberto(false)}
                  >
                    Fechar
                  </button>
                  <button className="mam-btn mam-btn-primary" type="submit">
                    Importar Capítulo Único
                  </button>
                </div>
              </>
            )}
            {resultadoImportacao && (
              <pre className="mam-pre">{JSON.stringify(resultadoImportacao, null, 2)}</pre>
            )}
          </form>
        </div>
      )}

      {(popupDownload || mensagem) && (
        <div
          style={{
            position: 'fixed',
            right: '1.25rem',
            bottom: '3rem',
            zIndex: 1200,
            display: 'flex',
            flexDirection: 'column',
            gap: '0.6rem',
            maxWidth: 380,
          }}
        >
          {mensagem && (
            <div
              style={{
                padding: '0.75rem 0.95rem',
                borderRadius: 12,
                background: 'var(--bg-elevated, #111827)',
                border: '1px solid var(--border-subtle, rgba(148,163,184,0.3))',
                boxShadow: '0 12px 30px rgba(0, 0, 0, 0.45)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: '0.75rem',
              }}
            >
              <span style={{ fontSize: '0.9rem' }}>{mensagem}</span>
              <button
                className="mam-btn"
                onClick={() => setMensagem(null)}
                style={{ padding: '0.2rem 0.45rem', flexShrink: 0 }}
              >
                ✕
              </button>
            </div>
          )}

          {popupDownload && (
            <div
              role="button"
              tabIndex={0}
              onClick={() => {
                setAbaAtiva('downloads')
                setPopupDownload(null)
                carregarObras()
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  setAbaAtiva('downloads')
                  setPopupDownload(null)
                  carregarObras()
                }
              }}
              style={{
                padding: '0.85rem 1.05rem',
                borderRadius: 12,
                background: 'var(--bg-elevated, #111827)',
                border: '1px solid var(--accent, #38bdf8)',
                boxShadow: '0 12px 30px rgba(0, 0, 0, 0.45)',
                cursor: 'pointer',
              }}
            >
              <div style={{ fontWeight: 700, marginBottom: '0.2rem' }}>
                ⬇️ Baixando conjunto: {popupDownload.obra} ({popupDownload.total} cap.)
              </div>
              <div className="mam-brand-sub">
                Clique aqui para acompanhar o progresso no Log de Downloads
              </div>
            </div>
          )}
        </div>
      )}

      <footer className="mam-statusbar">
        <span>MAM v0.1.0 • Portátil</span>
        <span className="mam-status-ready">
          <span className="mam-status-dot" />
          System Ready
        </span>
      </footer>
    </div>
  )
}

export default App
