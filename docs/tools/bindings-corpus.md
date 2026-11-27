# Bindings corpus

Pinned C libraries that `Bindings_Generator` binds, checked struct by struct against clang at each target triple. It catches generator layout bugs that the hand-written C tests do not reach: field offsets, sizes and alignments on 64-bit, 32-bit, 16-bit and bare-metal targets.

## How it works

`tools/corpus/bindings_corpus.py` runs every library in `corpus/bindings/libraries.json` against every triple in the same file:

1. Downloads the archive with `toolkit.download.pinned`, which checks its SHA-256. The tree is extracted once under `~/.cache/jaic/bindings-corpus/trees/`.
2. For each listed header, runs the generator (`tools/corpus/bindings-corpus/generate.jai`) with that header as its source. The output holds `#assert(size_of(R) == N)`, `align_of` and `offset_of` lines.
3. Parses the same header with clang (`-ast-dump=json`, same triple and include paths) to learn each record's C name (the dump goes to a file and is read node by node, keeping only records and typedefs, so its size does not bound memory), then compiles a probe that includes the header and holds `sizeof`, `_Alignof` and `offsetof` of every record and field the generator asserts on. The numbers are read from the probe's LLVM IR.
4. Compares every record and field both sides name. Any difference is a `MISMATCH` line.

Each library is one row, each triple one column. A cell is `ok`, `MISMATCH`, `ERROR` (generation or the oracle failed) or `skip` (the manifest gives a reason).

```bash
JAIC=/path/to/target/debug/jaic PYTHONPATH=tools python3 tools/corpus/bindings_corpus.py
JAIC=... PYTHONPATH=tools python3 tools/corpus/bindings_corpus.py --library lwip --triple avr --json report.json
```

Exit status is 1 on any `MISMATCH` or `ERROR`. Skips do not fail the run.

### One source per header

Each listed header is generated and parsed on its own, so a failure names its header. Declarations of the headers it includes are bound too, unless a header marks itself a system header (`#pragma clang system_header`, which CMSIS uses): the generator ignores such included headers, but never a header it was given as a source. Headers that need an earlier one (`task.h` needs `FreeRTOS.h`) name it in `prelude`, which becomes a `-include`.

### Matching records

A record with a C tag is named `struct tag` (or `union tag`). A typedef'd anonymous record is named by its typedef, which the AST links to the record. Bitfields and anonymous members have no `offsetof` and are skipped. A generated record clang has no complete definition for is unmatched, counted in the report and not checked.

The probe is compiled instead of dumping layouts with `-fdump-record-layouts-complete` because the dump runs at the record's closing brace. A trailing `__attribute__((packed))` (lwIP writes its headers that way) applies after that, so the dump shows an unpacked record the compiler does not use.

### Sandbox

The headers are third-party code, so the generator and the clang oracle run under `tools/corpus/upstream_sandbox.py` (no network; reads limited to the libc shim, config, tree and stdlib; writes limited to a scratch directory). Each command starts in that scratch directory, because the repository as working directory is not readable inside the sandbox. Outside CI, a platform without a sandbox refuses to run unless `--no-sandbox` is given.

## Manifest

`corpus/bindings/libraries.json`:

```json
{"id": "littlefs", "url": "https://github.com/littlefs-project/littlefs/archive/refs/tags/v2.9.3.tar.gz",
 "sha256": "9cf2e7db...", "root": "littlefs-2.9.3", "headers": ["lfs.h"], "include": ["."], "triples": {}}
```

- `sha256`: digest of the archive at `url`. A new pin is the digest the download prints, after checking it is the release's own archive.
- `root`: the archive's single top-level directory.
- `headers` and `include`: the sources, and the include directories relative to `root`.
- `config`: a directory under `corpus/bindings/config/` with the configuration headers the library needs (`FreeRTOSConfig.h`, `lwipopts.h`, `tusb_config.h`, `lv_conf.h`, `ffconf.h`, the CMSIS device header).
- `prelude`: headers force-included before each listed one.
- `clang`: extra flags for every triple of the library.
- `triples`: per triple, either `{"skip": "reason"}` or `include` / `clang` additions. A skip needs a reason; the driver rejects a skip without one.

Top level: `triples` maps each triple to its generator `os` (`linux` or `none`) and clang flags (`-ffreestanding` everywhere, `-mmcu=atmega328p` on AVR). `libc` is `corpus/bindings/libc/`.

## The libc shim

Targets without a sysroot (macOS hosts, bare metal) have no C library headers. `corpus/bindings/libc/` holds declaration-only headers for the parts the libraries name (`string.h`, `stdio.h`, `stdlib.h`, `ctype.h`, `assert.h`, `limits.h`, ...). `-ffreestanding` makes clang take `stdint.h`, `stddef.h` and `stdbool.h` from its own resource directory. The generator and the oracle use the same shim, so both see the same typedef widths. The comparison checks the generator, not glibc.

## Results

Recorded with the driver at this commit (see the manifest for the pins):

| Library | x86_64 | i686 | thumbv7em | avr | Notes |
|---|---|---|---|---|---|
| littlefs | ok | ok | ok | ok | |
| FatFs R0.15 | ok | ok | ok | ok | |
| FreeRTOS 11.1.0 | ok | ok | ok | ok | Posix port on x86, ARM_CM4F, ATMega323 |
| mbedTLS 3.6.2 | ok | ok | ok | ok | |
| nanopb 0.4.9.1 | ok | ok | ok | ok | |
| TinyUSB 0.17.0 | ok | ok | ok | ok | |
| CMSIS-Core 5.9.0 | skip | skip | ok | skip | Arm-only; see the manifest reasons |
| lwIP 2.2.1 | ok | ok | ok | ok | |
| LVGL 9.2.2 | ok | ok | ok | ok | |


## Change it

- New library: add an entry; put its config headers in `corpus/bindings/config/<id>/`. Run the driver with `--library <id>` and fix the first `ERROR` line it prints.
- New triple: add it to the top-level `triples` with its `os` and flags, then give each library a row (a run or a reason to skip).
- Changing the generator: rerun the driver. A regression shows as a new `MISMATCH` or `ERROR`.
- Tests: `PYTHONPATH=tools python3 -m unittest corpus.test_bindings_corpus` covers the manifest checks, the parsers and the comparison. It needs no network and no jaic. One test compiles a packed record with `clang` from `PATH`: a local run skips it without clang, and with `CI` set it fails.

## Configuration

| Name | Meaning |
|---|---|
| `JAIC` | The jaic that builds `generate.jai`. Use a build of this checkout (`cargo build -p jaic-cli --bin jaic`); the PATH `jaic` may be an older release. |
| `clang` on PATH | The oracle. Use the LLVM that the generator loads (`llvm-config`'s prefix). |
| `--no-sandbox` | Run the upstream headers without the sandbox. Not for CI. |
| `--json PATH` | Write every cell's result, including unmatched records and mismatches. |

The generator also needs the Bindings_Generator helper library, built by `tools/developer_checks/build_generator_platform.py` (see [development](development.md)).

## Dependencies

- `toolkit.download` (pinned downloads, cache), `corpus.upstream_sandbox` (sandboxing), `toolkit.repo` (paths).
- `stdlib/Bindings_Generator` through `generate.jai`.
- LLVM with clang (generator and oracle).
- Upstream archives at the URLs in the manifest; nothing else is fetched.
