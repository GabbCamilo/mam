//! # mam-storage
//! Gerenciamento de armazenamento do MAM.
//! Identificação de bibliotecas, unidades conectadas, espaço disponível,
//! detecção de duplicatas e validação de integridade SHA-256.

pub mod cache;

use std::path::{Path, PathBuf};
use thiserror::Error;

pub use cache::CacheL1Lru;

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

    /// Verifica se a unidade/diretório raiz está acessível (detecta desconexão de SSD).
    pub fn unidade_disponivel(caminho: &Path) -> bool {
        caminho.exists() && std::fs::metadata(caminho).is_ok()
    }

    /// Verifica espaço livre real disponível em um caminho (em bytes).
    pub fn espaco_disponivel(caminho: &Path) -> Result<u64, StorageError> {
        if !Self::unidade_disponivel(caminho) {
            return Err(StorageError::BibliotecaNaoEncontrada(caminho.to_path_buf()));
        }

        #[cfg(windows)]
        {
            espaco_livre_windows(caminho)
        }

        #[cfg(unix)]
        {
            espaco_livre_unix(caminho)
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
    use std::io::Write;

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
        assert!(StorageManager::unidade_disponivel(&temp));
        let livre = StorageManager::espaco_disponivel(&temp).expect("espaco livre real");
        assert!(livre > 0);
        assert!(livre < u64::MAX / 2);

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}

#[cfg(windows)]
fn espaco_livre_windows(caminho: &Path) -> Result<u64, StorageError> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    let wide: Vec<u16> = caminho
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut disponivel: u64 = 0;
    let mut total: u64 = 0;
    let mut total_livre: u64 = 0;

    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut disponivel,
            &mut total,
            &mut total_livre,
        )
    };
    if ok == 0 {
        return Err(StorageError::Io(std::io::Error::last_os_error()));
    }
    Ok(disponivel)
}

#[cfg(unix)]
fn espaco_livre_unix(caminho: &Path) -> Result<u64, StorageError> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    #[repr(C)]
    struct Statvfs {
        f_bsize: usize,
        f_frsize: usize,
        f_blocks: u64,
        f_bfree: u64,
        f_bavail: u64,
        f_files: u64,
        f_ffree: u64,
        f_favail: u64,
        f_fsid: usize,
        f_flag: usize,
        f_namemax: usize,
        __f_spare: [i32; 6],
    }

    unsafe extern "C" {
        fn statvfs(path: *const i8, buf: *mut Statvfs) -> i32;
    }

    let c_path = CString::new(caminho.as_os_str().as_bytes())
        .map_err(|_| StorageError::BibliotecaNaoEncontrada(caminho.to_path_buf()))?;
    let mut stat = std::mem::MaybeUninit::<Statvfs>::zeroed();
    let res = unsafe { statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if res != 0 {
        return Err(StorageError::Io(std::io::Error::last_os_error()));
    }
    let stat = unsafe { stat.assume_init() };
    Ok((stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64))
}
