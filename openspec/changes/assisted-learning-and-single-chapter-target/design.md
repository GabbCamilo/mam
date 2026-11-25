# Design: Aprendizado Progressivo, Auto-Captura Assistida e Destino de Capítulo Único

## 1. Memória Progressiva Cross-Site (`backend/source-integration/src/ai_memory.rs`)
- **Arquivo portátil:** `<MAM_LIBRARY>/System/connectors/ai_memory.json` (escrita atômica via arquivo temporário + rename).
- **Estrutura `MemoriaAprendizadoIA`:**
  - `schema_version: u32` (v1)
  - `padroes_url: Vec<PadraoMemoria>` (ex.: `/manga/{slug}/{num}`, `/{seg}/`, `?episode_no=`, etc.)
  - `seletores_capitulo: Vec<PadraoMemoria>` (seletores CSS de links de capítulo observados em listas irmãs)
  - `seletores_imagem: Vec<PadraoMemoria>` (seletores CSS / atributos de imagens de leitura)
  - Cada `PadraoMemoria` mantém `valor`, `hits`, `misses`, `ultima_confirmacao_ts`, `dominios_origem` (amostra) e método `score() -> f64`.
- **Ciclo de reforço:**
  - `registrar_sucesso(...)` incrementa `hits` e atualiza timestamp.
  - `registrar_falha(...)` incrementa `misses`.
  - Na descoberta de capítulos e extração de imagens de sites sem conector específico em `sites.json`, os padrões da memória ordenados por `score()` são aplicados como primeira camada inteligente antes do fallback genérico bruto.

## 2. Aprendizado Profundo DOM + URL (`GerenciadorConectores::aprender_de_sessao`)
- Mantém `aprender_padrao_url` compatível para chamadas simples e adiciona `aprender_de_sessao`:
  - **URL:** identifica prefixos comuns e segmentos numéricos/slug (`/{seg}/`, `/{seg}/{num}`, query params) sem falhar quando a diferença já é o número do capítulo.
  - **DOM da Obra (`html_obra`):** usa `scraper` para localizar `<a>` cujo `href` aponta para `url_capitulo`, extraindo o seletor CSS mais específico e compartilhado pelos links irmãos (classe/tag/container).
  - **DOM do Capítulo (`html_capitulo`):** identifica atributos/seletores predominantes das imagens de leitura (`data-url`, `data-original`, `data-lazy-src`, `data-src`, `src` dentro de containers de leitor).
  - Atualiza `sites.json` e alimenta `ai_memory.json` simultaneamente.

## 3. Auto-Captura e Aprendizado Passivo na Janela Assistida (`backend/src-tauri/src/lib.rs`)
- Ao criar `WebviewWindow` `mam-assistido`, registrar `on_page_load`:
  - Quando `PageLoadEvent::Finished` ocorrer, aguardar um breve debounce para hidratação JS e verificar se a página não é um desafio de borda (`Just a moment...`, `cf-challenge`, `Verify you are human`).
  - Se estiver pronta, executar a captura leve (`capturar_janela_assistida_do_webview`) e emitir `assisted-window://captured` para a janela principal.
  - Se a URL anterior na mesma sessão parecia uma página de obra e a nova URL tem padrão de capítulo (ou imagens de leitura), emitir `assisted-window://chapter-detected` e opcionalmente aprender/reforçar o padrão automaticamente.
- Manter comandos manuais (`browser_capturar_janela_assistida`, `browser_ensinar_padrao_capitulo`) funcionando normalmente como fallback.

## 4. Destino no Modo Capítulo Único (`EntradaImportacao` + UI React)
- **`EntradaImportacao`** (`backend/source-integration/src/extractors/mod.rs` e `backend/frontend/src/api.ts`):
  - Adicionar `#[serde(default)] pub obra_id_destino: Option<String>`.
  - Quando `obra_id_destino` for informado em `importar_capitulo`: buscar a `Obra` no `Database` pelo `ObraId`; se encontrada, salvar o `.cbz` dentro do `caminho_relativo` dessa obra e vincular o capítulo ao seu `ObraId`.
- **Frontend (`backend/frontend/src/App.tsx`):**
  - No modo **Capítulo Único / URLs Manuais**, exibir um seletor de destino (`auto` | `existente` | `novo`):
    - `auto`: detecção automática atual (ou título opcional).
    - `existente`: `<select>` com as obras já cadastradas em `obras`, enviando `obra_id_destino` e `obra_titulo` da obra selecionada.
    - `novo`: input para nome da nova obra/biblioteca + atalho rápido *"Capítulos Unitários"* para agrupar avulsos.
