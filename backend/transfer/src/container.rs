//! Empacotamento e comparação/sincronização de bibliotecas MAM via container `.zip`.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::copia::{calcular_sha256, listar_arquivos_relativos};
use crate::{DiferencaBibliotecas, RelatorioTransferencia, TransferError};

/// Resultado da compressão de uma biblioteca em container `.zip`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatorioContainer {
    pub caminho_arquivo: String,
    pub quantidade_arquivos: usize,
    pub bytes_totais: u64,
}

fn sanitizar_nome_container(nome: &str) -> String {
    let limpo: String = nome
        .trim()
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect();
    let base = if limpo.is_empty() {
        "MAM_Biblioteca_Container".to_string()
    } else {
        limpo
    };
    if base.to_lowercase().ends_with(".zip") {
        base
    } else {
        format!("{}.zip", base)
    }
}

fn deve_incluir_na_biblioteca(rel_norm: &str) -> bool {
    if rel_norm.is_empty() {
        return false;
    }
    let lower = rel_norm.to_lowercase();
    if lower.starts_with("exports/") || lower == "exports" {
        return false;
    }
    if lower.ends_with(".mam_tmp")
        || lower.ends_with("database.db-wal")
        || lower.ends_with("database.db-shm")
    {
        return false;
    }
    true
}

fn eh_ja_comprimido(rel_norm: &str) -> bool {
    let ext = rel_norm.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "cbz" | "zip" | "jpg" | "jpeg" | "png" | "webp" | "gif" | "avif"
    )
}

fn validar_caminho_relativo(rel: &str) -> Result<(), TransferError> {
    if rel.is_empty() || rel.starts_with('/') || rel.starts_with('\\') || rel.contains("..") {
        return Err(TransferError::CaminhoInvalido(rel.to_string()));
    }
    if rel.len() >= 2 && rel.as_bytes()[1] == b':' {
        return Err(TransferError::CaminhoInvalido(rel.to_string()));
    }
    Ok(())
}

fn sha256_reader<R: Read>(mut reader: R) -> Result<String, TransferError> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Comprime toda a biblioteca (`Manga/`, `System/`, etc.) em um arquivo `.zip` nomeável.
pub fn empacotar_biblioteca_container(
    raiz_biblioteca: &Path,
    pasta_destino: &Path,
    nome_container: &str,
) -> Result<RelatorioContainer, TransferError> {
    if !raiz_biblioteca.exists() || !raiz_biblioteca.is_dir() {
        return Err(TransferError::OrigemNaoEncontrada(
            raiz_biblioteca.display().to_string(),
        ));
    }

    fs::create_dir_all(pasta_destino)?;
    let nome_arquivo = sanitizar_nome_container(nome_container);
    let destino_final = pasta_destino.join(&nome_arquivo);
    let destino_tmp = pasta_destino.join(format!(".{}.mam_tmp", nome_arquivo));

    let file = File::create(&destino_tmp)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));

    let mut quantidade_arquivos = 0usize;

    for entry in WalkDir::new(raiz_biblioteca).follow_links(false) {
        let entry = entry.map_err(|e| TransferError::Io(io::Error::other(e.to_string())))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let abs = entry.path();
        if abs == destino_final || abs == destino_tmp {
            continue;
        }
        let rel = abs
            .strip_prefix(raiz_biblioteca)
            .map_err(|e| TransferError::CaminhoInvalido(e.to_string()))?;
        let rel_norm = rel.to_string_lossy().replace('\\', "/");
        if !deve_incluir_na_biblioteca(&rel_norm) {
            continue;
        }

        let metodo = if eh_ja_comprimido(&rel_norm) {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };
        let opts = FileOptions::default().compression_method(metodo);
        zip.start_file(&rel_norm, opts)
            .map_err(|e| TransferError::Storage(format!("Erro ao escrever entrada ZIP: {}", e)))?;

        let mut src = BufReader::new(File::open(abs)?);
        io::copy(&mut src, &mut zip)?;
        quantidade_arquivos += 1;
    }

    let writer = zip
        .finish()
        .map_err(|e| TransferError::Storage(format!("Erro ao finalizar container ZIP: {}", e)))?;
    drop(writer);

    if destino_final.exists() {
        let _ = fs::remove_file(&destino_final);
    }
    fs::rename(&destino_tmp, &destino_final)?;
    let bytes_totais = fs::metadata(&destino_final)?.len();

    Ok(RelatorioContainer {
        caminho_arquivo: destino_final.to_string_lossy().replace('\\', "/"),
        quantidade_arquivos,
        bytes_totais,
    })
}

