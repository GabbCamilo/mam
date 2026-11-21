"""Pipeline local de IA (OCR, detecção de balões e tradução) com CPU obrigatório."""

from __future__ import annotations

import importlib.util
from typing import Any

from .bubble_detector import detectar_baloes_cpu
from .context_translator import traduzir_balao_com_contexto, traduzir_baloes_pagina_com_contexto
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
    baloes = detectar_baloes_cpu(caminho_imagem)
    return {
        "tipo": "detect_bubbles",
        "caminho_imagem": caminho_imagem,
        "provedor": detectar_provedores_inferencia()["prioritario"],
        "baloes": baloes,
        "estado_revisao": "nao_revisado",
    }


def detectar_idioma_texto(texto: str) -> str:
    """Identifica automaticamente o idioma predominante de um texto (estilo detector automático)."""
    if not texto or not texto.strip():
        return "auto"

    tem_kana = any("\u3040" <= ch <= "\u30ff" for ch in texto)
    if tem_kana:
        return "ja"
    tem_hangul = any("\uac00" <= ch <= "\ud7af" for ch in texto)
    if tem_hangul:
        return "ko"
    tem_cjk = any("\u4e00" <= ch <= "\u9fff" for ch in texto)
    if tem_cjk:
        return "zh"

    lower = f" {texto.lower()} "
    if any(ch in lower for ch in "ãõçáéíóúâêôà"):
        return "pt"
    if any(w in lower for w in (" não ", " você ", " para ", " que ", " com ", " está ", " sim ")):
        return "pt"
    return "en"


def normalizar_idioma_destino(idioma_destino: str) -> str:
    """Normaliza o idioma de destino para as duas opções principais: 'pt-BR' ou 'en'."""
    norm = (idioma_destino or "pt-BR").strip().lower()
    if norm in ("en", "en-us", "en-gb", "eng", "english", "ingles", "inglês"):
        return "en"
    return "pt-BR"


def traduzir_texto_isolado(texto: str, idioma_origem: str, idioma_destino: str) -> tuple[str, str]:
    """Identifica o idioma do balão isolado e executa a tradução contextual para pt-BR ou en."""
    res = traduzir_balao_com_contexto(
        texto=texto,
        idioma_origem=idioma_origem,
        idioma_destino=idioma_destino,
    )
    return str(res["idioma_detectado"]), str(res["texto_traduzido"])


