# Design Técnico

1. **Backend (`source-integration/src/lib.rs`, `interface/src/lib.rs`)**:
   - Incluir `operacao_id`, `obra_id`, `obra_titulo`, `capitulo_id` e `caminho_relativo` no JSON salvo em `historico_operacoes` para cada capítulo baixado.
   - Estender `DownloadLogItem` em `interface/src/lib.rs` e `api.ts` com esses campos opcionais.
2. **Frontend (`App.tsx`)**:
   - Agrupar `operacoesDownload` e `downloadsLog` por `operacao_id` (e fallback para registros antigos sem `operacao_id`).
   - Cada grupo possui cabeçalho clicável (mostrando status, progresso `X/Y (%)` e ícone `▾`/`▸`) que expande ou recolhe os capítulos daquele lote.
   - Cada capítulo expandido possui botão **Ler** que carrega `obterDetalhesObra` e abre `CbzReader` diretamente no capítulo correspondente.
