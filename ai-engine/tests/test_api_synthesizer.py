import unittest
from mam_ai_engine.api_synthesizer import infer_search_api_from_responses


class TestApiSynthesizer(unittest.TestCase):
    def test_infere_api_busca_de_payload_json_com_query(self) -> None:
        captured = [
            (
                "https://exemplo.net/api/v2/search?query=medaka+box",
                {
                    "result": [
                        {
                            "id": 623916,
                            "english_title": "Medaka Box Extra",
                            "thumbnail": "galleries/1/thumb.webp",
                        }
                    ]
                },
            )
        ]
        cfg = infer_search_api_from_responses(
            captured, "medaka box", base_url="https://exemplo.net/"
        )
        self.assertIsNotNone(cfg)
        assert cfg is not None
        self.assertEqual(cfg["endpoint"], "https://exemplo.net/api/v2/search?query={query}")
        self.assertEqual(cfg["caminho_itens"], "result")
        self.assertEqual(cfg["campo_titulo"], "english_title")
        self.assertEqual(cfg["campo_id_ou_url"], "id")
        self.assertEqual(cfg["campo_capa"], "thumbnail")


if __name__ == "__main__":
    unittest.main()
