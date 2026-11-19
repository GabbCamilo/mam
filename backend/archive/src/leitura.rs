//! Leitura de arquivos CBZ existentes.
//! Utiliza ZipArchive para listar entradas sem extração permanente.

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use zip::ZipArchive;
use zip::write::ZipWriter;
use zip::write::FileOptions;

use super::ArchiveError;

/// Lê um arquivo CBZ e retorna informações das páginas.
pub fn ler_cbz<P: AsRef<Path>>(caminho_cbz: P) -> Result<super::CbzLido, ArchiveError> {
    let file = File::open(caminho_cbz.as_ref()).map_err(ArchiveError::Io)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| ArchiveError::CbzInvalido(format!("CBZ inválido: {}", e)))?;

    let mut paginas: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            archive.by_index(i).ok().and_then(|entry| {
                let nome = entry.enclosed_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if eh_imagem_entrada(&nome) {
                    Some(nome)
                } else {
                    None
                }
            })
        })
        .collect();

    paginas.sort();

    let quantidade = paginas.len();

    // Detectar páginas ausentes (numeração não sequencial)
    let paginas_ausentes = detectar_paginas_ausentes_numericas(&paginas);

    Ok(super::CbzLido {
        caminho: caminho_cbz.as_ref().to_string_lossy().to_string(),
        paginas,
        quantidade_paginas: quantidade,
        paginas_ausentes,
    })
}

/// Verifica se a entrada do ZIP é uma imagem.
fn eh_imagem_entrada(nome: &str) -> bool {
    let ext = nome.rsplit('.').next().unwrap_or("");
    matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "avif"
    )
}

/// Detecta páginas ausentes com base na numeração sequencial dos nomes.
/// Ex: ["001.jpg", "003.jpg"] -> pagina 2 está ausente.
fn detectar_paginas_ausentes_numericas(paginas: &[String]) -> Vec<i32> {
    let mut ausentes = Vec::new();

    if paginas.is_empty() {
        return ausentes;
    }

    // Extrair números dos nomes de arquivo
    let mut numeros: Vec<i32> = paginas
        .iter()
        .filter_map(|nome| {
            let digitos: String = nome.chars().filter(|c| c.is_ascii_digit()).collect();
            if digitos.is_empty() {
                None
            } else {
                digitos.parse::<i32>().ok()
            }
        })
        .collect();

    numeros.sort();
    numeros.dedup();

    if let (Some(&primeiro), Some(&ultimo)) = (numeros.first(), numeros.last()) {
        let conjunto: std::collections::HashSet<i32> = numeros.into_iter().collect();
        for n in primeiro..=ultimo {
            if !conjunto.contains(&n) {
                ausentes.push(n);
            }
        }
    }

    ausentes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use zip::write::FileOptions;

    #[test]
    fn ler_cbz_valido() {
        let temp = std::env::temp_dir().join(format!("mam_test_ler_cbz_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("capitulo.cbz");

        // Cria um CBZ com 3 páginas
        let file = fs::File::create(&caminho_cbz).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("001.jpg", options).unwrap();
        zip.write_all(b"img1").unwrap();
        zip.start_file("002.jpg", options).unwrap();
        zip.write_all(b"img2").unwrap();
        zip.start_file("003.jpg", options).unwrap();
        zip.write_all(b"img3").unwrap();
        zip.finish().unwrap();

        let cbz = ler_cbz(&caminho_cbz).unwrap();
        assert_eq!(cbz.quantidade_paginas, 3);
        assert_eq!(cbz.paginas.len(), 3);
        assert_eq!(cbz.paginas_ausentes.len(), 0);

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn ler_cbz_com_paginas_ausentes() {
        let temp = std::env::temp_dir().join(format!("mam_test_ausentes_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("capitulo.cbz");

        let file = fs::File::create(&caminho_cbz).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("001.jpg", options).unwrap();
        zip.write_all(b"img1").unwrap();
        // pula 002.jpg
        zip.start_file("003.jpg", options).unwrap();
        zip.write_all(b"img3").unwrap();
        zip.finish().unwrap();

        let cbz = ler_cbz(&caminho_cbz).unwrap();
        assert_eq!(cbz.quantidade_paginas, 2);
        assert_eq!(cbz.paginas_ausentes, vec![2]);

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn cbz_invalido_retorna_erro() {
        let temp = std::env::temp_dir().join(format!("mam_test_invalido_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("invalido.cbz");
        fs::write(&caminho_cbz, "invalid zip content".as_bytes()).unwrap();

        let resultado = ler_cbz(&caminho_cbz);
        assert!(resultado.is_err());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}
