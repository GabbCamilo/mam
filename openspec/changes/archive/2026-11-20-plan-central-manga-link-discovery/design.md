# Design Técnico: Descoberta Estrutural e Modal de Importação

## Arquitetura Proposta

### 1. Frontend — Modal em 2 Abas (`backend/frontend/src/App.tsx`)
- **Aba 1: Página Central da Obra (Lote)**
  - Campo principal: `URL da obra` (ex.: `https://mangafire.to/title/jw2-medaka-boxx` ou `https://mangalivre.to/manga/chainsaw-man-pt-br/`).
  - Campo opcional recolhível: `HTML copiado da página` (para quando o site carregar a lista apenas via JS no navegador).
  - Botão principal: **🔎 Descobrir Capítulos**.
- **Aba 2: Capítulo Único / URLs Manuais**
  - Título da obra, número do capítulo, URL do capítulo, lista ordenada de imagens ou HTML.

### 2. Backend Rust — Analisador Estrutural (`backend/source-integration/src/extractors/`)
- **Camada 1 — Adaptadores de Padrão de Site**:
  - `MangaFire`: detecta `/title/<slug>/chapter/<id>` e `/read/<slug>/<lang>/chapter-<N>`, além de atributos `data-number` e blocos `.chapter-item` / `ul.scroll-content`.
  - `MangaLivre / Madara`: detecta `/manga/<slug>/capitulo-<N>/` e listas `.wp-manga-chapter > a`.
- **Camada 2 — Heurística Estrutural Genérica**:
  - Agrupa links `<a>` por prefixo de rota comum e frequência de repetição, extraindo número do capítulo (`capitulo-`, `chapter-`, `cap.`, `ch.`), título e idioma (`pt-br`, `en`, `ja`).
  - Inspeciona objetos JSON embutidos em `<script>` (ex.: `__NEXT_DATA__`) quando presentes no HTML público.
- **Camada 3 — Projeção Estrutural via `ai-engine` (Opcional / Fallback)**:
  - Quando as camadas 1 e 2 retornarem `0` capítulos em um layout inédito, envia apenas um resumo compacto dos blocos de links (`{ tag, class, href, text }`) via JSON-RPC stdio para o `ai-engine` local identificar o seletor/agrupamento de capítulos.

### 3. Restrições de Segurança e Conformidade
- Nenhuma técnica de contorno de anti-bot, CAPTCHA, Cloudflare Turnstile ou DRM é implementada; quando um site exigir navegador, o usuário cola o HTML público da página no campo opcional.
