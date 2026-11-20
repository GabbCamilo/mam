# Proposal — implement-ai-part2-reader-overlay

## Why

Com a Parte 1 do Manga AI Engine (`process_page` e `OnnxSessionAdapter`) concluída, a **Parte 2** conecta o processamento de IA diretamente ao leitor CBZ (`CbzReader`), permitindo processar a página atual sob demanda sem alterar o `.cbz` original, persistir o resultado em `resultados_ia` (`nao_revisado`) e exibir a camada visual de sobreposição por coordenadas normalizadas (`bbox`).

## What Changes

- `backend/ai-engine/src/lib.rs`: adicionar `AiEngineManager::processar_pagina` chamando `process_page` via JSON-RPC stdio.
- `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs`: implementar e registrar `ai_processar_e_salvar_pagina` (extrai página temporariamente em `temp_dir`, executa `process_page`, remove temporário, salva em `resultados_ia` no SQLite).
- `backend/frontend/src/api.ts` e `backend/frontend/src/components/CbzReader.tsx`: adicionar botão **Processar Página (IA)** e camada de **Sobreposição Visual** sobre a imagem da página usando `bbox` normalizado (`[x, y, w, h]`).

## Capabilities

### New Capabilities

- `ai-part2-reader-overlay`: processamento sob demanda de páginas CBZ pelo AI Engine, persistência em `resultados_ia` e renderização visual de sobreposição no leitor.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos modificados: `backend/ai-engine/`, `backend/interface/`, `backend/src-tauri/`, `backend/frontend/src/`.
- Arquivos `.cbz` originais permanecem intocados.
