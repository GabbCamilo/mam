//! # mam-source-integration
//! Busca e obtenção autorizada de obras e capítulos em fontes compatíveis.
//!
//! A integração com fontes é modular e desacoplada. Cada fonte implementa o
//! trait `Fonte`. Somente fontes autorizadas (eh_autorizada == true) permitem
//! arquivamento. Não há mecanismos para contornar DRM, paywalls ou controles
//! de acesso.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use mam_database::Database;


use serde::{Deserialize, Serialize};

pub mod connectors;
pub mod extractors;
pub mod fonte;
pub mod mangadex;
mod busca;
mod capitulos;
mod operacao;
mod duplicatas;

pub use fonte::Fonte;

/// Erros do módulo de integração com fontes.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("Erro de rede: {0}")]
    Rede(#[from] reqwest::Error),
    #[error("Erro de serialização: {0}")]
    Serializacao(#[from] serde_json::Error),
    #[error("Erro de banco: {0}")]
    Banco(#[from] mam_database::DatabaseError),
    #[error("Fonte não encontrada: {0}")]
    FonteNaoEncontrada(String),
    #[error("Fonte não autorizada: {0}")]
    FonteNaoAutorizada(String),
    #[error("Erro de fonte: {0}")]
    Fonte(#[from] crate::fonte::FonteError),
    #[error("Obra não encontrada: {0}")]
    ObraNaoEncontrada(String),
    #[error("Capítulo não encontrado: {0}")]
    CapituloNaoEncontrado(String),
    #[error("Erro de operação: {0}")]
    Operacao(String),
}

/// Resultado de busca de obra em uma fonte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObraBuscaResultado {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub titulo: String,
    pub autor: Option<String>,
    pub descricao: Option<String>,
    #[serde(default)]
    pub capa_url: Option<String>,
}

/// Resultado de listagem de capítulos em uma fonte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapituloBuscaResultado {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub capitulo_id_fonte: String,
    pub numero: Option<i32>,
    pub titulo: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub idioma: Option<String>,
}

/// Gerenciador de integração com fontes.
/// Orquestra o registro de fontes, busca de obras, listagem de capítulos
/// e operações de arquivamento.
pub struct SourceIntegrationManager {
    fontes: Arc<RwLock<HashMap<String, Arc<dyn Fonte>>>>,
    db: Arc<Database>,
    operacoes: Arc<RwLock<HashMap<String, operacao::OperacaoArquivamento>>>,
}

impl SourceIntegrationManager {
    pub fn nova(db: Arc<Database>) -> Self {
        Self {
            fontes: Arc::new(RwLock::new(HashMap::new())),
            db,
            operacoes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registra uma nova fonte no gerenciador.
    pub async fn registrar_fonte(
        &self,
        fonte: Arc<dyn Fonte>,
    ) -> Result<(), SourceError> {
        let id = fonte.id().to_string();
        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(id.clone()));
        }
        self.fontes.write().await.insert(id.clone(), fonte);
        Ok(())
    }

    /// Lista todas as fontes registradas.
    pub async fn listar_fontes(&self) -> Vec<FonteInfo> {
        let fontes = self.fontes.read().await;
        fontes
            .values()
            .map(|f| FonteInfo {
                id: f.id().to_string(),
                nome: f.nome().to_string(),
                autorizada: f.eh_autorizada(),
            })
            .collect()
    }

    /// Busca obras por título em todas as fontes autorizadas.
    pub async fn buscar_obras(
        &self,
        consulta: &str,
    ) -> Result<Vec<ObraBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let mut resultados = Vec::new();

        for fonte in fontes.values() {
            if !fonte.eh_autorizada() {
                continue;
            }
            match fonte.buscar_obras(consulta).await {
                Ok(obras) => {
                    for obra in obras {
                        resultados.push(ObraBuscaResultado {
                            fonte_id: fonte.id().to_string(),
                            obra_id_fonte: obra.obra_id_fonte,
                            titulo: obra.titulo,
                            autor: obra.autor,
                            descricao: obra.descricao,
                            capa_url: obra.capa_url,
                        });
                    }
                }
                Err(_) => {
                    // Fonte falhou, continua com as outras
                    continue;
                }
            }
        }

        Ok(resultados)
    }

    /// Lista capítulos de uma obra em uma fonte específica.
    pub async fn listar_capitulos(
        &self,
        fonte_id: &str,
        obra_id_fonte: &str,
    ) -> Result<Vec<CapituloBuscaResultado>, SourceError> {
        let fontes = self.fontes.read().await;
        let fonte = fontes
            .get(fonte_id)
            .ok_or_else(|| SourceError::FonteNaoEncontrada(fonte_id.to_string()))?;

        if !fonte.eh_autorizada() {
            return Err(SourceError::FonteNaoAutorizada(fonte_id.to_string()));
        }

        let capitulos = fonte.listar_capitulos(obra_id_fonte).await?;

        Ok(capitulos
            .into_iter()
            .map(|c| CapituloBuscaResultado {
                fonte_id: fonte.id().to_string(),
                obra_id_fonte: obra_id_fonte.to_string(),
                capitulo_id_fonte: c.capitulo_id_fonte,
                numero: c.numero,
                titulo: c.titulo,
                url: c.url,
                idioma: c.idioma,
            })
            .collect())
    }

    /// Verifica se uma obra já está arquivada no banco (detecção de duplicata).
    pub async fn obra_ja_arquivada(&self, titulo: &str) -> Result<bool, SourceError> {
        Ok(self.db.buscar_obra_por_titulo(titulo).await?.is_some())
    }

    /// Verifica se um capítulo já está arquivado (detecção de duplicata por caminho).
    pub async fn capitulo_ja_arquivado(
        &self,
        caminho_relativo: &str,
    ) -> Result<bool, SourceError> {
        Ok(self.db.consultar_integridade(caminho_relativo).await?.is_some())
    }

    /// Inicia uma operação de arquivamento de capítulos e persiste o estado inicial no SQLite.
    pub async fn iniciar_operacao_arquivamento(
        &self,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
    ) -> Result<String, SourceError> {
        let id_operacao = uuid::Uuid::new_v4().to_string();

        let operacao = operacao::OperacaoArquivamento::nova(
            id_operacao.clone(),
            obra_titulo.clone(),
            capitulos,
        );
        if let Ok(json) = serde_json::to_string(&operacao) {
            let _ = self
                .db
                .salvar_estado_operacao_arquivamento(&id_operacao, &obra_titulo, "executando", &json)
                .await;
        }

        self.operacoes.write().await.insert(id_operacao.clone(), operacao);

        Ok(id_operacao)
    }

    async fn persistir_operacao(&self, id_operacao: &str) {
        let snapshot = {
            let operacoes = self.operacoes.read().await;
            operacoes.get(id_operacao).cloned()
        };
        if let Some(op) = snapshot {
            if let Ok(json) = serde_json::to_string(&op) {
                let _ = self
                    .db
                    .salvar_estado_operacao_arquivamento(&op.id, &op.obra_titulo, &op.estado, &json)
                    .await;
            }
        }
    }

    /// Carrega operações que estavam em andamento ou pausadas no banco após fechar o app.
    pub async fn carregar_operacoes_pausadas_do_banco(&self) -> usize {
        let Ok(rows) = self.db.listar_estados_operacoes_arquivamento().await else {
            return 0;
        };
        let mut carregadas = 0usize;
        let mut guard = self.operacoes.write().await;
        for (id, _titulo, estado, payload_json) in rows {
            if guard.contains_key(&id) {
                continue;
            }
            if estado == "executando" || estado == "pausado" {
                if let Ok(mut op) = serde_json::from_str::<operacao::OperacaoArquivamento>(&payload_json) {
                    op.pausada = true;
                    op.estado = "pausado".to_string();
                    guard.insert(id, op);
                    carregadas += 1;
                }
            }
        }
        carregadas
    }

    /// Pausa uma operação ativa.
    pub async fn pausar_operacao(&self, id_operacao: &str) -> Result<(), SourceError> {
        {
            let mut operacoes = self.operacoes.write().await;
            let op = operacoes
                .get_mut(id_operacao)
                .ok_or_else(|| SourceError::Operacao("Operação não encontrada".to_string()))?;
            if !op.esta_concluida() && !op.cancelada {
                op.pausada = true;
                op.estado = "pausado".to_string();
            }
        }
        self.persistir_operacao(id_operacao).await;
        Ok(())
    }

    /// Pausa automaticamente todas as operações em andamento (ex.: ao fechar o aplicativo).
    pub async fn pausar_todas_operacoes_ativas(&self) -> usize {
        let ids: Vec<String> = {
            let mut operacoes = self.operacoes.write().await;
            let mut alterados = Vec::new();
            for op in operacoes.values_mut() {
                if !op.esta_concluida() && !op.cancelada && op.estado == "executando" {
                    op.pausada = true;
                    op.estado = "pausado".to_string();
                    alterados.push(op.id.clone());
                }
            }
            alterados
        };
        for id in &ids {
            self.persistir_operacao(id).await;
        }
        ids.len()
    }

    /// Prepara uma operação pausada para retomada: se ainda houver tarefa ativa em memória,
    /// apenas despausa; se foi restaurada do banco após reinício, retorna os dados para reexecução.
    pub async fn preparar_retomada_operacao(
        &self,
        id_operacao: &str,
    ) -> Result<Option<(String, Vec<CapituloBuscaResultado>, Option<String>)>, SourceError> {
        let dados_respawn = {
            let mut operacoes = self.operacoes.write().await;
            let op = operacoes
                .get_mut(id_operacao)
                .ok_or_else(|| SourceError::Operacao("Operação não encontrada".to_string()))?;
            if op.cancelada || op.esta_concluida() {
                return Ok(None);
            }
            op.pausada = false;
            op.estado = "executando".to_string();
            let caps = op
                .capitulos
                .iter()
                .map(|c| CapituloBuscaResultado {
                    fonte_id: c.fonte_id.clone(),
                    obra_id_fonte: c.obra_id_fonte.clone(),
                    capitulo_id_fonte: c.capitulo_id_fonte.clone(),
                    numero: c.numero,
                    titulo: c.titulo.clone(),
                    url: c.url.clone(),
                    idioma: c.idioma.clone(),
                })
                .collect::<Vec<_>>();
            Some((op.obra_titulo.clone(), caps, op.capa_url.clone()))
        };
        self.persistir_operacao(id_operacao).await;
        Ok(dados_respawn)
    }

    /// Cancela uma operação e executa rollback estilo Docker (remove os CBZs e registros deste conjunto).
    pub async fn cancelar_operacao(
        &self,
        id_operacao: &str,
        raiz_biblioteca: &std::path::Path,
    ) -> Result<(), SourceError> {
        let estava_pausada = {
            let mut operacoes = self.operacoes.write().await;
            let op = operacoes
                .get_mut(id_operacao)
                .ok_or_else(|| SourceError::Operacao("Operação não encontrada".to_string()))?;
            op.cancelada = true;
            op.pausada = false;
            op.estado = "cancelado".to_string();
            op.capitulos_gravados.clone()
        };
        // Sempre executa rollback imediatamente para garantir limpeza tanto se pausada quanto ativa
        let _ = estava_pausada;
        self.executar_rollback_operacao(id_operacao, raiz_biblioteca)
            .await;
        Ok(())
    }

    async fn executar_rollback_operacao(
        &self,
        id_operacao: &str,
        raiz_biblioteca: &std::path::Path,
    ) {
        let (gravados, obra_id_criada, obra_nova, obra_titulo) = {
            let mut operacoes = self.operacoes.write().await;
            if let Some(op) = operacoes.get_mut(id_operacao) {
                op.cancelada = true;
                op.pausada = false;
                op.estado = "cancelado".to_string();
                let grav = std::mem::take(&mut op.capitulos_gravados);
                (grav, op.obra_id_criada.clone(), op.obra_nova, op.obra_titulo.clone())
            } else {
                (Vec::new(), None, false, String::new())
            }
        };

        for (cap_id, caminho_rel) in &gravados {
            let abs = raiz_biblioteca.join(caminho_rel);
            let _ = std::fs::remove_file(&abs);
            let _ = std::fs::remove_file(abs.with_extension("cbz.mam_tmp"));
            let _ = self.db.excluir_capitulo_por_id(cap_id).await;
        }

        let _ = self.db.remover_historico_por_operacao(id_operacao).await;

        if obra_nova {
            if let Some(oid) = obra_id_criada {
                if let Ok(true) = self.db.excluir_obra_se_vazia(&oid).await {
                    let pasta_obra = raiz_biblioteca.join("Manga").join(sanitizar_nome(&obra_titulo));
                    let _ = std::fs::remove_dir_all(&pasta_obra);
                }
            }
        }

        let _ = self
            .db
            .remover_estado_operacao_arquivamento(id_operacao)
            .await;
    }

    /// Retorna o status de uma operação de arquivamento.
    pub async fn status_operacao(
        &self,
        id_operacao: &str,
    ) -> Result<Option<StatusOperacao>, SourceError> {
        let operacoes = self.operacoes.read().await;
        Ok(operacoes.get(id_operacao).map(|o| o.status()))
    }

    /// Lista todas as operações de arquivamento.
    pub async fn listar_operacoes(&self) -> Vec<StatusOperacao> {
        let operacoes = self.operacoes.read().await;
        operacoes.values().map(|o| o.status()).collect()
    }

    /// Executa o arquivamento de um capítulo (atualiza progresso).
    pub async fn arquivar_capitulo(
        &self,
        id_operacao: &str,
        _fonte_id: &str,
        capitulo: &CapituloBuscaResultado,
    ) -> Result<(), SourceError> {
        let mut operacoes = self.operacoes.write().await;
        if let Some(op) = operacoes.get_mut(id_operacao) {
            op.atualizar_progresso(capitulo.capitulo_id_fonte.as_str());
        }
        Ok(())
    }

    /// Executa o arquivamento completo de uma lista de capítulos na biblioteca:
    /// obtém bytes da fonte autorizada, grava `.cbz` atomicamente (`temp` + `rename`),
    /// calcula SHA-256, salva capa local (quando informada) e registra obra/capítulo/log no SQLite.
    pub async fn arquivar_para_biblioteca(
        &self,
        raiz_biblioteca: &std::path::Path,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
    ) -> Result<String, SourceError> {
        self.arquivar_para_biblioteca_com_capa(raiz_biblioteca, obra_titulo, capitulos, None)
            .await
    }

    /// Variante de `arquivar_para_biblioteca` que também persiste `cover.jpg` localmente quando `capa_url` é fornecida.
    pub async fn arquivar_para_biblioteca_com_capa(
        &self,
        raiz_biblioteca: &std::path::Path,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
        capa_url: Option<String>,
    ) -> Result<String, SourceError> {
        let id_operacao = self
            .iniciar_operacao_arquivamento(obra_titulo.clone(), capitulos.clone())
            .await?;
        self.executar_arquivamento_operacao(
            &id_operacao,
            raiz_biblioteca,
            obra_titulo,
            capitulos,
            capa_url,
        )
        .await?;
        Ok(id_operacao)
    }

    /// Executa o download e arquivamento de uma operação já iniciada em `self.operacoes`.
    pub async fn executar_arquivamento_operacao(
        &self,
        id_operacao: &str,
        raiz_biblioteca: &std::path::Path,
        obra_titulo: String,
        capitulos: Vec<CapituloBuscaResultado>,
        capa_url: Option<String>,
    ) -> Result<(), SourceError> {

        let nome_pasta_obra = sanitizar_nome(&obra_titulo);
        let caminho_rel_obra = format!("Manga/{}", nome_pasta_obra);
        let pasta_obra_abs = raiz_biblioteca.join(&caminho_rel_obra);
        let _ = std::fs::create_dir_all(&pasta_obra_abs);

        if let Some(ref url_capa) = capa_url {
            let _ = salvar_capa_local(url_capa, &pasta_obra_abs).await;
        }

        let (obra, obra_criada_agora) = match self.db.buscar_obra_por_titulo(&obra_titulo).await? {
            Some(o) => (o, false),
            None => (
                self.db
                    .criar_obra(
                        mam_core::ObraId::new(),
                        obra_titulo.clone(),
                        mam_core::CaminhoRelativo::from_str(caminho_rel_obra.clone()),
                        capitulos.first().map(|c| c.fonte_id.clone()),
                    )
                    .await?,
                true,
            ),
        };

        {
            let mut operacoes = self.operacoes.write().await;
            if let Some(op) = operacoes.get_mut(id_operacao) {
                op.capa_url = capa_url.clone();
                if op.obra_id_criada.is_none() {
                    op.obra_id_criada = Some(obra.id.to_string());
                    op.obra_nova = obra_criada_agora;
                }
            }
        }
        self.persistir_operacao(id_operacao).await;

        for cap in capitulos {
            // Verifica se capítulo já foi processado (retomada após pausa/reinício)
            {
                let operacoes = self.operacoes.read().await;
                if let Some(op) = operacoes.get(id_operacao) {
                    if op.capitulos_processados.get(&cap.capitulo_id_fonte).copied().unwrap_or(false) {
                        continue;
                    }
                }
            }

            // Checa cancelamento / pausa antes de iniciar cada capítulo
            loop {
                let (cancelada, pausada) = {
                    let operacoes = self.operacoes.read().await;
                    match operacoes.get(id_operacao) {
                        Some(op) => (op.cancelada, op.pausada),
                        None => (true, false),
                    }
                };
                if cancelada {
                    self.executar_rollback_operacao(id_operacao, raiz_biblioteca).await;
                    return Ok(());
                }
                if !pausada {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(350)).await;
            }
            let numero = cap.numero.unwrap_or(0);
            let idioma_limpo = cap
                .idioma
                .as_deref()
                .map(|s| sanitizar_nome(s))
                .filter(|s| !s.is_empty());
            let pasta_destino_rel = match &idioma_limpo {
                Some(lang) => format!("{}/{}", caminho_rel_obra, lang),
                None => caminho_rel_obra.clone(),
            };

            let cbz_bytes = if cap.fonte_id == "extrator-link" {
                match extractors::RegistroExtratores::padrao()
                    .baixar_capitulo_cbz_por_url(&cap.capitulo_id_fonte)
                    .await
                {
                    Ok(b) => b,
                    Err(err) => {
                        let detalhes_erro = serde_json::json!({
                            "nome": format!("❌ Falha: {} — Cap. {} ({})", obra_titulo, numero, err),
                            "pasta": pasta_destino_rel,
                            "operacao_id": id_operacao,
                            "obra_id": obra.id.to_string(),
                            "obra_titulo": obra_titulo,
                            "erro": true,
                        })
                        .to_string();
                        let _ = self
                            .db
                            .registrar_operacao(
                                "download".to_string(),
                                Some(cap.capitulo_id_fonte.clone()),
                                Some(detalhes_erro),
                                None,
                            )
                            .await;
                        self.arquivar_capitulo(id_operacao, &cap.fonte_id, &cap)
                            .await?;
                        continue;
                    }
                }
            } else {
                let fonte = {
                    let fontes = self.fontes.read().await;
                    fontes
                        .get(&cap.fonte_id)
                        .cloned()
                        .ok_or_else(|| SourceError::FonteNaoEncontrada(cap.fonte_id.clone()))?
                };

                if !fonte.eh_autorizada() {
                    return Err(SourceError::FonteNaoAutorizada(cap.fonte_id));
                }

                match fonte.obter_capitulo(&cap.capitulo_id_fonte).await {
                    Ok(b) => b,
                    Err(_) => {
                        self.arquivar_capitulo(id_operacao, &cap.fonte_id, &cap)
                            .await?;
                        continue;
                    }
                }
            };
            let qtd_paginas = contar_paginas_cbz_bytes(&cbz_bytes) as i32;
            if qtd_paginas <= 0 {
                self.arquivar_capitulo(id_operacao, &cap.fonte_id, &cap)
                    .await?;
                continue;
            }
            use sha2::Digest;
            let hash = format!("{:x}", sha2::Sha256::digest(&cbz_bytes));

            let mut ja_existe_identico = false;
            let mut nome_arquivo = String::new();
            let mut caminho_rel_cap = String::new();
            let mut caminho_abs = std::path::PathBuf::new();

            let mut versao: usize = 1;
            loop {
                let nome_cand = if versao == 1 {
                    format!("Capitulo_{:04}.cbz", numero)
                } else {
                    format!("Capitulo_{:04}_v{}.cbz", numero, versao)
                };
                let rel_cand = format!("{}/{}", pasta_destino_rel, nome_cand);
                let abs_cand = raiz_biblioteca.join(&rel_cand);

                let existe_disco = abs_cand.exists();
                let existe_banco = self
                    .db
                    .buscar_capitulo_por_caminho(&rel_cand)
                    .await?
                    .is_some();

                if !existe_disco && !existe_banco {
                    nome_arquivo = nome_cand;
                    caminho_rel_cap = rel_cand;
                    caminho_abs = abs_cand;
                    break;
                }

                // Avalia se o arquivo existente é idêntico pelo hash SHA-256
                let hash_existente = if existe_disco {
                    std::fs::read(&abs_cand)
                        .ok()
                        .map(|bytes| format!("{:x}", sha2::Sha256::digest(&bytes)))
                } else {
                    self.db
                        .consultar_integridade(&rel_cand)
                        .await?
                        .map(|i| i.hash_sha256)
                };

                if hash_existente.as_deref() == Some(hash.as_str()) {
                    ja_existe_identico = true;
                    break;
                }

                versao += 1;
            }

            if ja_existe_identico {
                self.arquivar_capitulo(id_operacao, &cap.fonte_id, &cap)
                    .await?;
                continue;
            }

            if let Some(pai) = caminho_abs.parent() {
                std::fs::create_dir_all(pai).map_err(|e| SourceError::Operacao(e.to_string()))?;
            }

            let temp_path = caminho_abs.with_extension("cbz.mam_tmp");
            std::fs::write(&temp_path, &cbz_bytes)
                .map_err(|e| SourceError::Operacao(e.to_string()))?;
            std::fs::rename(&temp_path, &caminho_abs)
                .map_err(|e| SourceError::Operacao(e.to_string()))?;

            self.db
                .registrar_integridade(&caminho_rel_cap, &hash, "valido")
                .await?;

            let cap_id = mam_core::CapituloId::new();
            self.db
                .criar_capitulo_com_idioma(
                    cap_id,
                    None,
                    obra.id,
                    numero,
                    cap.titulo.clone(),
                    mam_core::CaminhoRelativo::from_str(caminho_rel_cap.clone()),
                    qtd_paginas,
                    cap.idioma.clone(),
                )
                .await?;

            let detalhes = serde_json::json!({
                "operacao_id": id_operacao,
                "obra_id": obra.id.to_string(),
                "obra_titulo": obra_titulo,
                "capitulo_id": cap_id.to_string(),
                "caminho_relativo": caminho_rel_cap,
                "nome": match &cap.idioma {
                    Some(lang) if !lang.trim().is_empty() => format!("{} — Cap. {} [{}]", obra_titulo, numero, lang),
                    _ => format!("{} — Cap. {}", obra_titulo, numero),
                },
                "pasta": pasta_destino_rel,
                "arquivo": nome_arquivo,
                "paginas": qtd_paginas,
            })
            .to_string();
            {
                let mut operacoes = self.operacoes.write().await;
                if let Some(op) = operacoes.get_mut(id_operacao) {
                    op.capitulos_gravados
                        .push((cap_id.to_string(), caminho_rel_cap.clone()));
                }
            }

            let _ = self
                .db
                .registrar_operacao(
                    "download".to_string(),
                    Some(caminho_rel_cap),
                    Some(detalhes),
                    None,
                )
                .await;

            self.arquivar_capitulo(id_operacao, &cap.fonte_id, &cap)
                .await?;
            self.persistir_operacao(id_operacao).await;

            // Se foi cancelado durante o download deste capítulo, aplica rollback imediato
            let cancelado_durante = {
                let operacoes = self.operacoes.read().await;
                operacoes.get(id_operacao).map(|o| o.cancelada).unwrap_or(false)
            };
            if cancelado_durante {
                self.executar_rollback_operacao(id_operacao, raiz_biblioteca).await;
                return Ok(());
            }
        }

        let _ = self
            .db
            .remover_estado_operacao_arquivamento(id_operacao)
            .await;

        Ok(())
    }
}

