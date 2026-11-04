# Image

Decoders and encoders for PNG, QOI, BMP and PNM, JPEG decoding, pixel format conversion and a few transforms (resize, crop, flip, rotate, premultiply), in plain Jai: no foreign code, so it runs natively, under `jaic -interp`, in `-os wasm` and in the playground.

```jai
#import "Extensions/Image";

image, ok, error := decode_image_as(bytes, .RGBA8);   // any supported file, converted
if !ok { print("%\n", error); return; }
defer deinit(*image);

small := resize(image, 64, 64, .BOX);
png := encode_png(small);                             // [] u8, free with array_free
```

An `Image` is `width`, `height`, `format` and `pixels`, rows packed top to bottom. Formats are `GRAY8`, `GRAY_ALPHA8`, `RGB8`, `RGBA8` and `BGRA8`. Everything allocates with `context.allocator`. Errors are static strings; a failed decode returns an empty `Image`. Every decoder refuses `width * height > max_pixels` (default 2^26) before allocating and never reads outside the data; pass a smaller limit for untrusted input.

## How it works

- `decode_image` picks the decoder from the signature (`detect_image_format`) and returns the file's own format; `decode_image_as` then calls `convert`.
- **PNG**: returns the nearest of the four 8-bit formats. Palettes expand, a `tRNS` key adds alpha, low-depth gray scales to 0..255, 16-bit keeps the high byte. All filters and Adam7 work. Critical-chunk CRCs are checked; other ancillary chunks are ignored, and APNG yields only the default image. The decoder inflates `IDAT` straight into one buffer sized from the header, unfilters with loops specialised on bytes per pixel (the Up filter uses SIMD), and for common layouts reuses that buffer as `pixels`.
- **`encode_png`**: levels 1-8 pick one filter per row by a sampled sum-of-absolute-values heuristic; level 9 tries all five over the whole row; level 0 stores. No palette reduction.
- **QOI**: 3 or 4 channels. **BMP**: OS/2, V3-V5 headers, palettes, RLE, bitfields, bottom-up and top-down; `encode_bmp` writes 24-bit or 32-bit with alpha. **PNM**: `P2/P3/P5/P6`; `encode_ppm` writes `P6` or `P5`.
- **JPEG**: baseline, extended and progressive Huffman, 8-bit, any sampling factors 1-4, restart intervals. One component gives `GRAY8`, everything else `RGB8` (YCbCr, Adobe RGB, CMYK/YCCK). Deferred height is resolved from the DNL marker after the first scan before allocating pixels. JFIF takes precedence over conflicting Adobe color hints; ordinary CMYK and Adobe-inverted CMYK use their respective conversions. Arithmetic, 12-bit, lossless and hierarchical files are refused; EXIF orientation is not applied. Structural errors, including invalid scan order or progressive refinement history, are rejected. DNL validates dimensions and placement; the permissive entropy reader does not certify that the encoded MCU-row count matches that declaration. Truncated entropy data is recovered with neutral samples, and a bad restart interval costs only its own MCUs; this recovery is not strict validation of every damaged stream.
- **`convert`** has one loop per format pair from a single template. Gray from colour is Rec. 709 luma in fixed point; alpha is added as 255 and dropped without compositing.
- **`resize`**: `NEAREST`, `BILINEAR`, or `BOX` (averages the exact area; best for big shrinks). Filtered modes are separable with weights summing to exactly one. Alpha is not premultiplied, so call `premultiply_alpha` first if transparent colours must not bleed. `crop` returns `ok = false` for an empty or outside rectangle; `flip_*` work in place; `rotate_90` returns a new image.
- ICO: `decode_ico(data, entry = -1)` decodes one icon entry (PNG-compressed or 32/24-bit truecolor BMP) and picks the largest when `entry` is `-1`. Lives in `ico.jai`; `decode_image` doesn't detect ICO files, so call it directly.

## How to change it

- New file format: its own file with `decode_<name>` returning `Image, bool, string`, an entry in `Image_File_Format`, `detect_image_format` and `decode_image` in `module.jai`, and a `#load`. Keep helpers in `#scope_file` with a format prefix (`png_`, `qoi_`); all files share one module scope.
- Check sizes with `image_size_ok` and the data length before allocating; call `image_is_valid` at the top of anything taking an `Image`.
- New pixel format: add it to `Pixel_Format`, `bytes_per_pixel`, `has_alpha` and every pair in `convert` (the `#complete` switches fail to compile until you do).
- PNG unfilter kernels are `png_unfilter_bpp` (`$BPP` is compile-time) and `png_paeth`; `png_filter_rows` is the encoder's mirror and must pick the same predictors.
- JPEG: sampling and colour are in `jpeg_output.jai`; a new scan type goes in `jpeg_decode_scan`. Keep DNL preflight and normal marker validation consistent, and maintain each coefficient's progressive bit history in `jpeg_read_scan`. The hot procedures are `#no_abc`, so keep every index guarded and rerun `tests/jpeg_robust.jai` plus a fuzz run under `-sanitize address`. Fixtures come from `tests/fixtures/jpeg/make_fixtures.sh` (needs `cjpeg`, `djpeg`, `python3`).
- Casts check overflow: narrowing needs `cast,trunc`, and `cast(u32) x << 8` casts the whole shift, so parenthesise.

## Tests and benchmarks

`stdlib/Extensions/Image/tests/` covers PNG (125 generated fixtures, regenerate with `python3 stdlib/Extensions/Image/tests/fixtures/generate_png_cases.py`), QOI, BMP, PNM, conversion, transforms, and `malformed.jai` (every decoder on files cut and mutated at every byte, under a tracking allocator). `stb_png.jai` needs the C libraries (native and interp only). Fuzz: `PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py Extensions/Image --jaic <jaic>`.

```sh
jaic -O2 stdlib/Extensions/Image/bench/bench.jai -exe /tmp/image-bench && /tmp/image-bench quick
```

The benchmark compares against `stb_image`, `stb_image_write` and `stb_image_resize` on synthetic photo and UI content (Apple M-series, 8.4 MP):

| Case | Speed vs stb |
| --- | --- |
| PNG decode, photo | 1.4-1.7x |
| PNG decode, UI | 1.1-1.3x |
| PNG encode level 6, photo | 1.0x, output 0.65 of stb's size |
| PNG encode level 6, UI | 2.1-2.4x, output 0.34-0.39 of stb's size |
| resize down 4x | 3.3-4.0x |
| resize up 2x | 1.1-1.3x |

PNG time is mostly [Deflate](deflate.md). Also uses `Extensions/SIMD` for unfilter and resize loops, and `Extensions/Byte_Search` for JPEG marker preflight.

`Ico_File.create_ico_file_from_bitmap_filename` uses these decoders on wasm. It applies `DEFAULT_MAX_PIXELS` (`1 << 26`) and requires RGB or RGBA output before conversion. Native filename loading retains stb and its format coverage. The consumer creates ICO bytes; `decode_image` does not read ICO files. See [ICO file creation](../stdlib/ico-files.md) for ownership, limits and WASI execution.
