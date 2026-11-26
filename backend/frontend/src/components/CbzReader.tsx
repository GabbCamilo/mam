import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { useI18n } from '../i18n'
import {
  Capitulo,
  MarcadorPagina,
  PaginaExtraida,
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
  /** Lacuna de extensão para controles futuros de IA externa na barra do leitor. */
  extraToolbarActions?: ReactNode
  /** Lacuna de extensão para camada de sobreposição externa alinhada à imagem da página. */
  renderOverlayPagina?: (paginaIndice: number) => ReactNode
}

export function CbzReader({
  capitulo,
  capitulosDisponiveis,
  volumesDisponiveis = [],
  paginaInicial = null,
  onSelecionarCapitulo,
  onFechar,
  extraToolbarActions,
  renderOverlayPagina,
}: CbzReaderProps) {
  const { tx } = useI18n()
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
  const [_carregando, setCarregando] = useState(false)

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
        const [progresso, listaMarcadores] = await Promise.all([
          mamApi.obterProgresso(capitulo.id),
          mamApi.listarMarcadores(capitulo.id),
        ])
        if (!ativo) return
        setIndiceAtual(paginaInicial !== null && paginaInicial !== undefined ? paginaInicial : progresso)
        setMarcadores(listaMarcadores)
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
        const inicio = Math.max(0, indiceAtual - 1)
        const fim = Math.min(totalPaginasLongStrip - 1, indiceAtual + 2)
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
              if (Math.abs(idx - indiceAtual) <= 5) {
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
          const inicio = Math.max(0, indiceAtual - 1)
          const fim = Math.min(p1.total_paginas - 1, indiceAtual + 2)
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

  return (
    <div className="mam-reader-shell">
      {barraVisivel && (
      <div className="mam-reader-toolbar">
        <div className="mam-toolbar-row">
          <button className="mam-btn" onClick={onFechar}>
            ← {tx('Biblioteca', 'Library')}
          </button>
          <button className="mam-btn" onClick={() => setModalCapitulosAberto(true)}>
            {tx('Cap.', 'Ch.')} {capitulo.numero} ({indiceAtual + 1}/{Math.max(totalPaginas, 1)}) ▾
          </button>
          <button className="mam-btn" onClick={voltarPagina}>
            ◀ {tx('Anterior', 'Previous')}
          </button>
          <button className="mam-btn mam-btn-primary" onClick={avancarPagina}>
            {tx('Próxima', 'Next')} ▶
          </button>
        </div>

        <div className="mam-toolbar-row">
          <button className="mam-btn" onClick={() => setModalLayoutAberto(true)}>
            ⚙ {tx('Layout', 'Layout')}
          </button>
          <button
            className={`mam-btn ${telaCheia ? 'mam-btn-active' : ''}`}
            onClick={alternarTelaCheia}
            title={tx('Tela Cheia (F11)', 'Fullscreen (F11)')}
          >
            {telaCheia ? `⤢ ${tx('Sair Tela Cheia', 'Exit Fullscreen')}` : `⛶ ${tx('Tela Cheia', 'Fullscreen')}`}
          </button>
          <button
            className={`mam-btn ${paginaAtualMarcada ? 'mam-btn-active' : ''}`}
            onClick={handleAlternarMarcador}
          >
            {paginaAtualMarcada ? `🔖 ${tx('Desmarcar', 'Unbookmark')}` : `🔖 ${tx('Marcar', 'Bookmark')}`}
          </button>
          {extraToolbarActions}
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
          <span style={{ color: '#94a3b8' }}>{tx('Marcadores:', 'Bookmarks:')}</span>
          {marcadores.map((m) => (
            <button key={m.pagina} className="mam-btn" onClick={() => irParaPagina(m.pagina)}>
              {m.nota}
            </button>
          ))}
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
          {tx('Erro ao carregar página:', 'Error loading page:')} {erro}
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
                {renderOverlayPagina?.(idx)}
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
                {tx('Página', 'Page')} {idx + 1}
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
            title={tx('Clique na esquerda para voltar ou na direita para avançar', 'Click left to go back or right to advance')}
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
                    decoding="async"
                    style={{
                      maxWidth: '100%',
                      maxHeight: `${0.82 * zoom}vh`,
                      objectFit: 'contain',
                      display: 'block',
                    }}
                  />
                  {renderOverlayPagina?.(indiceAtual)}
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
                    decoding="async"
                    style={{
                      maxWidth: '100%',
                      maxHeight: `${0.82 * zoom}vh`,
                      objectFit: 'contain',
                      display: 'block',
                    }}
                  />
                  {renderOverlayPagina?.(indiceAtual + 1)}
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
                  decoding="async"
                  onClick={handleClickNavegacaoImagem}
                  title={tx('Clique na esquerda para voltar ou na direita para avançar', 'Click left to go back or right to advance')}
                  style={{
                    ...estiloImagem,
                    width: ajuste === 'fit-width' ? '100%' : 'auto',
                    cursor: 'pointer',
                  }}
                />
                {renderOverlayPagina?.(indiceAtual)}
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
              <strong>{tx('Layout & Visualização', 'Layout & View')}</strong>
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
              <small style={{ color: '#94a3b8' }}>{tx('ZOOM PADRÃO', 'DEFAULT ZOOM')}</small>
              <div className="mam-toolbar-row" style={{ marginTop: '0.35rem' }}>
                <button
                  className={`mam-btn ${usarZoomPadrao ? 'mam-btn-active' : ''}`}
                  onClick={() => setUsarZoomPadrao((v) => !v)}
                >
                  {usarZoomPadrao
                    ? `✓ ${tx('Zoom Padrão', 'Default Zoom')} (${zoomPadraoPorModo[modo]}%)`
                    : tx('Zoom Padrão: OFF', 'Default Zoom: OFF')}
                </button>
                <button
                  className="mam-btn"
                  onClick={() => {
                    setZoomPadraoPorModo((prev) => ({ ...prev, [modo]: zoom }))
                    setUsarZoomPadrao(true)
                  }}
                >
                  {tx('Usar {zoom}% como padrão', 'Use {zoom}% as default', { zoom })}
                </button>
              </div>
            </div>

            <div>
              <small style={{ color: '#94a3b8' }}>{tx('AJUSTE À TELA', 'FIT TO SCREEN')}</small>
              <select
                className="mam-select"
                style={{ width: '100%', marginTop: '0.35rem' }}
                value={ajuste}
                onChange={(e) => setAjuste(e.target.value as AjusteTela)}
              >
                <option value="fit-width">{tx('Ajustar à Largura', 'Fit Width')}</option>
                <option value="fit-height">{tx('Ajustar à Altura', 'Fit Height')}</option>
                <option value="original">{tx('Tamanho Original', 'Original Size')}</option>
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
                  {tx('Capítulos', 'Chapters')}
                </button>
                <button
                  className={`mam-segmented-btn ${abaModalCap === 'volumes' ? 'active' : ''}`}
                  onClick={() => setAbaModalCap('volumes')}
                >
                  {tx('Volumes', 'Volumes')}
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
                    placeholder={tx('🔍 Buscar capítulo...', '🔍 Search chapter...')}
                    value={buscaModalCap}
                    onChange={(e) => setBuscaModalCap(e.target.value)}
                  />
                  <button
                    className="mam-btn mam-btn-primary"
                    onClick={() => setOrdemModalCap((o) => (o === 'desc' ? 'asc' : 'desc'))}
                  >
                    {tx('Cap', 'Ch')} {ordemModalCap === 'desc' ? '↓' : '↑'}
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
                      <span>{tx('Cap.', 'Ch.')} {c.numero}</span>
                      <span className="mam-brand-sub">{c.quantidade_paginas} {tx('págs', 'pgs')}</span>
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
                        <span>{tx('Cap.', 'Ch.')} {c.numero}</span>
                      </div>
                    ))}
                  </div>
                ))}
                {volumesDisponiveis.length === 0 && (
                  <p className="mam-section-desc">{tx('Nenhum volume cadastrado para esta obra.', 'No volumes registered for this series.')}</p>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
