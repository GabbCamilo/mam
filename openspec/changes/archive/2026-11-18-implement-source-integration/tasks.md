# Tasks

## 1. Estrutura do crate mam-source-integration

- [x] 1.1 Adicionar `mam-source-integration` ao `backend/Cargo.toml` workspace members
- [x] 1.2 Criar `backend/source-integration/Cargo.toml` com dependências (reqwest, serde, serde_json, url, mam-core, mam-database, mam-archive, thiserror, tokio)
- [x] 1.3 Criar `backend/source-integration/src/lib.rs` (SourceIntegrationManager ponto de entrada)
- [x] 1.4 Criar `backend/source-integration/src/fonte.rs` (trait Fonte e gerenciamento de fontes)
- [x] 1.5 Criar `backend/source-integration/src/busca.rs` (pesquisa de obras)
- [x] 1.6 Criar `backend/source-integration/src/capitulos.rs` (listagem de capítulos)
- [x] 1.7 Criar `backend/source-integration/src/operacao.rs` (gerenciamento de operações com progresso)
- [x] 1.8 Criar `backend/source-integration/src/duplicatas.rs` (detecção de conteúdos existentes)

## 2. Comandos Tauri para integração com fontes

- [x] 2.1 Comando `source_listar_fontes` (listar fontes registradas)
- [x] 2.2 Comando `source_buscar_obras` (pesquisar obras por título)
- [x] 2.3 Comando `source_listar_capitulos` (listar capítulos de uma obra)
- [x] 2.4 Comando `source_arquivar_capitulos` (iniciar arquivamento de capítulos selecionados)
- [x] 2.5 Comando `source_verificar_duplicatas` (verificar se obra/capítulo já existe)
- [x] 2.6 Registrar comandos no `invoke_handler` do src-tauri

## 3. Validação e teste

- [x] 3.1 Criar teste: trait Fonte (implementação de fonte fictícia autorizada)
- [x] 3.2 Criar teste: detecção de duplicatas via banco
- [x] 3.3 Criar teste: gerenciamento de operações de arquivamento
- [x] 3.4 `cargo build` e `cargo test` passam sem erros
