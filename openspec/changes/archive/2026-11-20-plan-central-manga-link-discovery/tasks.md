# Tarefas de Implementação (Planejadas)

- [x] Reformular o modal `+ Importar por Link / HTML` em `App.tsx` com 2 abas (`Página Central da Obra` e `Capítulo Único`)
- [x] Expandir `extractors/mod.rs` e `mangafire.rs` com analisador estrutural de blocos repetitivos, atributos `data-*`, JSON embutido e suporte dedicado ao padrão `mangalivre.to/manga/.../capitulo-.../`
- [x] Adicionar método opcional de classificação estrutural de links no `ai-engine` (fallback para layouts desconhecidos)
- [x] Garantir extração completa de páginas por capítulo descoberto (`baixar_capitulo_cbz_por_url`) integrada à fila assíncrona, deduplicação SHA-256 e `Log de Downloads`
- [x] Criar testes unitários cobrindo MangaFire, MangaLivre e agrupamento genérico, validar `openspec validate` e arquivar a change
