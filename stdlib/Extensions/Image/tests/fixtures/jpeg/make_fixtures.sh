#!/bin/sh
# Regenerates the JPEG test fixtures in this directory: small synthetic pictures encoded with
# cjpeg, and the reference decode of each from djpeg (`-dct int`, fancy upsampling) as raw
# PPM/PGM next to it. The CMYK/YCCK files are assembled from four grayscale scans by hand (cjpeg
# cannot write them); their references come from djpeg decodes of the planes.
#
#     sh stdlib/Extensions/Image/tests/fixtures/jpeg/make_fixtures.sh
#
# Needs libjpeg-turbo's cjpeg and djpeg (3.x for the 12-bit, lossless and arithmetic samples)
# and python3.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"

python3 - <<'PY'
import math

def lcg(seed):
    state = seed
    while True:
        state = (state * 1103515245 + 12345) & 0x7fffffff
        yield (state >> 8) & 0xff

def write_ppm(name, w, h, channels, fn):
    noise = lcg(w * 31 + h)
    out = bytearray()
    for y in range(h):
        for x in range(w):
            n = (next(noise) % 17) - 8
            out += bytes(max(0, min(255, v + n)) for v in fn(x, y, w, h))
    magic = b"P6" if channels == 3 else b"P5"
    with open(name, "wb") as f:
        f.write(b"%s\n%d %d\n255\n" % (magic, w, h))
        f.write(out)

