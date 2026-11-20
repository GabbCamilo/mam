# Design — resolve-mvp-pendencies

## Context

Consolidação das pendências levantadas na auditoria do projeto MAM após as Fases 1–12.

## Goals / Non-Goals

**Goals:**
- Implementar `GetDiskFreeSpaceExW` (Windows) / `statvfs` (Unix) em `mam-storage` sem adicionar dependências externas pesadas.
- Implementar `CacheL1Lru` em `mam-storage` e integrá-lo ao `reader_obter_pagina` em `mam-interface`.
- Persistir e revisar resultados de IA na tabela `resultados_ia` já existente em `001_initial.sql`.
- Priorizar `<exe_dir>/python/python.exe` no `AiEngineManager`.

**Non-Goals:**
- Baixar modelos ONNX/PyTorch pesados automaticamente (proibido pelas regras do projeto).
