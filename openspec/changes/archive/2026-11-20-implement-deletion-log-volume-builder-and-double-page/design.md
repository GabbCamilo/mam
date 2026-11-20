# Design Técnico

1. **Database (`backend/database/src/operations.rs`)**:
   - `excluir_obra`, `atualizar_volume`, `vincular_capitulos_volume`, `renomear_capitulo`, `listar_historico_exclusoes`.
2. **Interface & Tauri (`backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs`)**:
   - Registrar `tipo = "exclusao"` em `library_excluir_capitulo` e `library_excluir_obra`.
   - Expor `library_excluir_obra`, `library_criar_volume_com_capitulos`, `library_renomear_volume`, `library_renomear_capitulo`, `library_listar_historico_exclusoes`.
3. **Frontend (`api.ts`, `App.tsx`, `CbzReader.tsx`)**:
   - `CbzReader.tsx`: container grid `1fr 1fr` para modo `dupla` (50%/50% com `objectFit: 'contain'`).
   - `App.tsx`: botão excluir obra, renomear capítulo/volume, construtor de volume com seleção de capítulos, e sub-aba `Log de Exclusões` dentro da aba de Logs.
