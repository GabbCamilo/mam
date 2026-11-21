"""Servidor JSON-RPC 2.0 sobre stdio (JSONL) do Manga AI Engine."""

from __future__ import annotations

import json
import sys
from typing import Any, Callable

from .models_manager import inspecionar_modelo, listar_modelos_locais
from .pipeline import (
    analyze_reader_dom,
    classificar_links_capitulos,
    detectar_baloes,
    detectar_provedores_inferencia,
    executar_ocr,
    processar_pagina_completa,
    smart_discover_chapters,
    smart_extract_chapter_pages,
    smart_search_works,
    traduzir_regioes,
)
from .protocol import (
    INTERNAL_ERROR,
    METHOD_NOT_FOUND,
    make_error,
    make_result,
    parse_request,
)


def handle_request(req: dict[str, Any]) -> tuple[dict[str, Any], bool]:
    """Processa uma requisição JSON-RPC 2.0 e retorna `(resposta, deve_encerrar)`."""
    req_id = req.get("id")
    method = req["method"]
    params = req.get("params") or {}
    if not isinstance(params, dict):
        params = {}

    handlers: dict[str, Callable[[dict[str, Any]], Any]] = {
        "ping": lambda _p: {
            "status": "ok",
            "provedores": detectar_provedores_inferencia(),
        },
        "ocr": executar_ocr,
        "detect_bubbles": detectar_baloes,
        "translate": traduzir_regioes,
        "process_page": processar_pagina_completa,
        "classify_chapter_links": classificar_links_capitulos,
        "analyze_reader_dom": analyze_reader_dom,
        "smart_discover_chapters": smart_discover_chapters,
        "smart_extract_chapter_pages": smart_extract_chapter_pages,
        "smart_search_works": smart_search_works,
        "models_list": lambda p: listar_modelos_locais(p.get("diretorios")),
        "models_inspect": lambda p: inspecionar_modelo(str(p.get("caminho_modelo", ""))),
    }

    if method == "shutdown":
        return make_result(req_id, {"encerrado": True}), True

    handler = handlers.get(method)
    if handler is None:
        return make_error(req_id, METHOD_NOT_FOUND, f"Método não encontrado: {method}"), False

    try:
        resultado = handler(params)
        return make_result(req_id, resultado), False
    except Exception as exc:  # noqa: BLE001
        return make_error(req_id, INTERNAL_ERROR, f"Erro interno: {exc}"), False


def main() -> int:
    """Loop principal JSONL lendo de stdin e escrevendo em stdout."""
    if hasattr(sys.stdin, "reconfigure"):
        sys.stdin.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue

        req, err_resp = parse_request(line)
        if err_resp is not None or req is None:
            sys.stdout.write(json.dumps(err_resp, ensure_ascii=False) + "\n")
            sys.stdout.flush()
            continue

        resp, should_exit = handle_request(req)
        sys.stdout.write(json.dumps(resp, ensure_ascii=False) + "\n")
        sys.stdout.flush()
        if should_exit:
            break

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
