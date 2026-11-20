# mvp-pendencies Specification

## Purpose
TBD - created by archiving change resolve-mvp-pendencies. Update Purpose after archive.

## Requirements

### Requirement: Espaço real em disco, detecção de unidade e Cache L1 LRU
The system SHALL query real available disk space for target paths, detect whether a storage root is available before writing, and provide an in-memory LRU cache (`CacheL1Lru`) for decoded pages.

#### Scenario: Consultar espaço real e disponibilidade da unidade
- **WHEN** `StorageManager::espaco_disponivel` e `StorageManager::unidade_disponivel` são chamados para um diretório válido existente
- **THEN** o sistema SHALL retornar `disponivel = true` e uma quantidade real de bytes livres maior que zero (e menor que `u64::MAX / 2` em unidades normais).

#### Scenario: Cache L1 LRU respeita capacidade máxima
- **WHEN** itens são inseridos além da capacidade configurada no `CacheL1Lru`
- **THEN** o item menos recentemente usado MUST ser descartado mantendo o tamanho dentro do limite.

### Requirement: Persistência e revisão manual de resultados de IA
The system SHALL persist AI results in `resultados_ia` with initial state `nao_revisado` and allow manual review updating the state to `revisado` without altering original `.cbz` archives.

#### Scenario: Salvar e revisar resultado de IA no SQLite
- **WHEN** um resultado de IA é salvo e posteriormente revisado via `revisar_resultado_ia`
- **THEN** o registro no banco SHALL ter `estado_revisao = "revisado"` e preservar o texto revisado.
