# Tasks — implement-ai-part2-reader-overlay

## 1. Backend Rust (`mam-ai-engine`, `mam-interface`, `src-tauri`)

- [x] 1.1 Adicionar `AiEngineManager::processar_pagina` em `backend/ai-engine/src/lib.rs`
- [x] 1.2 Implementar e registrar `ai_processar_e_salvar_pagina` em `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs` (com teste de integração)

## 2. Frontend (`CbzReader`)

- [x] 2.1 Adicionar `aiProcessarESalvarPagina` em `backend/frontend/src/api.ts`
- [x] 2.2 Adicionar botão de processamento IA e camada visual de sobreposição por `bbox` em `backend/frontend/src/components/CbzReader.tsx`

## 3. Validação

- [x] 3.1 Executar `npm run build`, `cargo test --workspace` e `openspec validate implement-ai-part2-reader-overlay`
