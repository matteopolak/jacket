# JPEG independent corpus gate

This runner verifies the complete pinned 258-file JPEG inventory against a chosen source receipt in native ASan/bounds, native O2 and WASI O2 configurations. It preserves strict suite failures separately from damaged-data recovery, unsupported process rejection and owner-reviewed suite classifications. T.83 paid compliance data is unavailable; this is not official certification.

Use `PYTHONPATH="$REPO/tools"` for these repository helpers. Run `python3 check.py` for inventory and policy invariants and `python3 test_inventory.py` for the derivation, without any compiler/decoder execution.

## Fetch and execute

`fetch.py` downloads all 296 pinned source files: JPEG cases, original attribution, licenses and other files in the two complete source directories. Existing files must match their exact hashes; a mismatch fails rather than overwriting. No third-party code or binaries from the corpus run. `manifest.json` pins each repository, commit and the SHA-256 of every file; the URLs and the 258-case list are derived from those pins (see `inventory.py`), and a JPEG that is present but unpinned fails the fetch.

```sh
python3 fetch.py --output "$CORPUS"
python3 run.py \
  --repo "$REPO" --jaic "$JAIC" --corpus "$CORPUS" --output "$NEW_RESULT_DIR" \
  --reference-include "$LIBJPEG_INCLUDE" --reference-library "$LIBJPEG_FILE" \
  --expected-source "$REVIEWED_SHA" --expected-jaic-sha256 "$JAIC_SHA256"
```

Use the reviewed exact candidate receipt for final execution. The optional `--expected-source` HEAD guard supports local uncommitted harness adoption; the runner always records dirty status and hashes every Image source file. CI can pass its actual checked-out Git SHA. Existing output directories are refused so older failures cannot disappear.

Local orchestration may pass `--heavy "$ADMISSION_SCRIPT"`, or wrap the command externally once. The option is unnecessary in CI, where concurrency is controlled by the workflow. Do not do both or nest admission locks. Nothing depends on an overnight coordination path. `--cc`, `--node`, include/library paths, source/compiler receipts and result paths are CLI parameters. There are no machine-specific paths in this harness.

The reference is built through the installed public libjpeg API; the expected libjpeg-turbo version defaults to the original oracle's 3.2.0. The runner records reference header/library hashes and refuses a version mismatch. Deliberate reference upgrades need reviewed warning/behavior baselines; changing a version flag does not update per-case expected reference outcomes. Link a matching library/header installation. `--reference-library` accepts an actual library file or a library directory; prefer the exact file and `--expected-reference-library-sha256` for an immutable oracle receipt. The displayed version is the public API/header version. The reference also reports its actual loaded library with `dladdr`; the runner requires that resolved file to match the configured library set and records its hash. Explicit runtime routing prevents accidentally using an older system SONAME. Native ASan requires the compiler's compatible sanitizer linker/runtime setup, as documented by the repository.

## Policy and modes

Every mode uses `Image.DEFAULT_MAX_PIXELS`, checks its actual emitted value against the manifest's `production_default_pixels`, and calls both `decode_jpeg` and `decode_image` on every input. Successful pixels must agree between entry points, dimensions/count must be valid, exact pixel caps must accept with identical pixels, and cap-minus-one calls must reject without pixels. Rejections must agree and return empty pixels. Error strings can differ for a non-JPEG input rejected by format detection.

All four formerly cap-limited inputs receive real calls in all three modes. The two valid-size 69M crash fixtures must reject at the actual default cap; two malformed extreme-dimension files may reject structurally earlier. None is counted as a successful full decode. Every input hash must match the pinned inventory. A focused `--modes` run is explicitly `PARTIAL`; only all three complete inventories can pass the final gate.

Independent pixel comparison uses accurate integer IDCT (`JDCT_ISLOW`) and fancy upsampling, with unchanged maximum channel error2 and mean absolute error0.5. CMYK output uses public `JCS_CMYK` samples, converted by the previously fixed unprofiled policy: Adobe inversion selects `C*K`, otherwise ordinary `(255-C)*(255-K)`, rounded nearest. ICC color management is outside the core pixel API. Reference warnings, exit status and diagnostics are compared with the original pinned oracle per input; new changes fail. Warning-free supported images require matching output. Known damaged-data differences remain explicit review items, and cannot silently become clean pixel passes.

The native ASan mode uses `-O0 -sanitize address,undefined`; native/WASI use O2. The repository's `undefined` instrumentation checks LLVM local bounds, not all C-style UBSan categories. Existing leak-check policy is `detect_leaks=0`; this harness makes no leak-safety claim. Address and bounds diagnostics remain fatal. WASI is separately executed, with no sanitizer claim.

