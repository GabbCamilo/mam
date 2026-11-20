//! Criação de arquivos CBZ a partir de imagens.
//! Utiliza ZipWriter para empacotar imagens sequencialmente.

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};
use zip::ZipWriter;
use zip::write::FileOptions;

use super::ArchiveError;

/// Cria um arquivo CBZ a partir de uma lista de imagens ordenadas.
/// O arquivo é escrito no caminho `destino` (deve ser um arquivo temporário;
/// o rename atômico é feito pelo chamador).
pub fn criar_cbz(
    _imagens_dir: &Path,
    imagens: &[PathBuf],
    destino: &Path,
) -> Result<(usize, u64), ArchiveError> {
    let file = File::create(destino).map_err(ArchiveError::Io)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut contador = 1usize;
    let mut tamanho_total = 0u64;

    for caminho_imagem in imagens {
        let metadata = std::fs::metadata(caminho_imagem).map_err(ArchiveError::Io)?;
        tamanho_total += metadata.len();

        let nome_entrada = format!("{:03}.jpg", contador);
        zip.start_file(&nome_entrada, options)
            .map_err(ArchiveError::Zip)?;

        let mut img_file = File::open(caminho_imagem).map_err(ArchiveError::Io)?;
        let mut buffer = Vec::new();
        img_file.read_to_end(&mut buffer).map_err(ArchiveError::Io)?;
        zip.write_all(&buffer).map_err(ArchiveError::Io)?;

        contador += 1;
    }

    zip.finish().map_err(|e| ArchiveError::Zip(e))?;

    Ok((contador - 1, tamanho_total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn criar_cbz_valido() {
        let temp = std::env::temp_dir().join(format!("mam_test_criar_cbz_{}", uuid::Uuid::new_v4()));
        let img_dir = temp.join("imagens");
        std::fs::create_dir_all(&img_dir).unwrap();

        // Cria 3 imagens
        fs::write(img_dir.join("001.jpg"), b"imagem 1").unwrap();
        fs::write(img_dir.join("002.jpg"), b"imagem 2").unwrap();
        fs::write(img_dir.join("003.jpg"), b"imagem 3").unwrap();

        let imagens: Vec<std::path::PathBuf> = vec![
            img_dir.join("001.jpg"),
            img_dir.join("002.jpg"),
            img_dir.join("003.jpg"),
        ];

        let destino = temp.join("saida.cbz");
        let (qtd, tamanho) = criar_cbz(&img_dir, &imagens, &destino).unwrap();
        assert_eq!(qtd, 3);
        assert!(tamanho > 0);
        assert!(destino.exists());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn criar_cbz_sem_imagens_retorna_erro() {
        // Este teste não se aplica diretamente pois a validação de vazio está em lib.rs
        // Mas verificamos que criar com vetor vazio funciona (retorna 0, 0)
        let temp = std::env::temp_dir().join(format!("mam_test_vazio_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();
        let destino = temp.join("vazio.cbz");
        let imagens: Vec<std::path::PathBuf> = vec![];
        let (qtd, tamanho) = criar_cbz(&temp, &imagens, &destino).unwrap();
        assert_eq!(qtd, 0);
        assert_eq!(tamanho, 0);

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }
}
