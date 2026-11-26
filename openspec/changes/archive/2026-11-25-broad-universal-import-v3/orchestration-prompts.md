# Prompts de Delegação e Supervisão — `broad-universal-import-v3`

> **Status do OpenCode CLI neste ambiente (verificado em 22/11/2026):**
> `opencode` está no `PATH`, porém falha ao iniciar com erro de empacotamento do Bun (`Bun failed to remap this bin to its proper location within node_modules`).
> Conforme a Seção 3 e Seção 6 do protocolo de orquestração:
> - **Não** tentar reinstalar/alterar configurações globais ou assumir modelos gratuitos sem comprovação.
> - Executar as tarefas via **Antigravity** (ou delegar ao OpenCode CLI com modelo gratuito comprovado após o usuário reparar a instalação do Bun/OpenCode).

---

## Prompt 1 — `MAM-AMP-001` + `MAM-AMP-002` (Frente A: Classificador DOM + Geometria no `ai-engine`)

```text
TASK: MAM-AMP-001-002

ROLE:
Atue como engenheiro de software Python focado em extração DOM via CDP no ai-engine do MAM.

PROJECT:
Manga Archive Manager (MAM).

CONTEXT:
Consulte AGENTS.md, openspec/changes/broad-universal-import-v3/proposal.md,
openspec/changes/broad-universal-import-v3/design.md e ai-engine/mam_ai_engine/web_reader.py.

OBJECTIVE:
1. Implementar classificador estrutural de grupos repetidos de <a> no DOM para detectar listas de capítulos mesmo sem palavras-chave na URL.
2. Aprimorar extract_chapter_pages com filtragem geométrica (naturalWidth >= 400, naturalHeight >= 500) e fallback para <canvas>.

SCOPE:
- ai-engine/mam_ai_engine/web_reader.py
- ai-engine/tests/test_web_reader.py

RESTRICTIONS:
- Usar apenas stdlib Python (sem novas dependências externas).
- Não modificar arquivos em backend/ ou connectors/.
- Não tentar burlar CAPTCHA ou Cloudflare Turnstile.

EXPECTED OUTPUT:
- Implementação limpa em web_reader.py.
- Novos testes unitários em test_web_reader.py cobrindo lista sem palavra "chapter" e filtro geométrico.

ACCEPTANCE:
python -m unittest discover -s ai-engine/tests -v passando 100%.
```

---

## Prompt 2 — `MAM-AMP-004` + `MAM-AMP-005` (Frente B: Janela WebView Assistida Tauri 2 + UI)

```text
TASK: MAM-AMP-004-005

ROLE:
Atue como engenheiro Rust/Tauri 2 + React/TypeScript.

PROJECT:
Manga Archive Manager (MAM).

CONTEXT:
Consulte AGENTS.md, openspec/changes/broad-universal-import-v3/design.md,
backend/src-tauri/src/lib.rs, backend/frontend/src/api.ts e backend/frontend/src/App.tsx.

OBJECTIVE:
Implementar comandos Tauri para abrir uma janela WebviewWindow secundária (modo assistido)
e capturar { url, html, urls_imagens } do DOM renderizado pelo usuário, integrando ao modal
de Importação por Link / HTML em App.tsx.

SCOPE:
- backend/src-tauri/src/lib.rs
- backend/frontend/src/api.ts
- backend/frontend/src/App.tsx

RESTRICTIONS:
- Não burlar nem automatizar resolução de CAPTCHA/Turnstile; a janela é interativa para o usuário.
- Não alterar o banco SQLite nem o formato dos arquivos .cbz.
- Não executar commit ou push.

EXPECTED OUTPUT:
- Comandos Tauri compilando sem erros e integrados ao fluxo sourceDescobrirPorLink(url, html).

ACCEPTANCE:
- npm run build (em backend/frontend) sem erros.
- cargo check --manifest-path backend/Cargo.toml -p app sem erros.
```

---

## Checklist de Supervisão do Antigravity (Pós-Execução)

1. `git status` e `git diff` inspecionados (nenhum arquivo fora do escopo alterado).
2. Verificação de segurança: nenhum bypass de controle de acesso/CAPTCHA e nenhuma credencial exposta.
3. Gates executados e observados:
   - `python -m unittest discover -s ai-engine/tests -v`
   - `cargo test --manifest-path backend/Cargo.toml --workspace --exclude app` (sem `LLVM-MinGW` no `PATH`)
   - `npm run build` em `backend/frontend`
4. Consolidação das especificações em `openspec/specs/` via fluxo `openspec-sync-specs`.
