//! Structs de modelo do banco de dados MAM.
//! Representam obras, volumes, capítulos, configurações, histórico, integridade e resultados de IA.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use mam_core::{ObraId, VolumeId, CapituloId, CaminhoRelativo, EstadoIntegridade};

/// Obra da biblioteca (mangá ou quadrinho).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obra {
    pub id: ObraId,
    pub titulo: String,
    pub caminho_relativo: CaminhoRelativo,
    #[serde(default)]
    pub origem: Option<String>,
    pub data_arquivamento: DateTime<Utc>,
    pub estado_integridade: EstadoIntegridade,
    #[serde(default)]
    pub metadados_complementares: Option<String>,
}

/// Volume de uma obra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: VolumeId,
    pub obra_id: ObraId,
    pub numero: i32,
    pub caminho_relativo: CaminhoRelativo,
    #[serde(default)]
    pub titulo: Option<String>,
}

/// Capítulo de um volume/obra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capitulo {
    pub id: CapituloId,
    #[serde(default)]
    pub volume_id: Option<VolumeId>,
    pub obra_id: ObraId,
    pub numero: i32,
    #[serde(default)]
    pub titulo_original: Option<String>,
    pub caminho_relativo: CaminhoRelativo,
    pub quantidade_paginas: i32,
    #[serde(default)]
    pub idioma: Option<String>,
    pub data_arquivamento: DateTime<Utc>,
    pub estado_integridade: EstadoIntegridade,
}

/// Metadados detalhados de um capítulo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadadosCapitulo {
    pub capitulo_id: CapituloId,
    pub paginas_json: String, // JSON array de páginas
    #[serde(default)]
    pub origem_conteudo: Option<String>,
    pub data_arquivamento: DateTime<Utc>,
}

/// Configuração da biblioteca (chave-valor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguracaoBiblioteca {
    pub chave: String,
    pub valor: String,
    pub atualizado_em: DateTime<Utc>,
}

/// Registro de operação no histórico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricoOperacao {
    pub id: Uuid,
    pub tipo: String,
    #[serde(default)]
    pub alvo_caminho: Option<String>,
    #[serde(default)]
    pub detalhes_json: Option<String>,
    #[serde(default)]
    pub usuario: Option<String>,
    pub criado_em: DateTime<Utc>,
}

/// Estado de integridade de um arquivo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegridadeArquivo {
    pub caminho_relativo: String,
    pub hash_sha256: String,
    pub estado: String,
    #[serde(default)]
    pub verificado_em: Option<DateTime<Utc>>,
}

/// Referência a um resultado de IA (OCR, balões, tradução).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoIA {
    pub id: Uuid,
    pub capitulo_id: CapituloId,
    pub pagina_numero: i32,
    pub modelo_id: String,
    pub versao_modelo: String,
    pub tipo_processamento: String,
    pub resultado_json: String, // JSON com coordenadas, texto, idioma, estado revisao
    #[serde(default)]
    pub idioma_origem: Option<String>,
    #[serde(default)]
    pub idioma_destino: Option<String>,
    pub estado_revisao: String,
    pub criado_em: DateTime<Utc>,
}
