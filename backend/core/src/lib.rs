//! # mam-core
//! Núcleo do MAM: orquestração, ciclo de vida da aplicação, configuração portátil
//! e tipos de domínio compartilhados entre crates.
//!
//! Tipos de domínio:
//! - ObraId, VolumeId, CapituloId: identificadores únicos (wrappers de Uuid)
//! - CaminhoRelativo: caminho de arquivo relativo à raiz da biblioteca
//! - EstadoIntegridade: estado de integridade de um arquivo

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Identificador único de uma obra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObraId(pub Uuid);

impl ObraId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: String) -> Self {
        Self(Uuid::parse_str(&s).unwrap_or(Uuid::nil()))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for ObraId {
    fn default() -> Self {
        Self(Uuid::nil())
    }
}

/// Identificador único de um volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VolumeId(pub Uuid);

impl VolumeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: String) -> Self {
        Self(Uuid::parse_str(&s).unwrap_or(Uuid::nil()))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for VolumeId {
    fn default() -> Self {
        Self(Uuid::nil())
    }
}

/// Identificador único de um capítulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapituloId(pub Uuid);

impl CapituloId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: String) -> Self {
        Self(Uuid::parse_str(&s).unwrap_or(Uuid::nil()))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for CapituloId {
    fn default() -> Self {
        Self(Uuid::nil())
    }
}

/// Caminho de arquivo relativo à raiz da biblioteca (não depende de letra de unidade).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaminhoRelativo(pub String);

impl CaminhoRelativo {
    pub fn from_str(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CaminhoRelativo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Estado de integridade de um arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EstadoIntegridade {
    Valido,
    Invalido,
    Desconhecido,
}

impl EstadoIntegridade {
    pub fn as_str(&self) -> &'static str {
        match self {
            EstadoIntegridade::Valido => "valido",
            EstadoIntegridade::Invalido => "invalido",
            EstadoIntegridade::Desconhecido => "desconhecido",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "valido" => EstadoIntegridade::Valido,
            "invalido" => EstadoIntegridade::Invalido,
            _ => EstadoIntegridade::Desconhecido,
        }
    }
}

impl Default for EstadoIntegridade {
    fn default() -> Self {
        EstadoIntegridade::Desconhecido
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_sao_unicos() {
        let id1 = ObraId::new();
        let id2 = ObraId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn estado_integridade_serializa() {
        assert_eq!(EstadoIntegridade::Valido.as_str(), "valido");
        assert_eq!(EstadoIntegridade::from_str("valido"), EstadoIntegridade::Valido);
        assert_eq!(
            EstadoIntegridade::from_str("desconhecido"),
            EstadoIntegridade::Desconhecido
        );
    }
}
