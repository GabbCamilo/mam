# Tasks: Universal Link/HTML Import v2

- [x] Criar proposta e design OpenSpec (`universal-link-import-v2`)
- [x] Adicionar `headers_imagem` e `estrategia_descoberta` em `ConectorSiteConfig` (`connectors.rs`) e atualizar `connectors/default_sites.json`
- [x] Aplicar `Referer` por domínio em `baixar_imagem_data_url`, `salvar_capa_local` e `RegistroExtratores`
- [x] Melhorar `extrair_urls_scripts` em `extractors/mod.rs` para JSON escapado (`\/`, `\u002F`)
- [x] Implementar interceptação `Network.enable` / `Network.getResponseBody` em `ai-engine/mam_ai_engine/web_reader.py`
- [x] Adicionar botão auxiliar de fluxo assistido no modal de importação por link/HTML (`App.tsx`)
- [x] Validar com testes Rust e Python e recompilar release
