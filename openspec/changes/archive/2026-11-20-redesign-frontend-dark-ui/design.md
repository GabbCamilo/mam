# Design — redesign-frontend-dark-ui

## Context

Reformulação visual pura em React + CSS (`backend/frontend/src/`) mantendo 100% dos contratos `mamApi` existentes.

## Decisions

1. **CSS puro com Custom Properties (`styles.css`)**: Zero dependências extras de UI, bundle pequeno e carregamento instantâneo no WebView2 do Tauri.
2. **Paleta inspirada nas referências**: Fundo `#0a0e17`, superfícies `#111827` / `#151e2e`, bordas `#1e293b` / `#1d4ed8`, destaque azul `#0ea5e9` / `#2563eb`.
