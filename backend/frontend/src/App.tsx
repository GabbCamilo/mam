import { useEffect, useState } from 'react'
import {
  Capitulo,
  DetalhesObra,
  DiferencaBibliotecas,
  ModeloInfo,
  Obra,
  RelatorioTransferencia,
  mamApi,
} from './api'
import { CbzReader } from './components/CbzReader'

type AbaAtiva = 'biblioteca' | 'transferencia' | 'ai'

function App() {
  const [abaAtiva, setAbaAtiva] = useState<AbaAtiva>('biblioteca')
  const [raizBiblioteca, setRaizBiblioteca] = useState<string>('')
  const [obras, setObras] = useState<Obra[]>([])
  const [detalhes, setDetalhes] = useState<DetalhesObra | null>(null)
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

  async function carregarObras() {
    try {
      const [lista, raiz] = await Promise.all([mamApi.listarObras(), mamApi.libraryRaiz()])
      setObras(lista)
      setRaizBiblioteca(raiz)
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
    try {
      const d = await mamApi.obterDetalhesObra(obraId)
      setDetalhes(d)
      setCapituloAtivo(null)
    } catch (err) {
      setMensagem(`Erro ao carregar detalhes: ${String(err)}`)
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

  const todosCapitulosObra: Capitulo[] = detalhes
    ? [...detalhes.volumes.flatMap((v) => v.capitulos), ...detalhes.capitulos_avulsos].sort(
        (a, b) => a.numero - b.numero,
      )
    : []

  if (capituloAtivo) {
    return (
      <div style={{ padding: '1.25rem', fontFamily: 'system-ui, sans-serif' }}>
        <CbzReader
          capitulo={capituloAtivo}
          capitulosDisponiveis={todosCapitulosObra}
          onSelecionarCapitulo={setCapituloAtivo}
          onFechar={() => setCapituloAtivo(null)}
        />
      </div>
    )
  }

  return (
    <div style={{ padding: '1.5rem', fontFamily: 'system-ui, sans-serif', maxWidth: 980, margin: '0 auto' }}>
      <h1>Manga Archive Manager (MAM)</h1>
      <p style={{ color: '#555' }}>
        Biblioteca portátil — Raiz ativa: <code>{raizBiblioteca || 'carregando...'}</code>
      </p>

      <nav style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
        <button onClick={() => setAbaAtiva('biblioteca')}>Biblioteca</button>
        <button onClick={() => setAbaAtiva('transferencia')}>Transferência & Sync</button>
        <button onClick={() => setAbaAtiva('ai')}>AI Engine</button>
      </nav>

      {mensagem && (
        <p style={{ padding: '0.5rem 0.75rem', background: '#eef2ff', borderRadius: 6 }}>{mensagem}</p>
      )}

      {abaAtiva === 'biblioteca' && (
        <>
          <form onSubmit={handleCriarObra} style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
            <input
              placeholder="Título da obra"
              value={tituloNovaObra}
              onChange={(e) => setTituloNovaObra(e.target.value)}
            />
            <input
              placeholder="Caminho relativo (ex: Manga/Obra)"
              value={caminhoNovaObra}
              onChange={(e) => setCaminhoNovaObra(e.target.value)}
            />
            <button type="submit">Cadastrar Obra</button>
            <button type="button" onClick={handleDetectarCapitulos}>
              Detectar Capítulos CBZ
            </button>
          </form>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '1rem' }}>
            <section>
              <h2>Obras ({obras.length})</h2>
              <ul style={{ listStyle: 'none', padding: 0, display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                {obras.map((obra) => (
                  <li key={obra.id}>
                    <button
                      onClick={() => handleSelecionarObra(obra.id)}
                      style={{ width: '100%', textAlign: 'left', padding: '0.5rem' }}
                    >
                      {obra.titulo}
                    </button>
                  </li>
                ))}
              </ul>
            </section>

            <section>
              {detalhes ? (
                <div>
                  <h2>{detalhes.obra.titulo}</h2>
                  <p style={{ color: '#555' }}>Caminho relativo: {detalhes.obra.caminho_relativo}</p>

                  {detalhes.volumes.map((vc) => (
                    <div key={vc.volume.id} style={{ marginBottom: '1rem' }}>
                      <h3>
                        Volume {vc.volume.numero} {vc.volume.titulo ? `— ${vc.volume.titulo}` : ''}
                      </h3>
                      <ul>
                        {vc.capitulos.map((cap) => (
                          <li key={cap.id}>
                            <button onClick={() => setCapituloAtivo(cap)}>
                              Ler Capítulo {cap.numero} ({cap.quantidade_paginas} págs)
                            </button>
                          </li>
                        ))}
                      </ul>
                    </div>
                  ))}

                  {detalhes.capitulos_avulsos.length > 0 && (
                    <div>
                      <h3>Capítulos Avulsos</h3>
                      <ul>
                        {detalhes.capitulos_avulsos.map((cap) => (
                          <li key={cap.id}>
                            <button onClick={() => setCapituloAtivo(cap)}>
                              Ler Capítulo {cap.numero} ({cap.quantidade_paginas} págs)
                            </button>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              ) : (
                <p>Selecione uma obra para visualizar volumes e capítulos.</p>
              )}
            </section>
          </div>
        </>
      )}

      {abaAtiva === 'transferencia' && (
        <section style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
          <h2>Transferência e Sincronização Portátil</h2>
          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
            <input
              placeholder="Caminho da biblioteca origem"
              value={origemTransfer}
              onChange={(e) => setOrigemTransfer(e.target.value)}
              style={{ minWidth: 260 }}
            />
            <input
              placeholder="Caminho da biblioteca destino"
              value={destinoTransfer}
              onChange={(e) => setDestinoTransfer(e.target.value)}
              style={{ minWidth: 260 }}
            />
          </div>
          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
            <button onClick={handleCompararBibliotecas}>Comparar Bibliotecas</button>
            <button onClick={() => handleSincronizar(false)}>Sincronizar (Sem Sobrescrita)</button>
            <button onClick={handleCopiaIntegral}>Cópia Integral (SHA-256)</button>
          </div>
          {diferenca && (
            <pre style={{ background: '#f4f4f6', padding: '0.75rem', borderRadius: 6 }}>
              {JSON.stringify(diferenca, null, 2)}
            </pre>
          )}
          {relatorioTransfer && (
            <pre style={{ background: '#f4f4f6', padding: '0.75rem', borderRadius: 6 }}>
              {JSON.stringify(relatorioTransfer, null, 2)}
            </pre>
          )}
        </section>
      )}

      {abaAtiva === 'ai' && (
        <section style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
          <h2>Manga AI Engine (Local — JSON-RPC stdio)</h2>
          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
            <button onClick={handleVerificarAi}>Verificar Status do AI Engine</button>
            <input
              placeholder="Diretório de modelos locais (opcional)"
              value={diretorioModelos}
              onChange={(e) => setDiretorioModelos(e.target.value)}
              style={{ minWidth: 260 }}
            />
            <button onClick={handleListarModelos}>Listar Modelos Locais</button>
          </div>
          {aiStatus && (
            <pre style={{ background: '#f4f4f6', padding: '0.75rem', borderRadius: 6 }}>
              {JSON.stringify(aiStatus, null, 2)}
            </pre>
          )}
          {modelos.length > 0 && (
            <ul>
              {modelos.map((m) => (
                <li key={m.caminho}>
                  {m.nome} ({m.formato}, {m.tamanho_bytes} bytes)
                </li>
              ))}
            </ul>
          )}
        </section>
      )}
    </div>
  )
}

export default App
