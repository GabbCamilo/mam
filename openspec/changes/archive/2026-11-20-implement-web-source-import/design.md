# Design — implement-web-source-import

## Context

`mam-source-integration` já possui `Fonte`, `SourceIntegrationManager`, `busca.rs`, `capitulos.rs`, `duplicatas.rs` e `operacao.rs`.

## Goals / Non-Goals

**Goals:**
- Implementar `MangaDexFonte` usando `reqwest` contra a API pública oficial (`api.mangadex.org`) com User-Agent explícito (`MAM/0.1.0`).
- Permitir arquivamento real em `.cbz` usando escrita atômica (`temp` + `rename`) e registro no SQLite.
- Integrar a aba **Fontes & Importação** em `backend/frontend/src/App.tsx`.

**Non-Goals:**
- Suportar login/contas privadas ou fontes com DRM/paywall.

## Decisions

1. **Empacotamento em `.cbz` no `obter_capitulo`**: `MangaDexFonte::obter_capitulo` baixa as páginas da API pública `at-home/server` e monta um buffer ZIP (CBZ) em memória com imagens ordenadas (`0001.jpg`, etc.), permitindo que `arquivar_para_biblioteca` valide e grave o `.cbz` atomicamente no disco.
2. **Testes offline determinísticos**: Os testes unitários/integração do arquivamento usam uma `Fonte` mock local (sem depender de internet no CI/local) para validar gravação CBZ, SHA-256 e progresso 100%.
