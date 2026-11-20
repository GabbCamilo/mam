# Design Técnico

1. **Source Integration (`backend/source-integration/src/lib.rs`)**:
   - Extrair `executar_arquivamento_operacao(&self, id_operacao: &str, raiz_biblioteca: &Path, obra_titulo: String, capitulos: Vec<CapituloBuscaResultado>, capa_url: Option<String>)` a partir de `arquivar_para_biblioteca_com_capa`.
   - Manter `arquivar_para_biblioteca` e `arquivar_para_biblioteca_com_capa` síncronos para compatibilidade com testes.
2. **Interface & Tauri (`backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs`)**:
   - Adicionar `source_iniciar_arquivamento` (registra operação, dispara `tokio::spawn` chamando `executar_arquivamento_operacao`, retorna `op_id` imediatamente).
   - Adicionar `source_listar_operacoes` expondo `SourceIntegrationManager::listar_operacoes`.
3. **Frontend (`backend/frontend/src/api.ts`, `App.tsx`)**:
   - Adicionar `sourceIniciarArquivamento` e `sourceListarOperacoes`.
   - Estado `popupDownload` (exibido por ~6 segundos ao iniciar lote; clicável para abrir aba `downloads`) e polling silencioso enquanto houver operações não concluídas, atualizando `operacoesAtivas` e `downloadsLog` (incrementando `Log de Downloads (N)` automaticamente a cada capítulo salvo).
