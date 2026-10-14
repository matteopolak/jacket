#!/usr/bin/env python3
"""Create original elementary test streams from reviewed MPEG VLC tables."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
spec = (ROOT / "docs/specs/stdlib/modules/pl-mpeg.md").read_text()
appendix = (ROOT / "docs/specs/stdlib/modules/pl-mpeg-public-tables.md").read_text()
declarations = {}
for line in spec.splitlines():
    match = re.match(r"- `([A-Z_]+ :: Vlc\.\[.*?)`", line)
    if match:
        declarations[match[1].split()[0]] = match[1]
declarations["DCT_COEFF"] = re.search(r"```jai\n(DCT_COEFF.*?);\n```", appendix, re.S)[1]


def integer(expression):
    parts = expression.strip().split("<<")
    value = int(parts[0].strip(), 0)
    if len(parts) == 2:
        value <<= int(parts[1].strip(), 0)
    else:
        assert len(parts) == 1
    return value


def vlc(name, value):
    nodes = [(integer(a), integer(b)) for a, b in
             re.findall(r"\.\{\s*([^,]+),\s*([^}]+)\}", declarations[name])]

    def find(index, prefix):
        for bit in range(2):
            branch, leaf = nodes[index + bit]
            code = prefix + str(bit)
            if branch == 0 and leaf == value:
                return code
            if branch > 0:
                found = find(branch, code)
                if found is not None:
                    return found
        return None

    result = find(0, "")
    assert result is not None, (name, value)
    return result


class Bits:
    def __init__(self):
        self.bits = ""

    def put(self, value, width):
        assert 0 <= value < (1 << width)
        self.bits += format(value, f"0{width}b")

    def code(self, name, value):
        self.bits += vlc(name, value)

    def finish(self):
        bits = self.bits + "0" * (-len(self.bits) % 8)
        return bytes(int(bits[i:i + 8], 2) for i in range(0, len(bits), 8))


SEQUENCE = bytes([0, 0, 1, 0xb3, 1, 0, 0x10, 0x13, 0, 0, 0x20, 0])


def picture(kind, dc, temporal, ac=False):
    header = Bits()
    header.put(temporal, 10)
    header.put(kind, 3)
    header.put(0xffff, 16)
    if kind in (2, 3):
        header.put(0, 1)
        header.put(1, 3)
    if kind == 3:
        header.put(0, 1)
        header.put(1, 3)
    header.put(0, 1)
    slice_bits = Bits()
    slice_bits.put(8, 5)
    slice_bits.put(0, 1)
    slice_bits.code("MACROBLOCK_ADDRESS_INCREMENT", 1)
    table = {1: "MACROBLOCK_TYPE_INTRA", 2: "MACROBLOCK_TYPE_PREDICTIVE", 3: "MACROBLOCK_TYPE_B"}[kind]
    slice_bits.code(table, 1)
    for block in range(6):
        difference = dc - 128 if block == 0 else 0
        size = abs(difference).bit_length()
        slice_bits.code("DCT_SIZE_LUMINANCE" if block < 4 else "DCT_SIZE_CHROMINANCE", size)
        if size:
            slice_bits.put(difference if difference > 0 else (1 << size) - 1 + difference, size)
        if ac and block == 0:
            slice_bits.code("DCT_COEFF", 1)
            slice_bits.put(1, 1)  # Distinguish run=0/level=1 from end-of-block.
            slice_bits.put(0, 1)  # Positive coefficient.
        slice_bits.code("DCT_COEFF", 1)
        slice_bits.put(0, 1)
    return bytes([0, 0, 1, 0]) + header.finish() + bytes([0, 0, 1, 1]) + slice_bits.finish()


output = ROOT / "target/portable-tests/mpeg"
output.mkdir(parents=True, exist_ok=True)
(output / "reordered.m1v").write_bytes(SEQUENCE + picture(1, 80, 0) + picture(2, 160, 2) + picture(3, 120, 1))
(output / "ac.m1v").write_bytes(SEQUENCE + picture(1, 128, 0, ac=True))


def audio_frame(mode, selected_subband, protected=False):
    # MPEG-1, 128 kbit/s stereo: reviewed table A, 27 subbands. Joint stereo
    # extension zero shares sample codes from subband four onward.
    widths = [4] * 11 + [3] * 12 + [2] * 4
    bound = 4 if mode == 1 else 27
    payload = Bits()
    for sb, width in enumerate(widths):
        payload.put(1 if sb == selected_subband else 0, width)
        if sb < bound:
            payload.put(1 if sb == selected_subband else 0, width)
    for _ in range(2):
        payload.put(2, 2)  # One scalefactor for all three parts.
    protected_bits = payload.bits
    payload.put(3, 6)  # Leave headroom for synthesis ringing.
    payload.put(9, 6)  # Two octave steps: right channel gain is one quarter.
    for _ in range(12):
        payload.put(21, 5)  # Grouped base-three samples [0, 1, 2].
        if selected_subband < bound:
            payload.put(21, 5)
    body = payload.finish()
    header = bytes([0xff, 0xfc if protected else 0xfd, 0x80, mode << 6])
    if protected:
        crc = 0xffff
        control = "".join(format(b, "08b") for b in header[2:]) + protected_bits
        for bit in control:
            feedback = (crc >> 15) ^ int(bit)
            crc = (crc << 1) & 0xffff
            if feedback:
                crc ^= 0x8005
        header += crc.to_bytes(2, "big")
    size = 417 - len(header)
    assert len(body) <= size
    return header + body + bytes(size - len(body))


for name, mode, sb in [("stereo", 0, 0), ("dual", 2, 0), ("joint", 1, 5)]:
    frame = audio_frame(mode, sb)
    assert len(frame) == 417
    (output / (name + ".mp2")).write_bytes(frame * 2)
frame = audio_frame(1, 5, protected=True)
(output / "joint-crc.mp2").write_bytes(frame * 2)
print("Built independent MPEG B/AC and nonzero stereo fixtures")