def color(x, y, w, h):
    dx, dy = x - w * 0.6, y - h * 0.4
    inside = dx * dx + dy * dy < (min(w, h) * 0.3) ** 2
    r = 40 + 200 * x // max(w - 1, 1)
    g = 220 - 180 * y // max(h - 1, 1)
    b = 128 + int(100 * math.sin(x * 0.4) * math.cos(y * 0.3))
    if inside:
        r, g, b = 250 - r // 2, b, 30 + g // 3
    if (x // 8 + y // 8) % 5 == 0:
        r, g, b = 255 - r, 255 - g, 255 - b
    return (r, g, b)

def gray(x, y, w, h):
    r, g, b = color(x, y, w, h)
    return ((r * 3 + g * 6 + b) // 10,)

def plane(k):
    def fn(x, y, w, h):
        return ((x * 7 * (k + 1) + y * 5 * (k + 2) + 64 * k + int(60 * math.sin(x * 0.3 + k))) % 256,)
    return fn

write_ppm("photo.ppm", 37, 23, 3, color)
write_ppm("gray.pgm", 37, 23, 1, gray)
write_ppm("tiny.ppm", 1, 1, 3, color)
write_ppm("odd.ppm", 17, 9, 3, color)
write_ppm("block.ppm", 8, 8, 3, color)
for k in range(4):
    write_ppm("plane%d.pgm" % k, 37, 23, 1, plane(k))
PY

# 16-bit quantization tables: a luma and a chroma table with entries above 255.
python3 - <<'PY'
rows = []
for table in range(2):
    rows.append(" ".join(str(6 + 5 * i + 40 * table) for i in range(64)))
open("wide.q", "w").write("\n".join(rows) + "\n")
PY

make() { # name input cjpeg-args...
    name=$1
    input=$2
    shift 2
    cjpeg "$@" -outfile "$here/$name.jpg" "$input"
    ext=ppm
    case $input in *.pgm) ext=pgm ;; esac
    djpeg -dct int -outfile "$here/$name.$ext" "$here/$name.jpg"
    # A grayscale file decodes to a PGM; a color one to a PPM.
    if [ "$(head -c 2 "$here/$name.$ext")" = P5 ] && [ $ext = ppm ]; then
        mv "$here/$name.ppm" "$here/$name.pgm"
    fi
}

make base444 photo.ppm -quality 85 -sample 1x1
make base422 photo.ppm -quality 85 -sample 2x1
make base420 photo.ppm -quality 85 -sample 2x2
make base440 photo.ppm -quality 85 -sample 1x2
make base411 photo.ppm -quality 85 -sample 4x1
make base410 photo.ppm -quality 85 -sample 4x2
make wide_chroma photo.ppm -quality 85 -sample 1x1,2x2,2x2
make odd_sampling photo.ppm -quality 85 -sample 2x2,1x2,2x1
make gray gray.pgm -quality 85
make gray_progressive gray.pgm -quality 85 -progressive
make progressive420 photo.ppm -quality 85 -progressive
make progressive444 photo.ppm -quality 85 -progressive -sample 1x1
make progressive_restart photo.ppm -quality 85 -progressive -restart 4B
make restart420 photo.ppm -quality 85 -restart 3B
make restart_each photo.ppm -quality 85 -sample 1x1 -restart 1B
make progressive_restart_each photo.ppm -quality 85 -progressive -sample 1x1 -restart 1B
make optimized photo.ppm -quality 85 -optimize
make progressive_optimized photo.ppm -quality 60 -progressive -optimize
make rgb photo.ppm -quality 85 -rgb
make wide_quant photo.ppm -qtables wide.q -sample 1x1
make tiny tiny.ppm -quality 90
make odd17x9 odd.ppm -quality 90
make block8 block.ppm -quality 90 -sample 1x1

# Files the decoder must refuse.
cjpeg -arithmetic -outfile "$here/arithmetic.jpg" block.ppm
cjpeg -precision 12 -outfile "$here/twelve_bit.jpg" block.ppm
cjpeg -lossless 1 -outfile "$here/lossless.jpg" block.ppm

# CMYK and YCCK: four grayscale scans (one per component, standard tables) under one frame header.
for k in 0 1 2 3; do
    cjpeg -quality 90 -outfile "plane$k.jpg" "plane$k.pgm"
    djpeg -dct int -outfile "plane$k.dec.pgm" "plane$k.jpg"
done

python3 - "$here" <<'PY'
import struct, sys

here = sys.argv[1]

def segments(data):
    pos = 2
    while pos < len(data):
        assert data[pos] == 0xFF
        code = data[pos + 1]
        length = struct.unpack(">H", data[pos + 2:pos + 4])[0]
        yield code, data[pos + 4:pos + 2 + length], pos + 2 + length
        if code == 0xDA:
            return
        pos += 2 + length

planes = [open("plane%d.jpg" % k, "rb").read() for k in range(4)]
tables = b""
for code, body, _ in segments(planes[0]):
    if code in (0xDB, 0xC4):
        tables += bytes([0xFF, code]) + struct.pack(">H", len(body) + 2) + body
    if code == 0xC0:
        height, width = struct.unpack(">HH", body[1:5])

def entropy(data):
    for code, body, end in segments(data):
        if code == 0xDA:
            return data[end:data.rindex(b"\xff\xd9")]

def build(transform):
    out = b"\xff\xd8"
    out += b"\xff\xee" + struct.pack(">H", 14) + b"Adobe" + struct.pack(">HHHB", 100, 0, 0, transform)
    sof = struct.pack(">BHHB", 8, height, width, 4) + b"".join(bytes([k + 1, 0x11, 0]) for k in range(4))
    out += b"\xff\xc0" + struct.pack(">H", len(sof) + 2) + sof
    out += tables
    for k in range(4):
        sos = bytes([1, k + 1, 0x00, 0, 63, 0])
        out += b"\xff\xda" + struct.pack(">H", len(sos) + 2) + sos + entropy(planes[k])
    return out + b"\xff\xd9"

def read_pgm(name):
    raw = open(name, "rb").read()
    parts = raw.split(b"\n", 3)
    return parts[3]

def scale(a, b):
    t = a * b + 128
    return (t + (t >> 8)) >> 8

def clamp(v):
    return max(0, min(255, v))

dec = [read_pgm("plane%d.dec.pgm" % k) for k in range(4)]
open(here + "/cmyk.jpg", "wb").write(build(0))
open(here + "/ycck.jpg", "wb").write(build(2))

cmyk = bytearray()
ycck = bytearray()
for i in range(width * height):
    c, m, y, k = (dec[j][i] for j in range(4))
    cmyk += bytes([scale(c, k), scale(m, k), scale(y, k)])
    cb, cr = m - 128, y - 128
    r = clamp(c + ((91881 * cr + 32768) >> 16))
    b = clamp(c + ((116130 * cb + 32768) >> 16))
    g = clamp(c + ((-22554 * cb + 32768 - 46802 * cr) >> 16))
    ycck += bytes([scale(255 - r, k), scale(255 - g, k), scale(255 - b, k)])

header = b"P6\n%d %d\n255\n" % (width, height)
open(here + "/cmyk.ppm", "wb").write(header + cmyk)
open(here + "/ycck.ppm", "wb").write(header + ycck)
PY

echo "fixtures: $(cat "$here"/*.jpg "$here"/*.ppm "$here"/*.pgm | wc -c | tr -d ' ') bytes"
