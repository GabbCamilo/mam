# source-integration Specification

## Purpose
TBD - created by archiving change smart-web-reader-cbz. Update Purpose after archive.

## Requirements

### Requirement: Descoberta Multi-Página e Multi-Idioma em Sites Dinâmicos
O extrator por link SHALL descobrir capítulos em todas as páginas de paginação disponíveis (sem limite artificial) e priorizar o idioma preferido (`pt-br` por padrão, suportando `en`, `ja` e `all`) utilizando o Leitor Web Inteligente (CDP) quando o HTML estático não expuser a lista completa.

#### Scenario: Obra paginada com seletor de idioma no DOM
- **WHEN** o usuário informa a URL de uma obra SPA paginada com idioma preferido `pt-br`
- **THEN** o controlador seleciona o idioma disponível correspondente, percorre as páginas `1..N` até o fim da paginação e retorna os capítulos únicos ordenados

### Requirement: Extração Integral de Páginas para CBZ
O pipeline de download por URL SHALL filtrar recursos não-imagem (como `.js`, `.css`, logos e ícones) e capturar todas as páginas reais do capítulo em leitores dinâmicos ou *single-page* antes de empacotar o `.cbz`.

#### Scenario: Leitor com lazy-loading ou modo página única
- **WHEN** um capítulo é baixado de uma página que inicialmente monta apenas 1 imagem
- **THEN** o extrator aciona o Leitor Web Inteligente para rolar/avançar páginas e retorna todas as URLs de imagens reais em ordem sequencial para o `.cbz`
