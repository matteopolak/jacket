# Image (`Extensions/Image`)

Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

## What it is

Decoders and encoders for PNG, QOI, BMP and PNM, JPEG decoding, pixel format conversion and a few transforms (resize, crop, flip, rotate, premultiply), in plain Jai: no foreign code, so it runs natively, under `jaic -interp`, in `-os wasm` programs and in the browser playground.

```jai
#import "Extensions/Image";

image, ok, error := decode_image_as(bytes, .RGBA8);   // any supported file, converted
if !ok { print("%\n", error); return; }
defer deinit(*image);

small := resize(image, 64, 64, .BOX);
png := encode_png(small);                             // [] u8, caller frees with array_free
```

An `Image` is `width`, `height`, `format` and `pixels`, rows packed top to bottom with no padding. Pixel formats are `GRAY8`, `GRAY_ALPHA8`, `RGB8`, `RGBA8` and `BGRA8` (8 bits per channel); `bytes_per_pixel` and `has_alpha` describe them. Everything that returns an `Image` or a `[] u8` allocates with `context.allocator`; free images with `deinit(*image)`, bytes with `array_free`. Errors are static strings; a failed decode returns an empty `Image`.

## How it works

| Procedure | Notes |
| --- | --- |
| `decode_image(data, max_pixels)` | Picks the decoder from the signature (`detect_image_format`) and returns the file's own format. |
| `decode_image_as(data, format, max_pixels)` | The same, then `convert`. |
| `decode_png`, `decode_jpeg`, `decode_qoi`, `decode_bmp`, `decode_pnm` | Per format. Every one refuses `width * height > max_pixels` (default 2^26) before allocating, and never reads outside the data. |
| `encode_png(image, level)`, `encode_qoi`, `encode_bmp`, `encode_ppm` | Invalid images (zero or negative size, a pixel array of the wrong length) give an empty result. |
| `convert(image, format)` | Between every pair of formats; always a new image. |
| `resize`, `crop`, `flip_vertical`, `flip_horizontal`, `rotate_90`, `premultiply_alpha` | See below. |

### PNG

`decode_png` returns the nearest of `GRAY8`, `GRAY_ALPHA8`, `RGB8`, `RGBA8`: palettes expand to RGB8 (RGBA8 when a `tRNS` chunk is present), a `tRNS` colour key on gray or RGB adds an alpha channel, 1, 2 and 4-bit gray is scaled to 0..255, and 16-bit samples keep their high byte (truncation, so a value `v * 257` comes back as `v`). Adam7 interlacing and all five filters are supported. Gamma, ICC profiles and other ancillary chunks are ignored; of an animated PNG only the default image (the `IDAT` data) is decoded. The CRC-32 of every critical chunk and of `tRNS` is checked, unknown ancillary chunks are skipped, an unknown critical chunk is an error, and `IEND` is required. A palette index past the end of the palette decodes as black.

The decoder reads the chunk list first, then inflates the concatenated `IDAT` data straight into one buffer sized exactly from the header (a lone `IDAT` chunk is read in place, several are gathered into one allocation). A header claiming more data than Deflate can possibly produce from the bytes present is refused before that buffer is allocated. Rows are then unfiltered by loops specialised on bytes per pixel (neighbours stay in registers; the Up filter uses `Extensions/SIMD`). For 8-bit gray, RGB, gray+alpha and RGBA images without a colour key the unfiltered rows are moved to the front of the same buffer and that buffer becomes `pixels`; other layouts expand each row (through a 256-entry table for palettes and gray of 8 bits or fewer) into a new image. Interlaced images unfilter each pass in place and scatter the pixels.

`encode_png` writes 8-bit PNGs of the four formats (`BGRA8` is written as RGBA). For each row it computes all five filters in one pass and keeps the one whose output has the smallest sum of absolute values (bytes read as signed); `level` 1 to 9 is passed to Deflate, and level 0 stores the rows unfiltered in uncompressed blocks. There is no palette reduction.

