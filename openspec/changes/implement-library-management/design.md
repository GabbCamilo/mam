# Design — implement-library-management

## Context

`mam-database` já fornece os modelos `Obra`, `Volume`, `Capitulo` e operações básicas no SQLite, enquanto `mam-core` define `ObraId`, `VolumeId`, `CapituloId`, `CaminhoRelativo` e `EstadoIntegridade`. `mam-library` atua como camada de domínio acima do `mam-database` para coordenar regras de catálogo, organização híbrida de volumes e metadados, sendo consumido por `mam-interface` e `src-tauri`.

## Goals / Non-Goals

**Goals:**
- Implementar `LibraryManager` em `mam-library` com operações assíncronas sobre `Arc<Database>`.
- Garantir que todos os caminhos manipulados sejam relativos à raiz da biblioteca.
- Suportar organização híbrida (volumes + capítulos avulsos).
- Validar e persistir `metadados_complementares` em JSON.
- Expor comandos Tauri via `mam-interface` e `src-tauri`.

**Non-Goals:**
- Implementar interface React completa de biblioteca (será tratada em Change específico de UI/Reader).
- Implementar transferência/sincronização entre bibliotecas (`mam-transfer`).
- Modificar arquivos `.cbz` durante operações de catálogo ou metadados.

## Decisions

1. **Composição sobre `Arc<Database>`**: `LibraryManager` recebe `Arc<Database>` e `PathBuf` (raiz da biblioteca), mantendo consistência com `ChapterManager`, `ArchiveManager` e `SourceIntegrationManager`.
2. **Validação estrita de caminhos relativos**: Rejeitar caminhos absolutos ou com `..` antes de delegar ao `mam-database`.
3. **Erros tipados com `thiserror`**: Definir `LibraryError` em `mam-library` para falhas de validação, obra/volume não encontrados, JSON inválido e erros de banco.

## Risks / Trade-offs

- [Concorrência no SQLite] Múltiplas operações simultâneas no catálogo devem continuar usando transações curtas via `mam-database`.
- [Raiz dinâmica em SSD externo] A raiz da biblioteca deve ser resolvida em tempo de execução sem gravar letras de unidade no banco.
