"""Leitor Web Inteligente baseado em Chrome DevTools Protocol (CDP) em stdlib Python pura.

Suporta sites SPA/dinâmicos sem dependências externas:
- Seleção automática de idioma preferido (PT-BR padrão, ENG, JPN, fallback).
- Paginação integral sem limite artificial (com detecção de ciclo).
- Extração completa de páginas de capítulos (scroll lazy-load + avanço single-page).
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import shutil
import socket
import struct
import subprocess
import tempfile
import time
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

_NON_IMAGE_EXTS = (".js", ".mjs", ".css", ".html", ".htm", ".svg", ".ico", ".json", ".woff", ".woff2", ".ttf")
_UI_NOISE_TOKENS = ("favicon", "logo", "avatar", "banner", "badge", "flag", "/assets/mangafire/logo")


def find_browser_executable() -> str | None:
    """Localiza Microsoft Edge ou Google Chrome instalado no sistema."""
    candidatos = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        shutil.which("msedge"),
        shutil.which("chrome"),
        shutil.which("google-chrome"),
        shutil.which("chromium"),
    ]
    for cand in candidatos:
        if cand and Path(cand).exists():
            return str(cand)
    return None


def is_valid_reader_image_url(url: str) -> bool:
    """Filtra URLs de imagens reais de leitura, descartando scripts, estilos, logos e ícones."""
    if not url:
        return False
    lower = url.strip().lower()
    if not (lower.startswith("http://") or lower.startswith("https://")):
        return False
    sem_query = lower.split("?", 1)[0].split("#", 1)[0]
    if any(sem_query.endswith(ext) for ext in _NON_IMAGE_EXTS):
        return False
    if any(tok in sem_query for tok in _UI_NOISE_TOKENS):
        return False
    return (
        sem_query.endswith((".jpg", ".jpeg", ".png", ".webp", ".avif"))
        or "/page" in lower
    )


def _encode_ws_frame(payload: bytes, opcode: int = 0x1) -> bytes:
    """Codifica um frame WebSocket mascarado (cliente -> servidor, RFC 6455)."""
    mask_key = os.urandom(4)
    length = len(payload)
    header = bytearray([0x80 | (opcode & 0x0F)])
    if length < 126:
        header.append(0x80 | length)
    elif length < 65536:
        header.append(0x80 | 126)
        header.extend(struct.pack("!H", length))
    else:
        header.append(0x80 | 127)
        header.extend(struct.pack("!Q", length))
    header.extend(mask_key)
    masked = bytes(b ^ mask_key[i % 4] for i, b in enumerate(payload))
    return bytes(header) + masked


def _recv_exact(sock: socket.socket, n: int) -> bytes:
    buf = bytearray()
    while len(buf) < n:
        chunk = sock.recv(n - len(buf))
        if not chunk:
            raise ConnectionError("Conexão WebSocket CDP encerrada inesperadamente")
        buf.extend(chunk)
    return bytes(buf)


def _decode_ws_frame(sock: socket.socket) -> tuple[int, bytes]:
    b1, b2 = _recv_exact(sock, 2)
    opcode = b1 & 0x0F
    masked = bool(b2 & 0x80)
    length = b2 & 0x7F
    if length == 126:
        (length,) = struct.unpack("!H", _recv_exact(sock, 2))
    elif length == 127:
        (length,) = struct.unpack("!Q", _recv_exact(sock, 8))
    mask_key = _recv_exact(sock, 4) if masked else b""
    payload = _recv_exact(sock, length)
    if masked:
        payload = bytes(b ^ mask_key[i % 4] for i, b in enumerate(payload))
    return opcode, payload


class _CdpClient:
    """Cliente WebSocket síncrono minimalista para Chrome DevTools Protocol."""

    def __init__(self, ws_url: str, timeout: float = 20.0) -> None:
        parsed = urllib.parse.urlparse(ws_url)
        host = parsed.hostname or "127.0.0.1"
        port = parsed.port or 80
        path = parsed.path or "/"
        if parsed.query:
            path = f"{path}?{parsed.query}"

        self._sock = socket.create_connection((host, port), timeout=timeout)
        self._sock.settimeout(timeout)
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        handshake = (
            f"GET {path} HTTP/1.1\r\n"
            f"Host: {host}:{port}\r\n"
            f"Upgrade: websocket\r\n"
            f"Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            f"Sec-WebSocket-Version: 13\r\n\r\n"
        )
        self._sock.sendall(handshake.encode("ascii"))

        resp = b""
        while b"\r\n\r\n" not in resp:
            chunk = self._sock.recv(4096)
            if not chunk:
                raise ConnectionError("Falha no handshake WebSocket CDP")
            resp += chunk
        if b"101" not in resp.split(b"\r\n", 1)[0]:
            raise ConnectionError(f"Handshake CDP rejeitado: {resp[:120]!r}")
        self._msg_id = 0

    def call(self, method: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        self._msg_id += 1
        target_id = self._msg_id
        msg = json.dumps({"id": target_id, "method": method, "params": params or {}})
        self._sock.sendall(_encode_ws_frame(msg.encode("utf-8"), opcode=0x1))

        while True:
            opcode, payload = _decode_ws_frame(self._sock)
            if opcode == 0x8:
                raise ConnectionError("Sessão CDP fechada pelo navegador")
            if opcode == 0x9:
                self._sock.sendall(_encode_ws_frame(payload, opcode=0xA))
                continue
            if opcode not in (0x1, 0x0):
                continue
            data = json.loads(payload.decode("utf-8", errors="replace"))
            if data.get("id") == target_id:
                if "error" in data:
                    raise RuntimeError(f"Erro CDP em {method}: {data['error']}")
                return data.get("result") or {}

    def eval_js(self, expression: str) -> Any:
        res = self.call(
            "Runtime.evaluate",
            {
                "expression": f"(() => {{ {expression} }})()",
                "returnByValue": True,
                "awaitPromise": True,
            },
        )
        return (res.get("result") or {}).get("value")

    def close(self) -> None:
        try:
            self._sock.close()
        except OSError:
            pass


class _BrowserSession:
    """Context manager que inicia Edge/Chrome headless com porta CDP dinâmica."""

    def __init__(self) -> None:
        self._tmp_dir: str | None = None
        self._proc: subprocess.Popen[bytes] | None = None
        self.cdp: _CdpClient | None = None

    def __enter__(self) -> _CdpClient:
        exe = find_browser_executable()
        if not exe:
            raise RuntimeError("Nenhum navegador Edge/Chrome encontrado para leitura web inteligente")
        self._tmp_dir = tempfile.mkdtemp(prefix="mam_cdp_")
        args = [
            exe,
            "--headless=new",
            "--disable-gpu",
            "--no-sandbox",
            "--disable-extensions",
            "--remote-debugging-port=0",
            f"--user-data-dir={self._tmp_dir}",
            "about:blank",
        ]
        creationflags = 0x08000000 if os.name == "nt" else 0
        self._proc = subprocess.Popen(
            args,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            creationflags=creationflags,
        )

        port_file = Path(self._tmp_dir) / "DevToolsActivePort"
        deadline = time.monotonic() + 12.0
        while time.monotonic() < deadline:
            if port_file.exists():
                lines = port_file.read_text(encoding="utf-8", errors="ignore").splitlines()
                if lines and lines[0].strip().isdigit():
                    port = int(lines[0].strip())
                    targets = json.loads(
                        urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=5)
                        .read()
                        .decode("utf-8")
                    )
                    page_ws = next(
                        (t["webSocketDebuggerUrl"] for t in targets if t.get("type") == "page" and t.get("webSocketDebuggerUrl")),
                        None,
                    )
                    if page_ws:
                        self.cdp = _CdpClient(page_ws, timeout=20.0)
                        self.cdp.call("Page.enable")
                        self.cdp.call("Runtime.enable")
                        return self.cdp
            time.sleep(0.15)
        raise RuntimeError("Navegador headless não expôs DevToolsActivePort a tempo")

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        if self.cdp is not None:
            self.cdp.close()
        if self._proc is not None:
            try:
                self._proc.terminate()
                self._proc.wait(timeout=3)
            except Exception:
                try:
                    self._proc.kill()
                except Exception:
                    pass
        if self._tmp_dir:
            shutil.rmtree(self._tmp_dir, ignore_errors=True)


def _wait_spa_settle(cdp: _CdpClient, check_js: str, max_wait_s: float = 8.0) -> None:
    deadline = time.monotonic() + max_wait_s
    while time.monotonic() < deadline:
        ready = cdp.eval_js(
            f"const r = document.readyState === 'complete'; const ok = Boolean({check_js}); return r && ok;"
        )
        if ready:
            time.sleep(0.35)
            return
        time.sleep(0.25)


def _lang_aliases(preferred_lang: str) -> list[str]:
    norm = (preferred_lang or "pt-br").strip().lower()
    if norm in ("pt", "pt-br", "pt_br", "portuguese"):
        return ["portuguese (br)", "pt-br", "português", "portuguese", "br", "pt"]
    if norm in ("en", "eng", "english"):
        return ["english", "en", "gb"]
    if norm in ("ja", "jp", "jpn", "japanese"):
        return ["japanese", "ja", "jp", "日本語"]
    return []


def _select_preferred_language(cdp: _CdpClient, preferred_lang: str) -> bool:
    aliases = _lang_aliases(preferred_lang)
    if not aliases:
        return False
    aliases_json = json.dumps(aliases)
    # 1. Tenta <select> nativo ou abre dropdown customizado
    state = cdp.eval_js(
        f"""
        const aliases = {aliases_json};
        const matches = (txt) => {{
          const t = (txt || '').toLowerCase().trim();
          return aliases.some(a => t === a || t.includes(a));
        }};
        for (const sel of document.querySelectorAll('select')) {{
          for (const opt of sel.options) {{
            if (matches(opt.textContent) || matches(opt.value)) {{
              sel.value = opt.value;
              sel.dispatchEvent(new Event('change', {{ bubbles: true }}));
              return 'selected';
            }}
          }}
        }}
        const triggers = Array.from(document.querySelectorAll('button.select, [aria-haspopup="menu"], [class*="lang"] button'));
        const langBtn = triggers.find(b => /lang|idioma/i.test(b.textContent || '') || /english|portuguese|japanese/i.test(b.textContent || ''));
        if (!langBtn) return 'none';
        if (matches(langBtn.textContent)) return 'selected';
        langBtn.click();
        return 'opened';
        """
    )
    if state == "selected":
        time.sleep(0.6)
        return True
    if state != "opened":
        return False

    # 2. Aguarda o menu portal renderizar e clica na opção correspondente
    time.sleep(0.35)
    clicked = cdp.eval_js(
        f"""
        const aliases = {aliases_json};
        const matches = (txt) => {{
          const t = (txt || '').toLowerCase().trim();
          return aliases.some(a => t === a || t.includes(a));
        }};
        const items = Array.from(document.querySelectorAll(
          '[role="menuitem"], [role="option"], [role="menu"] button, [role="menu"] a, .select__menu button, .select__menu li, .dropdown-menu a, .dropdown-menu button, .popover button, .menu button'
        ));
        const target = items.find(el => matches(el.textContent) || matches(el.getAttribute('data-value') || '') || matches(el.getAttribute('title') || ''));
        if (target) {{
          target.click();
          return true;
        }}
        return false;
        """
    )
    if clicked:
        time.sleep(0.9)
    return bool(clicked)


def _collect_paginated_chapters(
    cdp: _CdpClient,
    url: str,
    max_pages: int | None = None,
    stop_when_all_numbers_in: set[int] | None = None,
) -> tuple[str, str | None, int, list[dict[str, Any]]]:
    vistos: set[str] = set()
    capitulos: list[dict[str, Any]] = []
    titulo_obra = ""
    capa_url = None
    idle_steps = 0
    page_idx = 0

    while True:
        page_idx += 1
        snap = cdp.eval_js(
            """
            const h1 = document.querySelector('h1');
            const ogImg = document.querySelector('meta[property="og:image"]');
            const coverImg = document.querySelector(
              '.title-detail__cover img, .title-detail__top img, .manga-poster img, img[class*="cover"], img[class*="poster"]'
            );
            const anchors = Array.from(document.querySelectorAll('a[href]')).map(a => {
              const row = a.closest('.title-detail__row, li, tr, .chapter-item');
              const flag = row ? row.querySelector('.title-detail__row-flag, [class*="flag"]') : null;
              return {
                href: a.href,
                text: (a.textContent || '').replace(/\\s+/g, ' ').trim(),
                dataNumber: a.getAttribute('data-number') || a.getAttribute('data-chapter') || '',
                rowLang: flag ? (flag.getAttribute('title') || flag.textContent || '') : ''
              };
            });
            return {
              title: h1 ? h1.textContent.trim() : document.title,
              cover: (ogImg && ogImg.content) || (coverImg && (coverImg.currentSrc || coverImg.src)) || null,
              anchors
            };
            """
        ) or {}

        if not titulo_obra and snap.get("title"):
            titulo_obra = str(snap["title"]).strip()
        if not capa_url and snap.get("cover"):
            capa_url = str(snap["cover"]).strip()

        novos_nesta_pagina = 0
        novos_fora_do_conjunto = 0
        for item in snap.get("anchors") or []:
            href = str(item.get("href") or "").strip()
            if not href:
                continue
            lower_href = href.lower()
            if not any(p in lower_href for p in ("/chapter/", "/capitulo-", "/chapter-", "/cap-", "/ch-")):
                continue
            norm = href.rstrip("/")
            if norm in vistos:
                continue
            vistos.add(norm)
            novos_nesta_pagina += 1
            if stop_when_all_numbers_in is not None:
                m = re.search(
                    r"(?:cap[íi]tulo|chapter|cap\.?|ch\.?)\s*[-:]?\s*(\d+)",
                    f"{item.get('text', '')} {href}".lower(),
                )
                num = int(m.group(1)) if m else None
                if num is None or num not in stop_when_all_numbers_in:
                    novos_fora_do_conjunto += 1
                    capitulos.append(item)
            else:
                capitulos.append(item)

        if stop_when_all_numbers_in is not None and novos_nesta_pagina > 0 and novos_fora_do_conjunto == 0:
            break

        if novos_nesta_pagina == 0:
            idle_steps += 1
        else:
            idle_steps = 0
        if idle_steps >= 2:
            break
        if max_pages is not None and page_idx >= max_pages:
            break

        avancou = cdp.eval_js(
            """
            const active = document.querySelector('.npager__num.is-active, .pagination .active, [aria-current="page"]');
            if (active) {
              let next = active.nextElementSibling;
              while (next && (next.disabled || next.classList.contains('disabled'))) {
                next = next.nextElementSibling;
              }
              if (next) {
                const txt = (next.textContent || '').trim();
                const aria = (next.getAttribute('aria-label') || '').toLowerCase();
                if (/^\\d+$|^>$|^›$|next|próxima/i.test(txt) || /next|próxim/i.test(aria) || (txt === '' && next.querySelector('svg'))) {
                  next.click();
                  return true;
                }
              }
            }
            const nextBtn = document.querySelector('a[rel="next"], button[aria-label*="Next" i], button.load-more, .load-more button');
            if (nextBtn && !nextBtn.disabled) {
              nextBtn.click();
              return true;
            }
            const scroller = document.querySelector('.scroll-area');
            if (scroller && scroller.scrollTop + scroller.clientHeight + 20 < scroller.scrollHeight) {
              scroller.scrollTop = scroller.scrollHeight;
              return true;
            }
            return false;
            """
        )
        if not avancou:
            break
        time.sleep(0.9)

    return titulo_obra, capa_url, page_idx, capitulos


def _extract_num_set(items: list[dict[str, Any]]) -> set[int]:
    nums: set[int] = set()
    for it in items:
        m = re.search(
            r"(?:cap[íi]tulo|chapter|cap\.?|ch\.?)\s*[-:]?\s*(\d+)",
            f"{it.get('text', '')} {it.get('href', '')}".lower(),
        )
        if m:
            nums.add(int(m.group(1)))
    return nums


def discover_work_chapters(
    url: str,
    preferred_lang: str = "pt-br",
    max_pages: int | None = None,
) -> dict[str, Any]:
    """Navega na página central da obra via CDP, seleciona idioma e percorre toda a paginação.

    Se `preferred_lang == 'pt-br'`, complementa automaticamente com capítulos pendentes de `en`
    cujos números ainda não existam em PT-BR (ex.: PT até 100 + ENG 101..150 em sequência).
    """
    with _BrowserSession() as cdp:
        cdp.call("Page.navigate", {"url": url})
        _wait_spa_settle(
            cdp,
            "document.querySelectorAll('a[href*=\"chapter\"], a[href*=\"capitulo\"], .title-detail__row-link').length > 0",
            max_wait_s=9.0,
        )
        selecionou_pref = _select_preferred_language(cdp, preferred_lang)
        titulo_obra, capa_url, paginas_varridas, capitulos = _collect_paginated_chapters(
            cdp, url, max_pages=max_pages
        )

        # Se PT-BR foi selecionado e existe inglês com capítulos além dos disponíveis em PT-BR,
        # adiciona os pendentes em sequência após os de português.
        if (preferred_lang or "pt-br").strip().lower() in ("pt", "pt-br", "pt_br", "portuguese") and selecionou_pref:
            numeros_pt = _extract_num_set(capitulos)
            if _select_preferred_language(cdp, "en"):
                _, capa_en, pags_en, extras_en = _collect_paginated_chapters(
                    cdp,
                    url,
                    max_pages=max_pages,
                    stop_when_all_numbers_in=numeros_pt if numeros_pt else None,
                )
                paginas_varridas += pags_en
                if not capa_url and capa_en:
                    capa_url = capa_en
                vistos_urls = {str(c.get("href", "")).rstrip("/") for c in capitulos}
                for ex in extras_en:
                    h = str(ex.get("href", "")).rstrip("/")
                    if h and h not in vistos_urls:
                        vistos_urls.add(h)
                        capitulos.append(ex)

        return {
            "titulo": titulo_obra or "Obra Importada",
            "capa_url": capa_url,
            "url_obra": url,
            "paginas_varridas": paginas_varridas,
            "capitulos": capitulos,
        }


def extract_chapter_pages(url: str) -> dict[str, Any]:
    """Extrai todas as URLs de imagens reais de um capítulo via CDP (scroll + avanço single-page)."""
    with _BrowserSession() as cdp:
        cdp.call("Page.navigate", {"url": url})
        _wait_spa_settle(
            cdp,
            "document.querySelectorAll('img').length > 0",
            max_wait_s=9.0,
        )

        # Tenta alternar para Long Strip caso o leitor tenha botão visível
        cdp.eval_js(
            """
            const btn = Array.from(document.querySelectorAll('button, a')).find(el =>
              /long strip|all pages|webtoon|todas as páginas/i.test(el.textContent || '')
            );
            if (btn) btn.click();
            """
        )
        time.sleep(0.4)

        vistos: set[str] = set()
        urls_ordenadas: list[str] = []
        idle_steps = 0

        for _ in range(120):
            snap = cdp.eval_js(
                """
                const imgs = Array.from(document.querySelectorAll('img')).map(img => ({
                  src: img.getAttribute('data-src') || img.currentSrc || img.src || '',
                  w: img.naturalWidth || 0,
                  inReader: Boolean(img.closest('[class*="reader"], [class*="chapter"], [class*="page"], main'))
                }));
                const text = document.body ? document.body.innerText : '';
                const m = text.match(/\\b(\\d+)\\s*\\/\\s*(\\d+)\\b/);
                return {
                  imgs,
                  curPage: m ? parseInt(m[1], 10) : null,
                  totalPages: m ? parseInt(m[2], 10) : null
                };
                """
            ) or {}

            novas = 0
            for im in snap.get("imgs") or []:
                src = str(im.get("src") or "").strip()
                if not is_valid_reader_image_url(src):
                    continue
                if im.get("w", 0) and int(im["w"]) < 200 and not im.get("inReader"):
                    continue
                if src not in vistos:
                    vistos.add(src)
                    urls_ordenadas.append(src)
                    novas += 1

            total_pages = snap.get("totalPages")
            if total_pages and len(urls_ordenadas) >= int(total_pages):
                break

            if novas == 0:
                idle_steps += 1
            else:
                idle_steps = 0
            if idle_steps >= 3:
                break

            # Faz scroll e tenta avançar página (modo single-page)
            cdp.eval_js(
                """
                window.scrollBy(0, window.innerHeight * 0.9);
                const nextBtn = document.querySelector('[aria-label*="Next" i], [title*="Next" i], .reader-next, .next-page');
                if (nextBtn) {
                  nextBtn.click();
                } else {
                  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', code: 'ArrowRight', bubbles: true }));
                }
                """
            )
            time.sleep(0.45)

        return {"url": url, "paginas": urls_ordenadas}


def search_works_cdp(url_busca: str, conector_id: str = "mangafire") -> list[dict[str, Any]]:
    """Pesquisa obras por título em um conector web (ex.: MangaFire) via CDP."""
    with _BrowserSession() as cdp:
        cdp.call("Page.navigate", {"url": url_busca})
        _wait_spa_settle(
            cdp,
            "document.querySelectorAll('a[href*=\"/title/\"], a[href*=\"/manga/\"]').length > 0",
            max_wait_s=8.0,
        )
        items = cdp.eval_js(
            """
            const seen = new Set();
            const out = [];
            for (const a of document.querySelectorAll('a[href*="/title/"], a[href*="/manga/"]')) {
              const href = (a.href || '').split('?')[0].split('#')[0];
              if (!href || href.includes('/chapter') || seen.has(href)) continue;
              const card = a.closest('article, .card, .unit, .manga-card, li, div') || a;
              const img = card.querySelector('img');
              const titleEl = card.querySelector('h3, h2, .title, .name, span') || a;
              const titulo = (titleEl.textContent || a.getAttribute('title') || '').replace(/\\s+/g, ' ').trim();
              if (!titulo || titulo.length < 2) continue;
              seen.add(href);
              out.push({
                obra_id_fonte: href,
                titulo,
                capa_url: img ? (img.currentSrc || img.src || null) : null
              });
              if (out.length >= 20) break;
            }
            return out;
            """
        ) or []
        return [
            {
                "fonte_id": conector_id,
                "obra_id_fonte": it["obra_id_fonte"],
                "titulo": it["titulo"],
                "autor": None,
                "descricao": None,
                "capa_url": it.get("capa_url"),
            }
            for it in items
        ]
