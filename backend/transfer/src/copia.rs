//! Operações de cópia integral e transferência seletiva com escrita atômica e SHA-256.

use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use mam_core::CaminhoRelativo;
use mam_storage::StorageManager;

use crate::{RelatorioTransferencia, TransferError};

/// Normaliza um caminho relativo para usar `/` como separador portátil.
pub fn normalizar_relativo(rel: &Path) -> String {
    rel.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Valida que uma string representa um caminho relativo seguro (sem prefixo, raiz ou `..`).
pub fn validar_caminho_relativo_seguro(caminho: &str) -> Result<CaminhoRelativo, TransferError> {
    let trimmed = caminho.trim();
    if trimmed.is_empty() {
        return Err(TransferError::CaminhoInvalido(
            "Caminho relativo não pode ser vazio".to_string(),
        ));
    }
    let bytes = trimmed.as_bytes();
    if (bytes.len() >= 2 && bytes[1] == b':')
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
    {
        return Err(TransferError::CaminhoInvalido(format!(
            "Caminho absoluto não permitido: '{}'",
            trimmed
        )));
    }
    let path = Path::new(trimmed);
    for comp in path.components() {
        if matches!(
            comp,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        ) {
            return Err(TransferError::CaminhoInvalido(format!(
                "Segmento inválido no caminho relativo: '{}'",
                trimmed
            )));
        }
    }
    Ok(CaminhoRelativo::from_str(normalizar_relativo(path)))
}

/// Calcula o hash SHA-256 de um arquivo em streaming (buffer de 64 KB).
pub fn calcular_sha256(caminho: &Path) -> Result<String, TransferError> {
    let arquivo = File::open(caminho)?;
    let mut reader = BufReader::new(arquivo);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let lidos = reader.read(&mut buffer)?;
        if lidos == 0 {
            break;
        }
        hasher.update(&buffer[..lidos]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Copia um único arquivo com escrita atômica (`*.mam_tmp` + `rename`) e validação SHA-256.
/// Nunca modifica ou remove o arquivo de origem.
pub fn copiar_arquivo_seguro(origem: &Path, destino: &Path) -> Result<u64, TransferError> {
    let meta = fs::metadata(origem)?;
    let tamanho = meta.len();

    if let Some(pai) = destino.parent() {
        fs::create_dir_all(pai)?;
        StorageManager::verificar_espaco(pai, tamanho)
            .map_err(|e| TransferError::Storage(e.to_string()))?;
    }

    let hash_origem = calcular_sha256(origem)?;
    let nome_temp = format!(
        ".{}.{}.mam_tmp",
        destino
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arquivo"),
        Uuid::new_v4()
    );
    let caminho_temp = destino
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(nome_temp);

    {
        let mut src = BufReader::new(File::open(origem)?);
        let mut dst = File::create(&caminho_temp)?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let lidos = src.read(&mut buffer)?;
            if lidos == 0 {
                break;
            }
            dst.write_all(&buffer[..lidos])?;
        }
        dst.sync_all()?;
    }

    let hash_destino = calcular_sha256(&caminho_temp)?;
    if hash_origem != hash_destino {
        let _ = fs::remove_file(&caminho_temp);
        return Err(TransferError::IntegridadeFalhou {
            arquivo: destino.to_string_lossy().to_string(),
            esperado: hash_origem,
            obtido: hash_destino,
        });
    }

    if destino.exists() {
        fs::remove_file(destino)?;
    }
    fs::rename(&caminho_temp, destino)?;
    Ok(tamanho)
}

/// Lista todos os arquivos regulares de um diretório retornando `(caminho_relativo_normalizado, caminho_absoluto)`.
pub fn listar_arquivos_relativos(raiz: &Path) -> Result<Vec<(String, PathBuf)>, TransferError> {
    if !raiz.exists() {
        return Err(TransferError::OrigemNaoEncontrada(
            raiz.to_string_lossy().to_string(),
        ));
    }
    let mut arquivos = Vec::new();
    for entry in WalkDir::new(raiz).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let abs = entry.path().to_path_buf();
            if let Ok(rel) = abs.strip_prefix(raiz) {
                arquivos.push((normalizar_relativo(rel), abs));
            }
        }
    }
    arquivos.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(arquivos)
}

/// Executa cópia integral de uma biblioteca origem para o diretório de destino.
pub fn copia_integral(origem: &Path, destino: &Path) -> Result<RelatorioTransferencia, TransferError> {
    let arquivos = listar_arquivos_relativos(origem)?;
    fs::create_dir_all(destino)?;

    let mut relatorio = RelatorioTransferencia::novo();
    for (rel, abs_origem) in arquivos {
        let abs_destino = destino.join(&rel);
        let bytes = copiar_arquivo_seguro(&abs_origem, &abs_destino)?;
        relatorio.arquivos_copiados.push(rel);
        relatorio.bytes_transferidos += bytes;
    }
    Ok(relatorio)
}

/// Executa transferência seletiva de caminhos relativos específicos (arquivos ou subdiretórios).
pub fn transferencia_seletiva(
    origem: &Path,
    destino: &Path,
    caminhos_relativos: &[String],
) -> Result<RelatorioTransferencia, TransferError> {
    if !origem.exists() {
        return Err(TransferError::OrigemNaoEncontrada(
            origem.to_string_lossy().to_string(),
        ));
    }
    fs::create_dir_all(destino)?;

    let mut alvos = Vec::new();
    for c in caminhos_relativos {
        alvos.push(validar_caminho_relativo_seguro(c)?);
    }

    let todos = listar_arquivos_relativos(origem)?;
    let mut relatorio = RelatorioTransferencia::novo();

    for (rel, abs_origem) in todos {
        let selecionado = alvos.iter().any(|alvo| {
            let a = alvo.as_str();
            rel == a || rel.starts_with(&format!("{}/", a))
        });
        if selecionado {
            let abs_destino = destino.join(&rel);
            let bytes = copiar_arquivo_seguro(&abs_origem, &abs_destino)?;
            relatorio.arquivos_copiados.push(rel);
            relatorio.bytes_transferidos += bytes;
        }
    }

    Ok(relatorio)
}
