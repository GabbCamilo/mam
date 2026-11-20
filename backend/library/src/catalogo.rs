//! Operações de catálogo para obras, volumes e capítulos com organização híbrida.

use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use mam_core::{CaminhoRelativo, CapituloId, ObraId, VolumeId};
use mam_database::models::{Capitulo, Obra, Volume};
use mam_database::Database;

use crate::LibraryError;

/// Volume com seus capítulos ordenados.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeComCapitulos {
    pub volume: Volume,
    pub capitulos: Vec<Capitulo>,
}

/// Estrutura completa de uma obra com organização híbrida (volumes + capítulos avulsos).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetalhesObra {
    pub obra: Obra,
    pub volumes: Vec<VolumeComCapitulos>,
    pub capitulos_avulsos: Vec<Capitulo>,
}

/// Valida que um caminho é estritamente relativo à raiz da biblioteca (sem prefixo de unidade,
/// barra inicial ou segmentos `..`).
pub fn validar_caminho_relativo(caminho: &str) -> Result<CaminhoRelativo, LibraryError> {
    let trimmed = caminho.trim();
    if trimmed.is_empty() {
        return Err(LibraryError::CaminhoInvalido(
            "O caminho relativo não pode ser vazio".to_string(),
        ));
    }

    // Rejeita caminhos com letra de unidade Windows (ex.: C:\...) ou barra inicial (/ ou \)
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        return Err(LibraryError::CaminhoInvalido(format!(
            "Caminho absoluto não permitido: '{}'",
            trimmed
        )));
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return Err(LibraryError::CaminhoInvalido(format!(
            "Caminho absoluto não permitido: '{}'",
            trimmed
        )));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(LibraryError::CaminhoInvalido(format!(
            "Caminho absoluto não permitido: '{}'",
            trimmed
        )));
    }

    for comp in path.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(LibraryError::CaminhoInvalido(format!(
                    "Segmento inválido no caminho relativo: '{}'",
                    trimmed
                )));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }

    Ok(CaminhoRelativo::from_str(trimmed.to_string()))
}

/// Organiza volumes e capítulos de uma obra no formato híbrido.
pub fn organizar_estrutura_hibrida(
    obra: Obra,
    mut volumes: Vec<Volume>,
    mut capitulos: Vec<Capitulo>,
) -> DetalhesObra {
    volumes.sort_by_key(|v| v.numero);
    capitulos.sort_by_key(|c| c.numero);

    let mut capitulos_avulsos = Vec::new();
    let mut volumes_com_capitulos: Vec<VolumeComCapitulos> = volumes
        .into_iter()
        .map(|volume| VolumeComCapitulos {
            volume,
            capitulos: Vec::new(),
        })
        .collect();

    for cap in capitulos {
        match cap.volume_id {
            Some(vol_id) => {
                if let Some(vc) = volumes_com_capitulos
                    .iter_mut()
                    .find(|v| v.volume.id == vol_id)
                {
                    vc.capitulos.push(cap);
                } else {
                    capitulos_avulsos.push(cap);
                }
            }
            None => capitulos_avulsos.push(cap),
        }
    }

    DetalhesObra {
        obra,
        volumes: volumes_com_capitulos,
        capitulos_avulsos,
    }
}

pub async fn criar_obra(
    db: &Database,
    titulo: String,
    caminho_relativo: String,
    origem: Option<String>,
) -> Result<Obra, LibraryError> {
    let titulo_limpo = titulo.trim().to_string();
    if titulo_limpo.is_empty() {
        return Err(LibraryError::Validacao(
            "O título da obra não pode ser vazio".to_string(),
        ));
    }

    let caminho = validar_caminho_relativo(&caminho_relativo)?;
    let obra = db
        .criar_obra(ObraId::new(), titulo_limpo, caminho, origem)
        .await?;
    Ok(obra)
}

pub async fn listar_obras(db: &Database) -> Result<Vec<Obra>, LibraryError> {
    Ok(db.listar_obras().await?)
}

pub async fn criar_volume(
    db: &Database,
    obra_id: ObraId,
    numero: i32,
    titulo: Option<String>,
    caminho_relativo: String,
) -> Result<Volume, LibraryError> {
    if db.buscar_obra_por_id(obra_id).await?.is_none() {
        return Err(LibraryError::ObraNaoEncontrada(obra_id.to_string()));
    }

    let caminho = validar_caminho_relativo(&caminho_relativo)?;
    let volume = db
        .criar_volume_com_numero(VolumeId::new(), obra_id, numero, titulo, caminho)
        .await?;
    Ok(volume)
}

pub async fn registrar_capitulo(
    db: &Database,
    obra_id: ObraId,
    volume_id: Option<VolumeId>,
    numero: i32,
    titulo_original: Option<String>,
    caminho_relativo: String,
    quantidade_paginas: i32,
) -> Result<Capitulo, LibraryError> {
    if db.buscar_obra_por_id(obra_id).await?.is_none() {
        return Err(LibraryError::ObraNaoEncontrada(obra_id.to_string()));
    }

    let caminho = validar_caminho_relativo(&caminho_relativo)?;
    let capitulo = db
        .criar_capitulo(
            CapituloId::new(),
            volume_id,
            obra_id,
            numero,
            titulo_original,
            caminho,
            quantidade_paginas,
        )
        .await?;
    Ok(capitulo)
}

pub async fn obter_detalhes_obra(
    db: &Database,
    obra_id: ObraId,
) -> Result<DetalhesObra, LibraryError> {
    let obra = db
        .buscar_obra_por_id(obra_id)
        .await?
        .ok_or_else(|| LibraryError::ObraNaoEncontrada(obra_id.to_string()))?;

    let volumes = db.listar_volumes_obra(obra_id).await?;
    let capitulos = db.listar_capitulos_obra(obra_id).await?;

    Ok(organizar_estrutura_hibrida(obra, volumes, capitulos))
}
