#!/usr/bin/env python3
"""Emit reviewed audio ABI shapes grouped by independent subsystem dependencies."""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/specs/stdlib/modules/sound-player.md"
ORDER = [
    "Audio_Channel", "Source_Direction", "Sound_Category", "Audio_Sample_Base",
    "Audio_Source_Sample", "Audio_Dest_Sample", "Scale_Mapping", "Sound_Data",
    "Sound_Stream", "Decoder_Type", "Cached_Decoder_Page", "Cached_Decoder",
    "Adpcm_Decoder", "Buffer_Bounds", "Fill_Region_Result", "Backend_Properties",
    "Output_Device", "Sound_Player_Config", "Vbap_Arc",
]
blocks = dict(re.findall(r"### `([^`]+)`\s+```jai\s+(.*?)```", SPEC.read_text(), re.S))
assert len(ORDER) == len(set(ORDER))
output = "// Generated reviewed public layouts; independent dependency grouping.\n"
output += "MAX_CHANNELS_PER_SOUND :: 8;\nACHANNEL_MAX :: 8;\nOUTPUT_CHANNELS_MAX :: ACHANNEL_MAX;\nNUM_SOURCE_DIRECTIONS :: 11;\n\n"
output += "\n\n".join(blocks[name].rstrip() for name in ORDER)
output += '\n\n#scope_file\n#import "Math";\n'
path = ROOT / "stdlib/internal/audio/records.jai"
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(output)
subprocess.run([str(ROOT / "target/jaifmt"), "--config", str(ROOT / "jaifmt.toml"), str(path)], check=True)
