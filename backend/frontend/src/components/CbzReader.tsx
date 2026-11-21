import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import {
  Capitulo,
  MarcadorPagina,
  PaginaExtraida,
  ResultadoIA,
  VolumeComCapitulos,
  mamApi,
} from '../api'

type ModoLeitura = 'horizontal' | 'vertical' | 'dupla'
type AjusteTela = 'fit-width' | 'fit-height' | 'original'

interface CbzReaderProps {
  capitulo: Capitulo
  capitulosDisponiveis: Capitulo[]
  volumesDisponiveis?: VolumeComCapitulos[]
  paginaInicial?: number | null
  onSelecionarCapitulo: (capitulo: Capitulo) => void
  onFechar: () => void
}

export function CbzReader({
  capitulo,
  capitulosDisponiveis,
  volumesDisponiveis = [],
  paginaInicial = null,
  onSelecionarCapitulo,
  onFechar,
}: CbzReaderProps) {
  const [indiceAtual, setIndiceAtual] = useState(0)
  const [paginaPrincipal, setPaginaPrincipal] = useState<PaginaExtraida | null>(null)
  const [paginaSecundaria, setPaginaSecundaria] = useState<PaginaExtraida | null>(null)
  const [paginasLongStrip, setPaginasLongStrip] = useState<Record<number, PaginaExtraida>>({})
  const [totalPaginasLongStrip, setTotalPaginasLongStrip] = useState(0)
  const [modo, setModo] = useState<ModoLeitura>('horizontal')
  const [ajuste, setAjuste] = useState<AjusteTela>('fit-width')
  const [zoom, setZoom] = useState(30)
  const [zoomPadraoPorModo, setZoomPadraoPorModo] = useState<Record<ModoLeitura, number>>({
    horizontal: 30,
    dupla: 110,
    vertical: 40,
  })
  const [usarZoomPadrao, setUsarZoomPadrao] = useState(true)
  const [barraVisivel, setBarraVisivel] = useState(true)
  const [telaCheia, setTelaCheia] = useState(false)
  const [paginaHover, setPaginaHover] = useState<number | null>(null)
  const viewportRef = useRef<HTMLDivElement | null>(null)
  const lastScrollTopRef = useRef(0)
  const longStripPageRefs = useRef<(HTMLElement | null)[]>([])
  const longStripCapituloRef = useRef<string | null>(null)
  const scrollRafRef = useRef<number | null>(null)
  const pendingScrollRatioRef = useRef<{ x: number; y: number } | null>(null)
  const [marcadores, setMarcadores] = useState<MarcadorPagina[]>([])
  const [erro, setErro] = useState<string | null>(null)
  const [carregando, setCarregando] = useState(false)

  const [mostrarIa, setMostrarIa] = useState(false)
  const [mostrarSobreposicao, setMostrarSobreposicao] = useState(true)
  const [idiomaDestinoIa, setIdiomaDestinoIa] = useState<'pt-BR' | 'en'>('pt-BR')
  const [processandoIa, setProcessandoIa] = useState(false)
  const [resultadosIa, setResultadosIa] = useState<ResultadoIA[]>([])
  const [textoRevisao, setTextoRevisao] = useState('')

  const [modalLayoutAberto, setModalLayoutAberto] = useState(false)
  const [modalCapitulosAberto, setModalCapitulosAberto] = useState(false)
  const [abaModalCap, setAbaModalCap] = useState<'capitulos' | 'volumes'>('capitulos')
  const [buscaModalCap, setBuscaModalCap] = useState('')
  const [ordemModalCap, setOrdemModalCap] = useState<'desc' | 'asc'>('desc')

  const indiceCapituloAtual = capitulosDisponiveis.findIndex((c) => c.id === capitulo.id)
  const capituloAnterior =
    indiceCapituloAtual > 0 ? capitulosDisponiveis[indiceCapituloAtual - 1] : null
  const proximoCapitulo =
    indiceCapituloAtual >= 0 && indiceCapituloAtual < capitulosDisponiveis.length - 1
      ? capitulosDisponiveis[indiceCapituloAtual + 1]
      : null

  useEffect(() => {
    let ativo = true
    async function inicializar() {
      try {
        const [progresso, listaMarcadores, listaIa] = await Promise.all([
          mamApi.obterProgresso(capitulo.id),
          mamApi.listarMarcadores(capitulo.id),
          mamApi.aiListarResultadosCapitulo(capitulo.id),
        ])
        if (!ativo) return
        setIndiceAtual(paginaInicial !== null && paginaInicial !== undefined ? paginaInicial : progresso)
        setMarcadores(listaMarcadores)
        setResultadosIa(listaIa)
      } catch {
        if (!ativo) return
        setIndiceAtual(paginaInicial ?? 0)
      }
    }
    inicializar()
    return () => {
      ativo = false
    }
  }, [capitulo.id, paginaInicial])

  useEffect(() => {
    if (usarZoomPadrao) {
      setZoom(zoomPadraoPorModo[modo])
    }
  }, [capitulo.id, modo, usarZoomPadrao, zoomPadraoPorModo])

  useEffect(() => {
    if (usarZoomPadrao && modo !== 'vertical') {
      setZoom(zoomPadraoPorModo[modo])
    }
  }, [indiceAtual])

  useEffect(() => {
    if (modo !== 'vertical') {
      setBarraVisivel(true)
    }
  }, [modo])

  useEffect(() => {
    let ativo = true
    async function carregarPaginas() {
      const mudouCapitulo = longStripCapituloRef.current !== capitulo.caminho_relativo
      if (modo === 'vertical' && !mudouCapitulo && totalPaginasLongStrip > 0) {
        await mamApi.salvarProgresso(capitulo.id, indiceAtual)
        if (indiceAtual >= totalPaginasLongStrip - 1) {
          await mamApi.marcarLido(capitulo.id, true)
        }
        const inicio = Math.max(0, indiceAtual - 2)
        const fim = Math.min(totalPaginasLongStrip - 1, indiceAtual + 4)
        const faltantes: number[] = []
        for (let i = inicio; i <= fim; i++) {
          if (!paginasLongStrip[i]) faltantes.push(i)
        }
        if (faltantes.length > 0) {
          const carregadas = await Promise.all(
            faltantes.map(async (i) => ({
              i,
              pag: await mamApi.obterPagina(capitulo.caminho_relativo, i),
            })),
          )
          if (!ativo) return
          setPaginasLongStrip((prev) => {
            const next: Record<number, PaginaExtraida> = {}
            for (const [k, v] of Object.entries(prev)) {
              const idx = Number(k)
              if (Math.abs(idx - indiceAtual) <= 10) {
                next[idx] = v
              }
            }
            for (const item of carregadas) {
              next[item.i] = item.pag
            }
            return next
          })
        }
        return
      }

      setCarregando(true)
      setErro(null)
      try {
        const p1 = await mamApi.obterPagina(capitulo.caminho_relativo, indiceAtual)
        if (!ativo) return
        setPaginaPrincipal(p1)
        await mamApi.salvarProgresso(capitulo.id, indiceAtual)

        if (p1.total_paginas > 0 && indiceAtual >= p1.total_paginas - 1) {
          await mamApi.marcarLido(capitulo.id, true)
        }

        if (modo === 'vertical') {
          const inicio = Math.max(0, indiceAtual - 2)
          const fim = Math.min(p1.total_paginas - 1, indiceAtual + 4)
          const mapaInicial: Record<number, PaginaExtraida> = { [indiceAtual]: p1 }
          const indicesExtras: number[] = []
          for (let i = inicio; i <= fim; i++) {
            if (i !== indiceAtual) indicesExtras.push(i)
          }
          const extras = await Promise.all(
            indicesExtras.map(async (i) => ({
              i,
              pag: await mamApi.obterPagina(capitulo.caminho_relativo, i),
            })),
          )
          if (!ativo) return
          for (const item of extras) {
            mapaInicial[item.i] = item.pag
          }
          longStripCapituloRef.current = capitulo.caminho_relativo
          setTotalPaginasLongStrip(p1.total_paginas)
          setPaginasLongStrip(mapaInicial)
          setPaginaSecundaria(null)
        } else if (modo === 'dupla' && indiceAtual + 1 < p1.total_paginas) {
          longStripCapituloRef.current = null
          setTotalPaginasLongStrip(0)
          setPaginasLongStrip({})
          const p2 = await mamApi.obterPagina(capitulo.caminho_relativo, indiceAtual + 1)
          if (ativo) setPaginaSecundaria(p2)
          if (indiceAtual + 2 < p1.total_paginas) {
            mamApi.obterPagina(capitulo.caminho_relativo, indiceAtual + 2).catch(() => {})
          }
        } else {
          longStripCapituloRef.current = null
          setTotalPaginasLongStrip(0)
          setPaginasLongStrip({})
          setPaginaSecundaria(null)
          if (indiceAtual + 1 < p1.total_paginas) {
            mamApi.obterPagina(capitulo.caminho_relativo, indiceAtual + 1).catch(() => {})
          }
        }
      } catch (e) {
        if (ativo) setErro(String(e))
      } finally {
        if (ativo) setCarregando(false)
      }
    }
    carregarPaginas()
    return () => {
      ativo = false
    }
  }, [capitulo.id, capitulo.caminho_relativo, indiceAtual, modo])

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'ArrowRight') avancarPagina()
      if (e.key === 'ArrowLeft') voltarPagina()
      if (e.key === 'Escape') {
        setModalLayoutAberto(false)
        setModalCapitulosAberto(false)
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  })

  const totalPaginas = paginaPrincipal?.total_paginas ?? capitulo.quantidade_paginas
  const passo = modo === 'dupla' ? 2 : 1

  async function alternarTelaCheia() {
    try {
      if (!document.fullscreenElement) {
        await document.documentElement.requestFullscreen()
      } else {
        await document.exitFullscreen()
      }
    } catch {
      // Ignora se bloqueado pelo ambiente
    }
  }

  function aplicarZoom(updater: number | ((prev: number) => number)) {
    const vp = viewportRef.current
    if (vp) {
      const ratioX =
        vp.scrollWidth > vp.clientWidth
          ? (vp.scrollLeft + vp.clientWidth / 2) / vp.scrollWidth
          : 0.5
      const ratioY =
        vp.scrollHeight > vp.clientHeight
          ? (vp.scrollTop + vp.clientHeight / 2) / vp.scrollHeight
          : 0.5
      pendingScrollRatioRef.current = { x: ratioX, y: ratioY }
    }
    setZoom((prev) => {
      const next = typeof updater === 'function' ? updater(prev) : updater
      return Math.min(200, Math.max(10, next))
    })
  }

  useLayoutEffect(() => {
    const vp = viewportRef.current
    if (!vp) return
    const ratio = pendingScrollRatioRef.current
    pendingScrollRatioRef.current = null
    if (!ratio) return

    if (vp.scrollWidth > vp.clientWidth) {
      vp.scrollLeft = Math.max(0, ratio.x * vp.scrollWidth - vp.clientWidth / 2)
    } else {
      vp.scrollLeft = 0
    }
    if (vp.scrollHeight > vp.clientHeight) {
      vp.scrollTop = Math.max(0, ratio.y * vp.scrollHeight - vp.clientHeight / 2)
    } else {
      vp.scrollTop = 0
    }
  }, [zoom, modo])

  function irParaPagina(idx: number) {
    setIndiceAtual(idx)
    if (modo === 'vertical') {
      requestAnimationFrame(() => {
        longStripPageRefs.current[idx]?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      })
    }
  }

  useEffect(() => {
    function onFullscreenChange() {
      setTelaCheia(Boolean(document.fullscreenElement))
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'F11') {
        e.preventDefault()
        alternarTelaCheia()
      } else if (modo === 'vertical' && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
        e.preventDefault()
        const delta = e.key === 'ArrowDown' ? 220 : -220
        viewportRef.current?.scrollBy({ top: delta, behavior: 'smooth' })
      }
    }
    document.addEventListener('fullscreenchange', onFullscreenChange)
    window.addEventListener('keydown', onKeyDown)
    return () => {
      document.removeEventListener('fullscreenchange', onFullscreenChange)
      window.removeEventListener('keydown', onKeyDown)
    }
  }, [modo])

  function avancarPagina() {
    if (totalPaginas > 0 && indiceAtual + passo < totalPaginas) {
      setIndiceAtual((prev) => prev + passo)
    } else if (proximoCapitulo) {
      onSelecionarCapitulo(proximoCapitulo)
    }
  }

  function voltarPagina() {
    if (indiceAtual - passo >= 0) {
      setIndiceAtual((prev) => prev - passo)
    } else if (indiceAtual > 0) {
      setIndiceAtual(0)
    } else if (capituloAnterior) {
      onSelecionarCapitulo(capituloAnterior)
    }
  }

  const paginaAtualMarcada = marcadores.some((m) => m.pagina === indiceAtual)

  async function handleAlternarMarcador() {
    if (paginaAtualMarcada) {
      await mamApi.removerMarcador(capitulo.id, indiceAtual)
    } else {
      await mamApi.adicionarMarcador(capitulo.id, indiceAtual, `Página ${indiceAtual + 1}`)
    }
    const atualizados = await mamApi.listarMarcadores(capitulo.id)
    setMarcadores(atualizados)
  }

  const estiloImagem: React.CSSProperties = {
    width: ajuste === 'fit-width' ? `${zoom}%` : 'auto',
    maxWidth: ajuste === 'fit-width' ? (zoom <= 100 ? '100%' : 'none') : undefined,
    maxHeight: ajuste === 'fit-height' ? `${0.82 * zoom}vh` : 'none',
    transform: ajuste === 'original' ? `scale(${zoom / 100})` : undefined,
    transformOrigin: 'center center',
    objectFit: 'contain',
    borderRadius: 6,
    boxShadow: '0 8px 28px rgba(0,0,0,0.55)',
    display: 'block',
  }

  function handleClickNavegacaoImagem(e: React.MouseEvent<HTMLElement>) {
    const rect = e.currentTarget.getBoundingClientRect()
    const relX = e.clientX - rect.left
    if (relX < rect.width / 2) {
      voltarPagina()
    } else {
      avancarPagina()
    }
  }

  const capsModalFiltrados = [...capitulosDisponiveis]
    .filter((c) =>
      buscaModalCap.trim() ? String(c.numero).includes(buscaModalCap.trim()) : true,
    )
    .sort((a, b) => (ordemModalCap === 'desc' ? b.numero - a.numero : a.numero - b.numero))

  function renderizarSobreposicaoPagina(paginaIndice: number) {
    if (!mostrarSobreposicao) return null
    const regioes = resultadosIa
      .filter((r) => r.pagina_numero === paginaIndice)
      .flatMap((r) => {
        try {
          const parsed = JSON.parse(r.resultado_json) as {
            regioes?: Array<{
              bbox?: [number, number, number, number]
              texto_traduzido?: string
              texto_original?: string
              cor_mascara?: string
            }>
          }
          return parsed.regioes ?? []
        } catch {
          return []
        }
      })

    if (regioes.length === 0) return null

    return regioes.map((reg, idx) => {
      const [x, y, w, h] = reg.bbox ?? [0, 0, 0.2, 0.1]
      return (
        <div
          key={`${paginaIndice}-reg-${idx}`}
          title={reg.texto_original || reg.texto_traduzido || 'Balão detectado'}
          style={{
            position: 'absolute',
            left: `${x * 100}%`,
            top: `${y * 100}%`,
            width: `${w * 100}%`,
            height: `${h * 100}%`,
            background: reg.cor_mascara || '#ffffff',
            color: '#0f172a',
            border: '1.5px solid rgba(56, 189, 248, 0.85)',
            boxShadow: '0 0 0 1px rgba(15, 23, 42, 0.12)',
            padding: '2px 4px',
            fontSize: 'clamp(9px, 1.05vw, 14px)',
            fontWeight: 700,
            lineHeight: 1.15,
            borderRadius: '40%',
            boxSizing: 'border-box',
            overflow: 'hidden',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            textAlign: 'center',
            pointerEvents: 'none',
            zIndex: 4,
          }}
        >
          {reg.texto_traduzido ?? reg.texto_original ?? ''}
        </div>
      )
    })
  }

  return (
    <div className="mam-reader-shell">
      {barraVisivel && (
      <div className="mam-reader-toolbar">
        <div className="mam-toolbar-row">
          <button className="mam-btn" onClick={onFechar}>
            ← Biblioteca
          </button>
          <button className="mam-btn" onClick={() => setModalCapitulosAberto(true)}>
            Cap. {capitulo.numero} ({indiceAtual + 1}/{Math.max(totalPaginas, 1)}) ▾
          </button>
          <button className="mam-btn" onClick={voltarPagina}>
            ◀ Anterior
          </button>
          <button className="mam-btn mam-btn-primary" onClick={avancarPagina}>
            Próxima ▶
          </button>
        </div>

        <div className="mam-toolbar-row">
          <button className="mam-btn" onClick={() => setModalLayoutAberto(true)}>
            ⚙ Layout
          </button>
          <button
            className={`mam-btn ${telaCheia ? 'mam-btn-active' : ''}`}
            onClick={alternarTelaCheia}
            title="Tela Cheia (F11)"
          >
            {telaCheia ? '⤢ Sair Tela Cheia' : '⛶ Tela Cheia'}
          </button>
          <button
            className={`mam-btn ${paginaAtualMarcada ? 'mam-btn-active' : ''}`}
            onClick={handleAlternarMarcador}
          >
            {paginaAtualMarcada ? '🔖 Desmarcar' : '🔖 Marcar'}
          </button>
          <button
            className={`mam-btn ${mostrarSobreposicao ? 'mam-btn-active' : ''}`}
            onClick={() => setMostrarSobreposicao((v) => !v)}
          >
            Sobreposição: {mostrarSobreposicao ? 'ON' : 'OFF'}
          </button>
          <select
            className="mam-select"
            aria-label="Idioma de destino da tradução IA"
            value={idiomaDestinoIa}
            onChange={(e) => setIdiomaDestinoIa(e.target.value as 'pt-BR' | 'en')}
            title="Idioma de destino para tradução dos balões (origem detectada automaticamente)"
          >
            <option value="pt-BR">Traduzir: PT-BR</option>
            <option value="en">Traduzir: EN</option>
          </select>
          <button
            className="mam-btn mam-btn-primary"
            disabled={processandoIa}
            onClick={async () => {
              setProcessandoIa(true)
              setErro(null)
              try {
                await mamApi.aiProcessarCapitulo({
                  capituloId: capitulo.id,
                  caminhoCbz: capitulo.caminho_relativo,
                  totalPaginas: Math.max(1, totalPaginas),
                  idiomaOrigem: capitulo.idioma || 'auto',
                  idiomaDestino: idiomaDestinoIa,
                })
                const atualizados = await mamApi.aiListarResultadosCapitulo(capitulo.id)
                setResultadosIa(atualizados)
                setMostrarSobreposicao(true)
                setMostrarIa(true)
              } catch (e) {
                setErro(String(e))
              } finally {
                setProcessandoIa(false)
              }
            }}
          >
            {processandoIa ? 'Processando IA...' : '✨ Processar IA'}
          </button>
          <button className="mam-btn" onClick={() => setMostrarIa((v) => !v)}>
            {mostrarIa ? 'Ocultar IA' : 'Revisão IA'}
          </button>
        </div>
      </div>
      )}

      {barraVisivel && marcadores.length > 0 && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.4rem',
            flexWrap: 'wrap',
            padding: '0.45rem 1rem',
            background: '#0d1320',
            borderBottom: '1px solid #1e293b',
            fontSize: '0.8rem',
          }}
        >
          <span style={{ color: '#94a3b8' }}>Marcadores:</span>
          {marcadores.map((m) => (
            <button key={m.pagina} className="mam-btn" onClick={() => irParaPagina(m.pagina)}>
              {m.nota}
            </button>
          ))}
        </div>
      )}

      {mostrarIa && (
        <div
          style={{
            padding: '0.75rem 1rem',
            background: '#111827',
            borderBottom: '1px solid #1e293b',
          }}
        >
          <strong>Resultados de IA — Página {indiceAtual + 1}</strong>
          {resultadosIa
            .filter((r) => r.pagina_numero === indiceAtual)
            .map((r) => {
              let parsedRegioes: Array<{
                bbox?: [number, number, number, number]
                texto_original?: string
                texto_traduzido?: string
              }> = []
              try {
                const p = JSON.parse(r.resultado_json) as { regioes?: typeof parsedRegioes }
                parsedRegioes = p.regioes ?? []
              } catch {
                parsedRegioes = []
              }

              return (
                <div key={r.id} style={{ marginTop: '0.5rem' }}>
                  <span className="mam-pill-badge">
                    {r.estado_revisao} • {r.modelo_id}
                  </span>
                  {parsedRegioes.length > 0 && (
                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '0.4rem',
                        marginTop: '0.4rem',
                      }}
                    >
                      {parsedRegioes.map((reg, idx) => (
                        <div key={idx} className="mam-panel">
                          <small>
                            Região #{idx + 1} — Original: {reg.texto_original || '(vazio)'}
                          </small>
                          <div className="mam-toolbar-row">
                            <input
                              className="mam-input mam-search-input"
                              defaultValue={reg.texto_traduzido || ''}
                              placeholder="Tradução revisada..."
                              onChange={(e) => {
                                reg.texto_traduzido = e.target.value
                              }}
                            />
                            <button
                              className="mam-btn mam-btn-primary"
                              onClick={async () => {
                                const obj = JSON.parse(r.resultado_json) as Record<string, unknown>
                                obj.regioes = parsedRegioes
                                obj.estado_revisao = 'revisado'
                                await mamApi.aiRevisarResultado(r.id, JSON.stringify(obj))
                                const atualizados = await mamApi.aiListarResultadosCapitulo(
                                  capitulo.id,
                                )
                                setResultadosIa(atualizados)
                              }}
                            >
                              Salvar Região
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                  <pre className="mam-pre" style={{ marginTop: '0.4rem' }}>
                    {r.resultado_json}
                  </pre>
                  <div className="mam-toolbar-row" style={{ marginTop: '0.4rem' }}>
                    <input
                      className="mam-input mam-search-input"
                      placeholder="JSON/texto revisado..."
                      value={textoRevisao}
                      onChange={(e) => setTextoRevisao(e.target.value)}
                    />
                    <button
                      className="mam-btn mam-btn-primary"
                      onClick={async () => {
                        if (!textoRevisao.trim()) return
                        await mamApi.aiRevisarResultado(r.id, textoRevisao.trim())
                        const atualizados = await mamApi.aiListarResultadosCapitulo(capitulo.id)
                        setResultadosIa(atualizados)
                        setTextoRevisao('')
                      }}
                    >
                      Salvar Revisão
                    </button>
                  </div>
                </div>
              )
            })}
          {resultadosIa.filter((r) => r.pagina_numero === indiceAtual).length === 0 && (
            <p className="mam-section-desc">Nenhum resultado de IA salvo para esta página.</p>
          )}
        </div>
      )}

      {erro && (
        <div
          style={{
            margin: '0.75rem 1rem 0',
            padding: '0.75rem',
            background: '#451a1a',
            color: '#fecaca',
            borderRadius: 8,
          }}
        >
          Erro ao carregar página: {erro}
        </div>
      )}

      <div
        ref={viewportRef}
        className="mam-reader-viewport"
        onScroll={(e) => {
          if (modo === 'vertical') {
            const vpEl = e.currentTarget
            const st = vpEl.scrollTop
            if (st > lastScrollTopRef.current + 6) {
              setBarraVisivel(false)
            } else if (st < lastScrollTopRef.current - 6) {
              setBarraVisivel(true)
            }
            lastScrollTopRef.current = st

            if (scrollRafRef.current !== null) return
            scrollRafRef.current = requestAnimationFrame(() => {
              scrollRafRef.current = null
              const vpRect = vpEl.getBoundingClientRect()
              const alvoY = vpRect.top + vpRect.height * 0.35
              let visivel = indiceAtual
              let melhorDist = Number.POSITIVE_INFINITY
              for (let i = 0; i < longStripPageRefs.current.length; i++) {
                const el = longStripPageRefs.current[i]
                if (!el) continue
                const r = el.getBoundingClientRect()
                if (r.top <= alvoY && r.bottom >= alvoY) {
                  visivel = i
                  break
                }
                const dist = Math.abs(r.top - alvoY)
                if (dist < melhorDist) {
                  melhorDist = dist
                  visivel = i
                }
              }
              if (visivel !== indiceAtual) {
                setIndiceAtual(visivel)
              }
            })
          }
        }}
        onWheel={(e) => {
          if (e.ctrlKey) {
            e.preventDefault()
            const delta = e.deltaY < 0 ? 10 : -10
            aplicarZoom((z) => z + delta)
          }
        }}
        style={{
          flexDirection: modo === 'vertical' ? 'column' : 'row',
          alignItems: modo === 'vertical' ? 'center' : 'flex-start',
          justifyContent: 'flex-start',
        }}
      >
        {modo === 'vertical' && totalPaginasLongStrip > 0 ? (
          Array.from({ length: totalPaginasLongStrip }, (_, idx) => {
            const pag = paginasLongStrip[idx]
            return pag ? (
              <div
                key={`${capitulo.id}-${idx}`}
                style={{
                  position: 'relative',
                  display: 'inline-block',
                  width: `${zoom}%`,
                  maxWidth: zoom <= 100 ? '100%' : 'none',
                  flexShrink: 0,
                  lineHeight: 0,
                }}
              >
                <img
                  ref={(el) => {
                    longStripPageRefs.current[idx] = el
                  }}
                  src={`data:${pag.mime_type};base64,${pag.dados_base64}`}
                  alt={pag.nome}
                  decoding="async"
                  style={{
                    ...estiloImagem,
                    width: '100%',
                    maxWidth: '100%',
                    maxHeight: 'none',
                  }}
                />
                {renderizarSobreposicaoPagina(idx)}
              </div>
            ) : (
              <div
                key={`${capitulo.id}-placeholder-${idx}`}
                ref={(el) => {
                  longStripPageRefs.current[idx] = el
                }}
                style={{
                  width: `${zoom}%`,
                  minHeight: '60vh',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: '#64748b',
                  fontSize: '0.85rem',
                  flexShrink: 0,
                }}
              >
                Página {idx + 1}
              </div>
            )
          })
        ) : modo === 'dupla' ? (
          <div
            onClick={handleClickNavegacaoImagem}
            style={{
              display: 'grid',
              gridTemplateColumns: paginaSecundaria ? '1fr 1fr' : '1fr',
              gap: '2px',
              width: `${zoom}%`,
              minWidth: zoom <= 100 ? '100%' : `${zoom}%`,
              margin: 'auto',
              alignItems: 'center',
              justifyItems: 'center',
              cursor: 'pointer',
            }}
            title="Clique na esquerda para voltar ou na direita para avançar"
          >
            {paginaPrincipal && (
              <div
                style={{
                  position: 'relative',
                  width: '100%',
                  display: 'flex',
                  justifyContent: paginaSecundaria ? 'flex-end' : 'center',
                }}
              >
                <div style={{ position: 'relative', display: 'inline-block', lineHeight: 0 }}>
                  <img
                    src={`data:${paginaPrincipal.mime_type};base64,${paginaPrincipal.dados_base64}`}
                    alt={paginaPrincipal.nome}
                    style={{
                      maxWidth: '100%',
                      maxHeight: `${0.82 * zoom}vh`,
                      objectFit: 'contain',
                      display: 'block',
                    }}
                  />
                  {renderizarSobreposicaoPagina(indiceAtual)}
                </div>
              </div>
            )}
            {paginaSecundaria && (
              <div
                style={{
                  position: 'relative',
                  width: '100%',
                  display: 'flex',
                  justifyContent: 'flex-start',
                }}
              >
                <div style={{ position: 'relative', display: 'inline-block', lineHeight: 0 }}>
                  <img
                    src={`data:${paginaSecundaria.mime_type};base64,${paginaSecundaria.dados_base64}`}
                    alt={paginaSecundaria.nome}
                    style={{
                      maxWidth: '100%',
                      maxHeight: `${0.82 * zoom}vh`,
                      objectFit: 'contain',
                      display: 'block',
                    }}
                  />
                  {renderizarSobreposicaoPagina(indiceAtual + 1)}
                </div>
              </div>
            )}
          </div>
        ) : (
          paginaPrincipal && (
            <div
              style={{
                position: 'relative',
                margin: 'auto',
                width: ajuste === 'fit-width' && zoom > 100 ? `${zoom}%` : '100%',
                minWidth: ajuste === 'fit-width' && zoom > 100 ? `${zoom}%` : undefined,
                display: 'flex',
                justifyContent: 'center',
              }}
            >
              <div
                style={{
                  position: 'relative',
                  display: 'inline-block',
                  lineHeight: 0,
                  width:
                    ajuste === 'fit-width'
                      ? zoom > 100
                        ? '100%'
                        : `${zoom}%`
                      : 'auto',
                }}
              >
                <img
                  src={`data:${paginaPrincipal.mime_type};base64,${paginaPrincipal.dados_base64}`}
                  alt={paginaPrincipal.nome}
                  onClick={handleClickNavegacaoImagem}
                  title="Clique na esquerda para voltar ou na direita para avançar"
                  style={{
                    ...estiloImagem,
                    width: ajuste === 'fit-width' ? '100%' : 'auto',
                    cursor: 'pointer',
                  }}
                />
                {renderizarSobreposicaoPagina(indiceAtual)}
              </div>
            </div>
          )
        )}
      </div>

      {totalPaginas > 0 && (
        <div
          onMouseLeave={() => setPaginaHover(null)}
          style={{
            position: 'fixed',
            left: 0,
            right: 0,
            bottom: 0,
            height: 28,
            zIndex: 1050,
            display: 'flex',
            alignItems: 'flex-end',
            userSelect: 'none',
          }}
        >
          {Array.from({ length: totalPaginas }, (_, idx) => {
            const lida = idx <= indiceAtual
            const hovered = paginaHover === idx
            return (
              <div
                key={idx}
                onMouseEnter={() => setPaginaHover(idx)}
                onClick={() => irParaPagina(idx)}
                style={{
                  flex: 1,
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'flex-end',
                  alignItems: 'center',
                  cursor: 'pointer',
                  position: 'relative',
                }}
              >
                {hovered && (
                  <div
                    style={{
                      width: '100%',
                      padding: '2px 0 3px',
                      textAlign: 'center',
                      fontSize: '0.75rem',
                      fontWeight: 600,
                      color: '#f8fafc',
                      background:
                        'linear-gradient(to top, rgba(15, 23, 42, 0.9), rgba(15, 23, 42, 0.15))',
                    }}
                  >
                    {idx + 1}
                  </div>
                )}
                <div
                  style={{
                    width: '100%',
                    height: hovered ? 3 : 2,
                    background: lida
                      ? '#ef4444'
                      : hovered
                        ? 'rgba(248, 250, 252, 0.85)'
                        : 'rgba(148, 163, 184, 0.35)',
                    borderRight: idx < totalPaginas - 1 ? '1px solid rgba(15, 23, 42, 0.7)' : 'none',
                  }}
                />
              </div>
            )
          })}
        </div>
      )}

      {/* Modal flutuante de Layout (inspirado na referência) */}
      {modalLayoutAberto && (
        <div className="mam-modal-backdrop" onClick={() => setModalLayoutAberto(false)}>
          <div className="mam-modal" onClick={(e) => e.stopPropagation()}>
            <div className="mam-modal-header">
              <strong>Layout & Visualização</strong>
              <button className="mam-btn" onClick={() => setModalLayoutAberto(false)}>
                ✕
              </button>
            </div>

            <div className="mam-layout-cards">
              <button
                className={`mam-layout-card ${modo === 'horizontal' ? 'active' : ''}`}
                onClick={() => {
                  setModo('horizontal')
                  setZoom(zoomPadraoPorModo.horizontal)
                }}
              >
                <span>▯</span>
                <span>Single</span>
              </button>
              <button
                className={`mam-layout-card ${modo === 'dupla' ? 'active' : ''}`}
                onClick={() => {
                  setModo('dupla')
                  setZoom(zoomPadraoPorModo.dupla)
                }}
              >
                <span>▯▯</span>
                <span>Double</span>
              </button>
              <button
                className={`mam-layout-card ${modo === 'vertical' ? 'active' : ''}`}
                onClick={() => {
                  setModo('vertical')
                  setZoom(zoomPadraoPorModo.vertical)
                }}
              >
                <span>☰</span>
                <span>Long strip</span>
              </button>
            </div>

            <div>
              <small style={{ color: '#94a3b8' }}>ZOOM</small>
              <div className="mam-stepper" style={{ marginTop: '0.35rem' }}>
                <button className="mam-btn" onClick={() => aplicarZoom((z) => z - 10)}>
                  −
                </button>
                <span>{zoom}%</span>
                <button className="mam-btn" onClick={() => aplicarZoom((z) => z + 10)}>
                  +
                </button>
                <button
                  className="mam-btn"
                  onClick={() => aplicarZoom(zoomPadraoPorModo[modo])}
                >
                  Reset
                </button>
              </div>
            </div>

            <div>
              <small style={{ color: '#94a3b8' }}>ZOOM PADRÃO</small>
              <div className="mam-toolbar-row" style={{ marginTop: '0.35rem' }}>
                <button
                  className={`mam-btn ${usarZoomPadrao ? 'mam-btn-active' : ''}`}
                  onClick={() => setUsarZoomPadrao((v) => !v)}
                >
                  {usarZoomPadrao
                    ? `✓ Zoom Padrão (${zoomPadraoPorModo[modo]}%)`
                    : 'Zoom Padrão: OFF'}
                </button>
                <button
                  className="mam-btn"
                  onClick={() => {
                    setZoomPadraoPorModo((prev) => ({ ...prev, [modo]: zoom }))
                    setUsarZoomPadrao(true)
                  }}
                >
                  Usar {zoom}% como padrão
                </button>
              </div>
            </div>

            <div>
              <small style={{ color: '#94a3b8' }}>AJUSTE À TELA</small>
              <select
                className="mam-select"
                style={{ width: '100%', marginTop: '0.35rem' }}
                value={ajuste}
                onChange={(e) => setAjuste(e.target.value as AjusteTela)}
              >
                <option value="fit-width">Ajustar à Largura</option>
                <option value="fit-height">Ajustar à Altura</option>
                <option value="original">Tamanho Original</option>
              </select>
            </div>
          </div>
        </div>
      )}

      {/* Modal flutuante de Capítulos / Volumes (inspirado na referência) */}
      {modalCapitulosAberto && (
        <div className="mam-modal-backdrop" onClick={() => setModalCapitulosAberto(false)}>
          <div className="mam-modal" onClick={(e) => e.stopPropagation()}>
            <div className="mam-modal-header">
              <div className="mam-segmented">
                <button
                  className={`mam-segmented-btn ${abaModalCap === 'capitulos' ? 'active' : ''}`}
                  onClick={() => setAbaModalCap('capitulos')}
                >
                  Capítulos
                </button>
                <button
                  className={`mam-segmented-btn ${abaModalCap === 'volumes' ? 'active' : ''}`}
                  onClick={() => setAbaModalCap('volumes')}
                >
                  Volumes
                </button>
              </div>
              <button className="mam-btn" onClick={() => setModalCapitulosAberto(false)}>
                ✕
              </button>
            </div>

            {abaModalCap === 'capitulos' ? (
              <>
                <div className="mam-toolbar-row">
                  <input
                    className="mam-input mam-search-input"
                    placeholder="🔍 Buscar capítulo..."
                    value={buscaModalCap}
                    onChange={(e) => setBuscaModalCap(e.target.value)}
                  />
                  <button
                    className="mam-btn mam-btn-primary"
                    onClick={() => setOrdemModalCap((o) => (o === 'desc' ? 'asc' : 'desc'))}
                  >
                    Cap {ordemModalCap === 'desc' ? '↓' : '↑'}
                  </button>
                </div>

                <div style={{ maxHeight: 320, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: '0.35rem' }}>
                  {capsModalFiltrados.map((c) => (
                    <div
                      key={c.id}
                      className={`mam-row-card ${c.id === capitulo.id ? 'mam-row-card-active' : ''}`}
                      onClick={() => {
                        onSelecionarCapitulo(c)
                        setModalCapitulosAberto(false)
                      }}
                    >
                      <span>Cap. {c.numero}</span>
                      <span className="mam-brand-sub">{c.quantidade_paginas} págs</span>
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <div style={{ maxHeight: 320, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                {volumesDisponiveis.map((vc) => (
                  <div key={vc.volume.id} className="mam-panel">
                    <strong>Volume {vc.volume.numero}</strong>
                    {vc.capitulos.map((c) => (
                      <div
                        key={c.id}
                        className="mam-row-card"
                        onClick={() => {
                          onSelecionarCapitulo(c)
                          setModalCapitulosAberto(false)
                        }}
                      >
                        <span>Cap. {c.numero}</span>
                      </div>
                    ))}
                  </div>
                ))}
                {volumesDisponiveis.length === 0 && (
                  <p className="mam-section-desc">Nenhum volume cadastrado para esta obra.</p>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
