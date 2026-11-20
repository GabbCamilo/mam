# Design — implement-ai-part3-review-and-cache

## Context

Parte 3 de 3 da evolução do Manga AI Engine.

## Decisions

1. **Chave SHA-256 composta**: `SHA-256(bytes_imagem || "|" || modelo || "|" || versao_pipeline || "|" || idioma_origem || "|" || idioma_destino)` armazenada em `configuracao_biblioteca` sob `ai:cache:<hex>`.
2. **Editor por região no `CbzReader.tsx`**: Mantém compatibilidade com resultados que possuem `regioes: [...]` e também preserva fallback de edição JSON bruto quando `regioes` estiver vazio.