fn eh_assinatura_imagem_valida(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xFF, 0xD8, 0xFF])
        || bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47])
        || bytes.starts_with(b"GIF8")
        || (bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP")
        || bytes.starts_with(b"BM")
        || (bytes.len() >= 12 && &bytes[4..8] == b"ftyp")
}

async fn baixar_bytes_imagem(url: &str) -> Result<(String, Vec<u8>), SourceError> {
    let eh_mangadex_upload = url.to_lowercase().contains("uploads.mangadex.org");
    let ua = if eh_mangadex_upload {
        "MAM-MangaArchiveManager/0.1.0"
    } else {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"
    };
    let client = reqwest::Client::builder()
        .user_agent(ua)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut req = client.get(url).header(
        reqwest::header::ACCEPT,
        "image/avif,image/webp,image/apng,image/*,*/*;q=0.8",
    );
    if !eh_mangadex_upload {
        for (k, v) in crate::connectors::GerenciadorConectores::headers_para_url(url, None) {
            req = req.header(&k, &v);
        }
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Err(SourceError::Operacao(format!(
            "Falha ao obter imagem: HTTP {}",
            resp.status()
        )));
    }
    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .split(';')
        .next()
        .unwrap_or("image/jpeg")
        .trim()
        .to_lowercase();
    if !mime.starts_with("image/") {
        return Err(SourceError::Operacao(format!(
            "Resposta não é uma imagem válida ({})",
            mime
        )));
    }
    let bytes = resp.bytes().await?.to_vec();
    if bytes.is_empty() || !eh_assinatura_imagem_valida(&bytes) {
        return Err(SourceError::Operacao(
            "Conteúdo retornado não possui assinatura de imagem válida".to_string(),
        ));
    }
    Ok((mime, bytes))
}

