# Design: Conector Universal (`universal-connector-ai`)

## 1. Orquestrador `ConectorUniversal` (`backend/source-integration/src/universal.rs`)
- **Estrutura `RotaCorrelacionada`:**
  - `conector_id: String` (id do conector correspondente ou `"memoria-ia"` / `"universal-heuristica"`)
  - `conector_nome: String`
  - `origem_correlacao: String` (`"dominio-registrado"` | `"memoria-progressiva"` | `"ai-engine"`)
  - `padroes_url: Vec<String>` (merge ordenado: conector do domínio + `GerenciadorMemoriaIA::padroes_ranqueados`)
  - `seletores_imagem: Vec<String>`
- **Métodos:**
  - `correlacionar_link(url: &str, raiz_biblioteca: Option<&Path>) -> RotaCorrelacionada`
  - `buscar_universal(source_manager: &SourceIntegrationManager, raiz_biblioteca: &Path, consulta: &str) -> Vec<ObraBuscaResultado>`
    - Executa em paralelo (`futures::join_all` ou `tokio::join!` / `JoinSet`) a busca no MangaDex e nos conectores com `url_busca` configurada, tolerando erros parciais por conector.

## 2. Integração com `ai-engine` (`web_reader.py` e `pipeline.py`)
- `smart_discover_chapters` e `smart_extract_chapter_pages` aceitam parâmetro opcional `memory_hints: { padroes_url?: string[], seletores_imagem?: string[] }`.
- Os `padroes_url` recebidos da memória portátil são incorporados às heurísticas de `discover_chapters` e `seletores_imagem` são avaliados prioritariamente em `extract_chapter_pages`.

## 3. UI (`App.tsx`)
- `conectorBuscaAtivo` inicia em `'universal'` (`Universal (IA Portátil)` como primeira `<option>`).
- Nos cards de resultado da aba **Fontes**, exibe badge sutil com `obra.fonte_id` quando em modo universal.
- Na importação por link, exibe a rota correlacionada retornada/detectada para o link importado.
