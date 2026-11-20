# Proposal — implement-ai-part3-review-and-cache

## Why

Para concluir as 3 partes do Manga AI Engine com máxima eficiência, a **Parte 3** adiciona:
1. **Cache de IA (L3)** indexado por `SHA-256(imagem) + modelo + versão + idiomas`, evitando reprocessamento redundante de páginas já analisadas e invalidando automaticamente quando qualquer parâmetro relevante muda.
2. **Revisão Manual por Região/Balão** no `CbzReader.tsx`, permitindo corrigir `texto_original` e `texto_traduzido` de cada balão individualmente e atualizar o estado para `revisado`.

## What Changes

- `backend/interface/Cargo.toml` e `backend/interface/src/lib.rs`: calcular chave SHA-256 de cache de IA (`calcular_chave_cache_ia`), consultar/gravar em `configuracao_biblioteca` (`ai:cache:<hash>`) em `ai_processar_e_salvar_pagina` e adicionar testes de *cache hit* e invalidação.
- `backend/frontend/src/components/CbzReader.tsx`: adicionar editor por região/balão no painel de revisão IA (`texto_original` e `texto_traduzido` editáveis, salvando `revisado` via `aiRevisarResultado`).

## Capabilities

### New Capabilities

- `ai-part3-review-and-cache`: cache determinístico de IA por hash da imagem/modelo/idiomas e editor de revisão manual por região no leitor.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos modificados: `backend/interface/`, `backend/frontend/src/components/CbzReader.tsx`.
- Arquivos `.cbz` originais permanecem intactos.
