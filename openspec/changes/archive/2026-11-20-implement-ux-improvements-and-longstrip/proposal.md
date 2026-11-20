# Proposal — implement-ux-improvements-and-longstrip

## Why

Após uso real do aplicativo, o usuário identificou melhorias essenciais de usabilidade: deseleção de obra ao clicar novamente, botão de selecionar todos os capítulos, badge de idioma (`ENG`, `PT`, `JPN`) e miniatura de capa no MangaDex e na Biblioteca, exclusão de capítulos arquivados, marcação de `Lido` / `Não lido` por capítulo na Biblioteca, rolagem vertical contínua real no modo `Long strip` do `CbzReader`, e modal flutuante para `+ Importar URL` para não ser empurrado para o fim da página.

## What Changes

- `backend/source-integration/`: adicionar `capa_url` em `ObraEncontrada`/`ObraBuscaResultado` (via `includes[]=cover_art` no MangaDex) e `idioma` em `CapituloEncontrado`/`CapituloBuscaResultado`.
- `backend/database/`, `backend/interface/`, `backend/src-tauri/`: adicionar `excluir_capitulo` (`library_excluir_capitulo`) e `reader_marcar_lido` / `reader_listar_lidos`.
- `backend/frontend/src/`:
  - `App.tsx`: toggle de deseleção na busca de fontes, botão `Selecionar Todas`, badges de idioma (`ENG`/`PT`/`JPN`), miniatura da capa da obra, botão de excluir capítulo na Biblioteca, badge clicável `Lido`/`Não lido` por capítulo, e modal flutuante para `+ Importar URL`.
  - `CbzReader.tsx`: renderização contínua de todas as páginas no modo `Long strip` (rolagem infinita vertical) mantendo `Single` e `Double` por clique/teclado, além de marcar capítulo como lido ao chegar à última página.

## Capabilities

### New Capabilities

- `ux-improvements`: melhorias de UX na Biblioteca, Fontes & Importação, exclusão de capítulos, status Lido/Não lido, Long strip contínuo e modal `+ Importar URL`.
