# Design — implement-ui-panels-and-docs

## Context

Todos os comandos Tauri de backend (`library_*`, `reader_*`, `chapter_*`, `transfer_*`, `sync_*`, `ai_*`) já estão registrados em `backend/src-tauri/src/lib.rs`. Resta conectar os painéis administrativos em `backend/frontend/src/` e atualizar `README.md`.

## Goals / Non-Goals

**Goals:**
- Expor detecção de capítulos, transferência/sincronização e status/modelos do AI Engine na interface React.
- Documentar claramente o uso portátil e o build a partir do GitHub no `README.md`.

**Non-Goals:**
- Instalar dependências adicionais de UI (manter React puro e leve).

## Decisions

1. **Abas simples em `App.tsx`**: Estado local `abaAtiva` (`biblioteca` | `transferencia` | `ai`) mantendo `CbzReader` em tela cheia quando um capítulo é aberto.
