//! # mam-library
//! Gerenciamento da biblioteca digital: catálogo de obras, volumes e capítulos,
//! metadados complementares e organização híbrida de volumes com caminhos relativos.

pub mod catalogo;
pub mod metadados;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use mam_core::{ObraId, VolumeId};
use mam_database::models::{Capitulo, Obra, Volume};
use mam_database::Database;

pub use catalogo::{DetalhesObra, VolumeComCapitulos};

/// Erros de domínio do Library Manager.
#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Erro de banco de dados: {0}")]
    Banco(#[from] mam_database::DatabaseError),

    #[error("Caminho inválido: {0}")]
    CaminhoInvalido(String),

    #[error("Erro de validação: {0}")]
    Validacao(String),

    #[error("Obra não encontrada: {0}")]
    ObraNaoEncontrada(String),

    #[error("Metadados inválidos: {0}")]
    MetadadosInvalidos(String),
}

/// Ponto de entrada do gerenciamento da biblioteca digital.
pub struct LibraryManager {
    db: Arc<Database>,
    raiz_biblioteca: PathBuf,
}

impl LibraryManager {
    /// Cria uma nova instância de `LibraryManager` vinculada ao banco e à raiz da biblioteca.
    pub fn nova(db: Arc<Database>, raiz_biblioteca: PathBuf) -> Self {
        Self { db, raiz_biblioteca }
    }

    /// Retorna o caminho da raiz da biblioteca configurada.
    pub fn raiz(&self) -> &Path {
        &self.raiz_biblioteca
    }

    /// Cadastra uma nova obra no catálogo utilizando caminho relativo.
    pub async fn criar_obra(
        &self,
        titulo: String,
        caminho_relativo: String,
        origem: Option<String>,
    ) -> Result<Obra, LibraryError> {
        catalogo::criar_obra(&self.db, titulo, caminho_relativo, origem).await
    }

    /// Lista todas as obras cadastradas no catálogo.
    pub async fn listar_obras(&self) -> Result<Vec<Obra>, LibraryError> {
        catalogo::listar_obras(&self.db).await
    }

    /// Cria um volume vinculado a uma obra existente.
    pub async fn criar_volume(
        &self,
        obra_id: ObraId,
        numero: i32,
        titulo: Option<String>,
        caminho_relativo: String,
    ) -> Result<Volume, LibraryError> {
        catalogo::criar_volume(&self.db, obra_id, numero, titulo, caminho_relativo).await
    }

    /// Registra um capítulo associado a um volume ou diretamente à obra.
    pub async fn registrar_capitulo(
        &self,
        obra_id: ObraId,
        volume_id: Option<VolumeId>,
        numero: i32,
        titulo_original: Option<String>,
        caminho_relativo: String,
        quantidade_paginas: i32,
    ) -> Result<Capitulo, LibraryError> {
        catalogo::registrar_capitulo(
            &self.db,
            obra_id,
            volume_id,
            numero,
            titulo_original,
            caminho_relativo,
            quantidade_paginas,
        )
        .await
    }

    /// Obtém os detalhes completos de uma obra (volumes ordenados + capítulos avulsos).
    pub async fn obter_detalhes_obra(&self, obra_id: ObraId) -> Result<DetalhesObra, LibraryError> {
        catalogo::obter_detalhes_obra(&self.db, obra_id).await
    }

