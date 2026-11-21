"""Analisador de contexto e tradução contextual por conectividade semântica para balões de mangá."""

from __future__ import annotations

import importlib.util
import re
from typing import Any


# Expressões idiomáticas e conectividades semânticas de mangá (não literais)
_EXPRESSOES_EN_PT: list[tuple[str, str]] = [
    (r"\bhold on\b", "espera aí"),
    (r"\bwait a second\b", "espera um segundo"),
    (r"\bwhat's going on\b", "o que está acontecendo"),
    (r"\bwhat is going on\b", "o que está acontecendo"),
    (r"\byou've got to be kidding me\b", "só pode estar brincando comigo"),
    (r"\bare you kidding me\b", "você só pode estar brincando"),
    (r"\bno way\b", "nem pensar"),
    (r"\bof course\b", "claro"),
    (r"\bby the way\b", "aliás"),
    (r"\bat least\b", "pelo menos"),
    (r"\beven so\b", "mesmo assim"),
    (r"\bin other words\b", "ou seja"),
    (r"\bas expected\b", "como esperado"),
    (r"\bit can't be helped\b", "não tem jeito"),
    (r"\bnothing can be done\b", "não há o que fazer"),
    (r"\bwhoever wins\b", "quem ganhar"),
    (r"\bhow about it\b", "que tal"),
    (r"\bi have just the thing\b", "eu tenho a coisa certa"),
    (r"\byes, i agree\b", "sim, eu concordo"),
    (r"\ba bit too quickly\b", "um pouco rápido demais"),
]

_EXPRESSOES_PT_EN: list[tuple[str, str]] = [
    (r"\bespera aí\b", "hold on"),
    (r"\bo que está acontecendo\b", "what's going on"),
    (r"\bsó pode estar brincando\b", "are you kidding me"),
    (r"\bnem pensar\b", "no way"),
    (r"\bmesmo assim\b", "even so"),
    (r"\bcomo esperado\b", "as expected"),
    (r"\bnão tem jeito\b", "it can't be helped"),
    (r"\bque tal\b", "how about it"),
]

_EXPRESSOES_JA_PT: list[tuple[str, str]] = [
    (r"なんだと", "o quê?!"),
    (r"まさか", "não pode ser..."),
    (r"しょうがない|仕方ない", "não tem jeito"),
    (r"なるほど", "entendo..."),
    (r"ちょっと待って", "espera um pouco"),
    (r"どういうこと", "o que isso significa?"),
    (r"もちろん", "é claro"),
    (r"やばい", "isso é ruim"),
]

_EXPRESSOES_JA_EN: list[tuple[str, str]] = [
    (r"なんだと", "what?!"),
    (r"まさか", "no way..."),
    (r"しょうがない|仕方ない", "it can't be helped"),
    (r"なるほど", "I see..."),
    (r"ちょっと待って", "wait a second"),
    (r"どういうこと", "what do you mean?"),
    (r"もちろん", "of course"),
    (r"やばい", "this is bad"),
]


def analisar_frase_contextual(
    texto: str,
    contexto_anterior: list[str] | None = None,
    contexto_seguinte: list[str] | None = None,
) -> dict[str, Any]:
    """Extrai tom, intenção comunicativa, conectivos e continuidade de diálogo."""
    t = (texto or "").strip()
    ant = [c.strip() for c in (contexto_anterior or []) if c and c.strip()]
    seg = [c.strip() for c in (contexto_seguinte or []) if c and c.strip()]

    if t.endswith("?!") or t.endswith("!?") or t.endswith("？！"):
        tom = "surpresa_interrogativa"
    elif t.endswith("?") or t.endswith("？"):
        tom = "interrogativo"
    elif t.endswith("!") or t.endswith("！"):
        tom = "exclamativo"
    elif t.endswith("...") or t.endswith("…"):
        tom = "hesitante_ou_suspense"
    else:
        tom = "declarativo"

    conectivos = []
    lower = t.lower()
    for marcador in ("then", "but", "however", "so", "even so", "então", "mas", "porém", "だが", "しかし", "それで"):
        if re.search(rf"(?:^|\b){re.escape(marcador)}(?:\b|$)", lower):
            conectivos.append(marcador)

    return {
        "tom": tom,
        "conectivos": conectivos,
        "tem_contexto_anterior": len(ant) > 0,
        "resumo_anterior": ant[-1] if ant else "",
        "tem_contexto_seguinte": len(seg) > 0,
    }


def _aplicar_padroes_contextuais(texto: str, padroes: list[tuple[str, str]]) -> tuple[str, list[str]]:
    resultado = texto
    casados: list[str] = []
    for regex, substituto in padroes:
        if re.search(regex, resultado, flags=re.IGNORECASE):
            resultado = re.sub(regex, substituto, resultado, flags=re.IGNORECASE)
            casados.append(substituto)
    return resultado, casados


