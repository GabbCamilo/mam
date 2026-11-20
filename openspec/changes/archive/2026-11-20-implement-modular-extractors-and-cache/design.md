# Design — implement-modular-extractors-and-cache

## Context

Consolidação da arquitetura modular de extratores web + Cache L1 LRU no leitor.

## Decisions

1. **Trait `ExtratorSite` + `RegistroExtratores`**: Cada site implementa `id`, `suporta_url` e `extrair_paginas`, mantendo isolamento completo entre parsers de sites diferentes.
2. **Parse HTML sem dependências pesadas**: Extração ordenada de atributos `data-src` / `src` de imagens em `mangafire.rs` e `generico.rs` sem introduzir crates pesados de DOM.
3. **`CachePaginas` com `tokio::sync::Mutex<CacheL1Lru<String, PaginaExtraida>>`**: Capacidade de 64 páginas em memória no estado gerenciado do Tauri.
