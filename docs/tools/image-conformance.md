# Image codec coverage

`Extensions/Image` provides portable byte codecs with owned pixel buffers. The conformance tools exercise pinned public corpora, direct and detected entry points, allocation limits, and encoder output decoded independently. They do not certify every feature of an image standard.

## Scope

The portable API returns 8-bit gray, gray-alpha, RGB, RGBA or BGRA pixels. PNG input can have 1/2/4/8/16-bit samples; 16-bit samples reduce to their high byte. JPEG returns gray or RGB from 8-bit Huffman baseline, extended sequential or progressive streams, including CMYK/YCCK conversion. JPEG arithmetic, lossless, hierarchical and 12-bit processes are outside this API. ICC color management, EXIF orientation and animation playback are not applied. The default raster cap is `2^26` pixels; ICO is capped at 256x256.

The `Ico_File` filename adapter builds five ICO entries from raster input. Its native stb path and its portable WASI path (portable codecs, raster cap) accept different formats. Filesystem access is a WASI host feature, not browser path access.

## Corpora

Every file of each pinned corpus is accounted for as accepted, rejected or explicitly unsupported; nothing is silently omitted.

| Corpus | What is checked |
| --- | --- |
| Original PngSuite | Valid files compare with pinned independent RGBA oracles; invalid files are rejected. The HTTPS mirror is pinned and its PNG bytes match the original archive. |
| BMP Suite | Mandatory good cases compare with the author's PNG references. Questionable, bad and related-format cases execute too; the author's rule requires safe handling, not universal rejection. |
| `imazen/codec-corpus` PNM | Supported P2/P3/P5/P6, malformed cases, and explicit PBM/PAM rejections (outside the declared API). |
| Official QOI reference | Reference-encoded cases, exhaustive opcode-range cases, reference-rejected negatives, and normative framing negatives the permissive reference accepts. |
| Pinned Pillow ICO fixtures | Every frame, default selection, invalid indices and every incomplete directory prefix. PNG and uncompressed 24/32-bit DIB frames compare independently; palette and compressed DIB rejection is explicit. The RGBA reader does not model legacy background-dependent XOR effects. |
| JPEG community and upstream corpora | Every pinned file at the production cap in every mode, including crash reproducers; above-cap inputs get actual rejection calls. |
| Encoders | Every valid PngSuite image, its native decoded format and BGRA conversion, all PNG levels 0-9, plus BMP/QOI/PNM. Independent Pillow and official QOI decoding check pixels; separate checks inspect PNG CRC/zlib framing, BMP/PNM framing and QOI padding. PNM drops alpha. |

The PNG oracle also agrees with an independent stb decode. Its low-depth `tRNS` comparison happens before rescaling, as [PNG specifies](https://www.w3.org/TR/2003/REC-PNG-20031110/#11tRNS). Regenerated oracle hashes must match `png_oracle` in the manifest.

The standalone JPEG API requires tables within the input: abbreviated MJPEG that needs external tables is rejected. P8 streams using 16-bit quantization tables are a documented extension, counted separately.

JPEG comparison against the libjpeg-turbo reference (built from pinned source) allows a maximum channel error of 2 and a mean absolute error of 0.5. Warning-free supported inputs must agree. Known damaged-entropy recovery differences are reported as differences, not clean passes. A reviewed classification may separate application-metadata labels from the core codestream, or documented recovery from structural validation; it cannot suppress a crash, a new reference warning, a cap/parity failure or a clean-reference pixel failure. See the [JPEG gate policy](../../tools/portable/image_conformance/jpeg/README.md).

The official classic JPEG T.83 test package is not part of the corpora. DNL regression pixels are checked against unchanged independently validated baseline/progressive scans, because libjpeg-turbo does not support DNL. DNL resolves dimensions before allocation and validates marker placement; entropy recovery can still tolerate malformed row-count inconsistencies and does not strictly validate MCU rows.

## Run and update

Needs the source-built compiler with its LLVM/sanitizer runtime, a C compiler, CMake, Node 24 and Python 3.10+ with the pinned Pillow oracle. The default modes are native O0 with `address,undefined`, native O2 and Node WASI O2. Leak detection is off, `undefined` checks LLVM local bounds only, and WASI has no sanitizer claim. WASI probes see only explicit input, oracle and output mounts. Non-JPEG probes time out after 600 seconds; JPEG probes use per-case CPU, wall and file-size limits. Timeouts kill the runner's process group, not descendants that start a new session.

```sh
python3 -m venv target/image-oracle
. target/image-oracle/bin/activate
python3 -m pip install --only-binary=:all: -r tools/portable/image_conformance/requirements.txt
PYTHONPATH=tools python3 tools/portable/image_conformance/run.py --jaic target/debug/jaic --output target/image-corpus-run
```

Use a fresh output directory per run; `result.json` from an earlier run is never overwritten. `--modes` is for diagnosis and proves only the modes it names. `--expected-source SHA` requires a clean checkout at that commit. `result.json` records source and compiler identity, input and oracle pins, every case outcome and the independent encoder counts. The JPEG runner takes its corpus, reference and output folders as arguments; see its [README](../../tools/portable/image_conformance/jpeg/README.md).

The manual-only `image-conformance` CI job runs everything on Linux and uploads no corpus files. Windows native legs run `portable_data`, including `GetIconInfo` mask extraction and `DrawIconEx` compositing on x64 and ARM64. UI acceptance uses WebGPU renderer readback; a missing adapter is a skip, never a pass.

Standalone regression programs need `.exit` expectations so Jacket runs them separately. `check_registration.py` runs `jpeg_dnl`, `jpeg_structure`, `jpeg_padding`, `qoi_framing` and `Ico_File::gdi` through Jacket in interpreted, sanitized native, optimized native and WASI modes; a missing ID or skipped case fails.

## Pins

Each corpus is pinned in `manifest.json` (`jpeg/manifest.json` for JPEG) as repository, revision and the SHA-256 of every file; the BMP Suite is one pinned zip. The case lists are derived from the downloaded files: PngSuite files starting with `x` are invalid, sizes come from IHDR, and JPEG classes come from the corpus folders. Only facts the files cannot show are stored: the independent PNG oracle hashes, JPEG expectation exceptions, unsupported processes and reference baselines. Downloads go through `toolkit.download` and are checked before use.

To extend a codec, add a regression and the full corpus first, then update the pins, license notes and expectations together. Keep tolerances and reference baselines fixed unless an independent review supports a change. Mixed-license corpora are downloaded into the output folder and never checked in.

## JPEG decode benchmark

[`jpeg_bench.jai`](../../stdlib/Extensions/Image/benches/jpeg_bench.jai) compares complete decode, allocation and free work against the pinned stb build with equal requested channel counts, on deterministic one- and twelve-megapixel workloads (baseline, progressive, grayscale and restart cases generated through native stb and `cjpeg`). Build the native providers first, and time in a quiet window, never alongside a compile.

```sh
JAIC_NATIVE_LIBS=target/native-libs/macos-arm64 JAIC_STDLIB=stdlib \
  jacket bench --path stdlib/Extensions/Image jpeg_
```

Each decoder/input pair is a separate `@bench`, reporting time and compressed MB/s. Filter cases by name to select a shape. Results are not checked in. The PNG/resize microbenchmark table is in [Image](../extensions/image.md).
