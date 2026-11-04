"""Filename WASI cases must receive distinct input and output argument slots."""

import ast
import subprocess
from pathlib import Path
import unittest


class WasmArguments(unittest.TestCase):
    def test_custom_case_arguments_are_forwarded(self):
        tree = ast.parse((Path(__file__).parent / "portable_data/run.py").read_text())
        function = next(
            n
            for n in tree.body
            if isinstance(n, ast.FunctionDef) and n.name == "wasm_test"
        )
        commands = []
        scope = dict(
            output=Path("/fixtures"),
            compiler="jaic",
            failures=[],
            run=lambda *args: commands.append(args),
            subprocess=subprocess,
        )
        exec(
            compile(ast.Module(body=[function], type_ignores=[]), "run.py", "exec"),
            scope,
        )
        scope["wasm_test"](
            "filename.jai", "filename", Path("/inputs"), Path("/outputs")
        )
        self.assertEqual(commands[-1][-2:], (Path("/inputs"), Path("/outputs")))
        commands.clear()
        scope["wasm_test"]("wire.jai", "wire")
        self.assertEqual(commands[-1][-1], Path("/fixtures"))


if __name__ == "__main__":
    unittest.main()
