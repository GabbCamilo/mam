//! Tipos JSON-RPC 2.0 e contratos do Manga AI Engine.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Erros de coordenação do Manga AI Engine.
#[derive(Debug, Error)]
pub enum AiEngineError {
    #[error("Erro de E/S ao comunicar com o AI Engine: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de serialização JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Timeout aguardando resposta do AI Engine")]
    Timeout,

    #[error("Processo auxiliar do AI Engine não está em execução")]
    ProcessoEncerrado,

    #[error("Erro retornado pelo AI Engine ({codigo}): {mensagem}")]
    Rpc { codigo: i64, mensagem: String },
}

/// Requisição JSON-RPC 2.0 enviada em JSONL para o processo Python.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

impl JsonRpcRequest {
    pub fn nova(id: u64, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// Erro JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

/// Resposta JSON-RPC 2.0 recebida do processo Python.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// Informações de modelo opcional retornado por `models_list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeloInfo {
    pub id: String,
    pub nome: String,
    pub formato: String,
    pub caminho: String,
    pub tamanho_bytes: u64,
    pub disponivel: bool,
}
