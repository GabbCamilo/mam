//! Operações de escrita seguras para arquivos CBZ.
//! Utiliza arquivo temporário + rename atômico para evitar arquivos corrompidos.

use std::path::Path;
use std::fs::File;
use std::io::Write;

use super::ArchiveError;

/// Retorna o caminho de um arquivo temporário no mesmo diretório do destino.
/// O arquivo temporário será usado para escrita atômica (temp + rename).
pub fn caminho_temporario(destino: &Path) -> Result<std::path::PathBuf, ArchiveError> {
    let pai = destino.parent().unwrap_or(Path::new("."));
    let nome_temp = formato::nome_temporario(destino)?;
    Ok(pai.join(nome_temp))
}

/// Escreve conteúdo em um arquivo temporário e depois renomeia para o destino final.
/// A operação de rename é atômica no mesmo filesystem.
pub fn escrever_atomico<P: AsRef<Path>, C: AsRef<[u8]>>(
    destino: P,
    conteudo: C,
) -> Result<(), ArchiveError> {
    let destino_path = destino.as_ref();
    let temp_path = caminho_temporario(destino_path)?;

    // Escrever no arquivo temporário
    {
        let mut file = File::create(&temp_path).map_err(ArchiveError::Io)?;
        file.write_all(conteudo.as_ref()).map_err(ArchiveError::Io)?;
        file.sync_all().map_err(ArchiveError::Io)?;
    }

    // Renomear para o destino (atômico no mesmo filesystem)
    std::fs::rename(&temp_path, destino_path).map_err(ArchiveError::Io)?;

    Ok(())
}

/// Módulo auxiliar para formatação de nomes temporários.
mod formato {
    use std::path::Path;
    use super::ArchiveError;

    /// Gera um nome de arquivo temporário baseado no destino com sufixo .tmp.
    pub fn nome_temporario(destino: &Path) -> Result<String, ArchiveError> {
        let nome_arquivo = destino
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ArchiveError::CbzInvalido("nome de arquivo inválido".to_string()))?;

        // Remove a extensão .cbz se presente, para não duplicar
        let nome_sem_ext = nome_arquivo.strip_suffix(".cbz").unwrap_or(nome_arquivo);

        Ok(format!("{}.cbz.tmp", nome_sem_ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn escrever_atomico_funciona() {
        let temp = std::env::temp_dir().join(format!("mam_test_atomico_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let destino = temp.join("saida.cbz");

        escrever_atomico(&destino, b"conteudo de teste").unwrap();

        assert!(destino.exists());
        let conteudo = fs::read_to_string(&destino).unwrap();
        assert_eq!(conteudo, "conteudo de teste");

        // Verificar que não há arquivo .tmp residual
        let temp_path = temp.join("saida.cbz.tmp");
        assert!(!temp_path.exists());

        // Limpeza
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn caminho_temporario_gera_sufixo_tmp() {
        let destino = std::path::Path::new("/biblioteca/capitulo.cbz");
        let temp = caminho_temporario(destino).unwrap();
        assert_eq!(temp.file_name().unwrap(), "capitulo.cbz.tmp");
    }
}
