# Proposta: Seletor de Idioma na Barra Superior (PT-BR / EN)

## Por quê

Atualmente toda a interface desktop do MAM (`App.tsx`, `CbzReader.tsx`, `ThemeModal.tsx`, `ConfirmModal.tsx` e `themes.ts`) possui textos hardcoded em português brasileiro (`pt-BR`). Usuários que preferem utilizar o aplicativo em inglês (`en`) não dispõem de um seletor de idioma na interface.

## O quê

Adicionar um controle de seleção/alternância de idioma (`PT` / `EN`) na barra superior (`mam-topbar`), ao lado dos controles de aparência/tema, capaz de converter os textos da interface do projeto entre português (`pt-BR`) e inglês (`en`), com persistência em `localStorage` e atualização imediata sem recarregar a aplicação.

## Impacto

- **Frontend (`backend/frontend/src/`)**:
  - Novo módulo de internacionalização leve (`i18n.ts`) com dicionário tipado `pt-BR` / `en`, helper de tradução e persistência em `localStorage` (`mam:idioma_v1`).
  - Atualização de `App.tsx` (barra superior, barra de navegação, abas `Biblioteca`, `Fontes & Importação`, `Log de Downloads`, `Transferência & Sync`, `AI Engine`, modais e mensagens de status/erro da UI).
  - Atualização de `CbzReader.tsx`, `ThemeModal.tsx`, `ConfirmModal.tsx` e rótulos de `themes.ts` para consumir o idioma ativo.
- **Backend Rust / AI Engine Python**: fora de escopo nesta mudança (mensagens de erro internas vindas do backend permanecem inalteradas ou são exibidas como fallback).

## Capacidades

- `interface-language-switcher`: Seletor de idioma na barra superior (`mam-topbar`) com alternância imediata entre `pt-BR` e `en`, persistência local e tradução completa da interface frontend.
