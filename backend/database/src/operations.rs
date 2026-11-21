//! Operações CRUD do banco de dados MAM.
//! Todas as operações utilizam transações curtas e são executadas dentro do contexto async
//! protegidas pelo Mutex da conexão (rusqlite é síncrono, mas as transações são curtas).

use crate::Database;
use crate::models::*;
use chrono::{DateTime, Utc};
use mam_core::{CapituloId, CaminhoRelativo, EstadoIntegridade, ObraId, VolumeId};
use rusqlite::params;
use uuid::Uuid;

/// Resultado genérico de operações de banco.
pub type DbResult<T> = Result<T, crate::DatabaseError>;

impl Database {
    // ── Obras ────

    pub async fn criar_obra(
        &self,
        id: ObraId,
        titulo: String,
        caminho_relativo: CaminhoRelativo,
        origem: Option<String>,
    ) -> DbResult<Obra> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO obras (id, titulo, caminho_relativo, origem, data_arquivamento, estado_integridade)
             VALUES (?1, ?2, ?3, ?4, datetime('now'), ?5)",
            params![
                id.to_string(),
                titulo,
                caminho_relativo.as_str(),
                origem,
                EstadoIntegridade::Desconhecido.as_str(),
            ],
        )?;
        tx.commit()?;
        Ok(Obra {
            id,
            titulo,
            caminho_relativo,
            origem,
            data_arquivamento: Utc::now(),
            estado_integridade: EstadoIntegridade::Desconhecido,
            metadados_complementares: None,
        })
    }

    pub async fn listar_obras(&self) -> DbResult<Vec<Obra>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, titulo, caminho_relativo, origem, data_arquivamento, estado_integridade, metadados_complementares FROM obras ORDER BY titulo"
        )?;
        let obras = stmt.query_map([], |row| {
            Ok(Obra {
                id: ObraId::from_string(row.get(0)?),
                titulo: row.get(1)?,
                caminho_relativo: CaminhoRelativo::from_str(row.get(2)?),
                origem: row.get(3)?,
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(5)?),
                metadados_complementares: row.get(6)?,
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(obras)
    }

    pub async fn buscar_obra_por_id(&self, id: ObraId) -> DbResult<Option<Obra>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, titulo, caminho_relativo, origem, data_arquivamento, estado_integridade, metadados_complementares FROM obras WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id.to_string()], |row| {
            Ok(Obra {
                id: ObraId::from_string(row.get(0)?),
                titulo: row.get(1)?,
                caminho_relativo: CaminhoRelativo::from_str(row.get(2)?),
                origem: row.get(3)?,
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(5)?),
                metadados_complementares: row.get(6)?,
            })
        })?;
        let obra = rows.next().transpose()?;
        Ok(obra)
    }

    // ── Capítulos ────

    pub async fn criar_capitulo(
        &self,
        id: CapituloId,
        volume_id: Option<VolumeId>,
        obra_id: ObraId,
        numero: i32,
        titulo_original: Option<String>,
        caminho_relativo: CaminhoRelativo,
        quantidade_paginas: i32,
    ) -> DbResult<Capitulo> {
        self.criar_capitulo_com_idioma(
            id,
            volume_id,
            obra_id,
            numero,
            titulo_original,
            caminho_relativo,
            quantidade_paginas,
            None,
        )
        .await
    }

    pub async fn criar_capitulo_com_idioma(
        &self,
        id: CapituloId,
        volume_id: Option<VolumeId>,
        obra_id: ObraId,
        numero: i32,
        titulo_original: Option<String>,
        caminho_relativo: CaminhoRelativo,
        quantidade_paginas: i32,
        idioma: Option<String>,
    ) -> DbResult<Capitulo> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO capitulos (id, volume_id, obra_id, numero, titulo_original, caminho_relativo, quantidade_paginas, idioma, data_arquivamento, estado_integridade)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), ?9)",
            params![
                id.to_string(),
                volume_id.map(|v| v.to_string()),
                obra_id.to_string(),
                numero,
                titulo_original,
                caminho_relativo.as_str(),
                quantidade_paginas,
                idioma,
                EstadoIntegridade::Desconhecido.as_str(),
            ],
        )?;
        tx.commit()?;
        Ok(Capitulo {
            id,
            volume_id,
            obra_id,
            numero,
            titulo_original,
            caminho_relativo,
            quantidade_paginas,
            idioma,
            data_arquivamento: Utc::now(),
            estado_integridade: EstadoIntegridade::Desconhecido,
        })
    }

    pub async fn listar_capitulos_obra(&self, obra_id: ObraId) -> DbResult<Vec<Capitulo>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, volume_id, obra_id, numero, titulo_original, caminho_relativo, quantidade_paginas, idioma, data_arquivamento, estado_integridade FROM capitulos WHERE obra_id = ?1 ORDER BY numero"
        )?;
        let capitulos = stmt.query_map(params![obra_id.to_string()], |row| {
            Ok(Capitulo {
                id: CapituloId::from_string(row.get(0)?),
                volume_id: row.get::<_, Option<String>>(1)?.map(|v| VolumeId::from_string(v)),
                obra_id: ObraId::from_string(row.get(2)?),
                numero: row.get(3)?,
                titulo_original: row.get(4)?,
                caminho_relativo: CaminhoRelativo::from_str(row.get(5)?),
                quantidade_paginas: row.get(6)?,
                idioma: row.get(7)?,
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(9)?),
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(capitulos)
    }

    // ── Integridade ────

    pub async fn registrar_integridade(
        &self,
        caminho_relativo: &str,
        hash_sha256: &str,
        estado: &str,
    ) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO integridade_arquivo (caminho_relativo, hash_sha256, estado, verificado_em)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![caminho_relativo, hash_sha256, estado],
        )?;
        Ok(())
    }

    pub async fn consultar_integridade(&self, caminho_relativo: &str) -> DbResult<Option<IntegridadeArquivo>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT caminho_relativo, hash_sha256, estado, verificado_em FROM integridade_arquivo WHERE caminho_relativo = ?1"
        )?;
        let mut rows = stmt.query_map(params![caminho_relativo], |row| {
            Ok(IntegridadeArquivo {
                caminho_relativo: row.get(0)?,
                hash_sha256: row.get(1)?,
                estado: row.get(2)?,
                verificado_em: row.get::<_, Option<String>>(3)?.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(Utc::now())
                }),
            })
        })?;
        let resultado = rows.next().transpose()?;
        Ok(resultado)
    }

    // ── Histórico ────

    pub async fn registrar_operacao(
        &self,
        tipo: String,
        alvo_caminho: Option<String>,
        detalhes: Option<String>,
        usuario: Option<String>,
    ) -> DbResult<Uuid> {
        let id = Uuid::new_v4();
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO historico_operacoes (id, tipo, alvo_caminho, detalhes_json, usuario, criado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            params![
                id.to_string(),
                tipo,
                alvo_caminho,
                detalhes,
                usuario,
            ],
        )?;
        Ok(id)
    }

    // ── Busca de obra por título ──

    pub async fn buscar_obra_por_titulo(&self, titulo: &str) -> DbResult<Option<Obra>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, titulo, caminho_relativo, origem, data_arquivamento, estado_integridade, metadados_complementares FROM obras WHERE titulo = ?1"
        )?;
        let mut rows = stmt.query_map(params![titulo], |row| {
            Ok(Obra {
                id: ObraId::from_string(row.get(0)?),
                titulo: row.get(1)?,
                caminho_relativo: CaminhoRelativo::from_str(row.get(2)?),
                origem: row.get(3)?,
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(5)?),
                metadados_complementares: row.get(6)?,
            })
        })?;
        let obra = rows.next().transpose()?;
        Ok(obra)
    }

    // ── Criação de volume ──

    pub async fn criar_volume(
        &self,
        id: VolumeId,
        obra_id: ObraId,
        titulo: String,
        caminho_relativo: CaminhoRelativo,
    ) -> DbResult<Volume> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO volumes (id, obra_id, numero, caminho_relativo, titulo)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                obra_id.to_string(),
                0, // número será atualizado posteriormente
                caminho_relativo.as_str(),
                titulo,
            ],
        )?;
        tx.commit()?;
        Ok(Volume {
            id,
            obra_id,
            numero: 0,
            caminho_relativo,
            titulo: Some(titulo),
        })
    }

    pub async fn criar_volume_com_numero(
        &self,
        id: VolumeId,
        obra_id: ObraId,
        numero: i32,
        titulo: Option<String>,
        caminho_relativo: CaminhoRelativo,
    ) -> DbResult<Volume> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO volumes (id, obra_id, numero, caminho_relativo, titulo)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                obra_id.to_string(),
                numero,
                caminho_relativo.as_str(),
                titulo,
            ],
        )?;
        tx.commit()?;
        Ok(Volume {
            id,
            obra_id,
            numero,
            caminho_relativo,
            titulo,
        })
    }

    pub async fn listar_volumes_obra(&self, obra_id: ObraId) -> DbResult<Vec<Volume>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, obra_id, numero, caminho_relativo, titulo FROM volumes WHERE obra_id = ?1 ORDER BY numero"
        )?;
        let volumes = stmt
            .query_map(params![obra_id.to_string()], |row| {
                Ok(Volume {
                    id: VolumeId::from_string(row.get(0)?),
                    obra_id: ObraId::from_string(row.get(1)?),
                    numero: row.get(2)?,
                    caminho_relativo: CaminhoRelativo::from_str(row.get(3)?),
                    titulo: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(volumes)
    }

    pub async fn atualizar_metadados_obra(&self, id: ObraId, metadados_json: &str) -> DbResult<()> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE obras SET metadados_complementares = ?1, atualizado_em = datetime('now') WHERE id = ?2",
            params![metadados_json, id.to_string()],
        )?;
        tx.commit()?;
        Ok(())
    }

    // ── Configurações chave-valor (estado do leitor, favoritos, marcadores) ──

    pub async fn salvar_configuracao(&self, chave: &str, valor: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO configuracao_biblioteca (chave, valor, atualizado_em)
             VALUES (?1, ?2, datetime('now'))",
            params![chave, valor],
        )?;
        Ok(())
    }

    pub async fn obter_configuracao(&self, chave: &str) -> DbResult<Option<String>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT valor FROM configuracao_biblioteca WHERE chave = ?1",
        )?;
        let mut rows = stmt.query_map(params![chave], |row| row.get(0))?;
        Ok(rows.next().transpose()?)
    }

    pub async fn listar_configuracoes_por_prefixo(
        &self,
        prefixo: &str,
    ) -> DbResult<Vec<(String, String)>> {
        let conn = self.conn.lock().await;
        let padrao = format!("{}%", prefixo);
        let mut stmt = conn.prepare(
            "SELECT chave, valor FROM configuracao_biblioteca WHERE chave LIKE ?1 ORDER BY chave",
        )?;
        let itens = stmt
            .query_map(params![padrao], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(itens)
    }

    /// Lista todos os marcadores de todos os capítulos de uma obra em uma única consulta SQL.
    pub async fn listar_marcadores_por_obra(
        &self,
        obra_id: ObraId,
    ) -> DbResult<Vec<(String, usize, String)>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT c.id, cfg.chave, cfg.valor
             FROM configuracao_biblioteca cfg
             INNER JOIN capitulos c
               ON cfg.chave LIKE 'reader:marcador:' || c.id || ':%'
             WHERE c.obra_id = ?1
             ORDER BY c.numero ASC, cfg.chave ASC",
        )?;
        let rows = stmt
            .query_map(params![obra_id.to_string()], |row| {
                let cap_id: String = row.get(0)?;
                let chave: String = row.get(1)?;
                let nota: String = row.get(2)?;
                Ok((cap_id, chave, nota))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rows
            .into_iter()
            .filter_map(|(cap_id, chave, nota)| {
                chave
                    .rsplit(':')
                    .next()
                    .and_then(|p| p.parse::<usize>().ok())
                    .map(|pagina| (cap_id, pagina, nota))
            })
            .collect())
    }

    /// Remove uma chave de configuração pelo nome exato.
    pub async fn remover_configuracao(&self, chave: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM configuracao_biblioteca WHERE chave = ?1",
            params![chave],
        )?;
        Ok(())
    }

    // ── Resultados de IA (OCR, balões, tradução e revisão) ──

    pub async fn salvar_resultado_ia(
        &self,
        capitulo_id: CapituloId,
        pagina_numero: i32,
        modelo_id: &str,
        versao_modelo: &str,
        tipo_processamento: &str,
        resultado_json: &str,
        idioma_origem: Option<&str>,
        idioma_destino: Option<&str>,
    ) -> DbResult<ResultadoIA> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO resultados_ia (id, capitulo_id, pagina_numero, modelo_id, versao_modelo, tipo_processamento, resultado_json, idioma_origem, idioma_destino, estado_revisao)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'nao_revisado')",
            params![
                id,
                capitulo_id.to_string(),
                pagina_numero,
                modelo_id,
                versao_modelo,
                tipo_processamento,
                resultado_json,
                idioma_origem,
                idioma_destino,
            ],
        )?;
        Ok(ResultadoIA {
            id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil()),
            capitulo_id,
            pagina_numero,
            modelo_id: modelo_id.to_string(),
            versao_modelo: versao_modelo.to_string(),
            tipo_processamento: tipo_processamento.to_string(),
            resultado_json: resultado_json.to_string(),
            idioma_origem: idioma_origem.map(|s| s.to_string()),
            idioma_destino: idioma_destino.map(|s| s.to_string()),
            estado_revisao: "nao_revisado".to_string(),
            criado_em: Utc::now(),
        })
    }

    pub async fn listar_resultados_ia_capitulo(
        &self,
        capitulo_id: CapituloId,
    ) -> DbResult<Vec<ResultadoIA>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, capitulo_id, pagina_numero, modelo_id, versao_modelo, tipo_processamento, resultado_json, idioma_origem, idioma_destino, estado_revisao
             FROM resultados_ia WHERE capitulo_id = ?1 ORDER BY pagina_numero",
        )?;
        let itens = stmt
            .query_map(params![capitulo_id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let cap_str: String = row.get(1)?;
                Ok(ResultadoIA {
                    id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::nil()),
                    capitulo_id: CapituloId::from_string(cap_str),
                    pagina_numero: row.get(2)?,
                    modelo_id: row.get(3)?,
                    versao_modelo: row.get(4)?,
                    tipo_processamento: row.get(5)?,
                    resultado_json: row.get(6)?,
                    idioma_origem: row.get(7)?,
                    idioma_destino: row.get(8)?,
                    estado_revisao: row.get(9)?,
                    criado_em: Utc::now(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(itens)
    }

    pub async fn revisar_resultado_ia(
        &self,
        resultado_id: &str,
        resultado_json_revisado: &str,
    ) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE resultados_ia SET resultado_json = ?1, estado_revisao = 'revisado' WHERE id = ?2",
            params![resultado_json_revisado, resultado_id],
        )?;
        Ok(())
    }

    /// Exclui um capítulo do catálogo e retorna o caminho relativo caso nenhum outro capítulo o referencie.
    pub async fn excluir_capitulo(&self, capitulo_id: CapituloId) -> DbResult<Option<String>> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        let caminho: Option<String> = {
            let mut stmt = tx.prepare("SELECT caminho_relativo FROM capitulos WHERE id = ?1")?;
            let mut rows = stmt.query_map(params![capitulo_id.to_string()], |row| row.get(0))?;
            rows.next().transpose()?
        };

        tx.execute(
            "DELETE FROM capitulos WHERE id = ?1",
            params![capitulo_id.to_string()],
        )?;

        let pode_remover_arquivo = if let Some(ref rel) = caminho {
            let restantes: i64 = tx.query_row(
                "SELECT COUNT(*) FROM capitulos WHERE caminho_relativo = ?1",
                params![rel],
                |row| row.get(0),
            )?;
            restantes == 0
        } else {
            false
        };

        tx.commit()?;
        Ok(if pode_remover_arquivo { caminho } else { None })
    }

    /// Retorna a quantidade de capítulos cadastrados por `obra_id`.
    pub async fn contar_capitulos_por_obra(&self) -> DbResult<std::collections::HashMap<String, usize>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT obra_id, COUNT(*) FROM capitulos GROUP BY obra_id",
        )?;
        let mut mapa = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            let obra_id: String = row.get(0)?;
            let qtd: i64 = row.get(1)?;
            Ok((obra_id, qtd.max(0) as usize))
        })?;
        for item in rows {
            let (id, qtd) = item?;
            mapa.insert(id, qtd);
        }
        Ok(mapa)
    }

    /// Lista registros de histórico por tipo (`download`, `exclusao`, etc.) do mais recente ao mais antigo.
    pub async fn listar_historico_por_tipo(
        &self,
        tipo: &str,
    ) -> DbResult<Vec<(String, Option<String>, Option<String>, String)>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, alvo_caminho, detalhes_json, criado_em FROM historico_operacoes
             WHERE tipo = ?1 ORDER BY criado_em DESC LIMIT 200",
        )?;
        let itens = stmt
            .query_map(params![tipo], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(itens)
    }

    /// Lista registros de downloads ordenados do mais recente para o mais antigo.
    pub async fn listar_historico_downloads(
        &self,
    ) -> DbResult<Vec<(String, Option<String>, Option<String>, String)>> {
        self.listar_historico_por_tipo("download").await
    }

    /// Lista registros de exclusões ordenados do mais recente para o mais antigo.
    pub async fn listar_historico_exclusoes(
        &self,
    ) -> DbResult<Vec<(String, Option<String>, Option<String>, String)>> {
        self.listar_historico_por_tipo("exclusao").await
    }

    /// Obtém um capítulo pelo ID.
    pub async fn obter_capitulo_por_id(&self, capitulo_id: CapituloId) -> DbResult<Option<Capitulo>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, volume_id, obra_id, numero, titulo_original, caminho_relativo, quantidade_paginas, idioma, data_arquivamento, estado_integridade FROM capitulos WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![capitulo_id.to_string()], |row| {
            Ok(Capitulo {
                id: CapituloId::from_string(row.get(0)?),
                volume_id: row.get::<_, Option<String>>(1)?.map(VolumeId::from_string),
                obra_id: ObraId::from_string(row.get(2)?),
                numero: row.get(3)?,
                titulo_original: row.get(4)?,
                caminho_relativo: CaminhoRelativo::from_str(row.get(5)?),
                quantidade_paginas: row.get(6)?,
                idioma: row.get(7)?,
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(9)?),
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// Exclui uma obra inteira do catálogo (CASCADE remove volumes e capítulos) e retorna `(titulo, caminho_relativo)`.
    pub async fn excluir_obra(&self, obra_id: ObraId) -> DbResult<Option<(String, String)>> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        let info: Option<(String, String)> = {
            let mut stmt = tx.prepare("SELECT titulo, caminho_relativo FROM obras WHERE id = ?1")?;
            let mut rows = stmt.query_map(params![obra_id.to_string()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            rows.next().transpose()?
        };
        if info.is_some() {
            tx.execute("DELETE FROM obras WHERE id = ?1", params![obra_id.to_string()])?;
        }
        tx.commit()?;
        Ok(info)
    }

    /// Atualiza número e título de um volume.
    pub async fn atualizar_volume(
        &self,
        volume_id: VolumeId,
        numero: i32,
        titulo: Option<String>,
    ) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE volumes SET numero = ?1, titulo = ?2 WHERE id = ?3",
            params![numero, titulo, volume_id.to_string()],
        )?;
        Ok(())
    }

    /// Vincula uma lista de capítulos a um volume (ou `None` para desvincular).
    pub async fn vincular_capitulos_volume(
        &self,
        volume_id: Option<VolumeId>,
        capitulo_ids: &[CapituloId],
    ) -> DbResult<()> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        let vol_str = volume_id.map(|v| v.to_string());
        for cap_id in capitulo_ids {
            tx.execute(
                "UPDATE capitulos SET volume_id = ?1 WHERE id = ?2",
                params![vol_str, cap_id.to_string()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Renomeia o título de um capítulo baixado.
    pub async fn renomear_capitulo(
        &self,
        capitulo_id: CapituloId,
        novo_titulo: Option<String>,
    ) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE capitulos SET titulo_original = ?1 WHERE id = ?2",
            params![novo_titulo, capitulo_id.to_string()],
        )?;
        Ok(())
    }

    /// Exclui um volume pelo ID (capítulos vinculados permanecem na obra com `volume_id = NULL`) e retorna `(numero, titulo)`.
    pub async fn excluir_volume(&self, volume_id: VolumeId) -> DbResult<Option<(i32, Option<String>)>> {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        let info: Option<(i32, Option<String>)> = {
            let mut stmt = tx.prepare("SELECT numero, titulo FROM volumes WHERE id = ?1")?;
            let mut rows = stmt.query_map(params![volume_id.to_string()], |row| {
                Ok((row.get::<_, i32>(0)?, row.get::<_, Option<String>>(1)?))
            })?;
            rows.next().transpose()?
        };
        if info.is_some() {
            tx.execute("DELETE FROM volumes WHERE id = ?1", params![volume_id.to_string()])?;
        }
        tx.commit()?;
        Ok(info)
    }

    /// Busca um capítulo pelo `caminho_relativo` para evitar duplicatas no catálogo.
    pub async fn buscar_capitulo_por_caminho(
        &self,
        caminho_relativo: &str,
    ) -> DbResult<Option<Capitulo>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, volume_id, obra_id, numero, titulo_original, caminho_relativo, quantidade_paginas, idioma, data_arquivamento, estado_integridade FROM capitulos WHERE caminho_relativo = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![caminho_relativo], |row| {
            Ok(Capitulo {
                id: CapituloId::from_string(row.get(0)?),
                volume_id: row.get::<_, Option<String>>(1)?.map(VolumeId::from_string),
                obra_id: ObraId::from_string(row.get(2)?),
                numero: row.get(3)?,
                titulo_original: row.get(4)?,
                caminho_relativo: CaminhoRelativo::from_str(row.get(5)?),
                quantidade_paginas: row.get(6)?,
                idioma: row.get(7)?,
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(9)?),
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// Atualiza páginas e título de um capítulo já existente no catálogo.
    pub async fn atualizar_capitulo_arquivado(
        &self,
        capitulo_id: CapituloId,
        titulo_original: Option<String>,
        quantidade_paginas: i32,
        idioma: Option<String>,
    ) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE capitulos SET titulo_original = ?1, quantidade_paginas = ?2, idioma = COALESCE(?3, idioma), data_arquivamento = datetime('now') WHERE id = ?4",
            params![titulo_original, quantidade_paginas, idioma, capitulo_id.to_string()],
        )?;
        Ok(())
    }

    /// Lista registros de renomeações ordenados do mais recente para o mais antigo.
    pub async fn listar_historico_renomeacoes(
        &self,
    ) -> DbResult<Vec<(String, Option<String>, Option<String>, String)>> {
        self.listar_historico_por_tipo("renomeacao").await
    }

    /// Limpa registros do histórico de operações preservando registros vinculados a
    /// operações de download em andamento ou pausadas.
    pub async fn limpar_historico(&self, tipo: Option<&str>) -> DbResult<()> {
        let conn = self.conn.lock().await;
        Self::garantir_tabela_operacoes_estado(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT id FROM operacoes_arquivamento_estado WHERE estado IN ('executando', 'pausado')",
        )?;
        let ids_protegidos: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        drop(stmt);

        if ids_protegidos.is_empty() {
            match tipo {
                Some(t) => {
                    conn.execute("DELETE FROM historico_operacoes WHERE tipo = ?1", params![t])?;
                }
                None => {
                    conn.execute("DELETE FROM historico_operacoes", [])?;
                }
            }
        } else {
            let mut stmt_hist = match tipo {
                Some(t) => {
                    let mut s = conn.prepare(
                        "SELECT id, COALESCE(detalhes_json, '') FROM historico_operacoes WHERE tipo = ?1",
                    )?;
                    let rows: Vec<(String, String)> = s
                        .query_map(params![t], |r| Ok((r.get(0)?, r.get(1)?)))?
                        .filter_map(Result::ok)
                        .collect();
                    rows
                }
                None => {
                    let mut s = conn.prepare(
                        "SELECT id, COALESCE(detalhes_json, '') FROM historico_operacoes",
                    )?;
                    let rows: Vec<(String, String)> = s
                        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                        .filter_map(Result::ok)
                        .collect();
                    rows
                }
            };
            for (row_id, detalhes) in stmt_hist.drain(..) {
                let protegido = ids_protegidos.iter().any(|op_id| detalhes.contains(op_id));
                if !protegido {
                    conn.execute("DELETE FROM historico_operacoes WHERE id = ?1", params![row_id])?;
                }
            }
        }
        Ok(())
    }

    fn garantir_tabela_operacoes_estado(conn: &rusqlite::Connection) -> DbResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS operacoes_arquivamento_estado (
                id TEXT PRIMARY KEY,
                obra_titulo TEXT NOT NULL,
                estado TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                atualizado_em TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Salva ou atualiza o estado persistido de uma operação de arquivamento (pausa/retomada/rollback).
    pub async fn salvar_estado_operacao_arquivamento(
        &self,
        id: &str,
        obra_titulo: &str,
        estado: &str,
        payload_json: &str,
    ) -> DbResult<()> {
        let conn = self.conn.lock().await;
        Self::garantir_tabela_operacoes_estado(&conn)?;
        conn.execute(
            "INSERT INTO operacoes_arquivamento_estado (id, obra_titulo, estado, payload_json, atualizado_em)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
               obra_titulo = excluded.obra_titulo,
               estado = excluded.estado,
               payload_json = excluded.payload_json,
               atualizado_em = excluded.atualizado_em",
            params![id, obra_titulo, estado, payload_json],
        )?;
        Ok(())
    }

    /// Lista operações de arquivamento persistidas (para retomada após fechar o app).
    pub async fn listar_estados_operacoes_arquivamento(
        &self,
    ) -> DbResult<Vec<(String, String, String, String)>> {
        let conn = self.conn.lock().await;
        Self::garantir_tabela_operacoes_estado(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, obra_titulo, estado, payload_json FROM operacoes_arquivamento_estado ORDER BY atualizado_em DESC",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
            .filter_map(Result::ok)
            .collect();
        Ok(rows)
    }

    /// Remove uma operação persistida (após conclusão ou cancelamento com rollback).
    pub async fn remover_estado_operacao_arquivamento(&self, id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        Self::garantir_tabela_operacoes_estado(&conn)?;
        conn.execute(
            "DELETE FROM operacoes_arquivamento_estado WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Remove um capítulo pelo ID (usado em rollback de cancelamento).
    pub async fn excluir_capitulo_por_id(&self, id: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM capitulos WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Remove uma obra pelo ID caso tenha ficado sem capítulos após rollback.
    pub async fn excluir_obra_se_vazia(&self, obra_id: &str) -> DbResult<bool> {
        let conn = self.conn.lock().await;
        let qtd: i64 = conn.query_row(
            "SELECT COUNT(*) FROM capitulos WHERE obra_id = ?1",
            params![obra_id],
            |r| r.get(0),
        )?;
        if qtd == 0 {
            conn.execute("DELETE FROM obras WHERE id = ?1", params![obra_id])?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Remove registros do histórico associados a uma operação cancelada.
    pub async fn remover_historico_por_operacao(&self, id_operacao: &str) -> DbResult<()> {
        let conn = self.conn.lock().await;
        let like_pat = format!("%{}%", id_operacao);
        conn.execute(
            "DELETE FROM historico_operacoes WHERE detalhes_json LIKE ?1",
            params![like_pat],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn criar_db_teste() -> Database {
        let temp_dir = env::temp_dir().join(format!("mam_test_ops_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("database.db");
        Database::abrir(db_path, temp_dir.clone()).await.expect("abrir banco teste")
    }

    #[tokio::test]
    async fn criar_e_listar_obra() {
        let db = criar_db_teste().await;
        let id = ObraId::new();
        let caminho = CaminhoRelativo::from_str("Manga/Obra A".to_string());
        db.criar_obra(id, "Obra Teste".to_string(), caminho, Some("fonte-teste".to_string()))
            .await
            .expect("criar obra");

        let obras = db.listar_obras().await.expect("listar obras");
        assert_eq!(obras.len(), 1);
        assert_eq!(obras[0].titulo, "Obra Teste");
    }

    #[tokio::test]
    async fn criar_capitulo_sem_volume() {
        let db = criar_db_teste().await;
        let obra_id = ObraId::new();
        let capitulo_id = CapituloId::new();
        let caminho = CaminhoRelativo::from_str("Manga/Obra A/Capitulo 0001.cbz".to_string());

        db.criar_obra(obra_id, "Obra Teste".to_string(), caminho.clone(), None)
            .await
            .expect("criar obra");

        let cap = db
            .criar_capitulo(
                capitulo_id,
                None,
                obra_id,
                1,
                Some("Capítulo 1".to_string()),
                caminho,
                20,
            )
            .await
            .expect("criar capítulo");

        assert_eq!(cap.numero, 1);
        assert_eq!(cap.volume_id, None);

        let caps = db.listar_capitulos_obra(obra_id).await.expect("listar capítulos");
        assert_eq!(caps.len(), 1);
    }

    #[tokio::test]
    async fn registrar_e_consultar_integridade() {
        let db = criar_db_teste().await;
        let caminho = "Manga/Obra A/Cap 001.cbz";
        db.registrar_integridade(caminho, "abc123", "valido")
            .await
            .expect("registrar integridade");

        let resultado = db.consultar_integridade(caminho).await.expect("consultar integridade");
        assert!(resultado.is_some());
        assert_eq!(resultado.unwrap().estado, "valido");
    }

    #[tokio::test]
    async fn configuracao_progresso_favorito_marcador() {
        let db = criar_db_teste().await;

        db.salvar_configuracao("reader:progresso:cap-1", "12")
            .await
            .expect("salvar progresso");
        assert_eq!(
            db.obter_configuracao("reader:progresso:cap-1").await.unwrap(),
            Some("12".to_string())
        );

        db.salvar_configuracao("reader:favorito:obra-1", "true")
            .await
            .expect("salvar favorito");
        db.salvar_configuracao("reader:marcador:cap-1:5", "Cena importante")
            .await
            .expect("salvar marcador");

        let marcadores = db
            .listar_configuracoes_por_prefixo("reader:marcador:cap-1:")
            .await
            .expect("listar marcadores");
        assert_eq!(marcadores.len(), 1);
        assert_eq!(marcadores[0].1, "Cena importante");
    }

    #[tokio::test]
    async fn salvar_e_revisar_resultado_ia() {
        let db = criar_db_teste().await;
        let obra_id = ObraId::new();
        let cap_id = CapituloId::new();
        let caminho = CaminhoRelativo::from_str("Manga/Obra/Cap 01.cbz".to_string());
        db.criar_obra(obra_id, "Obra IA".to_string(), caminho.clone(), None)
            .await
            .unwrap();
        db.criar_capitulo(cap_id, None, obra_id, 1, None, caminho, 10)
            .await
            .unwrap();

        let res = db
            .salvar_resultado_ia(
                cap_id,
                0,
                "ocr-ja",
                "1.0",
                "translation",
                r#"{"texto":"olá"}"#,
                Some("ja"),
                Some("pt-BR"),
            )
            .await
            .expect("salvar resultado ia");
        assert_eq!(res.estado_revisao, "nao_revisado");

        db.revisar_resultado_ia(&res.id.to_string(), r#"{"texto":"Olá!"}"#)
            .await
            .expect("revisar resultado ia");

        let lista = db.listar_resultados_ia_capitulo(cap_id).await.unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].estado_revisao, "revisado");
        assert_eq!(lista[0].resultado_json, r#"{"texto":"Olá!"}"#);
    }
}