### QOI, BMP, PNM

- **QOI**: 3 or 4 channels to `RGB8`/`RGBA8`. The encoder writes RGBA when the image has alpha, otherwise RGB (gray is written as colour).
- **BMP**: OS/2 core, `BITMAPINFOHEADER`, V4 and V5 headers; 1, 4, 8 bits with a palette (also RLE8 and RLE4), 16 and 32 bits with `BI_BITFIELDS` (any contiguous masks, including alpha) or the default 5-5-5 and 8-8-8 layouts, 24 bits; bottom-up and top-down. Palettised and 24-bit files decode to `RGB8`. 16 and 32-bit files give `RGBA8` when an alpha mask exists (for 32-bit `BI_RGB`, when any fourth byte is nonzero), else `RGB8`. Pixels RLE never writes stay black. `encode_bmp` writes a bottom-up 24-bit file, or a 32-bit V4 file with an alpha mask when the image has alpha.
- **PNM**: `P2`/`P3` (text) and `P5`/`P6` (binary) with `#` comments; maxval 1..65535 is scaled to 8 bits with rounding (binary samples are 16-bit big-endian above 255). `P1`/`P4` bitmaps are not supported. `encode_ppm` writes `P6`, or `P5` for gray, dropping alpha.

### Conversion and transforms

`convert` has one loop per (source, destination) pair, instantiated from a single template. Gray from colour is Rec. 709 luma in 16-bit fixed point (`(13933 r + 46871 g + 4732 b + 32768) >> 16`, weights sum to 65536); gray to colour repeats the value; alpha is added as 255 and dropped without compositing.

`resize(image, width, height, filter)`: `NEAREST` picks the closest source pixel; `BILINEAR` blends the four pixels around each sample point and does not widen its footprint when shrinking; `BOX` averages the exact source area under each output pixel (use it to shrink by much). Both filtered modes are separable: a horizontal pass into 16-bit samples then a vertical pass, with 14-bit weights adjusted to sum to exactly one, so a flat image stays flat. Channels are filtered independently and alpha is not premultiplied: call `premultiply_alpha` before resizing if colours of transparent pixels must not bleed. `crop` returns `ok = false` when the rectangle is empty or leaves the image. `flip_*` work in place, `rotate_90` returns a new image whose width is the old height.

### JPEG

`decode_jpeg` reads baseline, extended sequential and progressive Huffman JPEG with 8-bit samples:
- any sampling factors from 1 to 4 (4:4:4, 4:2:2, 4:2:0, 4:4:0, 4:1:1 and odd mixes);
- restart intervals and non-interleaved scans;
- 8- and 16-bit quantization tables;
- sizes that are not a multiple of the MCU.

One component gives `GRAY8` and everything else gives `RGB8`:
- YCbCr is converted with JFIF full-range coefficients.
- Adobe transform 0, or components named R, G, B, are read as RGB.
- Four components are read as Adobe-style inverted CMYK, or YCCK when the Adobe transform is 2, and converted to RGB.

Arithmetic coding, 12-bit, lossless and hierarchical files, DNL and 2-component files are refused with a specific error. EXIF orientation is not applied. The output matches libjpeg-turbo's `djpeg -dct int` exactly.

The work is split across four files:
- `jpeg.jai` reads the marker segments.
- `jpeg_scan.jai` decodes each scan in a restart-aware MCU loop. Sequential blocks are dequantized in the zigzag loop and transformed straight into per-component planes; progressive scans accumulate coefficients and transform at the end.
- `jpeg_huffman.jai` has the tables (a 9-bit lookahead that also carries the decoded AC run, size and value) and a 64-bit bit reader.
- `jpeg_output.jai` has the integer IDCT (all-zero rows and columns and DC-only blocks are skipped), triangle-filter upsampling for h2v1, h2v2 and h1v2 (other ratios repeat samples), and the fixed-point colour conversion.

