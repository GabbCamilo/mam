# Change: Universal Link/HTML Import v2

## Why
Sites de mangá modernos utilizam arquiteturas diversas além de HTML estático (`<a href>`):
- **HTMX / Fragmentos** (`weebcentral.com`): lista completa de capítulos/volumes e páginas carregadas em sub-rotas HTMX.
- **Next.js / RSC** (`mangaball.com`): rotas `/chapter-detail/` e dados hidratados em scripts JSON.
- **SPAs com API REST** (`atsu.moe`): HTML inicial sem links de capítulos; dados carregados via `fetch()` JSON.
- **Anti-hotlink / AJAX** (`mangago.me`): exige `Referer` por domínio para capas/páginas e suporte a fluxo assistido.

## What Changes
1. Evolução declarativa de `connectors/default_sites.json` com `headers_imagem` por domínio e padrões de capítulo atualizados (`/chapter-detail/`, etc.).
2. Suporte automático a `Referer`/headers por domínio ao baixar capas e páginas de capítulos.
3. Descoberta estática aprimorada para JSON embutido (desescape de `\/` e `\u002F`) e padrões universais.
4. Interceptação `Network.enable` + `Network.getResponseBody` no `web_reader.py` (CDP) para capturar JSONs de capítulos e páginas em SPAs puras.
5. Botão auxiliar de fluxo assistido no modal de importação por link/HTML.