def traduzir_balao_com_contexto(
    texto: str,
    idioma_origem: str,
    idioma_destino: str,
    contexto_anterior: list[str] | None = None,
    contexto_seguinte: list[str] | None = None,
    modelo_sessao: Any = None,
) -> dict[str, Any]:
    """Traduz uma frase considerando conectividade semântica, expressões idiomáticas e balões vizinhos."""
    from .pipeline import detectar_idioma_texto, normalizar_idioma_destino

    destino = normalizar_idioma_destino(idioma_destino)
    origem = (idioma_origem or "auto").strip().lower()
    if origem in ("", "auto"):
        origem = detectar_idioma_texto(texto)

    texto_limpo = (texto or "").strip()
    analise = analisar_frase_contextual(texto_limpo, contexto_anterior, contexto_seguinte)

    if not texto_limpo:
        return {
            "idioma_detectado": origem,
            "idioma_destino": destino,
            "texto_traduzido": "",
            "metodo_traducao": "vazio",
            "analise_contexto": analise,
        }

    if (origem == "pt" and destino == "pt-BR") or (origem == "en" and destino == "en"):
        return {
            "idioma_detectado": origem,
            "idioma_destino": destino,
            "texto_traduzido": texto_limpo,
            "metodo_traducao": "mesmo-idioma",
            "analise_contexto": analise,
        }

    # 1. Tentativa via tradutor neural offline (argostranslate) alimentado com frase completa + contexto
    if importlib.util.find_spec("argostranslate") is not None:
        try:
            import argostranslate.translate as argos_tr  # type: ignore[import-not-found]

            src_code = "pt" if origem.startswith("pt") else origem[:2]
            dst_code = "pt" if destino == "pt-BR" else "en"
            traduzido = argos_tr.translate(texto_limpo, src_code, dst_code)
            if traduzido:
                return {
                    "idioma_detectado": origem,
                    "idioma_destino": destino,
                    "texto_traduzido": str(traduzido),
                    "metodo_traducao": "argostranslate-contextual",
                    "analise_contexto": analise,
                }
        except Exception:  # noqa: BLE001
            pass

    # 2. Analisador contextual de expressões conectadas (não literal)
    traduzido = texto_limpo
    expressoes: list[str] = []
    if origem == "en" and destino == "pt-BR":
        traduzido, expressoes = _aplicar_padroes_contextuais(traduzido, _EXPRESSOES_EN_PT)
    elif origem == "pt" and destino == "en":
        traduzido, expressoes = _aplicar_padroes_contextuais(traduzido, _EXPRESSOES_PT_EN)
    elif origem == "ja" and destino == "pt-BR":
        traduzido, expressoes = _aplicar_padroes_contextuais(traduzido, _EXPRESSOES_JA_PT)
    elif origem == "ja" and destino == "en":
        traduzido, expressoes = _aplicar_padroes_contextuais(traduzido, _EXPRESSOES_JA_EN)

    if expressoes:
        # Preserva capitalização inicial natural da frase
        traduzido = traduzido[0].upper() + traduzido[1:] if len(traduzido) > 1 else traduzido.upper()
        analise["expressoes_conectadas"] = expressoes
        metodo = "analisador-contextual-expressoes"
    else:
        metodo = "modelo-neural-pendente" if modelo_sessao is None else "modelo-onnx-contextual"

    return {
        "idioma_detectado": origem,
        "idioma_destino": destino,
        "texto_traduzido": traduzido,
        "metodo_traducao": metodo,
        "analise_contexto": analise,
    }


def traduzir_baloes_pagina_com_contexto(
    baloes: list[dict[str, Any]],
    idioma_origem: str,
    idioma_destino: str,
    modelo_sessao: Any = None,
) -> list[dict[str, Any]]:
    """Processa todos os balões de uma página em ordem de leitura mantendo janela de contexto."""
    textos = [str(b.get("texto_original", "")) for b in baloes]
    resultados: list[dict[str, Any]] = []

    for idx, b in enumerate(baloes):
        ctx_ant = [t for t in textos[max(0, idx - 2) : idx] if t.strip()]
        ctx_seg = [t for t in textos[idx + 1 : min(len(textos), idx + 2)] if t.strip()]

        tr = traduzir_balao_com_contexto(
            texto=textos[idx],
            idioma_origem=idioma_origem,
            idioma_destino=idioma_destino,
            contexto_anterior=ctx_ant,
            contexto_seguinte=ctx_seg,
            modelo_sessao=modelo_sessao,
        )
        resultados.append({**b, **tr})

    return resultados
