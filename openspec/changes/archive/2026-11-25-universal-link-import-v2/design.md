# Design: Universal Link/HTML Import v2

## Arquitetura em 4 Camadas
1. **Camada 1 (HTTP + HTML/HTMX/JSON embutido):** Parsing de `<meta og:image>`, fragmentos HTMX (`/full-chapter-list`), WordPress Madara (`/ajax/chapters/`) e URLs escapadas em scripts (`__NEXT_DATA__`/RSC).
2. **Camada 2 (CDP Network JSON + DOM):** `web_reader.py` habilita `Network.enable` e inspeciona respostas `application/json` via `Network.getResponseBody` para extrair capítulos e imagens de SPAs (`atsu.moe`, `mangaball.com`, `comick.dev`).
3. **Camada 3 (Fluxo Assistido):** Botão para abrir a URL no navegador do usuário e colar HTML/DOM renderizado para sites com verificação humana ou anti-bot.
4. **Camada 4 (Conectores Declarativos):** `ConectorSiteConfig` ganha `headers_imagem` (opcional, retrocompatível via `#[serde(default)]`) para enviar `Referer`/`Origin` adequados por domínio.
