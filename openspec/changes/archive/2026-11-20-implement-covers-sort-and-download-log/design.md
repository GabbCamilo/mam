# Design — implement-covers-sort-and-download-log

## Decisions

1. **Proxy Base64 via `reqwest`**: Evita bloqueio de `Referer` / CSP no WebView2 do Windows ao carregar capas de `uploads.mangadex.org`.
2. **Persistência de `cover.jpg` local**: Ao arquivar capítulos de uma obra com `capa_url`, grava `cover.jpg` em `Manga/<Obra>/cover.jpg` para exibição offline em destaque na Biblioteca.
3. **Uso da tabela `historico_operacoes`**: Reaproveita a tabela já existente em `001_initial.sql` (`tipo = 'download'`) sem alterar o schema SQLite.
