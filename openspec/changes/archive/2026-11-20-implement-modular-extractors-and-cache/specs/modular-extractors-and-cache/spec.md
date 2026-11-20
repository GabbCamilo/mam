# Specification — modular-extractors-and-cache

## ADDED Requirements

### Requirement: Arquitetura modular de extratores por site
The system SHALL provide a pluggable `ExtratorSite` trait and `RegistroExtratores` supporting `mangadex`, `mangafire`, and a generic fallback (`generico`), allowing new sites to be added without altering the core importer.

#### Scenario: Selecionar extrator por URL e extrair páginas ordenadas
- **WHEN** `RegistroExtratores` recebe uma URL do `mangafire.to` com HTML ou lista ordenada de URLs
- **THEN** o sistema SHALL selecionar o `MangaFireExtractor`, extrair as URLs de imagens na ordem exata apresentada e preservar a sequência ao gerar o `.cbz`.

### Requirement: Cache L1 LRU integrado ao leitor CBZ
The system SHALL consult `CacheL1Lru` in `reader_obter_pagina` before reading from the `.cbz` archive and store decoded pages in memory up to the configured capacity.

#### Scenario: Cache hit na segunda leitura da mesma página
- **WHEN** `reader_obter_pagina` é chamado duas vezes para a mesma página e caminho
- **THEN** a segunda chamada SHALL retornar os dados diretamente do cache L1 em memória sem reabrir o arquivo `.cbz`.
