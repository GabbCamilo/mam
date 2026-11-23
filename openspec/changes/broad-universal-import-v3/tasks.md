# Tasks: Broad Universal Import v3

> Metodologia: OpenSpec (`Explore -> Propose -> Specify -> Design -> Tasks -> Apply -> Verify -> Archive`).
> Supervisão: Antigravity coordena, valida `git status`/`git diff` e executa todos os gates de qualidade antes de aceitar qualquer tarefa.

## Frente A — Amplificação do `ai-engine` (`web_reader.py`)

- [x] **MAM-AMP-001 — Classificador estrutural de grupos de capítulos no DOM**
  - **Arquivos:** `ai-engine/mam_ai_engine/web_reader.py`, `ai-engine/tests/test_web_reader.py`
  - **Critério de aceitação:** Identificar lista de capítulos em HTML/DOM sintético onde as URLs não contêm `chapter`/`capitulo` (ex.: `/v1/101`, `/v1/102`, `/v1/103`) com ordenação correta.
  - **Gate:** `python -m unittest discover -s ai-engine/tests -v`

- [x] **MAM-AMP-002 — Seleção geométrica de `<img>` e fallback `<canvas>` no CDP**
  - **Arquivos:** `ai-engine/mam_ai_engine/web_reader.py`, `ai-engine/tests/test_web_reader.py`
  - **Critério de aceitação:** Filtrar ícones/banners por dimensões (`naturalWidth >= 400`, `naturalHeight >= 500`), aceitar URLs de imagem sem extensão explícita quando dentro do leitor e suportar extração de `<canvas>` quando aplicável.
  - **Gate:** `python -m unittest discover -s ai-engine/tests -v`

- [ ] **MAM-AMP-003 — Inferência de template de URL em respostas JSON/GraphQL**
  - **Arquivos:** `ai-engine/mam_ai_engine/web_reader.py`, `ai-engine/tests/test_web_reader.py`
  - **Critério de aceitação:** Extrair capítulos e páginas de payloads JSON aninhados (GraphQL/REST) sem duplicatas.
  - **Gate:** `python -m unittest discover -s ai-engine/tests -v`

## Frente B — Captura Assistida para Sites Protegidos (`mgread.io` e similares)

- [x] **MAM-AMP-004 — Janela WebView Assistida no Tauri 2 (`abrir` e `capturar`)**
  - **Arquivos:** `backend/src-tauri/src/lib.rs`, `backend/src-tauri/capabilities/default.json`
  - **Critério de aceitação:** Comandos Tauri compilam em release, abrem janela secundária com a URL informada e retornam `{ url, html, urls_imagens }` do DOM renderizado pelo usuário, sem burlar desafios anti-bot.
  - **Gate:** `cargo check --manifest-path backend/Cargo.toml -p app`

- [x] **MAM-AMP-005 — Integração na UI ("Capturar da Janela Assistida")**
  - **Arquivos:** `backend/frontend/src/api.ts`, `backend/frontend/src/App.tsx`
  - **Critério de aceitação:** Botões no modal **Importar por Link / Página da Obra** permitem abrir a janela assistida e capturar o DOM/imagens direto para descoberta ou importação em lote.
  - **Gate:** `npm run build` em `backend/frontend/`

## Supervisão e Consolidação

- [ ] **MAM-AMP-006 — Revisão de segurança, qualidade e sincronização OpenSpec**
  - **Critério de aceitação:** `git diff` revisado, nenhum segredo/bypass anti-bot introduzido, `cargo test`, `unittest` e `npm run build` verdes, e delta-specs sincronizadas em `openspec/specs/`.
