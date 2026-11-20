# Tasks — implement-ux-improvements-and-longstrip

## 1. Backend

- [x] 1.1 Adicionar `idioma` e `capa_url` em `backend/source-integration/` (`fonte.rs`, `lib.rs`, `busca.rs`, `mangadex.rs`)
- [x] 1.2 Adicionar `excluir_capitulo` em `backend/database/src/operations.rs` e comandos `library_excluir_capitulo`, `reader_marcar_lido`, `reader_listar_lidos` em `backend/interface/` e `backend/src-tauri/`

## 2. Frontend

- [x] 2.1 Atualizar `backend/frontend/src/api.ts`, `App.tsx` e `CbzReader.tsx` com todas as melhorias de UX e modo `Long strip` contínuo

## 3. Validação e Empacotamento

- [x] 3.1 Executar `npm run build`, `cargo test --workspace`, `openspec validate`, arquivar, push e regerar `dist/MAM-windows-x86_64.zip`
