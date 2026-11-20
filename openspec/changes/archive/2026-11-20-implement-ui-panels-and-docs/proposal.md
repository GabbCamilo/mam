# Proposal — implement-ui-panels-and-docs

## Why

O backend Rust e o AI Engine Python já expõem comandos completos para biblioteca, leitor CBZ, detecção de capítulos, transferência/sincronização e Manga AI Engine, porém o frontend React (`backend/frontend/src/App.tsx`) expõe apenas a aba de Biblioteca/Leitor e o `README.md` raiz ainda não documenta o fluxo de compilação, testes e uso portátil em SSD externo para quem clona o repositório do GitHub.

## What Changes

- `backend/frontend/src/api.ts`: adicionar wrappers tipados para `library_raiz`, `chapter_detectar`, `transfer_copia_integral`, `sync_comparar_bibliotecas`, `sync_sincronizar_bibliotecas`, `ai_health` e `ai_listar_modelos`.
- `backend/frontend/src/App.tsx`: adicionar navegação por abas (`Biblioteca`, `Transferência & Sync`, `AI Engine`), botão de detecção de capítulos e painéis funcionais para transferência/sincronização e inspeção do AI Engine.
- `README.md`: documentar pré-requisitos, execução em desenvolvimento, testes (`cargo test --workspace`, `unittest`, `npm run build`), empacotamento portátil (`packaging/`) e variáveis de portabilidade (`MAM_LIBRARY_DIR`, `MAM_AI_ENGINE_DIR`).

## Capabilities

### New Capabilities

- `ui-panels`: painéis administrativos no frontend React integrados aos comandos Tauri de detecção de capítulos, transferência/sincronização e Manga AI Engine.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos modificados: `backend/frontend/src/api.ts`, `backend/frontend/src/App.tsx`, `README.md`.
- Sem alterações destrutivas ou novas dependências externas.
