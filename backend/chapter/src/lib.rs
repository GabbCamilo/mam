//! # mam-chapter
//! Detecção e parsing de capítulos CBZ para o MAM.
//!
//! Varre a estrutura hierárquica da biblioteca (Manga/ > Obra > [Volume/] > Capítulo.cbz),
//! extrai metadados de cada CBZ, calcula hash SHA-256 e registra no banco SQLite.
//! Nunca modifica os arquivos CBZ originais (leitura apenas).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mam_core::{CaminhoRelativo, EstadoIntegridade, ObraId, CapituloId, VolumeId};
use mam_database::Database;

use crate::detector::detectar_cbzs;
use crate::parser::extrair_paginas;
use crate::integrity::calcular_sha256;

use serde::{Deserialize, Serialize};

/// Erros do módulo de capítulos.
#[derive(Debug, thiserror::Error)]
pub enum ChapterError {
    #[error("Erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("Erro ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Erro de banco: {0}")]
    Database(#[from] mam_database::DatabaseError),
    #[error("CBZ inválido: {0}")]
    CbzInvalido(String),
}

/// Resultado de detecção de um capítulo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapituloDetectado {
    pub caminho_relativo: String,
    pub obra_titulo: String,
    pub volume_titulo: Option<String>,
    pub numero_capitulo: Option<i32>,
    pub titulo_original: Option<String>,
    pub quantidade_paginas: usize,
    pub hash_sha256: String,
    pub tamanho_bytes: u64,
}

/// Gerenciador de capítulos. Orquestra detecção, parsing e registro no banco.
pub struct ChapterManager {
    db: Arc<Database>,
    raiz_biblioteca: PathBuf,
}

impl ChapterManager {
    pub fn nova(db: Arc<Database>, raiz_biblioteca: PathBuf) -> Self {
        Self {
            db,
            raiz_biblioteca,
        }
    }

    /// Detecta todos os capítulos CBZ na biblioteca e os registra no banco.
    /// Retorna a lista de capítulos detectados/atualizados.
    pub async fn detectar_todos(&self) -> Result<Vec<CapituloDetectado>, ChapterError> {
        let manga_dir = self.raiz_biblioteca.join("Manga");
        if !manga_dir.exists() {
            return Ok(vec![]);
        }

        let cbz_files = detectar_cbzs(&manga_dir)?;
        let mut resultados = Vec::new();

        for caminho_cbz in cbz_files {
            let detectado = self.processar_cbz(&caminho_cbz).await?;
            resultados.push(detectado);
        }

        Ok(resultados)
    }

    /// Processa um único arquivo CBZ: extrai metadados, calcula hash, registra.
    async fn processar_cbz(
        &self,
        caminho_cbz: &Path,
    ) -> Result<CapituloDetectado, ChapterError> {
        // Caminho relativo da raiz da biblioteca
        let caminho_relativo = caminho_cbz
            .strip_prefix(&self.raiz_biblioteca)
            .map_err(|_| ChapterError::CbzInvalido("caminho fora da biblioteca".to_string()))?
            .to_string_lossy()
            .replace('\\', "/");

        // Hash SHA-256 (operação síncrona, executada em blocking)
        let hash = calcular_sha256(caminho_cbz)?;

        // Metadados do arquivo
        let metadata = std::fs::metadata(caminho_cbz)?;
        let tamanho = metadata.len();

        // Extrair páginas do CBZ (operação síncrona, executada em blocking)
        let paginas = extrair_paginas(caminho_cbz)?;
        let quantidade_paginas = paginas.len();

        // Determinar obra e volume a partir da estrutura de pastas
        let (obra_titulo, volume_titulo, numero_capitulo) =
            extrair_estrutura_pastas(caminho_cbz, &self.raiz_biblioteca)?;

        // Registrar obra (se não existir)
        let obra_id = match self.db.buscar_obra_por_titulo(&obra_titulo).await? {
            Some(obra) => obra.id,
            None => {
                let id = ObraId::new();
                let obra_pasta = caminho_cbz
                    .parent()
                    .and_then(|p| p.strip_prefix(&self.raiz_biblioteca).ok())
                    .unwrap_or(Path::new(""))
                    .to_string_lossy()
                    .replace('\\', "/");
                let obra_caminho = CaminhoRelativo::from_str(obra_pasta);
                self.db
                    .criar_obra(id, obra_titulo.clone(), obra_caminho, None)
                    .await?
                    .id
            }
        };

        // Registrar volume (se aplicável)
        let volume_id = if let Some(vtitulo) = &volume_titulo {
            let id = VolumeId::new();
            let vol_pasta = caminho_cbz
                .parent()
                .and_then(|p| p.strip_prefix(&self.raiz_biblioteca).ok())
                .unwrap_or(Path::new(""))
                .to_string_lossy()
                .replace('\\', "/");
            let vol_caminho = CaminhoRelativo::from_str(vol_pasta);
            Some(
                self.db
                    .criar_volume(id, obra_id, vtitulo.clone(), vol_caminho)
                    .await?
                    .id,
            )
        } else {
            None
        };

        // Registrar capítulo
        let capitulo_id = CapituloId::new();
        let capitulo_caminho = CaminhoRelativo::from_str(caminho_relativo.clone());

        self.db
            .criar_capitulo(
                capitulo_id,
                volume_id,
                obra_id,
                numero_capitulo.unwrap_or(0),
                None,
                capitulo_caminho,
                quantidade_paginas as i32,
            )
            .await?;

        // Registrar integridade
        self.db
            .registrar_integridade(
                &caminho_relativo,
                &hash,
                EstadoIntegridade::Valido.as_str(),
            )
            .await?;

        Ok(CapituloDetectado {
            caminho_relativo,
            obra_titulo,
            volume_titulo,
            numero_capitulo,
            titulo_original: None,
            quantidade_paginas,
            hash_sha256: hash,
            tamanho_bytes: tamanho,
        })
    }
}

