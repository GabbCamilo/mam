# Tasks — implement-covers-sort-and-download-log

## 1. Backend

- [x] 1.1 Adicionar `contar_capitulos_por_obra` e `listar_historico_downloads` em `backend/database/src/operations.rs`
- [x] 1.2 Adicionar `baixar_imagem_data_url`, `salvar_capa_local` e registro em `historico_operacoes` em `backend/source-integration/`
- [x] 1.3 Expor `source_obter_capa`, `library_listar_obras_resumo` e `source_listar_historico_downloads` em `backend/interface/` e `backend/src-tauri/`

## 2. Frontend

- [x] 2.1 Atualizar `backend/frontend/src/api.ts` e `backend/frontend/src/App.tsx` (capas via backend, ordenação da Biblioteca por nome/qtd. capítulos e aba Log de Downloads)

## 3. Validação e Empacotamento

- [x] 3.1 Executar `npm run build`, `cargo test --workspace`, `openspec validate`, arquivar, push e regerar `dist/MAM-windows-x86_64.zip`
