# Proposal — implement-web-source-import

## Why

O módulo `mam-source-integration` possui o contrato `Fonte` e comandos Tauri básicos, porém ainda não tem uma fonte pública autorizada concreta registrada na inicialização, não conclui o pipeline de gravação do `.cbz` + registro no banco durante o arquivamento, e não está conectado ao frontend React (`App.tsx`).

## What Changes

- `backend/source-integration/src/mangadex.rs`: implementar adaptador autorizado para a API pública do MangaDex (`https://api.mangadex.org`, sem DRM, paywall ou contorno de autenticação) implementando o trait `Fonte`.
- `backend/source-integration/src/lib.rs`: implementar `arquivar_para_biblioteca` gravando o `.cbz` de forma atômica na raiz da biblioteca, registrando integridade SHA-256 e catálogo no `Database`.
- `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs`: registrar a fonte autorizada na inicialização, conectar `source_arquivar_capitulos` ao arquivamento real na biblioteca e expor `source_status_operacao`.
- `backend/frontend/src/api.ts` e `backend/frontend/src/App.tsx`: adicionar aba **Fontes & Importação** com listagem de fontes, pesquisa de obras, listagem/seleção de capítulos, verificação de duplicatas e acompanhamento de operações.

## Capabilities

### New Capabilities

- `web-source-import`: pesquisa e importação ponta a ponta de conteúdos de fontes públicas autorizadas para arquivos `.cbz` na biblioteca portátil, integradas ao frontend React.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados/modificados: `backend/source-integration/`, `backend/interface/`, `backend/src-tauri/`, `backend/frontend/src/`.
- Respeita estritamente a proibição de contornar DRM, paywalls ou autenticação.
