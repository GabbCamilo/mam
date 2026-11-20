# Empacotamento Portátil — Manga Archive Manager (MAM)

O MAM é distribuído em pacotes portáteis separados por sistema operacional e arquitetura **x86-64** (macOS está fora do escopo).

## Formatos Suportados

| Plataforma | Arquitetura | Formato | Script |
|---|---|---|---|
| Windows | x86-64 | `.zip` portátil | [build-portable-zip.ps1](windows/build-portable-zip.ps1) |
| Linux | x86-64 | `.AppImage` (prioritário) e `.tar.gz` (alternativo) | [build-portable-linux.sh](linux/build-portable-linux.sh) |

## Estrutura Interna do Pacote Portátil

```text
MAM-<plataforma>-x86_64/
├── app.exe (ou app no Linux)
├── ai-engine/
│   ├── mam_ai_engine/
│   └── requirements.txt
└── models/                  # Diretório vazio (modelos opcionais instalados sob demanda)
```

## Regras de Portabilidade

- O usuário final não precisa instalar Python, Node.js ou SQLite.
- Nenhum modelo volumoso de IA é baixado ou instalado automaticamente durante o empacotamento.
- Caminhos de biblioteca e banco SQLite são relativos à raiz escolhida pelo usuário (ex.: SSD externo).