/// Extrai obra, volume e número do capítulo a partir da estrutura de pastas.
fn extrair_estrutura_pastas(
    caminho_cbz: &Path,
    raiz: &Path,
) -> Result<(String, Option<String>, Option<i32>), ChapterError> {
    let relativo = caminho_cbz
        .strip_prefix(raiz)
        .map_err(|_| ChapterError::CbzInvalido("caminho inválido".to_string()))?;
    let relativo_str = relativo.to_string_lossy().replace('\\', "/");
    let partes: Vec<&str> = relativo_str.split('/').collect();

    // Estrutura esperada: Manga/Obra/[Volume/]Capítulo.cbz
    if partes.len() < 3 {
        return Err(ChapterError::CbzInvalido(
            "estrutura de pastas inválida (esperado: Manga/Obra/Capítulo.cbz)".to_string(),
        ));
    }

    let obra_titulo = partes[1].to_string();
    let volume_titulo = if partes.len() >= 4 {
        Some(partes[2].to_string())
    } else {
        None
    };

    let nome_arquivo = partes.last().unwrap_or(&"");
    let numero_capitulo = extrair_numero_capitulo(nome_arquivo);

    Ok((obra_titulo, volume_titulo, numero_capitulo))
}

/// Extrai o número do capítulo a partir do nome do arquivo.
fn extrair_numero_capitulo(nome: &str) -> Option<i32> {
    let digitos: String = nome.chars().filter(|c| c.is_ascii_digit()).collect();
    if digitos.is_empty() {
        None
    } else {
        digitos.parse::<i32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrair_numero_capitulo_testa() {
        assert_eq!(extrair_numero_capitulo("Capítulo 001.cbz"), Some(1));
        assert_eq!(extrair_numero_capitulo("Cap 12.cbz"), Some(12));
        assert_eq!(extrair_numero_capitulo("sem_numeros.cbz"), None);
    }

    #[test]
    fn estrutura_pastas_valida() {
        let raiz = Path::new("/biblioteca");
        let cbz = raiz.join("Manga/Obra A/Volume 001/Capítulo 001.cbz");
        let (obra, volume, numero) = extrair_estrutura_pastas(&cbz, raiz).unwrap();
        assert_eq!(obra, "Obra A");
        assert_eq!(volume, Some("Volume 001".to_string()));
        assert_eq!(numero, Some(1));
    }

    #[test]
    fn estrutura_pastas_sem_volume() {
        let raiz = Path::new("/biblioteca");
        let cbz = raiz.join("Manga/Obra B/Capítulo 001.cbz");
        let (obra, volume, numero) = extrair_estrutura_pastas(&cbz, raiz).unwrap();
        assert_eq!(obra, "Obra B");
        assert_eq!(volume, None);
        assert_eq!(numero, Some(1));
    }
}

mod detector;
mod parser;
mod integrity;
mod operations;
