import { useEffect, useState } from 'react'
import { Capitulo, MarcadorPagina, PaginaExtraida, ResultadoIA, mamApi } from '../api'

type ModoLeitura = 'horizontal' | 'vertical' | 'dupla'
type AjusteTela = 'fit-width' | 'fit-height' | 'original'

interface CbzReaderProps {
  capitulo: Capitulo
  capitulosDisponiveis: Capitulo[]
  onSelecionarCapitulo: (capitulo: Capitulo) => void
  onFechar: () => void
}

export function CbzReader({
  capitulo,
  capitulosDisponiveis,
  onSelecionarCapitulo,
  onFechar,
}: CbzReaderProps) {
  const [indiceAtual, setIndiceAtual] = useState(0)
  const [paginaPrincipal, setPaginaPrincipal] = useState<PaginaExtraida | null>(null)
  const [paginaSecundaria, setPaginaSecundaria] = useState<PaginaExtraida | null>(null)
  const [modo, setModo] = useState<ModoLeitura>('horizontal')
  const [ajuste, setAjuste] = useState<AjusteTela>('fit-width')
  const [zoom, setZoom] = useState(100)
  const [favorito, setFavorito] = useState(false)
  const [marcadores, setMarcadores] = useState<MarcadorPagina[]>([])
  const [erro, setErro] = useState<string | null>(null)
  const [carregando, setCarregando] = useState(false)
  const [mostrarIa, setMostrarIa] = useState(false)
  const [resultadosIa, setResultadosIa] = useState<ResultadoIA[]>([])
  const [textoRevisao, setTextoRevisao] = useState('')

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

        if (modo === 'dupla' && indiceAtual + 1 < p1.total_paginas) {
          const p2 = await mamApi.obterPagina(capitulo.caminho_relativo, indiceAtual + 1)
          if (ativo) setPaginaSecundaria(p2)
        } else {
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
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  })

  const totalPaginas = paginaPrincipal?.total_paginas ?? capitulo.quantidade_paginas
  const passo = modo === 'dupla' ? 2 : 1

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

  async function handleFavorito() {
    const novo = await mamApi.alternarFavorito(capitulo.id)
    setFavorito(novo)
  }

  async function handleAdicionarMarcador() {
    await mamApi.adicionarMarcador(capitulo.id, indiceAtual, `Página ${indiceAtual + 1}`)
    const atualizados = await mamApi.listarMarcadores(capitulo.id)
    setMarcadores(atualizados)
  }

  const estiloImagem: React.CSSProperties = {
    width: ajuste === 'fit-width' ? `${zoom}%` : 'auto',
    maxHeight: ajuste === 'fit-height' ? '80vh' : 'none',
    transform: ajuste === 'original' ? `scale(${zoom / 100})` : undefined,
    objectFit: 'contain',
    borderRadius: 4,
    boxShadow: '0 2px 12px rgba(0,0,0,0.4)',
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
      <div
        style={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: '0.5rem',
          alignItems: 'center',
          padding: '0.75rem',
          background: '#1e1e24',
          color: '#f4f4f6',
          borderRadius: 8,
        }}
      >
        <button onClick={onFechar}>← Biblioteca</button>
        <strong>
          Capítulo {capitulo.numero} ({indiceAtual + 1} / {Math.max(totalPaginas, 1)})
        </strong>

        <button onClick={voltarPagina}>Página Anterior</button>
        <button onClick={avancarPagina}>Próxima Página</button>

        {capituloAnterior && (
          <button onClick={() => onSelecionarCapitulo(capituloAnterior)}>Cap. Anterior</button>
        )}
        {proximoCapitulo && (
          <button onClick={() => onSelecionarCapitulo(proximoCapitulo)}>Próx. Cap.</button>
        )}

        <select value={modo} onChange={(e) => setModo(e.target.value as ModoLeitura)}>
          <option value="horizontal">Horizontal</option>
          <option value="vertical">Vertical</option>
          <option value="dupla">Página Dupla</option>
        </select>

        <select value={ajuste} onChange={(e) => setAjuste(e.target.value as AjusteTela)}>
          <option value="fit-width">Ajustar à Largura</option>
          <option value="fit-height">Ajustar à Altura</option>
          <option value="original">Tamanho Original</option>
        </select>

        <button onClick={() => setZoom((z) => Math.max(40, z - 10))}>-</button>
        <span>{zoom}%</span>
        <button onClick={() => setZoom((z) => Math.min(200, z + 10))}>+</button>

        <button onClick={handleFavorito}>{favorito ? '★ Favorito' : '☆ Favoritar'}</button>
        <button onClick={handleAdicionarMarcador}>🔖 Marcar Página</button>
        <button onClick={() => setMostrarIa((v) => !v)}>
          {mostrarIa ? 'Ocultar Camada IA' : 'Camada IA / Revisão'}
        </button>
      </div>

      {mostrarIa && (
        <div style={{ padding: '0.75rem', background: '#f4f4f6', borderRadius: 6 }}>
          <strong>Resultados de IA — Página {indiceAtual + 1}</strong>
          {resultadosIa
            .filter((r) => r.pagina_numero === indiceAtual)
            .map((r) => (
              <div key={r.id} style={{ marginTop: '0.5rem' }}>
                <span>
                  Estado: <code>{r.estado_revisao}</code> | Modelo: {r.modelo_id}
                </span>
                <pre style={{ background: '#fff', padding: '0.5rem', borderRadius: 4 }}>
                  {r.resultado_json}
                </pre>
                <div style={{ display: 'flex', gap: '0.5rem' }}>
                  <input
                    placeholder="JSON/texto revisado..."
                    value={textoRevisao}
                    onChange={(e) => setTextoRevisao(e.target.value)}
                    style={{ minWidth: 260 }}
                  />
                  <button
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
            ))}
          {resultadosIa.filter((r) => r.pagina_numero === indiceAtual).length === 0 && (
            <p style={{ margin: '0.4rem 0 0' }}>Nenhum resultado de IA salvo para esta página.</p>
          )}
        </div>
      )}

      {marcadores.length > 0 && (
        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap', fontSize: '0.9rem' }}>
          <span>Marcadores:</span>
          {marcadores.map((m) => (
            <button key={m.pagina} onClick={() => setIndiceAtual(m.pagina)}>
              {m.nota}
            </button>
          ))}
        </div>
      )}

      {erro && (
        <div style={{ padding: '0.75rem', background: '#4a1c1c', color: '#ffd6d6', borderRadius: 6 }}>
          Erro ao carregar página: {erro}
        </div>
      )}

      {carregando && <p>Carregando página...</p>}

      <div
        style={{
          display: 'flex',
          flexDirection: modo === 'vertical' ? 'column' : 'row',
          justifyContent: 'center',
          alignItems: 'center',
          gap: '0.75rem',
          minHeight: '60vh',
          background: '#121216',
          padding: '1rem',
          borderRadius: 8,
          overflow: 'auto',
        }}
      >
        {paginaPrincipal && (
          <img
            src={`data:${paginaPrincipal.mime_type};base64,${paginaPrincipal.dados_base64}`}
            alt={paginaPrincipal.nome}
            style={estiloImagem}
          />
        )}
        {modo === 'dupla' && paginaSecundaria && (
          <img
            src={`data:${paginaSecundaria.mime_type};base64,${paginaSecundaria.dados_base64}`}
            alt={paginaSecundaria.nome}
            style={estiloImagem}
          />
        )}
      </div>
    </div>
  )
}
