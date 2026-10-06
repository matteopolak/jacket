# Upstream corpus

## What it is

Pinned snapshots of open-source Jai projects that `jaic` is measured against: applications (Focus, Jails, The Way to Jai, open-jai, Vk-Engine, sgpu, chess-jai, forbear, ...), their dependencies (jai_parser, Linalg, Jolt-Jai, Tracy's client sources) and libraries (metaprogramming libraries such as match-jai, yield-jai and AST_Utils, and rluba's family: jaison, uniform, stubborn, jai-tracy, hyperserve, cluster, ...; and libraries with their own test suites: reflector, jai-format, jai-protobuf, toml-jai and jai-xml). The full lists are `REPOSITORIES`, `DEPENDENCIES` and `LIBRARIES` in `tools/fetch_upstreams.py`.

The files live in the gitignored `corpus/upstream/`. The committed `corpus/upstreams.json` records exact commits and file hashes.

## How it works

`tools/fetch_upstreams.py` downloads the pinned `.jai` files, READMEs, licenses, C-family sources (`NATIVE_SOURCE_SUFFIXES`, for native-library builds) and compile-time data files (`RESOURCE_PREFIXES`, like focus's `config/` and `fonts/`) into `corpus/upstream/<owner>--<repo>/`. It never builds or runs anything from a project.

- Existing pins are kept. `--since YYYY-MM-DD` is the recency cutoff for newly added repositories; `DEPENDENCIES` (submodules) and `LIBRARIES` (which rarely change) are exempt.
- `MODULE_LINKS` symlinks dependencies into the consumers' `modules/` directories, and libraries imported by name into `corpus/upstream/_modules/` (cases pass `-I ../../_modules`). rluba's libraries import each other this way (`uniform`, `wait_group`, `cluster`, `date`, `stubborn`, `tracy`, `hyperserve`), as does `AST_Utils`.
- `SOURCE_PREFIXES` limits a repository to some directories (Tracy: only `public/`, the client library).
- Dependencies are pinned like projects: jai_parser for Jails; Linalg, Jolt-Jai and its JoltC submodule for Vk-Engine (JoltC has no Jai files, so the fetcher takes its `CMakeLists.txt` and `Examples/`); Tracy at jai-tracy's submodule commit, linked as `rluba--jai-tracy/tracy`.
- `corpus/upstream` and the Git cache live in the main checkout (found through the Git common dir), so worktrees share them.

`tools/verify_upstreams.py` re-hashes the tree against the manifest and rejects modified or missing files. Unlisted files are reported, not rejected, because building the projects (sweep `build` cases, `build_native_libs.py`, `build_vk_engine_libs.py`) leaves libraries, executables and generated files next to the sources.

Entry points that work are sweep cases in `tools/upstream-cases.json`, so [jaic-sweep](jaic-sweep.md) keeps them working.

### Recorded outputs

A case that only has to exit 0 says little about whether jaic computes the right thing. Where a project
documents what its programs print, the case in `tools/upstream-cases.json` also carries an `expect` record that
[jaic-sweep](jaic-sweep.md) checks:

```json
{"id": "ttwj-16-16.3-enum-specified", "path": "Ivo-Balbaert--The_Way_to_Jai/examples/16/16.3_enum_specified.jai",
 "mode": "run",
 "expect": {"stdout_ordered": ["enum 'Direction' is", "*NOT* specified.", "specified"],
            "source": "The_Way_to_Jai `// =>` annotations: examples/16/16.3_enum_specified.jai lines 20, 22, 26"}}
```

Values must come from the project itself: its docs, test files or golden outputs, written by people who ran
Jai. Never copy jaic's output into an expectation; that only freezes current behaviour.
`source` names the file and lines.

**The_Way_to_Jai.** The book writes the output next to the printing line as `// => text`.
`tools/upstream_expectations.py --write` turns these into `stdout_ordered` records: 164 cases with 459 lines.
`stdout_ordered` checks that each line appears in order, so output without an annotation does not matter.
The tool skips:

- addresses, timings and lines that describe a diagnostic rather than stdout;
- `REJECTED`, 63 annotations that were reviewed by hand, each with a reason. Most are typos or commentary.
  Eight are stale: written for an older Jai. For example, 15.8 assumes `for < a..b` counted down before
  beta 0.1.094.

`--verify JAIC` runs the annotated cases and lists annotations missing from stdout. Treat each one as a jaic
bug until proven otherwise:

- `enum_type_flags` lacked `#specified`/`#complete` (16.3; `tests/corpus/positive/enum-type-flags.jai`);
- a null `Type` printed as `<null type>` instead of `(null)` (26.31; `tests/corpus/positive/print-null-type.jai`).

Reject an annotation only when the book is wrong about current Jai, and say why.

**Not adopted: open-jai.** open-jai ships per-example expected outputs, but it produced them with its own
reimplementation. They are not evidence about the real compiler. One even contradicts TTWJ 14.2 and how_to 025
(the implicit-then rule of `ifx`).

**Libraries with self-checking tests.** The `reflector-tests` case builds reflector's unotest suite and runs it:
2 compile-time tests and 10 runtime tests (binary, Glowmade, Flatbuffers and JSON reflectors). The case expects
the suite's own `ALL PASSED.` and exit code 0. The project's `build.jai` also builds a Windows-only benchmark
(`#foreign kernel32`), so the case copies `tools/upstream-drivers/reflector-tests.jai` into a scratch copy of the
project. That driver makes the same test workspace calls. The suite found these jaic bugs, all fixed with tests:

- a tagged-union tag with a default (`union tag := Kind.A { ... }`): `tests/stdlib/tagged-union-tag-default.jai`;
- `for < :it_name`: `tests/stdlib/for-reverse-named-iterator.jai`;
- `#assert` with format arguments: `tests/corpus/negative/assert-format-arguments.jai`;
- a bare polymorphic struct parameter (`r: *BinaryReflector`) given a struct that `#as`-uses an instance. The body
  now sees the argument's own type, and the match costs an `#as` conversion:
  `tests/stdlib/bare-poly-struct-derived-argument.jai`;
- `#if R.FLAG` on a macro's constant `Type` argument;
- a parameter type that names a later parameter (`info: *r.Info, r: *$R`);
- a macro overload taking `Code`, which stopped the other overloads from binding `$T`;
- `.FLAG & x.flags` inside `cast(int)`: the inferred member took `int`, not the flags type;
- `compiler_get_struct_location` (it used to be unsupported): `tests/stdlib/compiler-struct-location.jai`.

reflector's readers call `Reset( d, count, initialized = false )` on arrays of structs, then
resize each element's `[] T` view. Those views are uninitialised memory, so the second resize reads a
garbage pointer. The official `array_resize` on a view `realloc`s the old data and its default
allocator doesn't zero memory either, so this is undefined behaviour in reflector itself; it only
works when the allocator happens to hand out fresh pages. The case's `setup` patches the scratch copy
to initialise those arrays (`Reset( d, count )`). Without it, `TestNestedType` crashed about one run
in five.

toml-jai's examples (run by its `tests.jai`) found three more, also fixed:

- integer and type tags on tagged unions (`4,, a: u8;`, `u16,, a: s8;`);
- `ifx c then x = 1 else x = 2;` as a statement;
- `ok=, p.* = f();`.

All three are in `tests/stdlib/tagged-union-constant-tags.jai`. The examples still stop on
`ok:, x.y = f();`, which declares one name and assigns to another place.

### Project status

| Project | Status | Notes |
| --- | --- | --- |
| [Focus](https://github.com/focus-editor/focus) | works | Builds and runs natively on macOS |
| [Jails](https://github.com/SogoCZE/Jails) | works | Builds a native language server |
| [jaison](https://github.com/rluba/jaison) | works | Tests and examples run, also natively |
| [uniform](https://github.com/rluba/uniform), [stubborn](https://github.com/rluba/stubborn) | works | uniform's stubborn test suite runs at compile time |
| [jai-date](https://github.com/rluba/jai-date), [wait_group](https://github.com/rluba/wait_group) | works | Self-tests and example run |
| [jai-csv](https://github.com/rluba/jai-csv) | works | Checks; no tests upstream |
| [cluster](https://github.com/rluba/cluster), [hyperserve](https://github.com/rluba/hyperserve) | works | Build natively and serve |
| [jai-redis](https://github.com/rluba/jai-redis) | works | Test builds; running needs a Redis server |
| [jai-postgres](https://github.com/rluba/jai-postgres) | partial | Checks; running needs libpq and a database |
| [jai-tracy](https://github.com/rluba/jai-tracy) | works | `-plug tracy` instruments and builds a profiled program |
| [sgpu](https://github.com/roeyb1/sgpu) | works | Examples build on macOS; mesh shaders need a driver MoltenVK lacks |
| [The Way to Jai](https://github.com/Ivo-Balbaert/The_Way_to_Jai) | works | Most programs run; the rest check |
| [Vk-Engine](https://github.com/ostef/Vk-Engine) | partial | Checks for Linux; no macOS support upstream |
| [chess-jai](https://github.com/danieltan1517/chess-jai) | works | UI and engine build natively; perft suite passes |
| [forbear](https://github.com/gabrielmfern/forbear) | works | Builds natively; the playground app runs |
| [rexim.github.io](https://github.com/rexim/rexim.github.io) | works | `rss.jai` runs |
| [ui_builder](https://github.com/kooparse/ui_builder) | partial | Demo checks; linking needs an unpinned prebuilt `libslang` |
| [Photon](https://github.com/DavidColson/Photon) | partial | Windows-only |
| [KodaJai](https://github.com/kujukuju/KodaJai) | partial | Needs the author's unpinned modules |
| [no_api](https://github.com/UnNabbo/no_api) | partial | Entry point loads a file missing upstream; Windows/Linux only |
| [reflector](https://github.com/n00bmind/reflector) | works | Its unotest suite builds natively and passes (`reflector-tests`) |
| [jai-format](https://github.com/OrangeLightning219/jai-format) | partial | Builds a `File` from the C `stdin` (`*FILE`); jaic's `File.handle` is an `s64` descriptor |
| [toml-jai](https://github.com/sjorsdonkers/toml-jai) | partial | Examples stop at `ok:, x.y = f();` (declare and assign in one list) |
| [jai-xml](https://github.com/smari/jai-xml) | partial | `test.jai` and the examples stop at `#if must` on a `$$must: bool = false` parameter called without that argument (the default should be baked) |
| [jai-protobuf](https://github.com/segcore/jai-protobuf) | partial | Pinned with its `.proto` inputs; its tests write generated code into the tree and do not pass yet |

### Notes per project

**Focus.** `jaic build first.jai` produces a working editor on macOS. It needs the stb libraries (`tools/build_native_libs.py`) and its own `modules/Objective_C/LightweightRenderingView/build.jai` run once. Debug builds need `~/Library/Application Support/dev.focus-editor` to exist, because upstream creates `.../debug` non-recursively.

**Jails.** `jaic build build.jai` produces a native `bin/jails` that answers LSP requests. `-os windows` needs a Windows host (it calls `MultiByteToWideChar` at compile time).

**Vk-Engine** (with Linalg and Jolt-Jai). The sweep cases `vk-engine-{core,renderer,game,editor}-check` run

```sh
jaic check Build.jai -I Modules -I Source -os linux - Core|Renderer|Game|Editor
```

on a scratch copy with empty `Libs/Linux` placeholders, as on a Linux machine whose libraries are built, so `Build.jai` doesn't regenerate bindings inside the corpus. Use a release `jaic`; each module takes tens of seconds. Without the placeholders `Build.jai` runs three generators: ImGui's works, Jolt's needs `cmake`, and Vulkan's stops with "expected *Declaration, found Enumerate" at `Modules/Vulkan/generate.jai:240`. That generator (like sgpu's `Vulkan_With_VMA/generate.jai`) targets an older `Bindings_Generator` where `Enum.enumerates` held declarations; the current API rejects it in Jai too. Natively on macOS it stops early: the upstream `Vulkan`, `ImGui` and `JoltPhysics` modules have no macOS branch. `tools/build_vk_engine_libs.py` builds the C++ libraries for macOS; see [Vk-Engine](../native/vk-engine.md).

**sgpu.** All examples check (host, Linux, Windows) and build natively on macOS after `tools/build_slang.py`. With MoltenVK all run except `04_mesh_shaders` (no `VK_EXT_mesh_shader`). Commands: [native libraries](native-libs.md#slang-sgpu).

**The Way to Jai.** Every entry point is a sweep case: most run to completion, the rest check (windowed Simp programs, interactive or endless ones, deliberate crashes, user-built libraries). The programs that fail `check` are not compiler bugs:

- Windows-only APIs (19.8, 33.2C, 33.6, 50.1) and the Windows-only raylib module (35.1, 52.2, 30/jai_raylib, and the `raylib/module.jai` files of 35 and 52, whose `raylib_native` library is declared only for Windows);
- files meant to be `#load`ed, not run on their own (8B `file_alpha.jai`, `file_beta.jai`), and 31 `build_gui.jai`, which uses an undeclared `success` in a procedure nothing calls (Jai checks the program's own procedures; see [dead-code elimination](../language/dead-code-elimination.md));
- intentional failures (20.2, 30.9, exercises/22);
- APIs that older Jai versions had (6.6 `random_seed` result, 26.27 and exercises/30 `builder_to_string(allocator=)`, 33.10 `Sound_Player` struct, 51.2 GetRect `dropdown`);
- missing command-line arguments (30.14, 8.2, 12.8) or a missing `cpp_library.cpp`;
- 31.2, which calls GL at compile time without a context.

19.5 frees an advanced pointer and 27/foldera writes through null; both are upstream bugs. 10.4 reads freed memory, so its third line is not checked.

**Metaprogramming libraries.** Sweep cases: match-jai `examples/first.jai`; yield-jai `constant`, `defer`, `first`, `if`, `if_case`, `while` (`expand` and `for` would need per-call `macro_expansion_block` export); AST_Utils `astTests.jai`; Jai-Shader-Transpiler `build.jai` (GLSL from `@glsl` procedures); jai-utils `closure.jai`; unotest. Checked by hand but not cases:

- AST_Utils `examples/build.jai` sets `FormatStruct.recursive_long_form_depth`, which current Basic does not have, in a procedure of its own files, so it fails to check.
- epic-fail is a `-plug` plugin ([metaprogram plugins](../metaprogramming/metaprogram-plugins.md)); its `assert` works when imported directly.
- MetaThreadSafe's examples fail on purpose. The diagnostics match, except that jaic still checks the untaken `#if` branch of a baked instance.
- jai-control-flow uses `%%` as an escaped percent, which current Jai reads as two arguments; a corrected copy passes all its tests.

**rluba's libraries.** All compile; what runs depends on the services they talk to.

| Library | What it is | Sweep cases |
| --- | --- | --- |
| jaison | JSON parse/print, typed and generic | `jaison-tests`, `jaison-example`, `jaison-native-build` |
| uniform | RE2-style regular expressions | `uniform-tests` (`jaic run first.jai - test`, the stubborn suite at compile time) |
| stubborn | Compile-time test runner and matchers | `stubborn-module`, `uniform-tests` |
| jai-date | Date parsing, formatting, arithmetic | `jai-date-module` (`#run` self-tests) |
| jai-csv | CSV parsing into typed arrays | `jai-csv-module` |
| wait_group | kqueue/epoll event loop | `wait-group-example` |
| cluster | Process clustering with a shared listen socket | `cluster-build`, `cluster-crashing-example` |
| hyperserve | HTTP server framework | `hyperserve-example-build`, `hyperserve-datastar-build` |
| jai-redis | Redis (RESP3) client | `jai-redis-test-build` |
| jai-postgres | libpq bindings and typed queries | `jai-postgres-module`, `jai-postgres-pgvector` |
| jai-tracy | Tracy bindings and instrumenting plugin | `jai-tracy-plugin-check`, `jai-tracy-plugin-build` |

- cluster: `cluster -n 2 -- crashing` starts, watches and reaps instances. This relies on `Process` giving children a socket as stdin.
- jai-tracy: the build case's setup compiles `macos/libtracy.a` from `tracy/public/TracyClient.cpp`. With `-min_size 1` the plugin wraps `main` in Tracy zones and the program runs. Its `generate.jai` also runs under jaic but isn't a case: it rewrites the pinned `bindings.jai`, and its enums come out different from upstream's (no `TracyPlotFormat` prefix stripping, `u32` instead of `s32`).
- Upstream bugs, not cases: jai-date's `example.jai` passes `allocator =` as an ordinary named argument (current Jai needs `,,`); cluster's `examples/http_server.jai` calls `cluster_accept` with the old signature; jai-postgres's `examples/example.jai` refers to `Uuid` but declares `UUID`.

**chess-jai.** `build.jai - ui` and `- ai cpu` are check cases (`chess-jai-ui-check`, `chess-jai-engine-check`). `jaic build build.jai - ui ai release` produces `chess` and the `ceij` UCI engine (both need stb_vorbis). The engine reads its 21 MB NNUE network into a `#no_reset` global at compile time, and `perft_all` passes.

**forbear.** `build.jai` checks and builds (`forbear-build`). The case's setup compiles `vendor/kb_text_shape.a` and `vendor/freetype.a` with clang first; otherwise `build.jai` regenerates `bindings-MACOS.jai` inside the corpus.

**rexim.github.io.** `rss.jai` runs (`rexim-rss`). It writes `event/<id>.json`, so the case creates `event/`.

**ui_builder.** `jaic check demo.jai` passes (`ui-builder-demo-check`). `jaic build` stops at the link step: Pixel_Maker's prebuilt `bindings/Slang/lib/macos/libslang` is not in the pinned corpus.

**Photon.** Windows-only: `Ico_File` and `Windows_Resources` are imported only for Windows, and `-os windows` calls `MultiByteToWideChar` at compile time.

**KodaJai.** Imports FixedStringJai, JaiGLFW, ContiguousJsonJai, JaiBoundingTree, KodaSerializer, BlockAllocatorJai, JaiMath, lz4_static and JaiParallel, none pinned.

**no_api.** `first.jai` loads `examples/sponza/sponza.jai`, which is not in the repository; the build copies DLLs and launches `wt`.

Excluded: jaithon, which is its own language in `.jai` files.

## How to change it

Add a repository to `REPOSITORIES`, `DEPENDENCIES` or `LIBRARIES` in `fetch_upstreams.py`, run it, review the manifest diff and commit `corpus/upstreams.json` (never `corpus/upstream/`). Add entry points that compile to `tools/upstream-cases.json`, and update the status table. Prefer projects that check themselves (test suites, golden files, documented output) and record what they promise in an `expect` record ([recorded outputs](#recorded-outputs)). After TTWJ changes, run `python3 tools/upstream_expectations.py --write` and then `--verify target/release/jaic`. The search procedure for new projects is the [third-party smoke test](third-party-smoke-test.md).

## Configuration

`--since YYYY-MM-DD` (default 2025-10-01). Tests: `tools/test_upstreams.py`.

## Dependencies

Git, Python 3.11+, network access to GitHub. Native libraries the projects need are built as described in [native libraries](native-libs.md).
