# portable-packaging Specification

## Purpose
TBD - created by archiving change implement-portable-packaging. Update Purpose after archive.

## Requirements

### Requirement: Empacotamento portátil para Windows (ZIP x86-64)
The system SHALL provide a packaging script for Windows x86-64 that produces a self-contained portable ZIP layout containing the compiled Tauri application, embedded frontend assets, and AI Engine runtime without requiring user installation of Python, Node.js, or SQLite.

#### Scenario: Montar estrutura portátil Windows sem modelos automáticos
- **WHEN** o script de empacotamento Windows é executado
- **THEN** o pacote SHALL incluir o executável compilado, o módulo `ai-engine/mam_ai_engine/`, um diretório `models/` vazio (sem download automático de modelos) e gerar o arquivo `.zip` portátil.

### Requirement: Empacotamento portátil para Linux (AppImage e TAR.GZ x86-64)
The system SHALL provide a packaging script and documentation for Linux x86-64 supporting AppImage (primary) and TAR.GZ (alternative), documenting required system libraries and execution permissions.

#### Scenario: Gerar artefatos Linux portáteis com permissões corretas
- **WHEN** o script de empacotamento Linux é executado em ambiente x86-64
- **THEN** o script SHALL montar o `AppDir` com `AppRun` executável, gerar `.tar.gz` portátil e invocar `appimagetool` quando disponível para produzir o `.AppImage`.
