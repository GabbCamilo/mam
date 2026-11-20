# Proposal — redesign-frontend-dark-ui

## Why

A interface React atual (`backend/frontend/src/`) usa estilos inline básicos sem identidade visual unificada. O usuário solicitou uma reformulação completa inspirada no visual escuro moderno de gerenciadores desktop (barra superior com ações rápidas e tema Dark/Light, lista em cards/linhas com bordas sutis, toggle `Capítulos | Volumes` com busca e ordenação, modais flutuantes no leitor para `Layout` e seleção de `Capítulos`, e barra de status inferior).

## What Changes

- `backend/frontend/src/styles.css` e `backend/frontend/src/main.tsx`: sistema de design com variáveis CSS (tema Dark padrão + alternância Light), botões pill, cards com borda sutil, modais flutuantes e barra de status.
- `backend/frontend/src/App.tsx`: novo layout desktop com cabeçalho de marca, botão `+ Importar Link`, abas em pílula (`Biblioteca`, `Fontes & Importação`, `Transferência & Sync`, `AI Engine`), busca/ordenação de obras, toggle `Capítulos | Volumes` e rodapé de status.
- `backend/frontend/src/components/CbzReader.tsx`: leitor em tela escura com modal flutuante de configurações (`Layout`: Single / Double / Long strip, Zoom, Ajuste) e modal flutuante de navegação de `Capítulos | Volumes`, mantendo favoritos, marcadores, sobreposição visual e revisão de IA.

## Capabilities

### New Capabilities

- `frontend-ui-redesign`: interface desktop dark-first com tema alternável, listagem estruturada de capítulos/volumes e modais flutuantes no leitor CBZ.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados/modificados: `backend/frontend/src/styles.css`, `backend/frontend/src/main.tsx`, `backend/frontend/src/App.tsx`, `backend/frontend/src/components/CbzReader.tsx`.
- Sem alterações de contratos Tauri ou dependências npm externas.
