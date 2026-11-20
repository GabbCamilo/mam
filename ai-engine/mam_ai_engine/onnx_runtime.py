"""Adaptador modular de inferência ONNX Runtime (CPU obrigatório, GPU opcional) e pré-processamento."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from typing import Any


def carregar_imagem_rgb(caminho_imagem: str) -> dict[str, Any]:
    """Lê dimensões reais e prepara metadados da imagem via Pillow/NumPy sem modificar o original."""
    alvo = Path(caminho_imagem)
    if not alvo.exists() or not alvo.is_file():
        return {"disponivel": False, "largura": 0, "altura": 0}

    try:
        from PIL import Image
        import numpy as np

        with Image.open(alvo) as img:
            rgb = img.convert("RGB")
            arr = np.asarray(rgb, dtype=np.uint8)
            altura, largura = int(arr.shape[0]), int(arr.shape[1])
            return {
                "disponivel": True,
                "largura": largura,
                "altura": altura,
                "canais": int(arr.shape[2]) if arr.ndim == 3 else 1,
            }
    except Exception:  # noqa: BLE001
        return {"disponivel": False, "largura": 0, "altura": 0}


class OnnxSessionAdapter:
    """Encapsula criação de `InferenceSession` ONNX Runtime com CPU obrigatório."""

    def __init__(self, caminho_modelo: str | None = None, usar_gpu_se_disponivel: bool = False) -> None:
        self.caminho_modelo = caminho_modelo
        self.usar_gpu_se_disponivel = usar_gpu_se_disponivel
        self._ort_disponivel = importlib.util.find_spec("onnxruntime") is not None
        self._sessao: Any = None
        self.ultimo_erro: str | None = None

    @property
    def disponivel(self) -> bool:
        return (
            self._ort_disponivel
            and self.caminho_modelo is not None
            and Path(self.caminho_modelo).is_file()
        )

    def carregar(self) -> bool:
        """Tenta inicializar a sessão ONNX com CPUExecutionProvider obrigatório."""
        if not self._ort_disponivel:
            self.ultimo_erro = "Biblioteca 'onnxruntime' não instalada no ambiente Python"
            return False

        if not self.caminho_modelo or not Path(self.caminho_modelo).is_file():
            self.ultimo_erro = "Arquivo de modelo .onnx não informado ou inexistente"
            return False

        import onnxruntime as ort  # type: ignore[import-not-found]

        provedores_disponiveis = ort.get_available_providers()
        providers: list[str] = []
        if self.usar_gpu_se_disponivel:
            for gpu_prov in ("DmlExecutionProvider", "ROCMExecutionProvider", "CUDAExecutionProvider"):
                if gpu_prov in provedores_disponiveis:
                    providers.append(gpu_prov)
        providers.append("CPUExecutionProvider")

        self._sessao = ort.InferenceSession(self.caminho_modelo, providers=providers)
        self.ultimo_erro = None
        return True

    def info_status(self) -> dict[str, Any]:
        return {
            "onnxruntime_instalado": self._ort_disponivel,
            "modelo_configurado": self.caminho_modelo,
            "sessao_ativa": self._sessao is not None,
            "ultimo_erro": self.ultimo_erro,
        }
