import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("webgpu_gen", Path(__file__).with_name("webgpu_gen.py"))
gen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen)

YAML = """\
name: webgpu
enum_prefix: 0x0000
doc: |
  First line.

  After a blank line.
constants:
  - name: whole_size
    value: uint64_max
typedefs: []
enums:
  - name: texture_dimension
    doc: Plain scalar
      continued here.
    entries:
      - null
      - name: 1D
      - name: "2D"
"""


class WebGPUGenTests(unittest.TestCase):
    def test_yaml_subset(self):
        doc = gen.parse_yaml(YAML)
        self.assertEqual(doc["doc"], "First line.\n\nAfter a blank line.\n")
        self.assertEqual(doc["typedefs"], [])
        enum = doc["enums"][0]
        self.assertEqual(enum["doc"], "Plain scalar continued here.")
        self.assertEqual(enum["entries"], [None, {"name": "1D"}, {"name": "2D"}])

    def test_names(self):
        self.assertEqual(gen.pascal("request_adapter_WebXR_options"), "RequestAdapterWebXROptions")
        self.assertEqual(gen.pascal("unorm10_10_10_2"), "Unorm10_10_10_2")
        self.assertEqual(gen.camel("vendor_ID"), "vendorID")
        self.assertEqual(gen.singular("entries"), "entry")
        self.assertEqual(gen.jai_ident("1D"), "_1D")

    def test_webidl_enum_strings(self):
        cases = {
            "RGBA8_unorm_srgb": "rgba8unorm-srgb",
            "depth24_plus_stencil8": "depth24plus-stencil8",
            "BC1_RGBA_unorm": "bc1-rgba-unorm",
            "ASTC_10x10_unorm_srgb": "astc-10x10-unorm-srgb",
            "etc2_RGB8A1_unorm": "etc2-rgb8a1unorm",
            "RGB9E5_ufloat": "rgb9e5ufloat",
        }
        for entry, js in cases.items():
            self.assertEqual(gen.js_enum_string("texture_format", entry), js)
        self.assertEqual(gen.js_enum_string("vertex_format", "unorm10_10_10_2"), "unorm10-10-10-2")
        self.assertEqual(gen.js_enum_string("texture_view_dimension", "2D_array"), "2d-array")


if __name__ == "__main__":
    unittest.main()
