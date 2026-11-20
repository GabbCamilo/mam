# Tasks — implement-cbz-reader

## 1. Leitura sob demanda no mam-archive

- [x] 1.1 Adicionar `PaginaExtraida` e `ler_pagina_cbz` em `backend/archive/src/leitura.rs` e expor em `ArchiveManager` (`backend/archive/src/lib.rs`)

## 2. Persistência de progresso, favoritos e marcadores no mam-database

- [x] 2.1 Adicionar operações de configuração chave-valor (`salvar_configuracao`, `obter_configuracao`, `listar_configuracoes_por_prefixo`) em `backend/database/src/operations.rs`

## 3. Comandos Tauri do leitor em mam-interface e src-tauri

- [x] 3.1 Implementar funções de negócio `reader_obter_pagina`, `reader_salvar_progresso`, `reader_obter_progresso`, `reader_alternar_favorito`, `reader_adicionar_marcador`, `reader_listar_marcadores` em `backend/interface/src/lib.rs`
- [x] 3.2 Registrar comandos Tauri no `invoke_handler` em `backend/src-tauri/src/lib.rs`

## 4. Interface Desktop e Leitor CBZ no frontend (React + TypeScript)

- [x] 4.1 Criar tipos e cliente de comandos Tauri em `backend/frontend/src/api.ts`
- [x] 4.2 Criar componente `CbzReader` (`backend/frontend/src/components/CbzReader.tsx`) com modos vertical, horizontal, dupla página, zoom, ajuste à tela, favoritos, marcadores e navegação entre capítulos
- [x] 4.3 Atualizar `backend/frontend/src/App.tsx` com visão de biblioteca (obras, volumes, capítulos híbridos) integrada ao `CbzReader`

## 5. Validação e testes

- [x] 5.1 Criar testes unitários em `mam-archive` para `ler_pagina_cbz` (página válida e índice fora do limite)
- [x] 5.2 Criar testes em `mam-database` para persistência de progresso, favoritos e marcadores
- [x] 5.3 Executar `cargo test --workspace` e `npm run build` em `backend/frontend` e confirmar sucesso sem erros
