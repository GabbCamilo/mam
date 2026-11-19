//! Validação de integridade de arquivos CBZ.
//! Detecta arquivos corrompidos e páginas ausentes.

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use zip::ZipArchive;
use zip::write::ZipWriter;
use zip::write::FileOptions;

use super::ArchiveError;

/// Valida se um arquivo CBZ é um ZIP válido com pelo menos uma imagem.
pub fn validar_cbz<P: AsRef<Path>>(caminho_cbz: P) -> Result<bool, ArchiveError> {
    let file = std::fs::File::open(caminho_cbz.as_ref()).map_err(ArchiveError::Io)?;
    let reader = std::io::BufReader::new(file);

    match zip::ZipArchive::new(reader) {
        Ok(mut archive) => {
            // Verificar se há pelo menos uma imagem
            let mut tem_imagem = false;
            for i in 0..archive.len() {
                if let Ok(entry) = archive.by_index(i) {
                    let nome = entry.enclosed_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if eh_imagem_entrada(&nome) {
                        tem_imagem = true;
                        break;
                    }
                }
            }
            Ok(tem_imagem)
        }
        Err(_) => Ok(false),
    }
}

/// Verifica se a entrada do ZIP é uma imagem.
fn eh_imagem_entrada(nome: &str) -> bool {
    let ext = nome.rsplit('.').next().unwrap_or("");
    matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "avif"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use zip::write::FileOptions;

    #[test]
    fn validar_cbz_valido() {
        let temp = std::env::temp_dir().join(format!("mam_test_validar_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("valido.cbz");

        let file = fs::File::create(&caminho_cbz).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("001.jpg", options).unwrap();
        zip.write_all(b"img1").unwrap();
        zip.finish().unwrap();

        assert!(validar_cbz(&caminho_cbz).unwrap());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn validar_cbz_invalido() {
        let temp = std::env::temp_dir().join(format!("mam_test_invalido2_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("invalido.cbz");
        fs::write(&caminho_cbz, "invalid zip content".as_bytes()).unwrap();

        assert!(!validar_cbz(&caminho_cbz).unwrap());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn validar_cbz_sem_imagens() {
        let temp = std::env::temp_dir().join(format!("mam_test_sem_img_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("sem_imagem.cbz");

        let file = fs::File::create(&caminho_cbz).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("nota.txt", options).unwrap();
        zip.write_all(b"texto").unwrap();
        zip.finish().unwrap();

        assert!(!validar_cbz(&caminho_cbz).unwrap());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}