Damaged data never reads or writes out of bounds, and decoding always ends:
- Truncated entropy data decodes as far as it goes; the rest stays mid-grey.
- A damaged restart interval costs only its own MCUs, and decoding picks up again at the next restart marker.
- Cut-off headers, bad tables, and scans that name unknown components or undefined tables are errors.

To change it:
- Sampling and colour handling are in `jpeg_output.jai` (`jpeg_component_row`, `jpeg_write_pixels`); a new scan type goes in `jpeg_decode_scan`.
- The hot procedures are `#no_abc` and use pointers. Keep every index guarded (zigzag position at most 63, table lookups masked), and after touching them rerun `tests/jpeg_robust.jai` and a fuzz run under `-sanitize address`.
- Fixtures come from `tests/fixtures/jpeg/make_fixtures.sh`, which needs `cjpeg`, `djpeg` and `python3`.
- The benchmark is `bench/jpeg_bench.jai`, run against stb_image (native only, with `JAIC_NATIVE_LIBS` set): about 0.7–0.95x stb_image's time on baseline, progressive and grayscale photos.

## How to change it

- **A new file format** goes in its own file with a `decode_<name>` returning `Image, bool, string`, an entry in `Image_File_Format`, `detect_image_format` and `decode_image` in `module.jai`, and an `#load`. Keep its helpers in `#scope_file` with a format prefix (`png_`, `qoi_`): all files share one module scope.
- Check sizes with `image_size_ok(width, height, max_pixels)` and the data length *before* allocating, and use `image_is_valid` at the top of anything that takes an `Image`.
- **A new pixel format**: add it to `Pixel_Format`, `bytes_per_pixel`, `has_alpha`, and every pair in `convert` (the `#complete` switches in `convert.jai` fail to compile until you do). `convert_pixels` reads a pixel as r, g, b, a and writes it back, so a new format needs a branch in each direction.
- **Unfilter kernels** are `png_unfilter_bpp` (`$BPP` is a compile-time constant); the Paeth predictor is `png_paeth`. `png_filter_rows` is the encoder's mirror image and must pick the same predictors.
- Casts check overflow in this compiler: narrowing needs `cast,trunc`, and `cast(u32) x << 8` parses as a cast of the whole shift, so parenthesise (`(cast(u32) x) << 8`).
- Run `jaifmt` and `jailint` on changes. File-format offsets are the content of the format files, so they carry `// jailint: allow-file(magic_number)`.

### Tests

`stdlib/Extensions/Image/tests/` (all run in every mode except `stb_png`, which needs the C libraries and is native/interp only):

| File | Covers |
| --- | --- |
| `png.jai` | 125 fixtures from `fixtures/generate_png_cases.py` (every colour type, depth, filter, interlace, colour key, palette alpha, split `IDAT`, ancillary and APNG chunks) with exact expected pixels; encoder round trips at several levels; chunk rules |
| `stb_png.jai` | our decoder on stb_image_write output, stb_image on our output, both decoders on every fixture |
| `qoi.jai`, `bmp.jai`, `pnm.jai` | hand-built files (every QOI op; every BMP header, depth, mask and RLE form; text and binary PNM) and round trips |
| `convert.jai`, `transform.jai` | every format pair against a per-pixel reference; exact checks of resize, crop, flips, rotation and premultiplication on small images |
| `malformed.jai` | every decoder on files cut at every byte and with every byte changed, under a tracking allocator (no leaks, no big allocation for a lying header) |
| `module.jai` | detection, `decode_image`, `decode_image_as`, `make_image` |

Regenerate the fixtures with `python3 stdlib/Extensions/Image/tests/fixtures/generate_png_cases.py`. The fuzz hooks (`tools/fuzz/properties/Image.jai`, target `Extensions/Image`) run the decoders on random and mutated bytes and check encode/decode and transform round trips: `PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py Extensions/Image --jaic <jaic>` ([stdlib fuzzing](../tools/stdlib-fuzzing.md)).

