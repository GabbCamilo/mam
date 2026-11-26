# Design: Seletor de Idioma na Barra Superior (`interface-language-switcher`)

## Contexto

O frontend do MAM (`backend/frontend/src/`) utiliza React 19 + TypeScript 5.7 + Vite 6 sem biblioteca externa de i18n. A barra superior (`mam-topbar` em `App.tsx`) já contém controles rápidos de aparência (`🎨 Temas` e `🌙 Dark` / `☀️ Light`) persistidos em `localStorage` via `themes.ts`.

## Goals / Non-Goals

- **Goals**:
  - Controle de idioma acessível e visível na `mam-topbar` (`🌐 PT / EN` ou seletor compacto) ao lado dos botões de tema.
  - Solução de i18n leve, zero-dependência externa e totalmente tipada em TypeScript (`src/i18n.ts`), evitando inchar o bundle ou quebrar o build offline/portátil.
  - Tradução consistente em `App.tsx`, `CbzReader.tsx`, `ThemeModal.tsx`, `ConfirmModal.tsx` e `themes.ts`.
  - Persistência automática em `localStorage` (`mam:idioma_v1`) e sincronização de `document.documentElement.lang`.
- **Non-Goals**:
  - Não alterar contratos Tauri, comandos Rust ou respostas JSON-RPC do `ai-engine`.
  - Não traduzir dados dinâmicos armazenados pelo usuário no SQLite (títulos de obras, nomes de arquivos `.cbz`, logs técnicos crus do backend).

## Decisões Técnicas

1. **Módulo `backend/frontend/src/i18n.ts`**:
   - Tipo `IdiomaUi = 'pt-BR' | 'en'`.
   - Funções `carregarIdioma()`, `salvarIdioma(idioma)` e `aplicarIdiomaNoDocumento(idioma)`.
   - Dicionário estruturado ou tabela chave-valor tipada com fallback seguro para `pt-BR` caso alguma chave falte.
   - Suporte a interpolação simples de parâmetros (ex.: `{count}`, `{titulo}`) para strings dinâmicas.
2. **Prop drilling / Contexto React**:
   - Criar um `LanguageContext` (ou `useI18n()`) leve em `i18n.ts` / `main.tsx` ou passar `idioma` e `t` para os componentes filhos (`CbzReader`, `ThemeModal`, `ConfirmModal`). Usar React Context (`I18nProvider` + `useI18n`) evita alterar excessivamente assinaturas de props e permite que qualquer componente leia e altere o idioma reativamente.
3. **Posicionamento na `mam-topbar`**:
   - Inserir o botão/seletor de idioma dentro de `<div className="mam-toolbar-row">` em `App.tsx`, mantendo a identidade visual `.mam-btn`.

## Riscos e Mitigações

- **Strings extensas em `App.tsx` (3.349 linhas) e `CbzReader.tsx` (928 linhas)**:
  - *Mitigação*: mapear e substituir sistematicamente por grupos (topbar/nav, biblioteca, fontes, downloads, transferência, AI Engine, modais, leitor CBZ) e validar com `tsc --noEmit && vite build` para garantir que nenhuma chave fique sem tipo ou quebrada.
- **Overflow visual na `mam-topbar`**:
  - *Mitigação*: usar rótulo compacto (`🌐 PT` / `🌐 EN`) compatível com a responsividade já existente de `.mam-toolbar-row`.
