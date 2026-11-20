# Tasks — implement-ai-part3-review-and-cache

## 1. Cache de IA em `mam-interface`

- [x] 1.1 Adicionar `sha2` em `backend/interface/Cargo.toml` e implementar `calcular_chave_cache_ia` + reuso/invalidação em `ai_processar_e_salvar_pagina` (`backend/interface/src/lib.rs`)
- [x] 1.2 Criar teste unitário validando determinismo e invalidação do cache de IA

## 2. Revisão manual por região no `CbzReader.tsx`

- [x] 2.1 Atualizar `backend/frontend/src/components/CbzReader.tsx` com editor por região (`texto_original` / `texto_traduzido`) salvando estado `revisado`

## 3. Validação

- [x] 3.1 Executar `npm run build`, `cargo test --workspace` e `openspec validate implement-ai-part3-review-and-cache`