def traduzir_regioes(params: dict[str, Any]) -> dict[str, Any]:
    """Traduz textos reconhecidos com janela de contexto entre balões vizinhos."""
    textos = [str(t) for t in (params.get("textos") or [])]
    idioma_origem = str(params.get("idioma_origem", "auto"))
    idioma_destino = normalizar_idioma_destino(str(params.get("idioma_destino", "pt-BR")))
    modalidade = str(params.get("modalidade", "sobreposicao"))

    baloes_ctx = [{"texto_original": t} for t in textos]
    processados = traduzir_baloes_pagina_com_contexto(
        baloes_ctx,
        idioma_origem=idioma_origem,
        idioma_destino=idioma_destino,
    )

    traducoes = [
        {
            "original": item["texto_original"],
            "traduzido": item["texto_traduzido"],
            "idioma_detectado": item["idioma_detectado"],
            "metodo_traducao": item["metodo_traducao"],
            "analise_contexto": item["analise_contexto"],
            "estado_revisao": "nao_revisado",
        }
        for item in processados
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
    idioma_origem = str(params.get("idioma_origem", "auto"))
    idioma_destino = normalizar_idioma_destino(str(params.get("idioma_destino", "pt-BR")))
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

    baloes = detectar_baloes_cpu(caminho_imagem)
    baloes_ctx = traduzir_baloes_pagina_com_contexto(
        baloes,
        idioma_origem=idioma_origem,
        idioma_destino=idioma_destino,
        modelo_sessao=adaptador._sessao,
    )
    regioes = []
    for idx, b in enumerate(baloes_ctx):
        texto_orig = str(b.get("texto_original", ""))
        texto_trad = str(b.get("texto_traduzido", ""))
        if not texto_trad:
            rotulo = "Bubble" if idioma_destino == "en" else "Balão"
            texto_trad = f"[{rotulo} {idx + 1}]"
        regioes.append(
            {
                "id": b["id"],
                "bbox": b["bbox"],
                "cor_mascara": b.get("cor_mascara", "#ffffff"),
                "confianca": b.get("confianca", 0.7),
                "texto_original": texto_orig,
                "texto_traduzido": texto_trad,
                "idioma_origem": b["idioma_detectado"],
                "idioma_destino": idioma_destino,
                "metodo_traducao": b["metodo_traducao"],
                "analise_contexto": b["analise_contexto"],
                "estado_revisao": "nao_revisado",
            }
        )

    return {
        "tipo": "process_page",
        "caminho_imagem": caminho_imagem,
        "idioma_origem": idioma_origem,
        "idioma_destino": idioma_destino,
        "provedor": detectar_provedores_inferencia()["prioritario"],
        "dimensoes_imagem": dimensoes,
        "onnx_status": adaptador.info_status(),
        "regioes": regioes,
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


def _profile_cache_path():
    import os
    from pathlib import Path

    base = Path(os.environ.get("LOCALAPPDATA") or Path.home()) / "MAM"
    base.mkdir(parents=True, exist_ok=True)
    return base / "site_profiles.json"


def analyze_reader_dom(params: dict[str, Any]) -> dict[str, Any]:
    """Guia IA estrutural que identifica seletores de capítulos/leitor e salva perfil por domínio."""
    import json
    from urllib.parse import urlparse

    url = str(params.get("url", "")).strip()
    dominio = urlparse(url).netloc.lower()
    candidatos = params.get("candidatos") or []
    classificados = classificar_links_capitulos({"url_obra": url, "candidatos": candidatos})

    perfil = {
        "dominio": dominio,
        "padrao_capitulo": "/chapter/" if any("/chapter/" in c["href"].lower() for c in classificados["capitulos"]) else "generico",
        "total_detectado": len(classificados["capitulos"]),
    }

    if dominio:
        cache_path = _profile_cache_path()
        try:
            perfis = json.loads(cache_path.read_text(encoding="utf-8")) if cache_path.exists() else {}
        except Exception:
            perfis = {}
        perfis[dominio] = perfil
        cache_path.write_text(json.dumps(perfis, ensure_ascii=False, indent=2), encoding="utf-8")

    return {"perfil": perfil, "capitulos": classificados["capitulos"]}


def smart_discover_chapters(params: dict[str, Any]) -> dict[str, Any]:
    """Orquestra o Leitor Web Inteligente (CDP) + classificação estrutural de capítulos."""
    import re
    from .web_reader import discover_work_chapters

    url_obra = str(params.get("url_obra", "")).strip()
    idioma_preferido = str(params.get("idioma_preferido") or "pt-br").strip()
    max_paginas = params.get("max_paginas")

    bruto = discover_work_chapters(
        url=url_obra,
        preferred_lang=idioma_preferido,
        max_pages=int(max_paginas) if max_paginas is not None else None,
    )

    capitulos_out: list[dict[str, Any]] = []
    for item in bruto.get("capitulos") or []:
        href = str(item.get("href") or "").strip()
        texto = str(item.get("text") or "").strip()
        row_lang = str(item.get("rowLang") or "").strip().lower()
        data_num = str(item.get("dataNumber") or "").strip()

        numero = None
        m = re.search(r"(?:cap[íi]tulo|chapter|cap\.?|ch\.?)\s*[-:]?\s*(\d+)", f"{texto} {href}".lower())
        if m:
            numero = int(m.group(1))
        elif data_num.isdigit():
            numero = int(data_num)

        idioma = None
        comb = f"{href} {texto} {row_lang}".lower()
        if "pt-br" in comb or "/pt/" in comb or "portuguese" in comb or row_lang in ("pt", "br", "pt-br"):
            idioma = "PT"
        elif "/en/" in comb or "english" in comb or row_lang in ("en", "gb"):
            idioma = "ENG"
        elif "/ja/" in comb or "/jp/" in comb or "japanese" in comb or row_lang in ("ja", "jp"):
            idioma = "JPN"

        capitulos_out.append({
            "fonte_id": "extrator-link",
            "obra_id_fonte": url_obra,
            "capitulo_id_fonte": href,
            "numero": numero,
            "titulo": texto or None,
            "url": href,
            "idioma": idioma,
        })

    capitulos_out.sort(key=lambda c: (c["numero"] is None, c["numero"] or 0))
    return {
        "titulo": bruto.get("titulo") or "Obra Importada",
        "capa_url": bruto.get("capa_url"),
        "url_obra": url_obra,
        "paginas_varridas": bruto.get("paginas_varridas", 1),
        "capitulos": capitulos_out,
    }


def smart_extract_chapter_pages(params: dict[str, Any]) -> dict[str, Any]:
    """Orquestra o Leitor Web Inteligente (CDP) para extrair todas as páginas de um capítulo."""
    from .web_reader import extract_chapter_pages

    url_capitulo = str(params.get("url_capitulo", "")).strip()
    return extract_chapter_pages(url_capitulo)


def smart_search_works(params: dict[str, Any]) -> dict[str, Any]:
    """Pesquisa obras por título em um conector web usando CDP."""
    from .web_reader import search_works_cdp

    url_busca = str(params.get("url_busca", "")).strip()
    conector_id = str(params.get("conector_id") or "mangafire").strip()
    return {"obras": search_works_cdp(url_busca, conector_id)}
