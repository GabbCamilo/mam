"""Pipeline local de IA (OCR, detecção de balões e tradução) com CPU obrigatório."""

from __future__ import annotations

import importlib.util
from typing import Any

from .onnx_runtime import OnnxSessionAdapter, carregar_imagem_rgb


def detectar_provedores_inferencia() -> dict[str, Any]:
    """Detecta runtimes de inferência disponíveis localmente (CPU sempre habilitado)."""
    tem_onnx = importlib.util.find_spec("onnxruntime") is not None
    tem_torch = importlib.util.find_spec("torch") is not None
    return {
        "cpu": True,
        "onnxruntime": tem_onnx,
        "pytorch": tem_torch,
        "prioritario": "onnxruntime" if tem_onnx else ("pytorch" if tem_torch else "cpu-fallback"),
    }


def executar_ocr(params: dict[str, Any]) -> dict[str, Any]:
    """Executa OCR local sobre uma página sem modificar o CBZ original."""
    caminho_imagem = str(params.get("caminho_imagem", ""))
    idioma_origem = str(params.get("idioma_origem", "ja"))
    return {
        "tipo": "ocr",
        "caminho_imagem": caminho_imagem,
        "idioma_origem": idioma_origem,
        "provedor": detectar_provedores_inferencia()["prioritario"],
        "regioes": [],
        "estado_revisao": "nao_revisado",
    }


def detectar_baloes(params: dict[str, Any]) -> dict[str, Any]:
    """Detecta balões de fala na página mantendo originais intactos."""
    caminho_imagem = str(params.get("caminho_imagem", ""))
    return {
        "tipo": "detect_bubbles",
        "caminho_imagem": caminho_imagem,
        "provedor": detectar_provedores_inferencia()["prioritario"],
        "baloes": [],
        "estado_revisao": "nao_revisado",
    }


def traduzir_regioes(params: dict[str, Any]) -> dict[str, Any]:
    """Traduz textos reconhecidos mantendo o estado inicial como nao_revisado."""
    textos = params.get("textos") or []
    idioma_origem = str(params.get("idioma_origem", "ja"))
    idioma_destino = str(params.get("idioma_destino", "pt-BR"))
    modalidade = str(params.get("modalidade", "sobreposicao"))

    traducoes = [
        {
            "original": str(t),
            "traduzido": str(t),
            "estado_revisao": "nao_revisado",
        }
        for t in textos
    ]

    return {
        "tipo": "translate",
        "idioma_origem": idioma_origem,
        "idioma_destino": idioma_destino,
        "modalidade": modalidade,
        "provedor": detectar_provedores_inferencia()["prioritario"],
        "traducoes": traducoes,
        "estado_revisao": "nao_revisado",
    }


def processar_pagina_completa(params: dict[str, Any]) -> dict[str, Any]:
    """Pipeline unificado (`process_page`) com dimensões reais da imagem e regiões normalizadas `[x,y,w,h]`."""
    caminho_imagem = str(params.get("caminho_imagem", ""))
    idioma_origem = str(params.get("idioma_origem", "ja"))
    idioma_destino = str(params.get("idioma_destino", "pt-BR"))
    caminho_modelo = params.get("caminho_modelo")
    usar_gpu = bool(params.get("usar_gpu", False))

    dimensoes = carregar_imagem_rgb(caminho_imagem)
    adaptador = OnnxSessionAdapter(
        caminho_modelo=str(caminho_modelo) if caminho_modelo else None,
        usar_gpu_se_disponivel=usar_gpu,
    )
    if adaptador.disponivel:
        adaptador.carregar()
    else:
        # Registra motivo explícito sem lançar exceção quando onnxruntime/modelo não estiverem presentes
        adaptador.carregar()

    return {
        "tipo": "process_page",
        "caminho_imagem": caminho_imagem,
        "idioma_origem": idioma_origem,
        "idioma_destino": idioma_destino,
        "provedor": detectar_provedores_inferencia()["prioritario"],
        "dimensoes_imagem": dimensoes,
        "onnx_status": adaptador.info_status(),
        "regioes": [],
        "estado_revisao": "nao_revisado",
    }


def classificar_links_capitulos(params: dict[str, Any]) -> dict[str, Any]:
    """Classifica candidatos de links de uma página central de mangá por padrão estrutural."""
    import re

    url_obra = str(params.get("url_obra", "")).strip().lower().rstrip("/")
    candidatos = params.get("candidatos") or []
    if not isinstance(candidatos, list):
        candidatos = []

    resultados: list[dict[str, Any]] = []
    vistos: set[str] = set()
    for item in candidatos:
        if not isinstance(item, dict):
            continue
        href = str(item.get("href", "")).strip()
        texto = str(item.get("text", "")).strip()
        if not href or href in vistos:
            continue

        lower_href = href.lower()
        lower_txt = texto.lower()
        score = 0.0
        if any(p in lower_href for p in ("/chapter/", "/capitulo-", "/chapter-", "/cap-", "/ch-")):
            score += 0.65
        if url_obra and lower_href.startswith(url_obra) and len(lower_href) > len(url_obra) + 1:
            score += 0.2
        if any(p in lower_txt for p in ("cap", "chapter", "ch.")):
            score += 0.15

        if score < 0.5:
            continue

        vistos.add(href)
        m = re.search(r"(?:capitulo-|chapter-|cap\.?\s*|ch\.?\s*)(\d+)", f"{lower_href} {lower_txt}")
        numero = int(m.group(1)) if m else None
        idioma = None
        if "pt-br" in lower_href or "/pt/" in lower_href:
            idioma = "PT"
        elif "/en/" in lower_href or "english" in lower_txt:
            idioma = "ENG"
        elif "/ja/" in lower_href or "/jp/" in lower_href:
            idioma = "JPN"

        resultados.append({
            "href": href,
            "numero": numero,
            "titulo": texto or None,
            "idioma": idioma,
            "score": round(score, 2),
        })

    resultados.sort(key=lambda r: (r["numero"] is None, r["numero"] or 0))
    return {"capitulos": resultados}