/// Baixa uma imagem externa via HTTP (aplicando `Referer`/headers do conector) e retorna Data URL Base64.
pub async fn baixar_imagem_data_url(url: &str) -> Result<String, SourceError> {
    let (mime, bytes) = baixar_bytes_imagem(url).await?;
    Ok(format!("data:{};base64,{}", mime, codificar_base64(&bytes)))
}

/// Baixa e salva `cover.jpg` na pasta da obra para exibição offline na Biblioteca.
/// Também aceita Data URLs (`data:image/...;base64,...`) já capturadas no frontend.
pub async fn salvar_capa_local(url: &str, pasta_obra: &std::path::Path) -> Result<(), SourceError> {
    let bytes: Vec<u8> = if let Some(b64_pos) = url.find("base64,") {
        if url.starts_with("data:image/") {
            decodificar_base64_simples(&url[b64_pos + 7..])
        } else {
            return Err(SourceError::Operacao("Data URL de capa inválida".to_string()));
        }
    } else {
        baixar_bytes_imagem(url).await?.1
    };

    if bytes.is_empty() || !eh_assinatura_imagem_valida(&bytes) {
        return Err(SourceError::Operacao("Imagem de capa vazia ou inválida".to_string()));
    }
    std::fs::create_dir_all(pasta_obra).map_err(|e| SourceError::Operacao(e.to_string()))?;
    let destino = pasta_obra.join("cover.jpg");
    let temp = pasta_obra.join(".cover.jpg.mam_tmp");
    std::fs::write(&temp, &bytes).map_err(|e| SourceError::Operacao(e.to_string()))?;
    if destino.exists() {
        let _ = std::fs::remove_file(&destino);
    }
    std::fs::rename(&temp, &destino).map_err(|e| SourceError::Operacao(e.to_string()))?;
    Ok(())
}

