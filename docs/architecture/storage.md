# Armazenamento — Manga Archive Manager

## Estrutura conceitual da biblioteca

```
MAM_LIBRARY/
├── Manga/
│   ├── Obra A/
│   │   ├── Volume 001/
│   │   │   ├── Capitulo 0001.cbz
│   │   │   └── Capitulo 0002.cbz
│   │   └── Volume 002/
│   │       └── Capitulo 0003.cbz
│   └── Obra B/
│       └── Capitulo 0001.cbz
│
├── System/
│   ├── database.db
│   ├── library_config.json
│   └── sources.json
│
├── Metadata/
├── Models/
├── Integrity/
└── Logs/
```

A estrutura poderá ser ajustada após a especificação dos domínios.

## Requisitos

- Utilizar caminhos relativos à raiz da biblioteca
- Não depender da letra da unidade (D: → E: não invalida referências)
- Reconhecer bibliotecas existentes em unidades externas
- Preservar metadados
- Permitir operação offline
- Evitar duplicatas
- Verificar integridade (SHA-256)
- Detectar desconexão do SSD
- Interromper escritas com segurança quando a unidade ficar indisponível

## Banco de dados

- Tecnologia: SQLite (arquivo único, sem instalação externa)
- Armazena: obras, volumes, capítulos, metadados, configurações, histórico, estado de integridade, referências a resultados de IA, localizações relativas dos arquivos
- Transações curtas e operações controladas
- WAL avaliado com atenção aos requisitos de portabilidade e transferência
- Não permitir que duas instâncias escrevam simultaneamente na mesma biblioteca sem mecanismo explícito de controle

## Preservação

- Arquivos originais preservados
- CBZ originais nunca modificados pelo processamento de IA
- Resultados de IA armazenados separadamente
