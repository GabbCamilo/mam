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
  onSelecionarCapitulo: (capitulo: Capitulo) => void
  onFechar: () => void
}

export function CbzReader({
  capitulo,
  capitulosDisponiveis,
  volumesDisponiveis = [],
  onSelecionarCapitulo,
  onFechar,
}: CbzReaderProps) {
  const [indiceAtual, setIndiceAtual] = useState(0)
  const [paginaPrincipal, setPaginaPrincipal] = useState<PaginaExtraida | null>(null)
  const [paginaSecundaria, setPaginaSecundaria] = useState<PaginaExtraida | null>(null)
  const [paginasLongStrip, setPaginasLongStrip] = useState<PaginaExtraida[]>([])
  const [modo, setModo] = useState<ModoLeitura>('horizontal')
  const [ajuste, setAjuste] = useState<AjusteTela>('fit-width')
  const [zoom, setZoom] = useState(30)
  const [telaCheia, setTelaCheia] = useState(false)
  const [paginaHover, setPaginaHover] = useState<number | null>(null)
  const viewportRef = useRef<HTMLDivElement | null>(null)
  const longStripPageRefs = useRef<(HTMLImageElement | null)[]>([])
  const pendingScrollRatioRef = useRef<{ x: number; y: number } | null>(null)
  const [marcadores, setMarcadores] = useState<MarcadorPagina[]>([])
  const [erro, setErro] = useState<string | null>(null)
  const [carregando, setCarregando] = useState(false)

  const [mostrarIa, setMostrarIa] = useState(false)
  const [mostrarSobreposicao, setMostrarSobreposicao] = useState(true)
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
        setIndiceAtual(progresso)
        setMarcadores(listaMarcadores)
        setResultadosIa(listaIa)
      } catch {
        if (!ativo) return
        setIndiceAtual(0)
      }
    }
    inicializar()
    return () => {
      ativo = false
    }
  }, [capitulo.id])

  useEffect(() => {
    let ativo = true
    async function carregarPaginas() {
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
          const todas: PaginaExtraida[] = []
          for (let i = 0; i < p1.total_paginas; i++) {
            if (!ativo) return
            const p = i === indiceAtual ? p1 : await mamApi.obterPagina(capitulo.caminho_relativo, i)
            todas.push(p)
          }
          if (ativo) {
            setPaginasLongStrip(todas)
            setPaginaSecundaria(null)
            await mamApi.marcarLido(capitulo.id, true)
          }
        } else if (modo === 'dupla' && indiceAtual + 1 < p1.total_paginas) {
          setPaginasLongStrip([])
          const p2 = await mamApi.obterPagina(capitulo.caminho_relativo, indiceAtual + 1)
          if (ativo) setPaginaSecundaria(p2)
        } else {
          setPaginasLongStrip([])
          setPaginaSecundaria(null)
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

  async function handleAdicionarMarcador() {
    await mamApi.adicionarMarcador(capitulo.id, indiceAtual, `Página ${indiceAtual + 1}`)
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
          <button className="mam-btn" onClick={handleAdicionarMarcador}>
            🔖 Marcar
          </button>
          <button
            className={`mam-btn ${mostrarSobreposicao ? 'mam-btn-active' : ''}`}
            onClick={() => setMostrarSobreposicao((v) => !v)}
          >
            Sobreposição: {mostrarSobreposicao ? 'ON' : 'OFF'}
          </button>
          <button
            className="mam-btn mam-btn-primary"
            disabled={processandoIa}
            onClick={async () => {
              setProcessandoIa(true)
              setErro(null)
              try {
                await mamApi.aiProcessarESalvarPagina({
                  capituloId: capitulo.id,
                  caminhoCbz: capitulo.caminho_relativo,
                  indice: indiceAtual,
                  idiomaOrigem: 'ja',
                  idiomaDestino: 'pt-BR',
                })
                const atualizados = await mamApi.aiListarResultadosCapitulo(capitulo.id)
                setResultadosIa(atualizados)
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

      {marcadores.length > 0 && (
        <div
          style={{
            display: 'flex',
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
            <button key={m.pagina} className="mam-btn" onClick={() => setIndiceAtual(m.pagina)}>
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
        {carregando && <p style={{ color: '#94a3b8', margin: 'auto' }}>Carregando página...</p>}

        {modo === 'vertical' && paginasLongStrip.length > 0 ? (
          paginasLongStrip.map((pag, idx) => (
            <img
              key={pag.nome + idx}
              ref={(el) => {
                longStripPageRefs.current[idx] = el
              }}
              src={`data:${pag.mime_type};base64,${pag.dados_base64}`}
              alt={pag.nome}
              style={{
                ...estiloImagem,
                width: `${zoom}%`,
                maxWidth: zoom <= 100 ? '100%' : 'none',
                maxHeight: 'none',
                flexShrink: 0,
              }}
            />
          ))
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
                <img
                  src={`data:${paginaPrincipal.mime_type};base64,${paginaPrincipal.dados_base64}`}
                  alt={paginaPrincipal.nome}
                  style={{
                    width: '100%',
                    maxHeight: `${0.82 * zoom}vh`,
                    objectFit: 'contain',
                    objectPosition: paginaSecundaria ? 'right center' : 'center',
                  }}
                />
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
                <img
                  src={`data:${paginaSecundaria.mime_type};base64,${paginaSecundaria.dados_base64}`}
                  alt={paginaSecundaria.nome}
                  style={{
                    width: '100%',
                    maxHeight: `${0.82 * zoom}vh`,
                    objectFit: 'contain',
                    objectPosition: 'left center',
                  }}
                />
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
              <img
                src={`data:${paginaPrincipal.mime_type};base64,${paginaPrincipal.dados_base64}`}
                alt={paginaPrincipal.nome}
                onClick={handleClickNavegacaoImagem}
                title="Clique na esquerda para voltar ou na direita para avançar"
                style={{
                  ...estiloImagem,
                  width:
                    ajuste === 'fit-width'
                      ? zoom > 100
                        ? '100%'
                        : `${zoom}%`
                      : 'auto',
                  cursor: 'pointer',
                }}
              />
              {mostrarSobreposicao &&
                resultadosIa
                  .filter((r) => r.pagina_numero === indiceAtual)
                  .flatMap((r) => {
                    try {
                      const parsed = JSON.parse(r.resultado_json) as {
                        regioes?: Array<{
                          bbox?: [number, number, number, number]
                          texto_traduzido?: string
                          texto_original?: string
                        }>
                      }
                      return parsed.regioes ?? []
                    } catch {
                      return []
                    }
                  })
                  .map((reg, idx) => {
                    const [x, y, w, h] = reg.bbox ?? [0, 0, 0.2, 0.1]
                    return (
                      <div
                        key={idx}
                        style={{
                          position: 'absolute',
                          left: `${x * 100}%`,
                          top: `${y * 100}%`,
                          width: `${w * 100}%`,
                          height: `${h * 100}%`,
                          background: 'rgba(255, 255, 255, 0.9)',
                          color: '#0f172a',
                          padding: '2px 4px',
                          fontSize: '0.8rem',
                          borderRadius: 4,
                          boxSizing: 'border-box',
                          overflow: 'hidden',
                        }}
                      >
                        {reg.texto_traduzido || reg.texto_original || ''}
                      </div>
                    )
                  })}
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
                  setZoom(30)
                }}
              >
                <span>▯</span>
                <span>Single</span>
              </button>
              <button
                className={`mam-layout-card ${modo === 'dupla' ? 'active' : ''}`}
                onClick={() => {
                  setModo('dupla')
                  setZoom(110)
                }}
              >
                <span>▯▯</span>
                <span>Double</span>
              </button>
              <button
                className={`mam-layout-card ${modo === 'vertical' ? 'active' : ''}`}
                onClick={() => {
                  setModo('vertical')
                  setZoom(100)
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
                  onClick={() =>
                    aplicarZoom(modo === 'horizontal' ? 30 : modo === 'dupla' ? 110 : 100)
                  }
                >
                  Reset
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
