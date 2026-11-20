import { useEffect, useState } from 'react'
import { Capitulo, DetalhesObra, Obra, mamApi } from './api'
import { CbzReader } from './components/CbzReader'

function App() {
  const [obras, setObras] = useState<Obra[]>([])
  const [detalhes, setDetalhes] = useState<DetalhesObra | null>(null)
  const [capituloAtivo, setCapituloAtivo] = useState<Capitulo | null>(null)
  const [tituloNovaObra, setTituloNovaObra] = useState('')
  const [caminhoNovaObra, setCaminhoNovaObra] = useState('')
  const [mensagem, setMensagem] = useState<string | null>(null)

  async function carregarObras() {
    try {
      const lista = await mamApi.listarObras()
      setObras(lista)
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

  async function handleSelecionarObra(obraId: string) {
    try {
      const d = await mamApi.obterDetalhesObra(obraId)
      setDetalhes(d)
      setCapituloAtivo(null)
    } catch (err) {
      setMensagem(`Erro ao carregar detalhes: ${String(err)}`)
    }
  }

  const todosCapitulosObra: Capitulo[] = detalhes
    ? [
        ...detalhes.volumes.flatMap((v) => v.capitulos),
        ...detalhes.capitulos_avulsos,
      ].sort((a, b) => a.numero - b.numero)
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
    <div style={{ padding: '1.5rem', fontFamily: 'system-ui, sans-serif', maxWidth: 960, margin: '0 auto' }}>
      <h1>Manga Archive Manager (MAM)</h1>
      <p>Biblioteca digital portátil com organização híbrida de volumes e leitor CBZ integrado.</p>

      {mensagem && (
        <p style={{ padding: '0.5rem 0.75rem', background: '#eef2ff', borderRadius: 6 }}>{mensagem}</p>
      )}

      <form onSubmit={handleCriarObra} style={{ display: 'flex', gap: '0.5rem', marginBottom: '1.25rem' }}>
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
    </div>
  )
}

export default App
