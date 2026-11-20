# Specification — transfer-sync

## ADDED Requirements

### Requirement: Cópia integral e transferência seletiva com integridade SHA-256
The system SHALL support full library copy and selective transfer of specified relative paths from a source library to a destination directory, validating SHA-256 integrity after copying and NEVER deleting source files.

#### Scenario: Cópia integral preserva origem e valida SHA-256
- **WHEN** uma cópia integral é executada de uma biblioteca origem para um destino
- **THEN** todos os arquivos da origem MUST ser copiados preservando seus caminhos relativos, cada arquivo copiado MUST ter seu hash SHA-256 comparado ao da origem, e todos os arquivos na origem MUST permanecer intactos.

#### Scenario: Transferência seletiva copia apenas os caminhos solicitados
- **WHEN** uma lista de caminhos relativos (obras, volumes ou capítulos) é informada para transferência seletiva
- **THEN** apenas os arquivos correspondentes aos caminhos informados SHALL ser copiados para o destino, rejeitando caminhos absolutos ou com `..`.

### Requirement: Comparação e sincronização entre bibliotecas
The system SHALL compare two libraries by relative path and SHA-256 hash, identifying missing and differing files, and synchronizing destination files without unexpected overwrites.

#### Scenario: Comparar bibliotecas identifica ausentes e diferentes
- **WHEN** `comparar_bibliotecas` analisa origem e destino
- **THEN** o sistema SHALL retornar listas separadas de caminhos relativos ausentes no destino, caminhos com conteúdo diferente (SHA-256 distinto) e caminhos idênticos.

#### Scenario: Evitar sobrescrita inesperada durante sincronização
- **WHEN** `sincronizar_bibliotecas` é executado com `permitir_sobrescrita = false` e existem arquivos diferentes no destino
- **THEN** o sistema SHALL copiar apenas os arquivos ausentes no destino, registrar os arquivos divergentes como ignorados/conflitantes no relatório e NÃO sobrescrever o destino nem excluir a origem.
