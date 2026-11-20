# Design — implement-cbz-reader

## Context

`mam-archive` já lista páginas de `.cbz` via `ZipArchive` sem extração permanente (`ler_cbz`), e `mam-library` já expõe catálogo de obras, volumes e capítulos. Falta a leitura sob demanda de uma página específica e a interface React + TypeScript integrada ao Tauri 2.

## Goals / Non-Goals

**Goals:**
- Ler páginas individuais de `.cbz` em memória sob demanda (`ler_pagina_cbz`).
- Persistir progresso de leitura, favoritos e marcadores no SQLite local sem alterar o schema de migração inicial (reutilizando `configuracao_biblioteca` com prefixos estruturados `reader:progresso:`, `reader:favorito:`, `reader:marcador:`).
- Construir UI limpa e responsiva em React + TypeScript (`backend/frontend/src/`) conectada aos comandos Tauri.

**Non-Goals:**
- Cache L1/L2 avançado de pré-carregamento em disco (pode ser evoluído posteriormente).
- Sobreposição de resultados de IA / tradução (pertence à Fase 10 — Manga AI Engine).

## Decisions

1. **Retorno de página como `PaginaExtraida` (bytes base64 + `mime_type`)**: Evita gravação de arquivos temporários em disco e funciona diretamente com `<img src="data:..." />` no WebView do Tauri 2.
2. **Persistência em `configuracao_biblioteca` (chave-valor SQLite)**: Evita criar nova migração SQLite nesta fase mantendo compatibilidade total com bancos existentes e portabilidade em SSD externo.
3. **Resolução de caminho relativo com `raiz_biblioteca`**: Comandos do leitor aceitam caminho relativo (ou absoluto resolvido a partir da raiz configurada) garantindo portabilidade.

## Risks / Trade-offs

- [Uso de memória com Base64] Páginas únicas de mangá (1–5 MB) são leves para decodificação sob demanda; não carregar o capítulo inteiro de uma vez.
