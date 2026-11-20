# Tarefas

- [x] Adicionar migração SQL `002_capitulos_idioma.sql` e campo `idioma` em `Capitulo` (`database`)
- [x] Atualizar `source-integration` para separar diretórios por idioma e persistir `idioma` ao arquivar
- [x] Atualizar `api.ts` e `App.tsx` com filtro `LANG` na fonte, filtro/espaçamento por diretório na biblioteca e navegação do leitor restrita ao idioma
- [x] Validar `npm run build`, `cargo test --workspace` e `openspec validate`
