//! Detecção de arquivos .cbz na estrutura da biblioteca.
//! Varre recursivamente o diretório Manga/ identificando arquivos CBZ.

use std::path::Path;
use walkdir::WalkDir;

/// Detecta todos os arquivos .cbz dentro de um diretório.
/// Retorna os caminhos ordenados alfabeticamente.
pub fn detectar_cbzs<P: AsRef<Path>>(diretorio: P) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut arquivos: Vec<std::path::PathBuf> = WalkDir::new(diretorio)
        .into_iter()
        .filter_entry(|e| {
            // Permite navegar em diretórios, e inclui apenas arquivos .cbz
            e.file_type().is_dir() || e.path().extension().map_or(false, |ext| {
                ext.eq_ignore_ascii_case("cbz")
            })
        })
        .filter_map(|e| {
            let e = e.ok()?;
            if e.file_type().is_file() && e.path().extension().map_or(false, |ext| {
                ext.eq_ignore_ascii_case("cbz")
            }) {
                Some(e.into_path())
            } else {
                None
            }
        })
        .collect();

    arquivos.sort();
    Ok(arquivos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn detecta_apenas_cbz() {
        let temp = std::env::temp_dir().join(format!("mam_test_detect_{}", uuid::Uuid::new_v4()));
        let manga = temp.join("Manga/Obra A");
        fs::create_dir_all(&manga).unwrap();

        // Cria arquivos .cbz e não-cbz
        fs::File::create(manga.join("cap1.cbz")).unwrap().write_all(b"fake cbz").unwrap();
        fs::File::create(manga.join("cap2.CBZ")).unwrap().write_all(b"fake cbz").unwrap();
        fs::File::create(manga.join("notacbz.txt")).unwrap().write_all(b"texto").unwrap();
        fs::File::create(manga.join("cap3.zip")).unwrap().write_all(b"fake zip").unwrap();

        let encontrados = detectar_cbzs(&temp.join("Manga")).unwrap();
        assert_eq!(encontrados.len(), 2); // cap1.cbz e cap2.CBZ

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}
