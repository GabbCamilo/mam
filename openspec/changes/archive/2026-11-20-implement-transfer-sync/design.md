# Design — implement-transfer-sync

## Context

`mam-storage` já fornece verificação de espaço (`StorageManager::verificar_espaco`) e identificação de bibliotecas. `mam-transfer` depende de `mam-core`, `mam-storage`, `sha2`, `walkdir`, `tokio`, `serde`, `thiserror` e `uuid`.

## Goals / Non-Goals

**Goals:**
- Implementar cópia integral e seletiva com escrita atômica (`*.mam_tmp` + `rename`) e validação SHA-256.
- Garantir invariante absoluta: nenhuma função de `mam-transfer` remove arquivos da origem.
- Implementar comparação (`DiferencaBibliotecas`) e sincronização (`sincronizar_bibliotecas`) com controle explícito de sobrescrita.
- Gerar `RelatorioTransferencia` detalhado e expor comandos Tauri.

**Non-Goals:**
- Sincronização via rede/nuvem (apenas armazenamento local/SSD externo).
- Resolução interativa de conflitos na UI (os conflitos são retornados no relatório estruturado).

## Decisions

1. **Escrita atômica no destino**: Cada arquivo é copiado para um arquivo temporário no diretório de destino, validado via SHA-256 contra a origem e só então renomeado para o caminho final.
2. **Normalização de caminhos relativos**: Utilizar `/` como separador lógico nas comparações entre bibliotecas para portabilidade entre Windows e Linux.
3. **Sem estado global mutável obrigatório**: `TransferManager` pode ser instanciado sem estado acoplado ao banco, recebendo caminhos de origem e destino diretamente.

## Risks / Trade-offs

- [Arquivos grandes em SSD] Leitura em streaming (`BufReader` / buffer de 64 KB) para cálculo de SHA-256 sem carregar arquivos grandes inteiros na memória.