## Benchmarks

```sh
jaic -O2 stdlib/Extensions/Image/bench/bench.jai -exe /tmp/image-bench
/tmp/image-bench            # 1 MP and 8 MP images
/tmp/image-bench quick      # 1 MP only
```

Megapixels per second on synthetic content ("photo": smooth gradients with noise; "ui": flat panels, lines and a few colours), best of several runs, against `stb_image`, `stb_image_write` (default settings) and `stb_image_resize` (box and triangle filters, linear). PNG decoding reads the file stb wrote in both libraries. "x stb" is our speed over stb's; "size" is our compressed size over stb's. Resize rates are input megapixels per second. Apple M-series, `-O2`, 8.4 MP images (1 MP within 3%):

| Case | Ours MP/s | stb MP/s | x stb | Size |
| --- | --- | --- | --- | --- |
| PNG decode, photo RGB8 | 130 | 81 | 1.60 | |
| PNG decode, photo RGBA8 | 109 | 69 | 1.59 | |
| PNG decode, photo GRAY8 | 315 | 183 | 1.73 | |
| PNG decode, ui RGB8 | 2044 | 1637 | 1.25 | |
| PNG decode, ui RGBA8 | 1557 | 1396 | 1.12 | |
| PNG decode, Adam7, photo RGB8 | 107 | 79 | 1.36 | |
| PNG decode, Adam7, ui RGB8 | 290 | 234 | 1.24 | |
| PNG encode level 6, photo RGB8 | 11.8 | 12.0 | 0.99 | 0.65 |
| PNG encode level 6, photo RGBA8 | 9.8 | 9.9 | 0.99 | 0.67 |
| PNG encode level 1, photo RGB8 | 30.1 | 12.0 | 2.51 | 0.63 |
| PNG encode level 6, ui RGB8 | 97.6 | 46.3 | 2.11 | 0.39 |
| PNG encode level 6, ui RGBA8 | 85.0 | 35.0 | 2.42 | 0.34 |
| QOI encode, photo RGB8 | 131 | | | 0.96 (of stb's PNG) |
| QOI decode, photo RGB8 | 236 | | | |
| QOI encode / decode, ui RGBA8 | 421 / 502 | | | 2.8 |
| convert RGBA8 to RGB8 / GRAY8 / BGRA8 | 605 / 951 / 580 | | | |
| convert GRAY8 to RGBA8 | 696 | | | |
| resize RGB8 NEAREST down 4x / up 2x | 7652 / 115 | | | |
| resize RGB8 BILINEAR down 4x / up 2x | 1157 / 85 | 294 / 65 | 3.93 / 1.32 | |
| resize RGB8 BOX down 4x / up 2x | 718 / 92 | 209 / 72 | 3.43 / 1.28 | |
| resize RGBA8 BILINEAR down 4x / up 2x | 1108 / 72 | 279 / 62 | 3.98 / 1.17 | |
| resize RGBA8 BOX down 4x / up 2x | 692 / 78 | 207 / 72 | 3.34 / 1.08 | |

PNG time is mostly Deflate: decoding a photo spends most of it in `inflate_into`, encoding in `deflate`. Without the SIMD Up kernel and register-carried predictors the flat-content decodes were 0.25 to 0.4 of stb's speed.

## Configuration

`DEFAULT_MAX_PIXELS` (2^26, about 67 million pixels) is the default `max_pixels` of every decoder; pass a smaller limit for untrusted input. Compression level is the `level` argument of `encode_png`. There are no environment variables.

## Dependencies

[`Extensions/Deflate`](deflate.md) (zlib inflate/deflate and CRC-32 for PNG), `Extensions/SIMD` (unfilter and resize inner loops), `Basic`. The tests and benchmark also use `stb_image`, `stb_image_write` and `stb_image_resize` (native libraries).
