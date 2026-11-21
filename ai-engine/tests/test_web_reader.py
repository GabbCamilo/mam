"""Testes unitários do Leitor Web Inteligente (CDP) e Guia IA Estrutural."""

from __future__ import annotations

import unittest
from unittest.mock import patch

from mam_ai_engine.pipeline import (
    analyze_reader_dom,
    smart_discover_chapters,
    smart_extract_chapter_pages,
)
from mam_ai_engine.web_reader import is_valid_reader_image_url


class TestWebReader(unittest.TestCase):
    def test_is_valid_reader_image_url_rejeita_scripts_e_logos(self) -> None:
        self.assertFalse(is_valid_reader_image_url("https://s.mfcdn.nl/build/mf/assets/main-B7h3oPSn.js"))
        self.assertFalse(is_valid_reader_image_url("https://s.mfcdn.nl/build/mf/assets/main.css"))
        self.assertFalse(is_valid_reader_image_url("https://mangafire.to/assets/mangafire/logo.png"))
        self.assertTrue(is_valid_reader_image_url("https://pw6.mfcdn2.xyz/mf/abc123/h/p.jpg"))
        self.assertTrue(is_valid_reader_image_url("https://cdn.exemplo.com/cap1/002.webp?v=1"))

    def test_smart_discover_chapters_extrai_idioma_e_numero(self) -> None:
        fake_cdp_res = {
            "titulo": "Medaka Box",
            "capa_url": "https://cdn.exemplo.com/cover.jpg",
            "url_obra": "https://mangafire.to/title/jw2-medaka-boxx",
            "paginas_varridas": 3,
            "capitulos": [
                {
                    "href": "https://mangafire.to/title/jw2-medaka-boxx/chapter/4570384",
                    "text": "Ch. 192.5 Good Loser Kumagawa 2",
                    "dataNumber": "",
                    "rowLang": "pt-br",
                },
                {
                    "href": "https://mangafire.to/title/jw2-medaka-boxx/chapter/4570300",
                    "text": "Ch. 173",
                    "dataNumber": "",
                    "rowLang": "pt-br",
                },
            ],
        }
        with patch("mam_ai_engine.web_reader.discover_work_chapters", return_value=fake_cdp_res):
            res = smart_discover_chapters({
                "url_obra": "https://mangafire.to/title/jw2-medaka-boxx",
                "idioma_preferido": "pt-br",
            })
        self.assertEqual(res["titulo"], "Medaka Box")
        self.assertEqual(res["paginas_varridas"], 3)
        self.assertEqual(len(res["capitulos"]), 2)
        self.assertEqual(res["capitulos"][0]["numero"], 173)
        self.assertEqual(res["capitulos"][0]["idioma"], "PT")
        self.assertEqual(res["capitulos"][1]["numero"], 192)

    def test_smart_discover_chapters_completa_sequencia_pt_com_eng(self) -> None:
        fake_cdp_seq = {
            "titulo": "Obra Mista",
            "capa_url": None,
            "url_obra": "https://mangafire.to/title/obra-mista",
            "paginas_varridas": 2,
            "capitulos": [
                {"href": "https://mangafire.to/title/obra-mista/chapter/100", "text": "Ch. 100", "dataNumber": "", "rowLang": "pt-br"},
                {"href": "https://mangafire.to/title/obra-mista/chapter/150", "text": "Ch. 150", "dataNumber": "", "rowLang": "en"},
            ],
        }
        with patch("mam_ai_engine.web_reader.discover_work_chapters", return_value=fake_cdp_seq):
            res = smart_discover_chapters({"url_obra": "https://mangafire.to/title/obra-mista", "idioma_preferido": "pt-br"})
        self.assertEqual([c["numero"] for c in res["capitulos"]], [100, 150])
        self.assertEqual([c["idioma"] for c in res["capitulos"]], ["PT", "ENG"])

    def test_smart_extract_chapter_pages_e_analyze_reader_dom(self) -> None:
        with patch(
            "mam_ai_engine.web_reader.extract_chapter_pages",
            return_value={"url": "https://exemplo/cap/1", "paginas": ["https://cdn/1.jpg", "https://cdn/2.jpg"]},
        ):
            res = smart_extract_chapter_pages({"url_capitulo": "https://exemplo/cap/1"})
        self.assertEqual(len(res["paginas"]), 2)

        dom_res = analyze_reader_dom({
            "url": "https://mangafire.to/title/jw2-medaka-boxx",
            "candidatos": [{"href": "https://mangafire.to/title/jw2-medaka-boxx/chapter/1", "text": "Ch. 1"}],
        })
        self.assertEqual(dom_res["perfil"]["dominio"], "mangafire.to")
        self.assertEqual(dom_res["perfil"]["padrao_capitulo"], "/chapter/")


if __name__ == "__main__":
    unittest.main()