The bounded `wasi_probe.mjs` exposes only `/input` (the corpus) and `/output` (the working folder), rather than preopening host `/`. POSIX hosts use the repository's `tools/bench/sandbox.py`: network denied, input corpus read only, writes limited to the result working directory. Compiler/build inputs are the explicitly reviewed repository and compiler. Decoder processes have CPU, wall-clock and output-file bounds; a timeout or signal is a process error, never a passed rejection. Defaults permit within-production-cap outputs (400MB file cap,180seconds CPU,240seconds wall per input). These are resource bounds, not benchmark settings or performance claims. Windows is an explicit unsupported harness-host error because the native ASan mode is unavailable; it is not silently skipped.

## Expected results and reviewed exceptions

A case's class is its corpus folder (`valid`, `invalid`, `non-conformant`, `crash-repro`, or `upstream-reference` for libjpeg-turbo's own images), and its expectation is the class default unless `expectations` in the manifest lists an exception. Valid supported files require acceptance; documented arithmetic/12-bit/etc negatives require rejection. Invalid-class files require rejection unless an exact-input-hash, reasoned owner/root disposition is provided. Non-conformant and crash reproducers retain both decode and independent reference evidence. In particular, partial/progressive/MJPEG labels may conflict with reference warnings or application framing assumptions; those remain reviewable findings.

`--reviewed-dispositions` accepts an explicit JSON document with `reviewed_by`, `review_receipt` and `cases`. Every entry requires `id`, exact `sha256`, `reason` and an expectation from `reject`, `accept`, `metadata-outside-core`, `documented-recovery`, `suite-label-inconsistency`, or `retained-compatibility-extension`. The reviewer must approve the classification before execution; a populated field is not itself proof of approval. Do not generate exceptions merely to turn the gate green. The checked-in reviewed classifications bind exact input hashes and reasons to their review receipt. Unresolved inputs retain their original strict expectations; this file is not a generic format or safety waiver.

Reviewed invalid-class acceptances are still reported as acceptances, never strict rejection passes. `documented-recovery` can dispose of known warning-bearing recovery pixel differences while retaining the measured differences. No disposition suppresses crashes, sanitizer errors, entry-point parity, cap behavior, new reference warnings or clean-reference pixel failures. Retained P8/Q16 compatibility is reported separately from strict T.81 conformance; it does not establish a standard-profile pass. All surviving strict policy failures remain failures.

## Evidence, adoption and remaining gaps

`receipt.json` pins source/compiler/harness/reference identities and bounds. `builds.json` retains each compile outcome. `cases.json` retains all input/mode outcomes and pixels, including process stderr and reference diagnostics; it is written incrementally. `summary.json` reports per-mode acceptance, rejection, failure/review counts, strict invalid rejections, reviewed invalid acceptances, process/sanitizer errors and damaged recovery mismatches. Inventory equality checks use case ID plus hash, not counts alone. Large temporary raw outputs are removed after comparison; binaries and compact ledgers remain for review.

The independent files have mixed MIT, IJG/BSD, AGPL, CC0 and incompletely enumerated upstream licenses. Each source set in the manifest carries its license note, including the per-file gaps; do not assume a uniform permissive license or redistribute unknown-attribution crash files. Fetch from their primary pinned repositories for private/CI testing, retain license documents, and review artifact contents before publication. Do not upload the raw mixed-license corpus as a project artifact by default.

T.83 document/data access, DNL positive-oracle support, generic component/profile applicability and above-default full decoding remain explicit gaps. The independent corpus is comprehensive for its pinned inventory, not an exhaustive official test-ID mapping. The frozen audit and its failure ledgers live outside this runner and must not be overwritten. Run against reviewed source, inspect all build diagnostics, and require the complete mode matrix before final acceptance. A launch error is an unverified decision, never a decoder rejection.

To extend coverage, add the complete new upstream source set (repository, revision, every file's hash and license note) and any expectation provenance, then review the inventory delta. Keep comparison tolerances fixed. Fix decoder causes or obtain evidence-backed scope decisions; do not filter inputs or alter reference warning baselines after a failure to make results pass.

## Reproduce the reference

`build_reference.py --output NEWDIR` fetches the independently pinned 3.2.0 implementation archive, verifies its hash/version, builds the shared libjpeg API with at most two workers, runs every configured upstream CTest, and installs into `NEWDIR/install`. The newer upstream test-image corpus revision is deliberately separate from this implementation pin. Pass its `install/include` and `install/lib` to the gate. `--archive FILE` can reuse identical locally cached bytes; mismatched bytes fail. Existing output directories are never overwritten.

The non-conformant `multiple-0xff-before-eoi.jpg` case has seven zero entropy alignment bits before its legal FF marker fill. Its required-rejection expectation records the independent full-Huffman-walk provenance in `expectation_provenance` in the manifest. The padding-only correction preserves all decoded coefficients and pixels with fill present or removed; marker fill remains a supported positive case in the generated regression.
