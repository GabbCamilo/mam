"""Testes unitários para o servidor JSON-RPC 2.0 do Manga AI Engine."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from mam_ai_engine.protocol import parse_request
from mam_ai_engine.server import handle_request


class AiEngineServerTests(unittest.TestCase):
    def test_ping_retorna_ok_e_cpu(self) -> None:
        req, err = parse_request('{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}')
        self.assertIsNone(err)
        assert req is not None
        resp, should_exit = handle_request(req)
        self.assertFalse(should_exit)
        self.assertEqual(resp["result"]["status"], "ok")
        self.assertTrue(resp["result"]["provedores"]["cpu"])

    def test_ocr_e_translate_marcam_nao_revisado(self) -> None:
        req_ocr, _ = parse_request(
            '{"jsonrpc":"2.0","id":2,"method":"ocr","params":{"caminho_imagem":"p001.jpg","idioma_origem":"ja"}}'
        )
        assert req_ocr is not None
        resp_ocr, _ = handle_request(req_ocr)
        self.assertEqual(resp_ocr["result"]["estado_revisao"], "nao_revisado")

        req_tr, _ = parse_request(
            '{"jsonrpc":"2.0","id":3,"method":"translate","params":{"textos":["こんにちは"],"idioma_origem":"ja","idioma_destino":"pt-BR"}}'
        )
        assert req_tr is not None
        resp_tr, _ = handle_request(req_tr)
        self.assertEqual(resp_tr["result"]["estado_revisao"], "nao_revisado")
        self.assertEqual(len(resp_tr["result"]["traducoes"]), 1)

    def test_models_list_sem_download_automatico(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            modelo = Path(tmp) / "ocr_ja.onnx"
            modelo.write_bytes(b"onnx-dummy")
            req, _ = parse_request(
                f'{{"jsonrpc":"2.0","id":4,"method":"models_list","params":{{"diretorios":["{tmp.replace(chr(92), "/")}"]}}}}'
            )
            assert req is not None
            resp, _ = handle_request(req)
            self.assertEqual(len(resp["result"]), 1)
            self.assertEqual(resp["result"][0]["formato"], "onnx")

    def test_shutdown_encerra_loop(self) -> None:
        req, _ = parse_request('{"jsonrpc":"2.0","id":5,"method":"shutdown"}')
        assert req is not None
        resp, should_exit = handle_request(req)
        self.assertTrue(should_exit)
        self.assertTrue(resp["result"]["encerrado"])

    def test_process_page_le_dimensoes_reais_e_status_onnx(self) -> None:
        from PIL import Image

        with tempfile.TemporaryDirectory() as tmp:
            img_path = Path(tmp) / "pagina.png"
            Image.new("RGB", (120, 240), color=(255, 255, 255)).save(img_path)

            req, _ = parse_request(
                f'{{"jsonrpc":"2.0","id":6,"method":"process_page","params":{{"caminho_imagem":"{str(img_path).replace(chr(92), "/")}","idioma_origem":"ja","idioma_destino":"pt-BR"}}}}'
            )
            assert req is not None
            resp, _ = handle_request(req)
            res = resp["result"]
            self.assertEqual(res["estado_revisao"], "nao_revisado")
            self.assertTrue(res["dimensoes_imagem"]["disponivel"])
            self.assertEqual(res["dimensoes_imagem"]["largura"], 120)
            self.assertEqual(res["dimensoes_imagem"]["altura"], 240)
            self.assertIn("onnx_status", res)

    def test_translate_contextual_preserva_expressoes_e_janela_baloes(self) -> None:
        req, _ = parse_request(
            '{"jsonrpc":"2.0","id":7,"method":"translate","params":{"textos":["Hold on...","What\'s going on?"],"idioma_origem":"auto","idioma_destino":"pt-BR"}}'
        )
        assert req is not None
        resp, _ = handle_request(req)
        traducoes = resp["result"]["traducoes"]
        self.assertEqual(len(traducoes), 2)
        self.assertEqual(traducoes[0]["idioma_detectado"], "en")
        self.assertIn("Espera aí", traducoes[0]["traduzido"])
        self.assertEqual(traducoes[0]["analise_contexto"]["tom"], "hesitante_ou_suspense")
        self.assertTrue(traducoes[1]["analise_contexto"]["tem_contexto_anterior"])
        self.assertIn("O que está acontecendo", traducoes[1]["traduzido"])


if __name__ == "__main__":
    unittest.main()
