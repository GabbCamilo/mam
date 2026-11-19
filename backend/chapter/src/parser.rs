//! Parsing de arquivos CBZ para extrair lista de páginas.
//! CBZ é um arquivo ZIP; usamos o crate `zip` para listar entradas
//! sem extração permanente.

use std::path::Path;
use zip::ZipArchive;
use zip::write::FileOptions;
use std::fs::File;
use std::io::BufReader;

/// Extrai a lista ordenada de páginas de um arquivo CBZ.
/// Retorna os nomes das páginas (imagens) ordenados alfabeticamente.
pub fn extrair_paginas<P: AsRef<Path>>(caminho_cbz: P) -> Result<Vec<String>, super::ChapterError> {
    let file = File::open(caminho_cbz.as_ref())
        .map_err(super::ChapterError::Io)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| super::ChapterError::CbzInvalido(format!("CBZ inválido: {}", e)))?;

    let mut paginas: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            archive.by_index(i).ok().and_then(|entry| {
                let nome = entry.enclosed_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                // Apenas arquivos de imagem (extensões comuns)
                if eh_imagem(&nome) {
                    Some(nome)
                } else {
                    None
                }
            })
        })
        .collect();

    paginas.sort();
    Ok(paginas)
}

/// Verifica se o nome do arquivo é uma imagem com base na extensão.
fn eh_imagem(nome: &str) -> bool {
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
    fn extrai_paginas_de_cbz_valido() {
        let temp = std::env::temp_dir().join(format!("mam_test_parse_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("capitulo.cbz");

        // Cria um CBZ (ZIP) com algumas imagens e um arquivo não-imagem
        let file = fs::File::create(&caminho_cbz).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("pag01.jpg", options).unwrap();
        zip.write_all(b"fake image 1").unwrap();
        zip.start_file("pag02.jpg", options).unwrap();
        zip.write_all(b"fake image 2").unwrap();
        zip.start_file("metadados.json", options).unwrap();
        zip.write_all(b"{}").unwrap();
        zip.finish().unwrap();

        let paginas = extrair_paginas(&caminho_cbz).unwrap();
        assert_eq!(paginas.len(), 2);
        assert_eq!(paginas[0], "pag01.jpg");
        assert_eq!(paginas[1], "pag02.jpg");

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn cbz_invalido_retorna_erro() {
        let temp = std::env::temp_dir().join(format!("mam_test_invalid_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("invalido.cbz");
        fs::write(&caminho_cbz, "invalid zip content".as_bytes()).unwrap();

        let resultado = extrair_paginas(&caminho_cbz);
        assert!(resultado.is_err());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}
