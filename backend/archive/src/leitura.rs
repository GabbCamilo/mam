//! Leitura de arquivos CBZ existentes.
//! Utiliza ZipArchive para listar entradas sem extração permanente.

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use zip::ZipArchive;
use serde::{Deserialize, Serialize};

use super::ArchiveError;

/// Página extraída sob demanda em memória (sem escrita permanente em disco).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginaExtraida {
    pub nome: String,
    pub mime_type: String,
    pub dados_base64: String,
    pub total_paginas: usize,
}

/// Lê um arquivo CBZ e retorna informações das páginas.
pub fn ler_cbz<P: AsRef<Path>>(caminho_cbz: P) -> Result<super::CbzLido, ArchiveError> {
    let file = File::open(caminho_cbz.as_ref()).map_err(ArchiveError::Io)?;
    let reader = BufReader::new(file);
    let archive = ZipArchive::new(reader)
        .map_err(|e| ArchiveError::CbzInvalido(format!("CBZ inválido: {}", e)))?;

    let mut paginas: Vec<String> = archive
        .file_names()
        .filter(|nome| eh_imagem_entrada(nome))
        .map(|nome| nome.to_string())
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

/// Lista os nomes de imagens ordenados dentro de um arquivo CBZ.
pub fn listar_paginas_ordenadas_cbz<P: AsRef<Path>>(caminho_cbz: P) -> Result<Vec<String>, ArchiveError> {
    let file = File::open(caminho_cbz.as_ref()).map_err(ArchiveError::Io)?;
    let reader = BufReader::new(file);
    let archive = ZipArchive::new(reader)
        .map_err(|e| ArchiveError::CbzInvalido(format!("CBZ inválido: {}", e)))?;

    let mut paginas: Vec<String> = archive
        .file_names()
        .filter(|nome| eh_imagem_entrada(nome))
        .map(|nome| nome.to_string())
        .collect();

    paginas.sort();
    Ok(paginas)
}

/// Lê os bytes de uma página específica de um CBZ dado o nome já resolvido da entrada.
pub fn ler_pagina_cbz_por_nome<P: AsRef<Path>>(
    caminho_cbz: P,
    nome_alvo: &str,
    total_paginas: usize,
) -> Result<PaginaExtraida, ArchiveError> {
    let file = File::open(caminho_cbz.as_ref()).map_err(ArchiveError::Io)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| ArchiveError::CbzInvalido(format!("CBZ inválido: {}", e)))?;

    let mut entrada = archive
        .by_name(nome_alvo)
        .map_err(|e| ArchiveError::ImagemNaoEncontrada(format!("{}: {}", nome_alvo, e)))?;

    let tamanho_estimado = usize::try_from(entrada.size()).unwrap_or(0);
    let mut buffer = Vec::with_capacity(tamanho_estimado);
    entrada.read_to_end(&mut buffer).map_err(ArchiveError::Io)?;

    Ok(PaginaExtraida {
        mime_type: inferir_mime_type(nome_alvo).to_string(),
        nome: nome_alvo.to_string(),
        dados_base64: codificar_base64(&buffer),
        total_paginas,
    })
}

/// Lê os bytes de uma página específica de um CBZ sob demanda em memória.
pub fn ler_pagina_cbz<P: AsRef<Path>>(
    caminho_cbz: P,
    indice: usize,
) -> Result<PaginaExtraida, ArchiveError> {
    let paginas = listar_paginas_ordenadas_cbz(&caminho_cbz)?;
    let total_paginas = paginas.len();

    let nome_alvo = paginas.get(indice).ok_or_else(|| {
        ArchiveError::ImagemNaoEncontrada(format!(
            "Índice de página {} fora dos limites (total: {})",
            indice, total_paginas
        ))
    })?;

    ler_pagina_cbz_por_nome(caminho_cbz, nome_alvo, total_paginas)
}

fn inferir_mime_type(nome: &str) -> &'static str {
    let ext = nome.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        _ => "image/jpeg",
    }
}

fn codificar_base64(bytes: &[u8]) -> String {
    const TABELA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut saida = Vec::with_capacity(((bytes.len() + 2) / 3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triplo = (b0 << 16) | (b1 << 8) | b2;

        saida.push(TABELA[((triplo >> 18) & 0x3F) as usize]);
        saida.push(TABELA[((triplo >> 12) & 0x3F) as usize]);
        if chunk.len() > 1 {
            saida.push(TABELA[((triplo >> 6) & 0x3F) as usize]);
        } else {
            saida.push(b'=');
        }
        if chunk.len() > 2 {
            saida.push(TABELA[(triplo & 0x3F) as usize]);
        } else {
            saida.push(b'=');
        }
    }
    String::from_utf8(saida).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

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

    #[test]
    fn ler_pagina_cbz_valida_e_indice_invalido() {
        let temp = std::env::temp_dir().join(format!("mam_test_pagina_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let caminho_cbz = temp.join("capitulo.cbz");

        let file = File::create(&caminho_cbz).unwrap();
        let mut zip = zip::write::ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("002.png", options).unwrap();
        zip.write_all(b"PNGDATA").unwrap();
        zip.start_file("001.jpg", options).unwrap();
        zip.write_all(b"JPGDATA").unwrap();
        zip.finish().unwrap();

        let pag0 = ler_pagina_cbz(&caminho_cbz, 0).unwrap();
        assert_eq!(pag0.nome, "001.jpg");
        assert_eq!(pag0.mime_type, "image/jpeg");
        assert_eq!(pag0.total_paginas, 2);
        assert!(!pag0.dados_base64.is_empty());

        let pag1 = ler_pagina_cbz(&caminho_cbz, 1).unwrap();
        assert_eq!(pag1.nome, "002.png");
        assert_eq!(pag1.mime_type, "image/png");

        assert!(ler_pagina_cbz(&caminho_cbz, 2).is_err());

        let _ = fs::remove_dir_all(&temp);
    }
}
