# interface-language-switcher Specification

## Purpose
Definir o seletor de idioma (`pt-BR` / `en`) na barra superior do MAM e o comportamento de internacionalização da interface desktop.

## ADDED Requirements

### Requirement: Seletor de idioma na barra superior
The frontend top bar (`mam-topbar`) SHALL provide a language toggle/selector control allowing the user to switch the UI language between Portuguese (`pt-BR`) and English (`en`).

#### Scenario: Alternar idioma para inglês pela barra superior
- **WHEN** o usuário aciona o seletor de idioma na barra superior (`mam-topbar`) selecionando `EN`
- **THEN** o idioma ativo SHALL mudar imediatamente para `en` sem recarregar a aplicação, atualizando `document.documentElement.lang` e os textos da interface.

#### Scenario: Alternar idioma de volta para português
- **WHEN** o usuário aciona o seletor de idioma selecionando `PT`
- **THEN** o idioma ativo SHALL retornar para `pt-BR` imediatamente.

### Requirement: Persistência da preferência de idioma
The frontend SHALL persist the selected language (`pt-BR` or `en`) in `localStorage` under a dedicated key (`mam:idioma_v1`) and restore it on startup, defaulting to `pt-BR` when absent or invalid.

#### Scenario: Restaurar idioma salvo ao reabrir o aplicativo
- **WHEN** o usuário seleciona `en`, fecha e reabre a aplicação
- **THEN** a interface SHALL iniciar automaticamente em `en`.

### Requirement: Cobertura de tradução nos componentes principais da UI
All user-facing labels, navigation tabs, section headers, buttons, placeholders, tooltips, and modals in `App.tsx`, `CbzReader.tsx`, `ThemeModal.tsx`, `ConfirmModal.tsx`, and `themes.ts` SHALL render in the active language (`pt-BR` or `en`).

#### Scenario: Navegar pelas abas e abrir modais em inglês
- **WHEN** o idioma ativo é `en` e o usuário navega entre `Library`, `Sources & Import`, `Download Log`, `Transfer & Sync`, `AI Engine` ou abre `CbzReader`, `ThemeModal` e `ConfirmModal`
- **THEN** todos os títulos, rótulos e ações visíveis SHALL ser exibidos em inglês sem quebrar o layout nem perder estado da sessão.
