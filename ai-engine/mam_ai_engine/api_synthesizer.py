"""Sintetizador automático de APIs declarativas de busca (`api_busca`) a partir de XHR/JSON."""

from __future__ import annotations

import time
from typing import Any
from urllib.parse import quote_plus, urlparse

from .web_reader import _BrowserSession, _collect_json_network_bodies, _wait_spa_settle

_TITLE_KEYS = ("english_title", "title", "name", "manga_title", "series_title", "label")
_ID_KEYS = ("url", "href", "slug", "id", "manga_id")
_COVER_KEYS = ("thumbnail", "cover", "cover_url", "image", "img", "poster")


def _find_candidate_arrays(node: Any, prefix: str = "") -> list[tuple[str, list[dict[str, Any]]]]:
    found: list[tuple[str, list[dict[str, Any]]]] = []
    if isinstance(node, list):
        dicts = [x for x in node if isinstance(x, dict)]
        if dicts:
            found.append((prefix, dicts))
    elif isinstance(node, dict):
        for k, v in node.items():
            sub = f"{prefix}.{k}" if prefix else str(k)
            found.extend(_find_candidate_arrays(v, sub))
    return found


def _pick_key(sample: dict[str, Any], candidates: tuple[str, ...]) -> str | None:
    lower_map = {str(k).lower(): str(k) for k in sample.keys()}
    for c in candidates:
        if c in lower_map:
            return lower_map[c]
    return None


def infer_search_api_from_responses(
    captured: list[tuple[str, Any]],
    query: str,
    base_url: str = "",
) -> dict[str, Any] | None:
    """Infere `ApiBuscaConfig` a partir de respostas `(url, json_body)` capturadas."""
    q_clean = query.strip()
    q_tokens = [t.lower() for t in q_clean.split() if len(t) >= 2]
    q_plus = quote_plus(q_clean)
    q_pct = q_clean.replace(" ", "%20")

    best_score = -1
    best_cfg: dict[str, Any] | None = None

    for req_url, body in captured:
        if not isinstance(req_url, str) or not req_url.startswith("http"):
            continue
        for caminho, arr in _find_candidate_arrays(body):
            sample = arr[0]
            campo_titulo = _pick_key(sample, _TITLE_KEYS)
            campo_id = _pick_key(sample, _ID_KEYS)
            if not campo_titulo or not campo_id:
                continue

            campo_capa = _pick_key(sample, _COVER_KEYS)
            titles = [str(it.get(campo_titulo) or "").lower() for it in arr[:10]]
            query_hits = sum(1 for t in titles if any(tok in t for tok in q_tokens))
            url_has_query = (
                q_clean.lower() in req_url.lower()
                or q_plus.lower() in req_url.lower()
                or q_pct.lower() in req_url.lower()
            )
            score = query_hits * 10 + (15 if url_has_query else 0) + min(len(arr), 10)
            if score <= best_score:
                continue

            endpoint = req_url
            for variant in (q_plus, q_pct, q_clean):
                if variant and variant in endpoint:
                    endpoint = endpoint.replace(variant, "{query}", 1)
                    break
            if "{query}" not in endpoint:
                sep = "&" if "?" in endpoint else "?"
                endpoint = f"{endpoint}{sep}query={{query}}"

            origin = ""
            if base_url:
                p = urlparse(base_url)
                if p.scheme and p.netloc:
                    origin = f"{p.scheme}://{p.netloc}"

            best_score = score
            best_cfg = {
                "endpoint": endpoint,
                "metodo": "GET",
                "formato": "json",
                "caminho_itens": caminho or None,
                "campo_id_ou_url": campo_id,
                "template_url_obra": None,
                "campo_titulo": campo_titulo,
                "campo_capa": campo_capa,
                "template_url_capa": None,
                "headers": {"Referer": f"{origin}/"} if origin else None,
            }

    return best_cfg


def synthesize_search_api_cdp(base_url: str, query: str) -> dict[str, Any]:
    """Abre o site via CDP, dispara uma busca na barra de pesquisa e sintetiza `api_busca`."""
    with _BrowserSession() as cdp:
        cdp.call("Network.enable")
        cdp.call("Page.navigate", {"url": base_url})
        _wait_spa_settle(
            cdp,
            "document.readyState === 'complete'",
            max_wait_s=7.0,
        )
        q_js = query.replace("\\", "\\\\").replace("'", "\\'")
        cdp.eval_js(
            f"""
            const inp = document.querySelector(
              'input[type="search"], input[name*="keyword"], input[name*="search"], input[name="q"], input[name="s"], input[placeholder*="Search" i], input[placeholder*="Buscar" i]'
            );
            if (inp) {{
              inp.focus();
              inp.value = '{q_js}';
              inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
              inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
              const form = inp.closest('form');
              if (form) {{
                const btn = form.querySelector('button[type="submit"], button');
                if (btn) btn.click(); else form.submit();
              }} else {{
                inp.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Enter', keyCode: 13, bubbles: true }}));
              }}
            }}
            """
        )
        time.sleep(2.5)
        captured = _collect_json_network_bodies(cdp)
        api_cfg = infer_search_api_from_responses(captured, query, base_url=base_url)
        return {"api_busca": api_cfg}
