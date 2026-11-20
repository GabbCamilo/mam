# Tasks — implement-web-source-import

## 1. Backend (`mam-source-integration`, `mam-interface`, `src-tauri`)

- [x] 1.1 Criar `backend/source-integration/src/mangadex.rs` implementando `Fonte` para a API pública do MangaDex
- [x] 1.2 Implementar `arquivar_para_biblioteca` em `backend/source-integration/src/lib.rs` (gravação atômica `.cbz`, SHA-256 e registro no banco)
- [x] 1.3 Registrar `MangaDexFonte` e expor `source_status_operacao` + arquivamento real em `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs`

## 2. Frontend (`backend/frontend/src/`)

- [x] 2.1 Adicionar wrappers `source_*` em `backend/frontend/src/api.ts`
- [x] 2.2 Adicionar aba **Fontes & Importação** em `backend/frontend/src/App.tsx`

## 3. Validação

- [x] 3.1 Criar teste de integração em `mam-source-integration` validando arquivamento real em `.cbz` com fonte local mockada
- [x] 3.2 Executar `npm run build`, `cargo test --workspace` e `openspec validate implement-web-source-import`
