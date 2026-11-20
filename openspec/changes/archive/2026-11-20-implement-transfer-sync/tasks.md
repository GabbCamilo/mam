# Tasks — implement-transfer-sync

## 1. Estrutura do crate mam-transfer

- [x] 1.1 Implementar `TransferError`, `RelatorioTransferencia` e `TransferManager` em `backend/transfer/src/lib.rs`
- [x] 1.2 Criar `backend/transfer/src/copia.rs` (cópia atômica com SHA-256, cópia integral e transferência seletiva)
- [x] 1.3 Criar `backend/transfer/src/sincronizacao.rs` (comparação de bibliotecas e sincronização sem sobrescrita inesperada)

## 2. Comandos Tauri em mam-interface e src-tauri

- [x] 2.1 Atualizar `backend/interface/Cargo.toml` e `backend/interface/src/lib.rs` com `transfer_copia_integral`, `transfer_seletiva`, `sync_comparar_bibliotecas`, `sync_sincronizar_bibliotecas`
- [x] 2.2 Registrar `TransferManager` e os 4 comandos Tauri em `backend/src-tauri/Cargo.toml` e `backend/src-tauri/src/lib.rs`

## 3. Validação e testes

- [x] 3.1 Criar testes unitários e de integração em `mam-transfer` (cópia integral, transferência seletiva, preservação da origem, comparação e bloqueio de sobrescrita)
- [x] 3.2 Executar `cargo test --workspace` e `openspec validate implement-transfer-sync` confirmando sucesso sem erros