/// Compara um container `.zip` diretamente com uma biblioteca de destino sem extrair tudo em disco.
pub fn comparar_container_com_biblioteca(
    caminho_zip: &Path,
    destino: &Path,
) -> Result<DiferencaBibliotecas, TransferError> {
    if !caminho_zip.exists() || !caminho_zip.is_file() {
        return Err(TransferError::OrigemNaoEncontrada(
            caminho_zip.display().to_string(),
        ));
    }

    let mapa_destino: BTreeMap<String, _> = if destino.exists() {
        listar_arquivos_relativos(destino)?.into_iter().collect()
    } else {
        BTreeMap::new()
    };

    let mut archive = ZipArchive::new(BufReader::new(File::open(caminho_zip)?))
        .map_err(|e| TransferError::Storage(format!("Container ZIP inválido: {}", e)))?;

    let mut ausentes_no_destino = Vec::new();
    let mut diferentes = Vec::new();
    let mut identicos = Vec::new();

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| TransferError::Storage(format!("Erro ao ler entrada ZIP: {}", e)))?;
        if entry.is_dir() {
            continue;
        }
        let rel = entry.name().replace('\\', "/");
        if validar_caminho_relativo(&rel).is_err() || !deve_incluir_na_biblioteca(&rel) {
            continue;
        }

        match mapa_destino.get(&rel) {
            None => ausentes_no_destino.push(rel),
            Some(abs_destino) => {
                let hash_zip = sha256_reader(entry)?;
                let hash_dst = calcular_sha256(abs_destino)?;
                if hash_zip == hash_dst {
                    identicos.push(rel);
                } else {
                    diferentes.push(rel);
                }
            }
        }
    }

    Ok(DiferencaBibliotecas {
        ausentes_no_destino,
        diferentes,
        identicos,
    })
}

/// Extrai e sincroniza apenas os arquivos necessários de um container `.zip` para o destino.
pub fn sincronizar_container_para_biblioteca(
    caminho_zip: &Path,
    destino: &Path,
    permitir_sobrescrita: bool,
) -> Result<RelatorioTransferencia, TransferError> {
    let diff = comparar_container_com_biblioteca(caminho_zip, destino)?;
    fs::create_dir_all(destino)?;

    let mut archive = ZipArchive::new(BufReader::new(File::open(caminho_zip)?))
        .map_err(|e| TransferError::Storage(format!("Container ZIP inválido: {}", e)))?;

    let mut relatorio = RelatorioTransferencia::novo();

    let mut extrair_entrada = |rel: &str| -> Result<u64, TransferError> {
        validar_caminho_relativo(rel)?;
        let mut entry = archive
            .by_name(rel)
            .map_err(|e| TransferError::Storage(format!("Entrada '{}' não encontrada no ZIP: {}", rel, e)))?;
        let alvo = destino.join(rel);
        if let Some(pai) = alvo.parent() {
            fs::create_dir_all(pai)?;
        }
        let tmp = alvo.with_extension("mam_tmp");
        let mut out = BufWriter::new(File::create(&tmp)?);
        let bytes = io::copy(&mut entry, &mut out)?;
        drop(out);
        if alvo.exists() {
            let _ = fs::remove_file(&alvo);
        }
        fs::rename(&tmp, &alvo)?;
        Ok(bytes)
    };

    for rel in diff.ausentes_no_destino {
        let bytes = extrair_entrada(&rel)?;
        relatorio.arquivos_copiados.push(rel);
        relatorio.bytes_transferidos += bytes;
    }

    for rel in diff.diferentes {
        if permitir_sobrescrita {
            let bytes = extrair_entrada(&rel)?;
            relatorio.arquivos_copiados.push(rel);
            relatorio.bytes_transferidos += bytes;
        } else {
            relatorio.arquivos_ignorados.push(rel);
        }
    }

    Ok(relatorio)
}