fn decodificar_base64_simples(input: &str) -> Vec<u8> {
    let mut saida = Vec::with_capacity((input.len() * 3) / 4);
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            saida.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    saida
}

pub fn codificar_base64(bytes: &[u8]) -> String {
    const TABELA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut saida = String::with_capacity(((bytes.len() + 2) / 3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triplo = (b0 << 16) | (b1 << 8) | b2;
        saida.push(TABELA[((triplo >> 18) & 0x3F) as usize] as char);
        saida.push(TABELA[((triplo >> 12) & 0x3F) as usize] as char);
        saida.push(if chunk.len() > 1 { TABELA[((triplo >> 6) & 0x3F) as usize] as char } else { '=' });
        saida.push(if chunk.len() > 2 { TABELA[(triplo & 0x3F) as usize] as char } else { '=' });
    }
    saida
}

pub fn sanitizar_nome(s: &str) -> String {
    let limpo: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    let trimmed = limpo.trim();
    if trimmed.is_empty() {
        "Obra".to_string()
    } else {
        trimmed.to_string()
    }
}

fn contar_paginas_cbz_bytes(bytes: &[u8]) -> usize {
    let cursor = std::io::Cursor::new(bytes);
    zip::ZipArchive::new(cursor).map(|z| z.len()).unwrap_or(0)
}

/// Informação de uma fonte registrada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FonteInfo {
    pub id: String,
    pub nome: String,
    pub autorizada: bool,
}