    /// Atualiza os metadados complementares JSON de uma obra sem alterar arquivos `.cbz`.
    pub async fn atualizar_metadados_obra(
        &self,
        obra_id: ObraId,
        metadados_json: &str,
    ) -> Result<(), LibraryError> {
        metadados::atualizar_metadados_obra(&self.db, obra_id, metadados_json).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use chrono::Utc;
    use mam_core::{CaminhoRelativo, CapituloId, EstadoIntegridade};

    #[test]
    fn rejeita_caminhos_absolutos_e_parent_dir() {
        assert!(catalogo::validar_caminho_relativo("Manga/Naruto").is_ok());
        assert!(catalogo::validar_caminho_relativo("C:\\Manga\\Naruto").is_err());
        assert!(catalogo::validar_caminho_relativo("/mnt/ssd/Manga").is_err());
        assert!(catalogo::validar_caminho_relativo("../fora").is_err());
        assert!(catalogo::validar_caminho_relativo("").is_err());
    }

    #[test]
    fn organiza_estrutura_hibrida_volumes_e_avulsos() {
        let obra_id = ObraId::new();
        let vol_id = VolumeId::new();
        let obra = Obra {
            id: obra_id,
            titulo: "Obra Híbrida".to_string(),
            caminho_relativo: CaminhoRelativo::from_str("Manga/Obra".to_string()),
            origem: None,
            data_arquivamento: Utc::now(),
            estado_integridade: EstadoIntegridade::Desconhecido,
            metadados_complementares: None,
        };
        let volumes = vec![Volume {
            id: vol_id,
            obra_id,
            numero: 1,
            caminho_relativo: CaminhoRelativo::from_str("Manga/Obra/Vol 01".to_string()),
            titulo: Some("Volume 1".to_string()),
        }];
        let capitulos = vec![
            Capitulo {
                id: CapituloId::new(),
                volume_id: Some(vol_id),
                obra_id,
                numero: 1,
                titulo_original: None,
                caminho_relativo: CaminhoRelativo::from_str("Manga/Obra/Vol 01/Cap 01.cbz".to_string()),
                quantidade_paginas: 20,
                data_arquivamento: Utc::now(),
                estado_integridade: EstadoIntegridade::Valido,
            },
            Capitulo {
                id: CapituloId::new(),
                volume_id: None,
                obra_id,
                numero: 2,
                titulo_original: None,
                caminho_relativo: CaminhoRelativo::from_str("Manga/Obra/Cap 02.cbz".to_string()),
                quantidade_paginas: 18,
                data_arquivamento: Utc::now(),
                estado_integridade: EstadoIntegridade::Valido,
            },
        ];

        let detalhes = catalogo::organizar_estrutura_hibrida(obra, volumes, capitulos);
        assert_eq!(detalhes.volumes.len(), 1);
        assert_eq!(detalhes.volumes[0].capitulos.len(), 1);
        assert_eq!(detalhes.capitulos_avulsos.len(), 1);
        assert_eq!(detalhes.capitulos_avulsos[0].numero, 2);
    }

    #[test]
    fn valida_metadados_json_apenas_objeto() {
        assert!(metadados::validar_metadados_json(r#"{"autor": "Oda"}"#).is_ok());
        assert!(metadados::validar_metadados_json("[]").is_err());
        assert!(metadados::validar_metadados_json("{invalido").is_err());
    }

    #[tokio::test]
    async fn integracao_crud_obra_volume_metadados() {
        let temp_dir = env::temp_dir().join(format!("mam_test_lib_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db = Arc::new(
            Database::abrir(temp_dir.join("database.db"), temp_dir.clone())
                .await
                .expect("abrir banco"),
        );
        let manager = LibraryManager::nova(db, temp_dir.clone());

        let obra = manager
            .criar_obra("One Piece".to_string(), "Manga/One Piece".to_string(), None)
            .await
            .expect("criar obra");

        let vol = manager
            .criar_volume(
                obra.id,
                1,
                Some("Romance Dawn".to_string()),
                "Manga/One Piece/Vol 01".to_string(),
            )
            .await
            .expect("criar volume");

        manager
            .registrar_capitulo(
                obra.id,
                Some(vol.id),
                1,
                Some("Cap 1".to_string()),
                "Manga/One Piece/Vol 01/Cap 001.cbz".to_string(),
                54,
            )
            .await
            .expect("registrar capítulo no volume");

        manager
            .registrar_capitulo(
                obra.id,
                None,
                1000,
                None,
                "Manga/One Piece/Cap 1000.cbz".to_string(),
                19,
            )
            .await
            .expect("registrar capítulo avulso");

        manager
            .atualizar_metadados_obra(obra.id, r#"{"autor":"Eiichiro Oda","status":"em andamento"}"#)
            .await
            .expect("atualizar metadados");

        let detalhes = manager
            .obter_detalhes_obra(obra.id)
            .await
            .expect("obter detalhes");

        assert_eq!(detalhes.obra.titulo, "One Piece");
        assert!(detalhes.obra.metadados_complementares.is_some());
        assert_eq!(detalhes.volumes.len(), 1);
        assert_eq!(detalhes.volumes[0].capitulos.len(), 1);
        assert_eq!(detalhes.capitulos_avulsos.len(), 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
