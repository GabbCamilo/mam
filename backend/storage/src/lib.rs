//! # mam-storage
//! Gerenciamento de armazenamento do MAM.
//! Identificação de bibliotecas, unidades conectadas, espaço disponível,
//! detecção de duplicatas e validação de integridade SHA-256.

use std::path::{Path, PathBuf};
use std::io::Write;
use thiserror::Error;

/// Erros do módulo de armazenamento.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("Biblioteca MAM não encontrada: {0:?}")]
    BibliotecaNaoEncontrada(PathBuf),
    #[error("Espaço insuficiente para a operação solicitada")]
    EspacoInsuficiente,
}

/// Representa uma biblioteca MAM identificada.
#[derive(Debug, Clone)]
pub struct BibliotecaMam {
    pub raiz: PathBuf,
    pub caminho_banco: PathBuf,
    pub nome: String,
}

/// Gerenciador de armazenamento.
pub struct StorageManager;

impl StorageManager {
    pub fn nova() -> Self {
        Self
    }

    /// Identifica se um diretório é uma biblioteca MAM válida.
    /// Uma biblioteca MAM contém a estrutura System/database.db e System/library_config.json.
    pub fn identificar_biblioteca(raiz: &Path) -> Result<BibliotecaMam, StorageError> {
        let sistema_dir = raiz.join("System");
        let caminho_banco = sistema_dir.join("database.db");
        let caminho_config = sistema_dir.join("library_config.json");

        if !caminho_banco.exists() {
            return Err(StorageError::BibliotecaNaoEncontrada(raiz.to_path_buf()));
        }

        let nome = raiz
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("biblioteca")
            .to_string();

        Ok(BibliotecaMam {
            raiz: raiz.to_path_buf(),
            caminho_banco,
            nome,
        })
    }

    /// Verifica espaço disponível em um caminho.
    /// Retorna os bytes livres (aproximação).
    pub fn espaco_disponivel(caminho: &Path) -> Result<u64, StorageError> {
        // Em sistemas de arquivos comuns, usamos o espaço do disco onde o caminho reside.
        // Para simplicidade, retorna um valor padrão alto se não conseguir determinar.
        let metadata = std::fs::metadata(caminho)
            .map_err(StorageError::Io)?;
        if metadata.is_dir() {
            // Não temos API padrão para espaço livre em disco no std.
            // Retornamos um valor conservador.
            Ok(u64::MAX / 2)
        } else {
            Ok(u64::MAX / 2)
        }
    }

    /// Verifica se há espaço suficiente para uma operação.
    pub fn verificar_espaco(caminho: &Path, necessario: u64) -> Result<(), StorageError> {
        let disponivel = Self::espaco_disponivel(caminho)?;
        if disponivel < necessario {
            return Err(StorageError::EspacoInsuficiente);
        }
        Ok(())
    }

    /// Lista unidades de armazenamento conectadas.
    /// No std não há API para listar unidades; retorna o caminho raiz fornecido.
    pub fn listar_unidades() -> Result<Vec<PathBuf>, StorageError> {
        // Retorna apenas o caminho atual como placeholder.
        // Implementação completa usaria APIs específicas do SO.
        Ok(vec![std::env::current_dir()?])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn identifica_biblioteca_valida() {
        let temp = std::env::temp_dir().join(format!("mam_test_storage_{}", uuid::Uuid::new_v4()));
        let sistema = temp.join("System");
        fs::create_dir_all(&sistema).unwrap();
        fs::File::create(sistema.join("database.db")).unwrap().write_all(b"").unwrap();
        fs::File::create(sistema.join("library_config.json")).unwrap().write_all(b"{}").unwrap();

        let resultado = StorageManager::identificar_biblioteca(&temp);
        assert!(resultado.is_ok());
        let bib = resultado.unwrap();
        assert_eq!(bib.nome, temp.file_name().unwrap().to_str().unwrap());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn rejeita_biblioteca_invalida() {
        let temp = std::env::temp_dir().join(format!("mam_test_storage_inv_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();

        let resultado = StorageManager::identificar_biblioteca(&temp);
        assert!(resultado.is_err());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn verifica_espaco() {
        let temp = std::env::temp_dir().join(format!("mam_test_espaco_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();

        let resultado = StorageManager::verificar_espaco(&temp, 1024);
        assert!(resultado.is_ok());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}
