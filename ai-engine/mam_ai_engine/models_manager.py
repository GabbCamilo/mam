"""Gerenciamento opcional de modelos locais de IA sem downloads automáticos."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
from typing import Any


SUPPORTED_EXTENSIONS = {".onnx", ".pt", ".pth", ".bin", ".safetensors"}


def calcular_sha256_arquivo(caminho: Path) -> str:
    """Calcula SHA-256 em streaming para validação de integridade do modelo."""
    hasher = hashlib.sha256()
    with caminho.open("rb") as f:
        for bloco in iter(lambda: f.read(65536), b""):
            hasher.update(bloco)
    return hasher.hexdigest()


def listar_modelos_locais(diretorios: list[str] | None = None) -> list[dict[str, Any]]:
    """Inspeciona diretórios configurados buscando modelos já instalados localmente.

    Nunca realiza downloads automáticos.
    """
    alvos = diretorios or _diretorios_padrao()
    modelos: list[dict[str, Any]] = []

    for dir_str in alvos:
        base = Path(dir_str)
        if not base.exists() or not base.is_dir():
            continue

        for item in sorted(base.rglob("*")):
            if item.is_file() and item.suffix.lower() in SUPPORTED_EXTENSIONS:
                stat = item.stat()
                modelos.append(
                    {
                        "id": item.stem,
                        "nome": item.name,
                        "formato": item.suffix.lower().lstrip("."),
                        "caminho": str(item),
                        "tamanho_bytes": stat.st_size,
                        "disponivel": True,
                    }
                )

    return modelos


def inspecionar_modelo(caminho_modelo: str) -> dict[str, Any]:
    """Retorna metadados de um modelo específico em disco."""
    alvo = Path(caminho_modelo)
    if not alvo.exists() or not alvo.is_file():
        return {
            "caminho": caminho_modelo,
            "disponivel": False,
            "erro": "Modelo não encontrado no caminho informado",
        }

    return {
        "id": alvo.stem,
        "nome": alvo.name,
        "formato": alvo.suffix.lower().lstrip("."),
        "caminho": str(alvo),
        "tamanho_bytes": alvo.stat().st_size,
        "sha256": calcular_sha256_arquivo(alvo),
        "prioridade_runtime": "onnxruntime" if alvo.suffix.lower() == ".onnx" else "pytorch",
        "disponivel": True,
    }


def _diretorios_padrao() -> list[str]:
    env_dir = os.environ.get("MAM_MODELS_DIR")
    if env_dir:
        return [p for p in env_dir.split(os.pathsep) if p]
    return []
