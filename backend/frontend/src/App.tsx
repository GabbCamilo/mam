import { useEffect, useState } from 'react'
import {
  Capitulo,
  CapituloBuscaResultado,
  DetalhesObra,
  DiferencaBibliotecas,
  ExtratorInfo,
  FonteInfo,
  ModeloInfo,
  Obra,
  ObraBuscaResultado,
  PaginaExtraida,
  RelatorioTransferencia,
  ResultadoImportacao,
  StatusOperacao,
  mamApi,
} from './api'
import { CbzReader } from './components/CbzReader'

type AbaAtiva = 'biblioteca' | 'fontes' | 'transferencia' | 'ai'
type SubAbaObra = 'capitulos' | 'volumes'

function App() {
  const [tema, setTema] = useState<'dark' | 'light'>('dark')
  const [abaAtiva, setAbaAtiva] = useState<AbaAtiva>('biblioteca')
  const [subAbaObra, setSubAbaObra] = useState<SubAbaObra>('capitulos')
  const [raizBiblioteca, setRaizBiblioteca] = useState<string>('')
  const [obras, setObras] = useState<Obra[]>([])
  const [buscaObra, setBuscaObra] = useState('')
  const [buscaCapitulo, setBuscaCapitulo] = useState('')
  const [ordemCaps, setOrdemCaps] = useState<'desc' | 'asc'>('desc')
  const [detalhes, setDetalhes] = useState<DetalhesObra | null>(null)
  const [capaBiblioteca, setCapaBiblioteca] = useState<PaginaExtraida | null>(null)
  const [capitulosLidos, setCapitulosLidos] = useState<string[]>([])
  const [capituloAtivo, setCapituloAtivo] = useState<Capitulo | null>(null)
  const [tituloNovaObra, setTituloNovaObra] = useState('')
  const [caminhoNovaObra, setCaminhoNovaObra] = useState('')
  const [mensagem, setMensagem] = useState<string | null>(null)

  const [origemTransfer, setOrigemTransfer] = useState('')
  const [destinoTransfer, setDestinoTransfer] = useState('')
  const [diferenca, setDiferenca] = useState<DiferencaBibliotecas | null>(null)
  const [relatorioTransfer, setRelatorioTransfer] = useState<RelatorioTransferencia | null>(null)

  const [aiStatus, setAiStatus] = useState<Record<string, unknown> | null>(null)
  const [diretorioModelos, setDiretorioModelos] = useState('')
  const [modelos, setModelos] = useState<ModeloInfo[]>([])

  const [fontes, setFontes] = useState<FonteInfo[]>([])
  const [consultaFonte, setConsultaFonte] = useState('')
  const [obrasFonte, setObrasFonte] = useState<ObraBuscaResultado[]>([])
  const [obraFonteSelecionada, setObraFonteSelecionada] = useState<ObraBuscaResultado | null>(null)
  const [capitulosFonte, setCapitulosFonte] = useState<CapituloBuscaResultado[]>([])
  const [selecionadosIds, setSelecionadosIds] = useState<string[]>([])
  const [statusArquivamento, setStatusArquivamento] = useState<StatusOperacao | null>(null)

  const [modalImportarUrlAberto, setModalImportarUrlAberto] = useState(false)
  const [extratores, setExtratores] = useState<ExtratorInfo[]>([])
  const [extObraTitulo, setExtObraTitulo] = useState('')
  const [extNumeroCap, setExtNumeroCap] = useState('1')
  const [extUrl, setExtUrl] = useState('')
  const [extHtml, setExtHtml] = useState('')
  const [extUrlsTexto, setExtUrlsTexto] = useState('')
  const [resultadoImportacao, setResultadoImportacao] = useState<ResultadoImportacao | null>(null)

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', tema)
  }, [tema])

  async function carregarObras() {
    try {
      const [lista, raiz, lidos] = await Promise.all([
        mamApi.listarObras(),
        mamApi.libraryRaiz(),
        mamApi.listarLidos(),
      ])
      setObras(lista)
      setRaizBiblioteca(raiz)
      setCapitulosLidos(lidos)
    } catch (e) {
      setMensagem(`Erro ao listar obras: ${String(e)}`)
    }
  }

  useEffect(() => {
    carregarObras()
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

  async function handleImportarExtrator(e: React.FormEvent) {
    e.preventDefault()
    if (!extObraTitulo.trim()) return
    const urls_ordenadas = extUrlsTexto
      .split(/\r?\n/)
      .map((u) => u.trim())
      .filter((u) => u.length > 0)

    try {
      const res = await mamApi.sourceImportarExtrator({
        obra_titulo: extObraTitulo.trim(),
        numero_capitulo: Number.parseInt(extNumeroCap, 10) || 1,
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
    if (!consultaFonte.trim()) return
    try {
      const res = await mamApi.sourceBuscarObras(consultaFonte.trim())
      setObrasFonte(res)
      setObraFonteSelecionada(null)
      setCapitulosFonte([])
      setSelecionadosIds([])
      setMensagem(`${res.length} obra(s) encontrada(s).`)
    } catch (err) {
      setMensagem(`Erro na busca de fontes: ${String(err)}`)
    }
  }

  async function handleSelecionarObraFonte(obra: ObraBuscaResultado) {
    if (obraFonteSelecionada?.obra_id_fonte === obra.obra_id_fonte) {
      setObraFonteSelecionada(null)
      setCapitulosFonte([])
      setSelecionadosIds([])
      return
    }
    setObraFonteSelecionada(obra)
    setSelecionadosIds([])
    try {
      const caps = await mamApi.sourceListarCapitulos(obra.fonte_id, obra.obra_id_fonte)
      setCapitulosFonte(caps)
    } catch (err) {
      setMensagem(`Erro ao listar capítulos da fonte: ${String(err)}`)
    }
  }

  function alternarCapituloSelecionado(id: string) {
    setSelecionadosIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id],
    )
  }

  function alternarSelecionarTodosCapitulos() {
    if (selecionadosIds.length === capitulosFonte.length) {
      setSelecionadosIds([])
    } else {
      setSelecionadosIds(capitulosFonte.map((c) => c.capitulo_id_fonte))
    }
  }

  async function handleArquivarSelecionados() {
    if (!obraFonteSelecionada || selecionadosIds.length === 0) return
    const escolhidos = capitulosFonte.filter((c) => selecionadosIds.includes(c.capitulo_id_fonte))
    try {
      const opId = await mamApi.sourceArquivarCapitulos(obraFonteSelecionada.titulo, escolhidos)
      const status = await mamApi.sourceStatusOperacao(opId)
      setStatusArquivamento(status)
      setMensagem(`Arquivamento concluído (operação ${opId}).`)
      await carregarObras()
    } catch (err) {
      setMensagem(`Erro ao arquivar capítulos: ${String(err)}`)
    }
  }

  const todosCapitulosObra: Capitulo[] = detalhes
    ? [...detalhes.volumes.flatMap((v) => v.capitulos), ...detalhes.capitulos_avulsos].sort(
        (a, b) => a.numero - b.numero,
      )
    : []

  const capitulosFiltrados = [...todosCapitulosObra]
    .filter((c) =>
      buscaCapitulo.trim()
        ? String(c.numero).includes(buscaCapitulo.trim()) ||
          (c.titulo_original ?? '').toLowerCase().includes(buscaCapitulo.trim().toLowerCase())
        : true,
    )
    .sort((a, b) => (ordemCaps === 'desc' ? b.numero - a.numero : a.numero - b.numero))

  const obrasFiltradas = obras.filter((o) =>
    buscaObra.trim() ? o.titulo.toLowerCase().includes(buscaObra.trim().toLowerCase()) : true,
  )

  if (capituloAtivo) {
    return (
      <CbzReader
        capitulo={capituloAtivo}
        capitulosDisponiveis={todosCapitulosObra}
        volumesDisponiveis={detalhes?.volumes ?? []}
        onSelecionarCapitulo={setCapituloAtivo}
        onFechar={async () => {
          setCapituloAtivo(null)
          await carregarObras()
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
          <span className="mam-pill-badge">{obras.length} obra(s)</span>
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
          onClick={() => setAbaAtiva('biblioteca')}
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
        {mensagem && (
          <div className="mam-alert">
            <span>{mensagem}</span>
            <button className="mam-btn" onClick={() => setMensagem(null)}>
              ✕
            </button>
          </div>
        )}

        {abaAtiva === 'biblioteca' && (
          <>
            <div className="mam-section-header">
              <div>
                <h1 className="mam-section-title">BIBLIOTECA</h1>
                <p className="mam-section-desc">
                  Gerencie obras, volumes híbridos e capítulos CBZ armazenados localmente.
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
                {obrasFiltradas.length === 0 ? (
                  <p className="mam-section-desc">Nenhuma obra encontrada.</p>
                ) : (
                  obrasFiltradas.map((obra) => (
                    <div
                      key={obra.id}
                      onClick={() => handleSelecionarObra(obra.id)}
                      className={`mam-row-card ${detalhes?.obra.id === obra.id ? 'mam-row-card-active' : ''}`}
                    >
                      <div>
                        <strong>{obra.titulo}</strong>
                        <div className="mam-brand-sub">{obra.caminho_relativo}</div>
                      </div>
                      <span className="mam-pill-badge">{obra.estado_integridade}</span>
                    </div>
                  ))
                )}
              </section>

              <section className="mam-panel">
                {detalhes ? (
                  <>
                    <div className="mam-section-header">
                      <div style={{ display: 'flex', gap: '0.85rem', alignItems: 'center' }}>
                        {capaBiblioteca && (
                          <img
                            src={`data:${capaBiblioteca.mime_type};base64,${capaBiblioteca.dados_base64}`}
                            alt={detalhes.obra.titulo}
                            style={{
                              width: 56,
                              height: 78,
                              objectFit: 'cover',
                              borderRadius: 8,
                              border: '1px solid var(--border-subtle)',
                            }}
                          />
                        )}
                        <div>
                          <h2 className="mam-section-title">{detalhes.obra.titulo}</h2>
                          <p className="mam-section-desc">{detalhes.obra.caminho_relativo}</p>
                        </div>
                      </div>
                      <div className="mam-segmented">
                        <button
                          className={`mam-segmented-btn ${subAbaObra === 'capitulos' ? 'active' : ''}`}
                          onClick={() => setSubAbaObra('capitulos')}
                        >
                          Capítulos ({todosCapitulosObra.length})
                        </button>
                        <button
                          className={`mam-segmented-btn ${subAbaObra === 'volumes' ? 'active' : ''}`}
                          onClick={() => setSubAbaObra('volumes')}
                        >
                          Volumes ({detalhes.volumes.length})
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
                          capitulosFiltrados.map((cap) => {
                            const lido = capitulosLidos.includes(cap.id)
                            return (
                              <div
                                key={cap.id}
                                className="mam-row-card"
                                onClick={() => setCapituloAtivo(cap)}
                              >
                                <div>
                                  <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', flexWrap: 'wrap' }}>
                                    <strong>Cap. {cap.numero}</strong>
                                    {cap.titulo_original && (
                                      <span style={{ color: 'var(--text-secondary)' }}>
                                        • {cap.titulo_original}
                                      </span>
                                    )}
                                    <button
                                      className={`mam-pill-badge ${lido ? '' : ''}`}
                                      style={{
                                        cursor: 'pointer',
                                        background: lido ? 'rgba(34, 197, 94, 0.16)' : 'rgba(148, 163, 184, 0.12)',
                                        color: lido ? 'var(--success)' : 'var(--text-secondary)',
                                        borderColor: lido ? 'rgba(34, 197, 94, 0.35)' : 'var(--border-subtle)',
                                      }}
                                      onClick={(e) => handleAlternarLido(e, cap.id)}
                                    >
                                      {lido ? '✓ Lido' : '○ Não lido'}
                                    </button>
                                  </div>
                                  <div className="mam-brand-sub">{cap.caminho_relativo}</div>
                                </div>
                                <div className="mam-toolbar-row">
                                  <span className="mam-pill-badge">{cap.quantidade_paginas} págs</span>
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
                          })
                        )}
                      </>
                    ) : (
                      <>
                        {detalhes.volumes.map((vc) => (
                          <div key={vc.volume.id} className="mam-panel">
                            <strong>
                              Volume {vc.volume.numero}{' '}
                              {vc.volume.titulo ? `— ${vc.volume.titulo}` : ''}
                            </strong>
                            {vc.capitulos.map((cap) => (
                              <div
                                key={cap.id}
                                className="mam-row-card"
                                onClick={() => setCapituloAtivo(cap)}
                              >
                                <span>
                                  Cap. {cap.numero} ({cap.quantidade_paginas} págs)
                                </span>
                                <div className="mam-toolbar-row">
                                  <button
                                    className="mam-btn"
                                    onClick={(e) => handleExcluirCapitulo(e, cap.id)}
                                  >
                                    🗑
                                  </button>
                                  <button className="mam-btn mam-btn-primary">Ler</button>
                                </div>
                              </div>
                            ))}
                          </div>
                        ))}
                        {detalhes.volumes.length === 0 && (
                          <p className="mam-section-desc">
                            Nenhum volume agrupado; capítulos listados na aba Capítulos.
                          </p>
                        )}
                      </>
                    )}
                  </>
                ) : (
                  <p className="mam-section-desc">
                    Selecione uma obra à esquerda para visualizar capítulos e volumes.
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
              <button
                className="mam-btn mam-btn-primary"
                onClick={() => setModalImportarUrlAberto(true)}
              >
                + Importar por Link / HTML
              </button>
            </div>

            <form onSubmit={handleBuscarNaFonte} className="mam-panel">
              <div className="mam-toolbar-row">
                <input
                  className="mam-input mam-search-input"
                  placeholder="🔍 Pesquisar obra por título no MangaDex..."
                  value={consultaFonte}
                  onChange={(e) => setConsultaFonte(e.target.value)}
                />
                <button className="mam-btn mam-btn-primary" type="submit">
                  Pesquisar
                </button>
              </div>
            </form>

            <div className="mam-grid-2">
              <div className="mam-panel">
                <strong>Resultados ({obrasFonte.length})</strong>
                {obrasFonte.map((o) => (
                  <div
                    key={`${o.fonte_id}-${o.obra_id_fonte}`}
                    className={`mam-row-card ${obraFonteSelecionada?.obra_id_fonte === o.obra_id_fonte ? 'mam-row-card-active' : ''}`}
                    onClick={() => handleSelecionarObraFonte(o)}
                  >
                    <div style={{ display: 'flex', gap: '0.65rem', alignItems: 'center' }}>
                      {o.capa_url && (
                        <img
                          src={o.capa_url}
                          alt={o.titulo}
                          style={{ width: 38, height: 54, objectFit: 'cover', borderRadius: 6 }}
                        />
                      )}
                      <span>{o.titulo}</span>
                    </div>
                  </div>
                ))}
              </div>

              <div className="mam-panel">
                {obraFonteSelecionada ? (
                  <>
                    <div className="mam-section-header">
                      <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                        {obraFonteSelecionada.capa_url && (
                          <img
                            src={obraFonteSelecionada.capa_url}
                            alt={obraFonteSelecionada.titulo}
                            style={{ width: 48, height: 68, objectFit: 'cover', borderRadius: 8 }}
                          />
                        )}
                        <strong>{obraFonteSelecionada.titulo}</strong>
                      </div>
                      <div className="mam-toolbar-row">
                        <button className="mam-btn" onClick={alternarSelecionarTodosCapitulos}>
                          {selecionadosIds.length === capitulosFonte.length && capitulosFonte.length > 0
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
                    {statusArquivamento && (
                      <div className="mam-alert">
                        Progresso: {statusArquivamento.capitulos_processados}/
                        {statusArquivamento.total_capitulos} ({statusArquivamento.percentual}%)
                      </div>
                    )}
                    {capitulosFonte.map((c) => (
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
                    ))}
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

        {abaAtiva === 'transferencia' && (
          <section className="mam-panel">
            <h1 className="mam-section-title">TRANSFERÊNCIA & SINCRONIZAÇÃO</h1>
            <p className="mam-section-desc">
              Compare bibliotecas, sincronize sem sobrescritas inesperadas e valide integridade SHA-256.
            </p>
            <div className="mam-toolbar-row">
              <input
                className="mam-input mam-search-input"
                placeholder="Caminho da biblioteca origem"
                value={origemTransfer}
                onChange={(e) => setOrigemTransfer(e.target.value)}
              />
              <input
                className="mam-input mam-search-input"
                placeholder="Caminho da biblioteca destino"
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
            {diferenca && <pre className="mam-pre">{JSON.stringify(diferenca, null, 2)}</pre>}
            {relatorioTransfer && (
              <pre className="mam-pre">{JSON.stringify(relatorioTransfer, null, 2)}</pre>
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

      {/* Modal flutuante de Importação por Link Direto / HTML / Lista Ordenada */}
      {modalImportarUrlAberto && (
        <div className="mam-modal-backdrop" onClick={() => setModalImportarUrlAberto(false)}>
          <form
            className="mam-modal"
            style={{ maxWidth: 620 }}
            onClick={(e) => e.stopPropagation()}
            onSubmit={handleImportarExtrator}
          >
            <div className="mam-modal-header">
              <strong>Importar por Link Direto / HTML / Lista Ordenada</strong>
              <button
                type="button"
                className="mam-btn"
                onClick={() => setModalImportarUrlAberto(false)}
              >
                ✕
              </button>
            </div>
            <p className="mam-section-desc">
              Extratores ativos: {extratores.map((e) => e.id).join(', ') || 'mangadex, mangafire, generico'}
            </p>
            <div className="mam-toolbar-row">
              <input
                className="mam-input mam-search-input"
                placeholder="Título da obra (ex: Medaka Box)"
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
            <input
              className="mam-input"
              placeholder="Link (mangadex.org/chapter/..., mangafire.to/...)"
              value={extUrl}
              onChange={(e) => setExtUrl(e.target.value)}
            />
            <textarea
              className="mam-textarea"
              placeholder="Lista ordenada de URLs de páginas (uma por linha, opcional)"
              value={extUrlsTexto}
              onChange={(e) => setExtUrlsTexto(e.target.value)}
              rows={3}
            />
            <textarea
              className="mam-textarea"
              placeholder="HTML da página do capítulo (opcional — útil para MangaFire/sites dinâmicos)"
              value={extHtml}
              onChange={(e) => setExtHtml(e.target.value)}
              rows={3}
            />
            <div className="mam-toolbar-row" style={{ justifyContent: 'flex-end' }}>
              <button
                type="button"
                className="mam-btn"
                onClick={() => setModalImportarUrlAberto(false)}
              >
                Fechar
              </button>
              <button className="mam-btn mam-btn-primary" type="submit">
                Importar e Gerar CBZ
              </button>
            </div>
            {resultadoImportacao && (
              <pre className="mam-pre">{JSON.stringify(resultadoImportacao, null, 2)}</pre>
            )}
          </form>
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
