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

The filename adapter keeps native coverage; the portable codecs add interpreter/WASI execution and one allocation contract. Features listed for the alternatives are not results of this suite.

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

The original PNG oracle also agrees with an independent stb decode. Its low-depth `tRNS` comparison happens before rescaling, as [PNG specifies](https://www.w3.org/TR/2003/REC-PNG-20031110/#11tRNS); this corrects a Pillow 10.4 oracle defect without changing expected codec behavior. Regenerated oracle hashes must match `png_oracle` in the manifest.

The standalone JPEG API requires tables within the input: abbreviated MJPEG requiring externally supplied tables is rejected, with no guessed defaults. P8 streams using 16-bit quantization tables remain a documented interoperability extension, separately counted rather than claimed as strict T.81 conformance.

JPEG comparison keeps maximum channel error 2 and mean absolute error 0.5. Warning-free supported inputs must agree. Known damaged-entropy recovery differences remain measured differences, not clean pixel passes. A reviewed classification may distinguish application-metadata labels from the core codestream or documented recovery from structural validation; it cannot suppress a crash, new reference warning, cap/parity failure or clean-reference pixel failure. See the [JPEG gate policy](../../tools/portable/image_conformance/jpeg/README.md).

The [official classic JPEG T.83 package](https://www.itu.int/rec/T-REC-T.83/en) requires paid document/test-data access and was not obtained. Community corpora do not provide its test-ID mapping or certification. JPEG2000 T.803 is a different standard. DNL regression pixels are checked against unchanged independently validated baseline/progressive scans because libjpeg-turbo explicitly lacks that feature. DNL resolves dimensions before allocation and validates marker placement. Existing entropy recovery can tolerate malformed row-count inconsistencies; it does not strictly validate MCU rows.

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

[`jpeg_bench.jai`](../../stdlib/Extensions/Image/bench/jpeg_bench.jai) compares complete decode, allocation and free work against the pinned stb build with equal requested channel counts, on a deterministic one-megapixel workload (baseline, progressive, grayscale and restart cases generated through native stb and `cjpeg`). Build it and the native providers first, and time in a quiet window, never alongside a compile.

```sh
JAIC_NATIVE_LIBS=target/native-libs/macos-arm64 JAIC_STDLIB=stdlib \
  target/debug/jaic -O2 stdlib/Extensions/Image/bench/jpeg_bench.jai -exe target/jpeg-bench
target/jpeg-bench 1
```

The program prints MP/s for both decoders per workload; a time ratio below 1 means Image is faster. Results are not checked in. The PNG/resize microbenchmark table is in [Image](../extensions/image.md).
