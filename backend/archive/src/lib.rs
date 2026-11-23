//! # mam-archive
//! Geração e leitura de arquivos CBZ para o MAM.
//!
//! CBZ é um arquivo ZIP contendo imagens sequenciais numeradas e ordenadas.
//! Nunca modifica arquivos CBZ originais (leitura apenas para validação).
//! Geração utiliza arquivos temporários e operações de escrita seguras.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use mam_database::Database;
use mam_storage::CacheL1Lru;

use crate::criacao::criar_cbz;
use crate::leitura::{ler_cbz, ler_pagina_cbz_por_nome, listar_paginas_ordenadas_cbz};
use crate::validacao::validar_cbz;

pub use crate::leitura::PaginaExtraida;

use serde::{Deserialize, Serialize};

/// Erros do módulo de archive.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("Erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("Erro ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Erro de banco: {0}")]
    Database(#[from] mam_database::DatabaseError),
    #[error("CBZ inválido: {0}")]
    CbzInvalido(String),
    #[error("Imagem não encontrada: {0}")]
    ImagemNaoEncontrada(String),
}

/// Resultado de criação de CBZ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbzCriado {
    pub caminho_destino: String,
    pub quantidade_paginas: usize,
    pub hash_sha256: String,
    pub tamanho_bytes: u64,
}

/// Resultado de leitura de CBZ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbzLido {
    pub caminho: String,
    pub paginas: Vec<String>,
    pub quantidade_paginas: usize,
    pub paginas_ausentes: Vec<i32>,
}

/// Gerenciador de archive CBZ.
pub struct ArchiveManager {
    db: Arc<Database>,
    indice_paginas_cache: Arc<tokio::sync::Mutex<CacheL1Lru<(PathBuf, u64, u128), Vec<String>>>>,
}

impl ArchiveManager {
    pub fn nova(db: Arc<Database>) -> Self {
        Self {
            db,
            indice_paginas_cache: Arc::new(tokio::sync::Mutex::new(CacheL1Lru::nova(64))),
        }
    }

    /// Cria um arquivo CBZ a partir de imagens em um diretório de origem.
    /// O arquivo de destino é criado com escrita atômica (temp + rename).
    pub async fn criar_cbz_de_imagens(
        &self,
        imagens_dir: &Path,
        caminho_destino: &Path,
    ) -> Result<CbzCriado, ArchiveError> {
        let imagens_dir = imagens_dir.to_path_buf();
        let caminho_destino = caminho_destino.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let imagens = listar_imagens_ordenadas(&imagens_dir)?;

            if imagens.is_empty() {
                return Err(ArchiveError::CbzInvalido(
                    "nenhuma imagem encontrada no diretório de origem".to_string(),
                ));
            }

            let temp_path = {
                let pai = caminho_destino.parent().unwrap_or(Path::new("."));
                let nome_temp = formato_nome_temporario(&caminho_destino)?;
                pai.join(nome_temp)
            };

            let (quantidade_paginas, tamanho_bytes) =
                criar_cbz(&imagens_dir, &imagens, &temp_path)?;

            std::fs::rename(&temp_path, &caminho_destino)
                .map_err(ArchiveError::Io)?;

            let hash = calcular_sha256_arquivo(&caminho_destino)?;

            Ok(CbzCriado {
                caminho_destino: caminho_destino
                    .to_string_lossy()
                    .replace('\\', "/"),
                quantidade_paginas,
                hash_sha256: hash,
                tamanho_bytes,
            })
        })
        .await
        .map_err(|e| ArchiveError::CbzInvalido(format!("Falha de tarefa em segundo plano: {}", e)))?
    }

    /// Lê um arquivo CBZ existente e retorna informações das páginas.
    pub async fn ler_cbz(&self, caminho_cbz: &Path) -> Result<CbzLido, ArchiveError> {
        let caminho = caminho_cbz.to_path_buf();
        tokio::task::spawn_blocking(move || ler_cbz(&caminho))
            .await
            .map_err(|e| ArchiveError::CbzInvalido(format!("Falha de tarefa em segundo plano: {}", e)))?
    }

    /// Valida a integridade de um arquivo CBZ existente.
    pub async fn validar_cbz(&self, caminho_cbz: &Path) -> Result<bool, ArchiveError> {
        let caminho = caminho_cbz.to_path_buf();
        tokio::task::spawn_blocking(move || validar_cbz(&caminho))
            .await
            .map_err(|e| ArchiveError::CbzInvalido(format!("Falha de tarefa em segundo plano: {}", e)))?
    }

    /// Detecta páginas ausentes em um CBZ.
    pub async fn detectar_paginas_ausentes(
        &self,
        caminho_cbz: &Path,
    ) -> Result<Vec<i32>, ArchiveError> {
        let cbz = self.ler_cbz(caminho_cbz).await?;
        Ok(cbz.paginas_ausentes)
    }

    /// Lê uma página específica de um CBZ sob demanda em memória, reutilizando o índice ordenado de entradas.
    pub async fn ler_pagina(
        &self,
        caminho_cbz: &Path,
        indice: usize,
    ) -> Result<PaginaExtraida, ArchiveError> {
        let caminho = caminho_cbz.to_path_buf();
        let meta = tokio::fs::metadata(&caminho).await.map_err(ArchiveError::Io)?;
        let modificado_ns = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let chave_indice = (caminho.clone(), meta.len(), modificado_ns);

        let paginas = {
            let mut guard = self.indice_paginas_cache.lock().await;
            guard.obter(&chave_indice).cloned()
        };

        let paginas = match paginas {
            Some(lista) => lista,
            None => {
                let caminho_idx = caminho.clone();
                let lista = tokio::task::spawn_blocking(move || listar_paginas_ordenadas_cbz(&caminho_idx))
                    .await
                    .map_err(|e| {
                        ArchiveError::CbzInvalido(format!("Falha de tarefa em segundo plano: {}", e))
                    })??;
                let mut guard = self.indice_paginas_cache.lock().await;
                guard.inserir(chave_indice, lista.clone());
                lista
            }
        };

        let total_paginas = paginas.len();
        let nome_alvo = paginas
            .get(indice)
            .cloned()
            .ok_or_else(|| {
                ArchiveError::ImagemNaoEncontrada(format!(
                    "Índice de página {} fora dos limites (total: {})",
                    indice, total_paginas
                ))
            })?;

        tokio::task::spawn_blocking(move || {
            ler_pagina_cbz_por_nome(&caminho, &nome_alvo, total_paginas)
        })
        .await
        .map_err(|e| ArchiveError::CbzInvalido(format!("Falha de tarefa em segundo plano: {}", e)))?
    }
}

