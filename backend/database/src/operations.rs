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
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO capitulos (id, volume_id, obra_id, numero, titulo_original, caminho_relativo, quantidade_paginas, data_arquivamento, estado_integridade)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), ?8)",
            params![
                id.to_string(),
                volume_id.map(|v| v.to_string()),
                obra_id.to_string(),
                numero,
                titulo_original,
                caminho_relativo.as_str(),
                quantidade_paginas,
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
            data_arquivamento: Utc::now(),
            estado_integridade: EstadoIntegridade::Desconhecido,
        })
    }

    pub async fn listar_capitulos_obra(&self, obra_id: ObraId) -> DbResult<Vec<Capitulo>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, volume_id, obra_id, numero, titulo_original, caminho_relativo, quantidade_paginas, data_arquivamento, estado_integridade FROM capitulos WHERE obra_id = ?1 ORDER BY numero"
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
                data_arquivamento: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
                estado_integridade: EstadoIntegridade::from_str(&row.get::<_, String>(8)?),
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
        detalhes: Option<serde_json::Value>,
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
                detalhes.map(|d| d.to_string()),
                usuario,
            ],
        )?;
        Ok(id)
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
}
