"""Detector modular de balões de fala em páginas de mangá (CPU leve com Pillow + NumPy)."""

from __future__ import annotations

from collections import deque
from pathlib import Path
from typing import Any


def _componentes_conexos_rapidos(mask: Any) -> list[tuple[int, int, int, int, int]]:
    """Encontra bounding boxes `(x0, y0, x1, y1, area)` de regiões True em matriz booleana 2D."""
    import numpy as np

    h, w = mask.shape
    visitado = np.zeros((h, w), dtype=bool)
    componentes: list[tuple[int, int, int, int, int]] = []

    ys, xs = np.nonzero(mask)
    for idx in range(len(ys)):
        sy, sx = int(ys[idx]), int(xs[idx])
        if visitado[sy, sx]:
            continue

        fila: deque[tuple[int, int]] = deque([(sy, sx)])
        visitado[sy, sx] = True
        min_x = max_x = sx
        min_y = max_y = sy
        area = 0

        while fila:
            cy, cx = fila.popleft()
            area += 1
            if cx < min_x:
                min_x = cx
            if cx > max_x:
                max_x = cx
            if cy < min_y:
                min_y = cy
            if cy > max_y:
                max_y = cy

            for ny, nx in ((cy - 1, cx), (cy + 1, cx), (cy, cx - 1), (cy, cx + 1)):
                if 0 <= ny < h and 0 <= nx < w and not visitado[ny, nx] and mask[ny, nx]:
                    visitado[ny, nx] = True
                    fila.append((ny, nx))

        componentes.append((min_x, min_y, max_x, max_y, area))

    return componentes


def detectar_baloes_cpu(caminho_imagem: str, max_dim: int = 560) -> list[dict[str, Any]]:
    """Detecta regiões candidatas a balões de fala usando análise de luminância e contraste local.

    Retorna coordenadas normalizadas `[x, y, w, h]` em `[0.0, 1.0]` e cor predominante de máscara.
    """
    alvo = Path(caminho_imagem)
    if not alvo.is_file():
        return []

    try:
        from PIL import Image
        import numpy as np
    except Exception:  # noqa: BLE001
        return []

    with Image.open(alvo) as img:
        gray_orig = img.convert("L")
        orig_w, orig_h = gray_orig.size
        if orig_w == 0 or orig_h == 0:
            return []

        escala = min(1.0, max_dim / float(max(orig_w, orig_h)))
        nw = max(32, int(round(orig_w * escala)))
        nh = max(32, int(round(orig_h * escala)))
        small = gray_orig.resize((nw, nh), Image.Resampling.BILINEAR)
        arr = np.asarray(small, dtype=np.uint8)

    # Regiões claras candidatas ao interior de balões
    bright = arr >= 228
    # Preenche pequenos pontos escuros (texto) dentro de balões usando dilatação/fechamento simples 3x3
    padded = np.pad(bright, 1, mode="constant", constant_values=False)
    dilated = (
        padded[:-2, :-2]
        | padded[:-2, 1:-1]
        | padded[:-2, 2:]
        | padded[1:-1, :-2]
        | padded[1:-1, 1:-1]
        | padded[1:-1, 2:]
        | padded[2:, :-2]
        | padded[2:, 1:-1]
        | padded[2:, 2:]
    )

    componentes = _componentes_conexos_rapidos(dilated)
    area_total = float(nw * nh)
    baloes: list[dict[str, Any]] = []

    for x0, y0, x1, y1, area in componentes:
        bw = x1 - x0 + 1
        bh = y1 - y0 + 1
        bbox_area = float(bw * bh)
        frac_area = bbox_area / area_total

        # Descarta ruídos minúsculos e fundos inteiros de página
        if frac_area < 0.004 or frac_area > 0.22:
            continue
        if bw < 14 or bh < 14:
            continue
        # Ignora margens externas coladas na borda da página
        if (x0 == 0 and x1 >= nw - 2) or (y0 == 0 and y1 >= nh - 2):
            continue

        recorte = arr[y0 : y1 + 1, x0 : x1 + 1]
        dark_ratio = float(np.mean(recorte < 115))
        bright_ratio = float(np.mean(recorte >= 220))
        fill_ratio = float(area) / bbox_area

        # Balão típico: maioria clara (>= 50%), com presença real de texto escuro (2%..42%) e boa compacidade
        if bright_ratio < 0.50 or dark_ratio < 0.02 or dark_ratio > 0.42 or fill_ratio < 0.45:
            continue

        # Ajusta pequena margem interna para mascarar o texto sem apagar a borda preta do balão
        pad_x = max(1, int(bw * 0.06))
        pad_y = max(1, int(bh * 0.06))
        ix0 = min(x1, x0 + pad_x)
        iy0 = min(y1, y0 + pad_y)
        ix1 = max(ix0 + 1, x1 - pad_x)
        iy1 = max(iy0 + 1, y1 - pad_y)

        nx = round(ix0 / float(nw), 4)
        ny = round(iy0 / float(nh), 4)
        nw_norm = round((ix1 - ix0 + 1) / float(nw), 4)
        nh_norm = round((iy1 - iy0 + 1) / float(nh), 4)

        baloes.append(
            {
                "id": f"balao-{len(baloes) + 1}",
                "bbox": [nx, ny, nw_norm, nh_norm],
                "cor_mascara": "#ffffff",
                "confianca": round(min(0.95, 0.55 + bright_ratio * 0.35), 2),
            }
        )

    # Ordena em ordem de leitura mangá (topo -> base, direita -> esquerda)
    baloes.sort(key=lambda b: (round(b["bbox"][1] * 8), -b["bbox"][0]))
    for idx, b in enumerate(baloes):
        b["id"] = f"balao-{idx + 1}"

    return baloes
