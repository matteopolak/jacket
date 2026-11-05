# Image codec coverage

`Extensions/Image` provides portable byte codecs with owned pixel buffers. The conformance tools exercise pinned public corpora, direct and detected entry points, allocation limits, and encoder output decoded independently. They do not certify every feature of an image standard.

## Profiles and alternatives

The portable API returns 8-bit gray, gray-alpha, RGB, RGBA or BGRA pixels. PNG input can have 1/2/4/8/16-bit samples; 16-bit samples reduce to their high byte. JPEG returns gray or RGB from 8-bit Huffman baseline, extended sequential or progressive streams, including CMYK/YCCK conversion. JPEG arithmetic, lossless, hierarchical and 12-bit processes are outside this API. ICC color management, EXIF orientation and animation playback are not applied.

| Implementation | Decode and encode scope | Precision and color | Portability and limits | Evidence here |
| --- | --- | --- | --- | --- |
| `Extensions/Image` | Decode PNG/JPEG/QOI/BMP/PNM; explicit ICO decode. Encode PNG/QOI/BMP/PNM. | 8-bit output; PNG transparency/palettes/Adam7; JPEG gray, YCbCr, RGB, CMYK/YCCK. | Native, interpreter and WASI corpus modes. Raster default cap `2^26` pixels; ICO cap 256×256. | Executable full inventories below; inspect the source-qualified receipts for actual results. |
| `Ico_File` filename adapter | Creates five ICO entries from raster input; native stb path and portable WASI path have different accepted formats. | Requires decoded RGB/RGBA. Straight BGRA entries, resized alpha and Windows AND masks. | Native retains stb coverage; WASI uses the portable codecs and raster cap. Filesystem access is a WASI host feature, not browser path access. | Filename fixtures, independent wire checks, and actual Windows GDI mask/render checks. |
| [n00bmind/qoi, `d26c7ab`](https://github.com/n00bmind/qoi/tree/d26c7ab39c6bd4d489c3c35d2feb281f9272dc01) (Jai) | QOI encoder/decoder; CLI advertises conversion through stb-image-write formats. | README does not establish a broader precision/color profile. | Platform guarantees, limits and our compiler compatibility unverified. | README comparison only; no source inspection, execution or timing here. |
| [stb, `2c980bb`](https://github.com/nothings/stb/tree/2c980bb59875b0d32144a71867fbdebb2f77cd20) (C) | PNG/JPEG/BMP/PNM plus native-only formats including GIF/TGA/HDR/PSD; separate image-write library. | PNG 8/16-bit interfaces; JPEG baseline/progressive without arithmetic/12-bit. Binary PNM. | C dependency; configurable dimension bound and implementation size limits, rather than this API's pixel cap. | Pinned native provider and JPEG benchmark comparator. No full stb conformance claim. |
| [libjpeg-turbo 3.2.0](https://github.com/libjpeg-turbo/libjpeg-turbo/tree/c85e6b905bf237038faa936dab160ebfc5da0344) (C) | JPEG codec and command-line tools, not a multi-format image API. | Reference uses accurate integer IDCT, fancy upsampling and unprofiled CMYK conversion. | Native reference built from pinned source; the reference rejects DNL. | Independent JPEG pixel/diagnostic oracle; not an oracle for every T.81 feature. |
| [Rust `image` 0.25.8](https://docs.rs/image/0.25.8/image/codecs/index.html) | PNG/JPEG/BMP/ICO/QOI encode/decode; PNM and additional formats behind features. | Generic pixel types; codec-specific precision and color behavior. | Feature-dependent dependencies; its platform/limit behavior was not executed here. | Published API comparison at [source `98b001d`](https://github.com/image-rs/image/tree/98b001da0ddcd91936a716696fba877df910b61d), not suite or timing evidence. |

These alternatives serve different purposes. The filename adapter preserves native coverage; the portable codecs provide interpreter/WASI execution and a common allocation contract. Features documented by alternatives are not interoperability or performance results for this suite.

## Complete pinned inventories

| Corpus | Required accounting |
| --- | --- |
| Original PngSuite 2017jul19 | All 176 files: 162 valid accepted, 14 invalid rejected. All valid gray/color/transparency/interlace cases compare with pinned independent RGBA oracles. The HTTPS mirror is pinned and its PNG bytes match the original archive. |
| BMP Suite 2.8 | All 91 files: 27 mandatory good cases compare with the author's PNG references. All questionable, bad and related-format cases execute too. The author's bad-case rule requires safe handling, not universal rejection. |
| `imazen/codec-corpus` PNM | All 51 files: supported P2/P3/P5/P6, malformed cases, and explicit PBM/PAM rejections. PBM/PAM are outside the declared API, not silently omitted. |
| Official QOI reference | 648 reference-encoded cases, 24 exhaustive opcode-range cases, 32 reference-rejected negatives, and 15 normative framing negatives that the permissive reference accepts. RGB/RGBA, both colorspaces, every index/diff/luma/run code, channel ranges and exact end padding are checked. No finite official certification suite was identified. |
| Pinned Pillow ICO fixtures | All 10 ICO files and all 20 frames, default selection, invalid indices and every incomplete directory prefix: 430 calls. PNG and uncompressed 24/32-bit DIB frames compare independently; palette/compressed DIB rejection remains explicit. The RGBA reader does not model legacy background-dependent XOR effects; Pillow frame agreement is not full Windows drawing parity. |
| JPEG community and upstream corpora | All 258 pinned files at the actual production cap in every mode, including large crash reproducers. Original valid/invalid/non-conformant/crash categories and independent reference warnings remain visible. Above-cap inputs receive actual rejection calls. |
| Encoders | Every valid PngSuite image, its native decoded format and BGRA conversion, all PNG levels 0–9, plus BMP/QOI/PNM. Independent Pillow and official QOI decoding check pixels; separate checks inspect PNG CRC/zlib framing, BMP/PNM framing and QOI padding. PNM intentionally drops alpha. |

The original PNG oracle also agrees with an independent stb decode. Its low-depth `tRNS` comparison happens before rescaling, as [PNG specifies](https://www.w3.org/TR/2003/REC-PNG-20031110/#11tRNS); this corrects a Pillow 10.4 oracle defect without changing expected codec behavior. Regenerated oracle hashes must match the checked-in manifest.

The standalone JPEG API requires tables within the input: abbreviated MJPEG requiring externally supplied tables is rejected, with no guessed defaults. P8 streams using 16-bit quantization tables remain a documented interoperability extension, separately counted rather than claimed as strict T.81 conformance.

JPEG comparison keeps maximum channel error 2 and mean absolute error 0.5. Warning-free supported inputs must agree. Known damaged-entropy recovery differences remain measured differences, not clean pixel passes. A reviewed classification may distinguish application-metadata labels from the core codestream or documented recovery from structural validation; it cannot suppress a crash, new reference warning, cap/parity failure or clean-reference pixel failure. See the [JPEG gate policy](../../tools/portable/image_conformance/jpeg/README.md).

The [official classic JPEG T.83 package](https://www.itu.int/rec/T-REC-T.83/en) requires paid document/test-data access and was not obtained. Community corpora do not provide its test-ID mapping or certification. JPEG2000 T.803 is a different standard. DNL regression pixels are checked against unchanged independently validated baseline/progressive scans because libjpeg-turbo explicitly lacks that feature. DNL resolves dimensions before allocation and validates marker placement. Existing entropy recovery can tolerate malformed row-count inconsistencies; it does not strictly validate MCU rows.

## Run and update

Dependencies: the source-built compiler and its LLVM/sanitizer runtime, C compiler, CMake, Node 24, and Python 3.10 or newer with the pinned Pillow oracle. The default modes are native O0 with `address,undefined`, native O2, and actual Node WASI O2. Leak detection is disabled; the compiler's `undefined` mode checks LLVM local bounds, not every C UBSan category. WASI has no sanitizer claim. WASI probes receive only explicit input, oracle and output directory mounts, without the host environment or root filesystem. Non-JPEG probes have a 600-second group timeout; JPEG probes use the documented per-case resource bounds. Compilation keeps the whole-job deadline. Cleanup covers the runner-owned process group, not descendants that create a new session.

```sh
python3 -m venv target/image-oracle
. target/image-oracle/bin/activate
python3 -m pip install --only-binary=:all: -r tools/portable/image_conformance/requirements.txt
PYTHONPATH=tools python3 tools/portable/image_conformance/run.py --jaic target/debug/jaic --output target/image-corpus-run
```

Use a fresh output directory for each run; prior receipts must remain intact. `--modes` supports diagnosis but proves only the named modes. `--expected-source SHA` requires a clean checkout at that commit. `result.json` records input/pixel pins, source/compiler identity, every case outcome and independent encoder counts; raw commands stay in the output folder. The JPEG runner has separate corpus/reference/output arguments and incremental ledgers. It does not overwrite the initial failure audit.

The manual-only `image-conformance` CI job runs the complete inventories on Linux without uploading raw corpus artifacts. Its sandbox must allow the Node installation to run before corpus execution. Existing Windows native legs run `portable_data`, including `GetIconInfo` mask extraction and actual `DrawIconEx` compositing after native and WASI production on x64 and ARM64. UI acceptance separately uses actual WebGPU renderer readback; a missing adapter is an explicit skip, never pixel acceptance.

Standalone regression programs need `.exit` expectations so Jacket discovers and runs them separately. The required-ID check executes `jpeg_dnl`, `jpeg_structure`, `jpeg_padding` and `qoi_framing` through Jacket in interpreted, sanitized native, optimized native and WASI modes; missing IDs, skipped cases and incomplete runs fail. Externally driven producers stay under `tests/driven/`, with the ICO producer explicitly invoked by the Windows SDK runner.

To extend a codec, first add a regression and the complete relevant corpus inventory. Update hashes, license/provenance records, expected case classes and direct/detected/cap checks together. Keep tolerances and reference warning baselines fixed until an independently reviewed reason supports changing them. Respect upstream redistribution terms: the runners download mixed-license corpora into their output directories rather than checking raw files into this repository.

## Equivalent-task measurements

The [JPEG benchmark](../../stdlib/Extensions/Image/bench/jpeg_bench.jai) compares complete decode, allocation and free work against pinned stb with equal requested channel counts. Its reproducible one-megapixel workload generates deterministic baseline, progressive, grayscale and restart cases through pinned native stb and `cjpeg`. Build the optimized benchmark and native providers before timing in an exclusive, quiet window; do not time alongside compilation.

```sh
JAIC_NATIVE_LIBS=target/native-libs/macos-arm64 JAIC_STDLIB=stdlib \
  target/debug/jaic -O2 stdlib/Extensions/Image/bench/jpeg_bench.jai -exe target/jpeg-bench
target/jpeg-bench 1
```

Measured on Apple M5, ARM64 macOS 27.0, with Image source `6e9d47004f7c` and the repository's default O2 stb build at `2c980bb`. Both decoders run in the same native O2 benchmark binary. `cjpeg` 3.2.0 generates the additional JPEG cases before timing. Each image is 1154×865 pixels. Each value is the median of three process minima; parentheses give the Image range across those minima. Each minimum covers 3–40 complete decode/allocation/free calls. Time excludes encoding and file I/O. A time ratio below 1 means Image completed the same decode task in less time than this stb build.

| JPEG workload | Image MP/s (range) | stb MP/s | Image time / stb |
| --- | ---: | ---: | ---: |
| baseline 4:2:0 q75 (stb) | 326.9 (325.0–327.2) | 243.8 | 0.75× |
| baseline 4:4:4 q95 (stb) | 165.2 (164.5–166.2) | 115.4 | 0.70× |
| baseline 4:2:2 q85 | 256.1 (248.1–257.2) | 187.4 | 0.73× |
| baseline 4:2:0 q75 optimized | 337.9 (328.3–338.7) | 251.2 | 0.74× |
| baseline 4:2:0 q75 restarts | 327.7 (318.8–329.9) | 245.9 | 0.75× |
| progressive 4:2:0 q75 | 142.8 (142.6–143.0) | 113.8 | 0.80× |
| progressive 4:4:4 q95 | 64.9 (64.2–65.0) | 48.9 | 0.75× |
| grayscale q85 | 476.5 (473.5–476.7) | 309.0 | 0.65× |

The [machine-readable measurement](../benchmarks/image-codecs.json) records source/compiler/library hashes and all three samples. These minima describe throughput on this workload, not a latency distribution. Repeat all cases with the same reference build and update the table and receipt together when changing the decoder or benchmark.

The existing PNG/resize microbenchmark table remains in [Image](../extensions/image.md). Neither table measures UI drawing or establishes untested competitor throughput.
