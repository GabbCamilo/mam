# Design — implement-source-integration

## Context

O banco de dados (Fase 3), gerenciamento de capítulos (Fase 4) e Archive Manager (Fase 5) já estão implementados. Agora é necessário a camada de integração com fontes autorizadas para pesquisar obras e obter capítulos para arquivamento. A integração deve ser modular, desacoplada, e respeitar estritamente as autorizações de acesso.

## Goals / Non-Goals

**Goals:**
- Interface modular para fontes (trait Fonte) com registro, listagem, ativação/desativação
- Pesquisa de obras por título em fontes autorizadas
- Listagem de capítulos disponíveis para uma obra
- Detecção de obras/capítulos já existentes na biblioteca (duplicatas)
- Gerenciamento de operações de arquivamento com progresso e retomada
- Registro de histórico de operações
- Integração com Archive Manager para gerar CBZ após obtenção autorizada

**Non-Goals:**
- Contornar DRM, paywalls, autenticação, controles de acesso ou restrições técnicas
- Download de conteúdo não autorizado
- Monitoramento automático de novas disponibilizações (funcionalidade futura, fora do MVP)
- Extensões/plugins de terceiros na primeira versão

## Decisions

1. **Trait Fonte** — interface comum para todas as fontes: `buscar_obras`, `listar_capitulos`, `obter_metadados`, `eh_autorizada`. Cada fonte implementa o trait.
2. **Fontes autorizadas apenas** — o sistema só permite arquivamento de fontes marcadas como autorizadas. Nenhuma fonte é pré-configurada com credenciais ou mecanismos de bypass.
3. **Progresso via eventos** — operações de arquivamento emitem eventos de progresso (percentual, etapa atual) que o frontend consome via Tauri events.
4. **Detecção de duplicatas** — antes de arquivar, consulta o banco SQLite se a obra/capítulo já existe (pelo título ou caminho relativo).
5. **Retomada de operações** — operações interrompidas podem ser retomadas consultando o histórico de operações no banco.
6. **reqwest** — cliente HTTP para requisições a fontes (apenas fontes públicas/autorizadas, sem auth bypass).
7. **Não amarrar a uma fonte específica** — o sistema é fonte-agnóstico; cada fonte é uma implementação separada do trait.

## Arquitetura

```
backend/
├── source-integration/   # mam-source-integration — busca e obtenção autorizada
│   ├── src/
│   │   ├── lib.rs        # SourceIntegrationManager ponto de entrada
│   │   ├── fonte.rs      # trait Fonte e registro de fontes
│   │   ├── busca.rs      # pesquisa de obras
│   │   ├── capitulos.rs  # listagem de capítulos
│   │   ├── operacao.rs   # gerenciamento de operações com progresso
│   │   └── duplicatas.rs # detecção de conteúdos existentes
├── archive/              # mam-archive — geração CBZ (já existe)
└── src-tauri/            # comandos Tauri expostos ao frontend
```

## Trait Fonte

```rust
pub trait Fonte: Send + Sync {
    fn id(&self) -> &str;
    fn nome(&self) -> &str;
    fn eh_autorizada(&self) -> bool;
    async fn buscar_obras(&self, consulta: &str) -> Result<Vec<ObraEncontrada>, SourceError>;
    async fn listar_capitulos(&self, obra_id: &str) -> Result<Vec<CapituloEncontrado>, SourceError>;
    async fn obter_capitulo(&self, capitulo_id: &str) -> Result<Bytes, SourceError>;
}
```

## Fluxo de arquivamento

```
usuário consulta "Naruto"
      ↓
source-integration.buscar_obras("Naruto") → lista de obras
      ↓
usuário seleciona obra
      ↓
source-integration.listar_capitulos(obra_id) → lista de capítulos
      ↓
usuário seleciona capítulos
      ↓
verificar duplicatas (banco SQLite) → filtrar já existentes
      ↓
iniciar operação de arquivamento (progresso via eventos)
      ↓
para cada capítulo: obter conteúdo → ArchiveManager.criar_cbz → registrar no banco
      ↓
registrar no histórico de operações
```

## Regras obrigatórias

- Somente fontes autorizadas (eh_autorizada() == true)
- Não contornar DRM, paywalls, autenticação ou controles de acesso
- Detectar conteúdos já existentes antes de arquivar
- Acompanhamento de progresso de operações
- Possibilidade de retomar operações interrompidas
- Registro de histórico de operações
- Caminhos relativos à raiz da biblioteca

## Risks / Trade-offs

- [Fonte-agnóstico] cada fonte requer implementação separada do trait; mitigado com documentação clara do contrato.
- [HTTP] reqwest requer conexão à internet para fontes; operação offline usa apenas dados já armazenados.
- [Retomada] retomada requer identificação estável de capítulos (URL ou identificador da fonte).

## Migration Plan

1. Criar crate `mam-source-integration` no workspace
2. Implementar trait Fonte, busca, capítulos, operacao, duplicatas
3. Adicionar comandos Tauri em `mam-interface` e `src-tauri`
4. Validar com `cargo build` e `cargo test`
5. Arquivar change

## Open Questions

- Quais fontes iniciais suportar? (nenhuma pré-configurada; usuário registra fontes autorizadas)
- Formato do identificador de capítulo por fonte? (URL, ID interno — deixado para cada fonte)
- Limite de concurrent downloads? (avaliar com filas controladas)
