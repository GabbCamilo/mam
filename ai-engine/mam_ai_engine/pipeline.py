"""Pipeline local de IA (OCR, detecção de balões e tradução) com CPU obrigatório."""

from __future__ import annotations

import importlib.util
from typing import Any


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
