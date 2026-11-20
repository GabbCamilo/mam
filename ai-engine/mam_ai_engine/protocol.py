"""Protocolo JSON-RPC 2.0 sobre stdio (JSONL) para o Manga AI Engine."""

from __future__ import annotations

import json
from typing import Any


PARSE_ERROR = -32700
INVALID_REQUEST = -32600
METHOD_NOT_FOUND = -32601
INVALID_PARAMS = -32602
INTERNAL_ERROR = -32603


def parse_request(line: str) -> tuple[dict[str, Any] | None, dict[str, Any] | None]:
    """Faz o parse de uma linha JSONL para requisição JSON-RPC 2.0."""
    try:
        payload = json.loads(line)
    except json.JSONDecodeError as exc:
        return None, make_error(None, PARSE_ERROR, f"JSON inválido: {exc}")

    if not isinstance(payload, dict) or payload.get("jsonrpc") != "2.0":
        req_id = payload.get("id") if isinstance(payload, dict) else None
        return None, make_error(req_id, INVALID_REQUEST, "Requisição JSON-RPC 2.0 inválida")

    method = payload.get("method")
    if not isinstance(method, str) or not method.strip():
        return None, make_error(payload.get("id"), INVALID_REQUEST, "Campo 'method' obrigatório")

    return payload, None


def make_result(req_id: Any, result: Any) -> dict[str, Any]:
    """Constrói uma resposta de sucesso JSON-RPC 2.0."""
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": result,
    }


def make_error(req_id: Any, code: int, message: str, data: Any = None) -> dict[str, Any]:
    """Constrói uma resposta de erro JSON-RPC 2.0."""
    err: dict[str, Any] = {"code": code, "message": message}
    if data is not None:
        err["data"] = data
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": err,
    }
