//! Cálculo de hash SHA-256 para validação de integridade de arquivos CBZ.
//! Utilizado para detectar arquivos corrompidos ou alterados.

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use sha2::{Sha256, Digest};

/// Calcula o hash SHA-256 de um arquivo.
/// Retorna a string hexadecimal do hash.
pub fn calcular_sha256<P: AsRef<Path>>(caminho: P) -> Result<String, super::ChapterError> {
    let file = File::open(caminho.as_ref())
        .map_err(super::ChapterError::Io)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_lidos = reader.read(&mut buffer)
            .map_err(super::ChapterError::Io)?;
        if bytes_lidos == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_lidos]);
    }

    let resultado = hasher.finalize();
    Ok(format!("{:x}", resultado))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn calcula_sha256_deterministico() {
        let temp = std::env::temp_dir().join(format!("mam_test_hash_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho = temp.join("arquivo.bin");
        fs::write(&caminho, b"conteudo de teste para hash").unwrap();

        let hash1 = calcular_sha256(&caminho).unwrap();
        let hash2 = calcular_sha256(&caminho).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 = 64 caracteres hex

        // Conteúdo diferente = hash diferente
        fs::write(&caminho, b"conteudo modificado").unwrap();
        let hash3 = calcular_sha256(&caminho).unwrap();
        assert_ne!(hash1, hash3);

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn arquivo_nao_encontrado_retorna_erro() {
        let caminho = std::path::Path::new("/caminho/inexistente.cbz");
        let resultado = calcular_sha256(caminho);
        assert!(resultado.is_err());
    }
}