pub use operacao::StatusOperacao;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonte_info_cria() {
        let info = FonteInfo {
            id: "fonte-teste".to_string(),
            nome: "Fonte Teste".to_string(),
            autorizada: true,
        };
        assert_eq!(info.id, "fonte-teste");
        assert!(info.autorizada);
    }

    #[test]
    fn status_operacao_calcula_percentual() {
        let status = StatusOperacao {
            id: "op-1".to_string(),
            obra_titulo: "Obra Teste".to_string(),
            total_capitulos: 10,
            capitulos_processados: 3,
            percentual: 30.0,
            concluida: false,
            estado: "executando".to_string(),
            pausada: false,
            cancelada: false,
        };
        assert_eq!(status.percentual, 30.0);
        assert!(!status.concluida);
    }

    #[tokio::test]
    async fn arquivar_para_biblioteca_grava_cbz_e_sha256() {
        use std::io::Write;
        use crate::fonte::{CapituloEncontrado, ObraEncontrada};
        struct FonteMock;
        #[async_trait::async_trait]
        impl Fonte for FonteMock {
            fn id(&self) -> &str { "mock" }
            fn nome(&self) -> &str { "Mock Autorizada" }
            fn eh_autorizada(&self) -> bool { true }
            async fn buscar_obras(&self, _: &str) -> Result<Vec<ObraEncontrada>, crate::fonte::FonteError> { Ok(vec![]) }
            async fn listar_capitulos(&self, _: &str) -> Result<Vec<CapituloEncontrado>, crate::fonte::FonteError> { Ok(vec![]) }
            async fn obter_capitulo(&self, _: &str) -> Result<Vec<u8>, crate::fonte::FonteError> {
                let mut cur = std::io::Cursor::new(Vec::new());
                {
                    let mut zw = zip::ZipWriter::new(&mut cur);
                    zw.start_file("0001.jpg", zip::write::FileOptions::default()).unwrap();
                    zw.write_all(b"JPG").unwrap();
                    zw.finish().unwrap();
                }
                Ok(cur.into_inner())
            }
        }

        let base = std::env::temp_dir().join(format!("mam_test_src_arch_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();
        let db = Arc::new(Database::abrir(base.join("db.sqlite"), base.clone()).await.unwrap());
        let sm = SourceIntegrationManager::nova(db.clone());
        sm.registrar_fonte(Arc::new(FonteMock)).await.unwrap();

        let caps = vec![
            CapituloBuscaResultado {
                fonte_id: "mock".to_string(),
                obra_id_fonte: "obra-1".to_string(),
                capitulo_id_fonte: "cap-1-pt".to_string(),
                numero: Some(1),
                titulo: Some("Capítulo 1".to_string()),
                url: None,
                idioma: Some("PT".to_string()),
            },
            CapituloBuscaResultado {
                fonte_id: "mock".to_string(),
                obra_id_fonte: "obra-1".to_string(),
                capitulo_id_fonte: "cap-1-eng".to_string(),
                numero: Some(1),
                titulo: Some("Chapter 1".to_string()),
                url: None,
                idioma: Some("ENG".to_string()),
            },
        ];

        let op_id = sm
            .arquivar_para_biblioteca(&base, "Obra Web".to_string(), caps)
            .await
            .expect("arquivar para biblioteca");

        let status = sm.status_operacao(&op_id).await.unwrap().unwrap();
        assert!(status.concluida);
        assert_eq!(status.percentual, 100.0);
        assert!(base.join("Manga/Obra Web/PT/Capitulo_0001.cbz").exists());
        assert!(base.join("Manga/Obra Web/ENG/Capitulo_0001.cbz").exists());
        assert!(sm.capitulo_ja_arquivado("Manga/Obra Web/PT/Capitulo_0001.cbz").await.unwrap());
        assert!(sm.capitulo_ja_arquivado("Manga/Obra Web/ENG/Capitulo_0001.cbz").await.unwrap());

        let obra = db.buscar_obra_por_titulo("Obra Web").await.unwrap().unwrap();
        let caps_db = db.listar_capitulos_obra(obra.id).await.unwrap();
        assert_eq!(caps_db.len(), 2);
        assert!(caps_db.iter().any(|c| c.idioma.as_deref() == Some("PT")));
        assert!(caps_db.iter().any(|c| c.idioma.as_deref() == Some("ENG")));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn ciclo_vida_pausa_retomada_cancelamento_rollback_e_limpar_log_preservado() {
        use std::io::Write;
        use crate::fonte::{CapituloEncontrado, ObraEncontrada};
        struct FonteLentaMock;
        #[async_trait::async_trait]
        impl Fonte for FonteLentaMock {
            fn id(&self) -> &str { "mock-lenta" }
            fn nome(&self) -> &str { "Mock Lenta" }
            fn eh_autorizada(&self) -> bool { true }
            async fn buscar_obras(&self, _: &str) -> Result<Vec<ObraEncontrada>, crate::fonte::FonteError> { Ok(vec![]) }
            async fn listar_capitulos(&self, _: &str) -> Result<Vec<CapituloEncontrado>, crate::fonte::FonteError> { Ok(vec![]) }
            async fn obter_capitulo(&self, _: &str) -> Result<Vec<u8>, crate::fonte::FonteError> {
                let mut cur = std::io::Cursor::new(Vec::new());
                {
                    let mut zw = zip::ZipWriter::new(&mut cur);
                    zw.start_file("0001.jpg", zip::write::FileOptions::default()).unwrap();
                    zw.write_all(b"JPG").unwrap();
                    zw.finish().unwrap();
                }
                Ok(cur.into_inner())
            }
        }

        let base = std::env::temp_dir().join(format!("mam_test_lifecycle_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();
        let db = Arc::new(Database::abrir(base.join("db.sqlite"), base.clone()).await.unwrap());
        let sm = Arc::new(SourceIntegrationManager::nova(db.clone()));
        sm.registrar_fonte(Arc::new(FonteLentaMock)).await.unwrap();

        let caps = vec![
            CapituloBuscaResultado {
                fonte_id: "mock-lenta".to_string(),
                obra_id_fonte: "o1".to_string(),
                capitulo_id_fonte: "c1".to_string(),
                numero: Some(1),
                titulo: Some("Cap 1".to_string()),
                url: None,
                idioma: Some("PT".to_string()),
            },
            CapituloBuscaResultado {
                fonte_id: "mock-lenta".to_string(),
                obra_id_fonte: "o1".to_string(),
                capitulo_id_fonte: "c2".to_string(),
                numero: Some(2),
                titulo: Some("Cap 2".to_string()),
                url: None,
                idioma: Some("PT".to_string()),
            },
        ];

        // 1. Inicia operação e pausa automaticamente (simulando fechamento do app)
        let op_id = sm.iniciar_operacao_arquivamento("Obra Pausa".to_string(), caps.clone()).await.unwrap();
        let pausadas = sm.pausar_todas_operacoes_ativas().await;
        assert_eq!(pausadas, 1);

        // 2. Simula reinício do app com novo SourceIntegrationManager carregando do SQLite
        let sm_novo = Arc::new(SourceIntegrationManager::nova(db.clone()));
        sm_novo.registrar_fonte(Arc::new(FonteLentaMock)).await.unwrap();
        let restauradas = sm_novo.carregar_operacoes_pausadas_do_banco().await;
        assert_eq!(restauradas, 1);
        let st_rest = sm_novo.status_operacao(&op_id).await.unwrap().unwrap();
        assert!(st_rest.pausada);
        assert_eq!(st_rest.estado, "pausado");

        // 3. Registrar um log ligado à operação pausada + um log antigo, e limpar o log:
        //    o log da operação pausada deve ser preservado!
        db.registrar_operacao(
            "download".to_string(),
            None,
            Some(serde_json::json!({"operacao_id": op_id, "nome": "Cap em Pausa"}).to_string()),
            None,
        ).await.unwrap();
        db.registrar_operacao(
            "download".to_string(),
            None,
            Some(serde_json::json!({"operacao_id": "op-antiga", "nome": "Cap Antigo"}).to_string()),
            None,
        ).await.unwrap();
        db.limpar_historico(Some("download")).await.unwrap();
        let logs = db.listar_historico_por_tipo("download").await.unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].2.as_deref().unwrap().contains(&op_id));

        // 4. Despausa e executa parcialmente / testa rollback ao cancelar
        let (_, caps_ret, capa_ret) = sm_novo.preparar_retomada_operacao(&op_id).await.unwrap().unwrap();
        sm_novo.executar_arquivamento_operacao(&op_id, &base, "Obra Pausa".to_string(), caps_ret, capa_ret).await.unwrap();
        assert!(base.join("Manga/Obra Pausa/PT/Capitulo_0001.cbz").exists());

        // Cancela o conjunto e valida rollback estilo Docker (apaga CBZs, obra e logs do conjunto)
        sm_novo.cancelar_operacao(&op_id, &base).await.unwrap();
        assert!(!base.join("Manga/Obra Pausa/PT/Capitulo_0001.cbz").exists());
        assert!(!base.join("Manga/Obra Pausa/PT/Capitulo_0002.cbz").exists());
        assert!(db.buscar_obra_por_titulo("Obra Pausa").await.unwrap().is_none());

        let _ = std::fs::remove_dir_all(&base);
    }
}
