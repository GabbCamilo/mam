# Specification — ai-part3-review-and-cache

## ADDED Requirements

### Requirement: Cache determinístico de IA com invalidação por parâmetros
The system SHALL compute a deterministic AI cache key from the original page image SHA-256 hash, model identifier, source language, target language, and pipeline version, reusing cached results on hit and invalidating automatically when any parameter changes.

#### Scenario: Cache hit e invalidação por alteração de idioma
- **WHEN** `calcular_chave_cache_ia` é calculado para os mesmos bytes, modelo e idiomas
- **THEN** a chave SHALL ser idêntica; quando o idioma de destino ou os bytes da imagem mudarem, a chave SHALL ser diferente, invalidando o cache anterior.

### Requirement: Revisão manual por região no CbzReader
The frontend `CbzReader` SHALL allow editing recognized (`texto_original`) and translated (`texto_traduzido`) text per detected region and saving the updated result as `revisado`.

#### Scenario: Editar região no painel de revisão IA
- **WHEN** o usuário edita o texto traduzido de uma região e clica em salvar revisão
- **THEN** o sistema SHALL persistir o JSON atualizado via `ai_revisar_resultado` com `estado_revisao = "revisado"`.
