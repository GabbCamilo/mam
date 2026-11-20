# Tasks — implement-library-management

## 1. Estrutura do crate mam-library

- [ ] 1.1 Atualizar `backend/library/Cargo.toml` com dependências (`mam-database`, `mam-storage`, `tokio`)
- [ ] 1.2 Implementar `LibraryError` e `LibraryManager` em `backend/library/src/lib.rs`
- [ ] 1.3 Criar `backend/library/src/catalogo.rs` (criação/listagem/detalhes de obras, volumes e capítulos híbridos)
- [ ] 1.4 Criar `backend/library/src/metadados.rs` (validação e atualização de metadados JSON)

## 2. Comandos Tauri para biblioteca

- [ ] 2.1 Adicionar funções de negócio em `backend/interface/src/lib.rs` (`library_criar_obra`, `library_listar_obras`, `library_obter_detalhes_obra`, `library_criar_volume`, `library_atualizar_metadados`)
- [ ] 2.2 Registrar `LibraryManager` no estado gerenciado e comandos no `invoke_handler` em `backend/src-tauri/src/lib.rs`

## 3. Validação e testes

- [ ] 3.1 Criar testes unitários para validação de caminhos relativos e organização híbrida de volumes/capítulos
- [ ] 3.2 Criar testes de integração com `Database` em diretório temporário (CRUD de obra, volume e metadados)
- [ ] 3.3 Executar `cargo test --workspace` e confirmar que todos os testes passam sem erros