/// Lista imagens ordenadas em um diretório.
fn listar_imagens_ordenadas(
    diretorio: &Path,
) -> Result<Vec<std::path::PathBuf>, ArchiveError> {
    let mut imagens: Vec<std::path::PathBuf> = std::fs::read_dir(diretorio)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| eh_imagem(ext.to_string_lossy().as_ref()))
        })
        .map(|e| e.path())
        .collect();

    imagens.sort();
    Ok(imagens)
}

/// Verifica se a extensão é uma imagem válida.
fn eh_imagem(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "avif"
    )
}

/// Calcula hash SHA-256 de um arquivo.
fn calcular_sha256_arquivo(caminho: &Path) -> Result<String, ArchiveError> {
    use sha2::{Sha256, Digest};
    use std::fs::File;
    use std::io::{BufReader, Read};

    let file = File::open(caminho).map_err(ArchiveError::Io)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_lidos = reader.read(&mut buffer).map_err(ArchiveError::Io)?;
        if bytes_lidos == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_lidos]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Gera um nome de arquivo temporário baseado no destino com sufixo .tmp.
fn formato_nome_temporario(destino: &Path) -> Result<String, ArchiveError> {
    let nome_arquivo = destino
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ArchiveError::CbzInvalido("nome de arquivo inválido".to_string()))?;

    // Remove a extensão .cbz se presente, para não duplicar
    let nome_sem_ext = nome_arquivo.strip_suffix(".cbz").unwrap_or(nome_arquivo);

    Ok(format!("{}.cbz.tmp", nome_sem_ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eh_imagem_testa() {
        assert!(eh_imagem("jpg"));
        assert!(eh_imagem("png"));
        assert!(eh_imagem("webp"));
        assert!(!eh_imagem("txt"));
        assert!(!eh_imagem("pdf"));
    }

    #[test]
    fn listar_imagens_ordenadas_testa() {
        let temp = std::env::temp_dir().join(format!("mam_test_archive_{}", uuid::Uuid::new_v4()));
        let img_dir = temp.join("imagens");
        std::fs::create_dir_all(&img_dir).unwrap();

        // Cria imagens em ordem não sequencial no nome para testar ordenação
        std::fs::write(img_dir.join("003.jpg"), b"img3").unwrap();
        std::fs::write(img_dir.join("001.jpg"), b"img1").unwrap();
        std::fs::write(img_dir.join("002.jpg"), b"img2").unwrap();
        std::fs::write(img_dir.join("nota.txt"), b"texto").unwrap();

        let imagens = listar_imagens_ordenadas(&img_dir).unwrap();
        assert_eq!(imagens.len(), 3);
        assert_eq!(imagens[0].file_name().unwrap(), "001.jpg");
        assert_eq!(imagens[1].file_name().unwrap(), "002.jpg");
        assert_eq!(imagens[2].file_name().unwrap(), "003.jpg");

        // Limpeza
        let _ = std::fs::remove_dir_all(&temp);
    }
}

mod criacao;
mod leitura;
mod validacao;
mod operacoes_seguras;
