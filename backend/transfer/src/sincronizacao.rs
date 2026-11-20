//! Comparação e sincronização não-destrutiva entre duas bibliotecas MAM.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::copia::{calcular_sha256, copiar_arquivo_seguro, listar_arquivos_relativos};
use crate::{RelatorioTransferencia, TransferError};

/// Resultado da comparação entre biblioteca de origem e biblioteca de destino.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiferencaBibliotecas {
    pub ausentes_no_destino: Vec<String>,
    pub diferentes: Vec<String>,
    pub identicos: Vec<String>,
}

/// Compara dois diretórios de biblioteca por caminho relativo e hash SHA-256.
pub fn comparar_bibliotecas(
    origem: &Path,
    destino: &Path,
) -> Result<DiferencaBibliotecas, TransferError> {
    let arquivos_origem = listar_arquivos_relativos(origem)?;
    let mapa_destino: BTreeMap<String, _> = if destino.exists() {
        listar_arquivos_relativos(destino)?.into_iter().collect()
    } else {
        BTreeMap::new()
    };

    let mut ausentes_no_destino = Vec::new();
    let mut diferentes = Vec::new();
    let mut identicos = Vec::new();

    for (rel, abs_origem) in arquivos_origem {
        match mapa_destino.get(&rel) {
            None => ausentes_no_destino.push(rel),
            Some(abs_destino) => {
                let hash_src = calcular_sha256(&abs_origem)?;
                let hash_dst = calcular_sha256(abs_destino)?;
                if hash_src == hash_dst {
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

/// Sincroniza arquivos da origem para o destino sem jamais remover arquivos da origem.
/// Quando `permitir_sobrescrita` é `false`, arquivos divergentes no destino são preservados
/// e listados em `arquivos_ignorados`.
pub fn sincronizar_bibliotecas(
    origem: &Path,
    destino: &Path,
    permitir_sobrescrita: bool,
) -> Result<RelatorioTransferencia, TransferError> {
    let diff = comparar_bibliotecas(origem, destino)?;
    fs::create_dir_all(destino)?;

    let mut relatorio = RelatorioTransferencia::novo();

    for rel in diff.ausentes_no_destino {
        let bytes = copiar_arquivo_seguro(&origem.join(&rel), &destino.join(&rel))?;
        relatorio.arquivos_copiados.push(rel);
        relatorio.bytes_transferidos += bytes;
    }

    for rel in diff.diferentes {
        if permitir_sobrescrita {
            let bytes = copiar_arquivo_seguro(&origem.join(&rel), &destino.join(&rel))?;
            relatorio.arquivos_copiados.push(rel);
            relatorio.bytes_transferidos += bytes;
        } else {
            relatorio.arquivos_ignorados.push(rel);
        }
    }

    Ok(relatorio)
}
