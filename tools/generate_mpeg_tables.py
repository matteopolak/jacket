#!/usr/bin/env python3
"""Emit reviewed immutable MPEG tables, grouped by independent codec responsibility."""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/specs/stdlib/modules"
groups = [
    ("Video scan and reconstruction", ["PICTURE_RATE", "ZIG_ZAG", "INTRA_QUANT_MATRIX",
                                      "NON_INTRA_QUANT_MATRIX", "PREMULTIPLIER_MATRIX"]),
    ("Video entropy coding", ["DCT_SIZE_LUMINANCE", "DCT_SIZE_CHROMINANCE", "DCT_COEFF",
                              "MOTION", "CODE_BLOCK_PATTERN", "MACROBLOCK_ADDRESS_INCREMENT",
                              "MACROBLOCK_TYPE_INTRA", "MACROBLOCK_TYPE_PREDICTIVE", "MACROBLOCK_TYPE_B"]),
    ("Audio framing", ["FRAME_SYNC", "MPEG_1", "MPEG_2", "MPEG_2_5", "LAYER_I", "LAYER_II",
                       "LAYER_III", "SAMPLE_RATE", "BIT_RATE", "SAMPLES_PER_FRAME"]),
    ("Audio quantization and synthesis", ["SCALEFACTOR_BASE", "QUANT_TAB_A", "QUANT_TAB_B",
                                         "QUANT_TAB_C", "QUANT_TAB_D", "QUANT_LUT_STEP_1",
                                         "QUANT_LUT_STEP_2", "QUANT_LUT_STEP_3", "QUANT_LUT_STEP4",
                                         "QUANT_TAB", "SYNTHESIS_WINDOW"]),
]
text = (SPEC / "pl-mpeg.md").read_text()
declarations = {}
for line in text.splitlines():
    match = re.match(r"- `([A-Z][A-Z_0-9]* (?:[^`]*))`", line)
    if match and "..." not in match[1]:
        declaration = match[1]
        declarations[declaration.split()[0]] = declaration + ";"
for declaration in re.findall(r"```jai\n(.*?)\n```", (SPEC / "pl-mpeg-public-tables.md").read_text(), re.S):
    declarations[declaration.split()[0]] = declaration.strip()
output = "// Generated from the public specifications by tools/generate_mpeg_tables.py.\n"
for title, names in groups:
    output += "\n// " + title + ".\n\n"
    output += "\n\n".join(declarations[name] for name in names) + "\n"
path = ROOT / "stdlib/internal/media/mpeg/tables.jai"
path.write_text(output)
subprocess.run([str(ROOT / "target/jaifmt"), "--config", str(ROOT / "jaifmt.toml"), str(path)], check=True)
