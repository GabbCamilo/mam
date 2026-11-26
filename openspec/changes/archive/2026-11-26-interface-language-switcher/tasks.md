# Tarefas: interface-language-switcher

- [x] 1. Criar módulo de internacionalização `backend/frontend/src/i18n.ts` com tipos (`IdiomaUi = 'pt-BR' | 'en'`), persistência (`mam:idioma_v1`), `I18nProvider`/`useI18n` e dicionário completo `pt-BR` e `en`.
- [x] 2. Adicionar o seletor de idioma (`PT` / `EN`) na barra superior (`mam-topbar`) em `backend/frontend/src/App.tsx` e envolver a aplicação no `I18nProvider`.
- [x] 3. Internacionalizar as abas e seções de `backend/frontend/src/App.tsx` (`Biblioteca`, `Fontes & Importação`, `Log de Downloads`, `Transferência & Sync`, `AI Engine`, barra inferior e modais).
- [x] 4. Internacionalizar os componentes auxiliares: `CbzReader.tsx`, `ThemeModal.tsx`, `ConfirmModal.tsx` e os rótulos de `themes.ts`.
- [x] 5. Verificar tipagem e build de produção com `npm run build` (`tsc --noEmit && vite build`) em `backend/frontend/` e validar que não há regressões visuais ou de compilação.
