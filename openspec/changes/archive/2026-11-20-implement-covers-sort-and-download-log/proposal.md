# Proposal — implement-covers-sort-and-download-log

## Why

O carregamento direto de capas externas (`uploads.mangadex.org`) no WebView falha por políticas de `Referer`/CSP, sendo necessário buscar as imagens via backend Rust (`reqwest`) e salvá-las localmente ao baixar a obra. Além disso, a Biblioteca precisa exibir automaticamente as obras com miniatura de capa, contagem de capítulos e ordenação por nome ou quantidade de capítulos, e o sistema precisa registrar e exibir um painel de Log de Downloads (nome, pasta de destino e hora).

## What Changes

- `backend/database/src/operations.rs`: adicionar `contar_capitulos_por_obra` e `listar_historico_downloads`.
- `backend/source-integration/`: adicionar `baixar_imagem_data_url`, salvamento de `cover.jpg` na pasta da obra e registro de `tipo = 'download'` em `historico_operacoes` ao arquivar capítulos.
- `backend/interface/` e `backend/src-tauri/`: expor `source_obter_capa`, `library_listar_obras_resumo` e `source_listar_historico_downloads`.
- `backend/frontend/src/`: atualizar `api.ts` e `App.tsx` com carregamento de capas via backend, ordenação da Biblioteca (`Nome` / `Capítulos`) e aba **Downloads (Log)**.

## Capabilities

### New Capabilities

- `covers-sort-and-download-log`: proxy/armazenamento local de capas, listagem resumida com ordenação na Biblioteca e histórico visual de downloads.
