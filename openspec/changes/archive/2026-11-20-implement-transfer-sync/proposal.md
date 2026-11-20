# Proposal — implement-transfer-sync

## Why

O MAM tem como objetivo central permitir uma biblioteca digital portátil armazenada em SSD externo, com capacidade de copiar integralmente a biblioteca, transferir obras/volumes/capítulos seletivamente e sincronizar duas bibliotecas em unidades diferentes com validação de integridade SHA-256 e sem jamais excluir arquivos da origem. O crate `mam-transfer` ainda é um esqueleto e precisa ser implementado.

## What Changes

- `backend/transfer/src/lib.rs`: implementar `TransferManager`, `TransferError` e `RelatorioTransferencia`.
- `backend/transfer/src/copia.rs`: cópia segura com verificação de espaço via `mam-storage`, escrita atômica (`temp` + `rename`) e validação SHA-256 pós-cópia; suporte a cópia integral e transferência seletiva por caminhos relativos.
- `backend/transfer/src/sincronizacao.rs`: comparação entre biblioteca origem e destino (`comparar_bibliotecas`), identificação de arquivos ausentes ou com hash diferente, e sincronização não-destrutiva (`sincronizar_bibliotecas`) preservando origem e evitando sobrescritas inesperadas quando `permitir_sobrescrita = false`.
- `backend/interface/Cargo.toml`, `backend/interface/src/lib.rs`, `backend/src-tauri/Cargo.toml`, `backend/src-tauri/src/lib.rs`: expor comandos Tauri `transfer_copia_integral`, `transfer_seletiva`, `sync_comparar_bibliotecas` e `sync_sincronizar_bibliotecas`.

## Capabilities

### New Capabilities

- `transfer-sync`: cópia integral de biblioteca, transferência seletiva (obras, volumes ou capítulos por caminho relativo), comparação de bibliotecas, sincronização controlada com validação SHA-256 e relatórios de operação.

### Modified Capabilities

- (nenhuma — utiliza `mam-storage` e `mam-core` existentes)

## Impact

- Arquivos criados/modificados: `backend/transfer/`, `backend/interface/`, `backend/src-tauri/`.
- Nenhum arquivo da origem é excluído automaticamente em qualquer operação de transferência ou sincronização.
