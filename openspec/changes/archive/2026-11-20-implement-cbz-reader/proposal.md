# Proposal — implement-cbz-reader

## Why

Com o catálogo da biblioteca (`mam-library`) e a leitura de estrutura CBZ (`mam-archive`) implementados, o MAM precisa do **Leitor CBZ integrado** e da **Interface Desktop** funcional no frontend React + TypeScript para permitir abrir capítulos `.cbz` sob demanda sem extração permanente, navegar entre páginas e capítulos, controlar zoom/modos de leitura e persistir histórico, favoritos e marcadores.

## What Changes

- `backend/archive/src/leitura.rs` e `backend/archive/src/lib.rs`: adicionar leitura de bytes de página individual sob demanda (`ler_pagina_cbz`) sem extração permanente em disco.
- `backend/database/src/operations.rs`: adicionar operações de estado do leitor (salvar/consultar última página lida, alternar favorito, gerenciar marcadores) usando `configuracao_biblioteca` / persistência SQLite local.
- `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs`: expor comandos Tauri do leitor (`reader_obter_pagina`, `reader_salvar_progresso`, `reader_obter_progresso`, `reader_alternar_favorito`, `reader_listar_marcadores`, `reader_adicionar_marcador`).
- `backend/frontend/src/`: implementar interface da biblioteca (catálogo de obras, detalhes híbridos de volumes/capítulos) e componente **CBZ Reader** (navegação página a página e entre capítulos consecutivos, modos vertical, horizontal e dupla página, zoom, ajuste à tela, favoritos e marcadores).

## Capabilities

### New Capabilities

- `cbz-reader`: leitura sob demanda de páginas CBZ sem extração permanente, controles de visualização (vertical, horizontal, dupla página, zoom, ajuste à tela), navegação entre capítulos e persistência de histórico, favoritos e marcadores.

### Modified Capabilities

- (nenhuma — integra-se aos comandos de `library-management` e `archive-cbz`)

## Impact

- Arquivos criados/modificados: `backend/archive/`, `backend/database/`, `backend/interface/`, `backend/src-tauri/`, `backend/frontend/src/`.
- Os arquivos `.cbz` originais nunca são modificados ou extraídos permanentemente.
- Nenhuma dependência npm ou cargo externa é necessária além das já instaladas.
