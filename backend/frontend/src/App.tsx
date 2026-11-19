import { invoke } from '@tauri-apps/api/core'

async function checkHealth() {
  try {
    const result = await invoke<string>('health_check')
    console.log('Health check:', result)
  } catch (error) {
    console.error('Health check failed:', error)
  }
}

function App() {
  return (
    <div style={{ padding: '2rem', fontFamily: 'system-ui, sans-serif' }}>
      <h1>Manga Archive Manager</h1>
      <p>Aplicação desktop para gerenciamento de mangás e quadrinhos digitais.</p>
      <button onClick={checkHealth}>Verificar status do backend</button>
    </div>
  )
}

export default App
