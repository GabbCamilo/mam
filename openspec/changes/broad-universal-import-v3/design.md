# Design: Broad Universal Import v3

## Princípios Arquiteturais (`AGENTS.md`)
- **Monólito modular Rust + processo auxiliar Python (JSON-RPC 2.0 stdio):** nenhuma nova dependência pesada ou serviço externo.
- **Hardware mínimo (i7-6700K, 16 GB RAM, SSD, CPU obrigatório):** o classificador estrutural de DOM usa apenas stdlib Python + avaliação JS no CDP, sem exigir GPU ou PyTorch.
- **Preservação de arquivos originais e portabilidade:** todas as importações continuam gerando `.cbz` atomicamente na `MAM_LIBRARY`.

## Arquitetura das Duas Frentes

### Frente A — Classificação Estrutural e Geométrica (`web_reader.py`)
1. **Agrupamento por ancestral comum:** um snippet JS no CDP agrupa elementos `<a>` pelo seu contêiner pai/avô (`ul`, `div`, `table`) e calcula:
   - quantidade de links irmãos (`>= 3`),
   - monotonia numérica nos textos/hrefs,
   - consistência de prefixo de caminho.
   O grupo com maior pontuação é selecionado mesmo sem tokens como `chapter` na URL.
2. **Seleção geométrica de páginas:** na extração de capítulo, além de extensão de arquivo, prioriza `<img>` visíveis com `naturalWidth >= 400 && naturalHeight >= 500` (descartando banners horizontais e ícones) e captura `<canvas>` renderizados quando não houver `<img>` válido.

### Frente B — Janela WebView Assistida (`src-tauri` + `frontend`)
1. Novo comando Tauri `browser_abrir_janela_assistida(url)` que abre uma `tauri::WebviewWindow` secundária controlada pelo usuário.
2. Novo comando Tauri `browser_capturar_janela_assistida()` que injeta um script de leitura somente-leitura na janela secundária para coletar `{ url, html, urls_imagens }` do DOM já carregado pelo usuário e fecha ou minimiza a janela após a captura.
3. O frontend preenche automaticamente o fluxo de descoberta (`source_descobrir_por_link(url, html)`) ou importação (`urls_ordenadas`).
