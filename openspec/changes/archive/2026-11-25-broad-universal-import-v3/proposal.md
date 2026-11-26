# Change: Broad Universal Import v3 (DOM Estrutural + Sessão Assistida)

## Why
Mesmo após `universal-link-import-v2` (HTMX, `Referer` por domínio, scripts escapados e interceptação JSON via CDP), dois cenários impedem a importação universal entre os 25+ sites de mangá:
1. **Sites com bloqueio de borda / desafio humano (ex.: `mgread.io`):** retornam HTTP 403 (`cf-mitigated: challenge`, `"Just a moment..."`) para `reqwest` e navegadores headless (`--headless=new`), impedindo qualquer leitura automatizada.
2. **Layouts sem palavras-chave na URL:** sites em que URLs de capítulos não contêm `/chapter/` ou `/read/` (ex.: `/v1/982341`) ou leitores que renderizam páginas em `<canvas>`.

## What Changes
- **Frente A (`ai-engine/mam_ai_engine/web_reader.py`):**
  - Classificador estrutural de grupos repetidos no DOM (irmãos `<a>` com sequência numérica monotônica e prefixo comum).
  - Detecção visual/geométrica de páginas (`<img>` por `naturalWidth >= 400` / `naturalHeight >= 500` e leitura opcional de `<canvas>`).
  - Inferência de templates de URL a partir de respostas JSON/GraphQL capturadas pelo CDP.
- **Frente B (`backend/src-tauri/`, `backend/interface/`, `backend/frontend/`):**
  - Fluxo de **Janela WebView Assistida** no Tauri 2: o usuário abre o site em janela visível, resolve normalmente eventuais verificações humanas, e aciona **"Capturar Página / Imagens da Janela"** para enviar DOM e URLs renderizadas diretamente ao pipeline existente (`source_descobrir_por_link` / `source_importar_extrator`).
  - Ponte HTTP local opcional (`127.0.0.1`) para receber capturas enviadas por extensão/bookmarklet do navegador do usuário.

## Out of Scope
- Qualquer tentativa de contornar, resolver ou burlar CAPTCHAs, Cloudflare Turnstile ou desafios anti-bot de forma automatizada.
- Criação de ADRs ou microsserviços externos.
