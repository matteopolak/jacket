# Upstream corpus

## What it is

Pinned source snapshots of recently maintained open-source Jai projects (focus-editor/focus, Ivo-Balbaert/The_Way_to_Jai, SogoCZE/Jails, rluba/jaison, withlang-dev/open-jai, ostef/Vk-Engine, roeyb1/sgpu, plus the dependencies SogoCZE/jai_parser, ostef/Linalg and ostef/Jolt-Jai, and libraries (`LIBRARIES`): the metaprogramming libraries match-jai, yield-jai, JaiModules-AST_Utils, Jai-Shader-Transpiler, jai-utils, unotest, and rluba's library family jai-tracy, jai-redis, uniform, cluster, jai-csv, jai-postgres, stubborn, hyperserve, wait_group, jai-date; plus Tracy's client sources (wolfpld/tracy, a dependency of jai-tracy). They are the real-world programs `jaic` is measured against. They live in the gitignored `corpus/upstream/`; the committed `corpus/upstreams.json` records exact commits and file hashes.

## How it works

`tools/fetch_upstreams.py` downloads the pinned `.jai` files, READMEs, licenses, C-family sources (`NATIVE_SOURCE_SUFFIXES`, needed by native-library builds) and compile-time data files (`RESOURCE_PREFIXES`, e.g. focus's `config/` and `fonts/`) into `corpus/upstream/<owner>--<repo>/`. Existing pins in the manifest are kept; `--since YYYY-MM-DD` (default 2025-10-01) is the recency cutoff for newly added repositories, and `DEPENDENCIES` (submodules) and `LIBRARIES` (metaprogramming libraries that target the Compiler node API and rarely change) are exempt. `MODULE_LINKS` symlinks dependencies into the consumers' `modules/` directories, and libraries imported by name into `corpus/upstream/_modules/` (cases pass `-I ../../_modules`): `AST_Utils`, and rluba's `uniform`, `wait_group`, `cluster`, `date` (jai-date), `stubborn`, `tracy` (jai-tracy) and `hyperserve`, which import each other by those names. `SOURCE_PREFIXES` limits a repository to some directories (Tracy: only `public/`, the client library; its profiler GUI and bundled libraries are not needed). Nothing from a project is built or executed by the fetcher.

Dependencies are pinned like projects: jai_parser for Jails; Linalg, Jolt-Jai and Jolt-Jai's JoltC submodule for Vk-Engine (JoltC has no Jai files; the fetcher takes its `CMakeLists.txt` and `Examples/`); wolfpld/tracy at jai-tracy's `tracy` submodule commit (v0.11.1), linked as `rluba--jai-tracy/tracy`. `corpus/upstream` and the Git cache live in the main checkout (found through the Git common dir), so worktrees share them.

`tools/verify_upstreams.py` re-hashes the fetched tree against the manifest and rejects modified or missing files. Unlisted files are reported, not rejected: building the projects (sweep `build` cases, `build_native_libs.py`, `build_vk_engine_libs.py`) leaves libraries, executables and generated files next to the sources.

### Project status

| Project | Status | Notes |
| --- | :---: | --- |
| [Focus](https://github.com/focus-editor/focus) | ✅ | Builds and runs natively on macOS |
| [Jails](https://github.com/SogoCZE/Jails) | ✅ | Builds a native language server |
| [jaison](https://github.com/rluba/jaison) | ✅ | Tests and examples run, also natively |
| [uniform](https://github.com/rluba/uniform), [stubborn](https://github.com/rluba/stubborn) | ✅ | uniform's stubborn test suite runs at compile time |
| [jai-date](https://github.com/rluba/jai-date), [wait_group](https://github.com/rluba/wait_group) | ✅ | jai-date's self-tests and wait_group's example run; jai-date's example has an upstream bug |
| [jai-csv](https://github.com/rluba/jai-csv) | ✅ | Checks (no tests upstream); parses correctly when tried |
| [cluster](https://github.com/rluba/cluster), [hyperserve](https://github.com/rluba/hyperserve) | ✅ | Build natively; cluster supervises instances, hyperserve's examples serve requests |
| [jai-redis](https://github.com/rluba/jai-redis) | ✅ | Its test builds; running it needs a Redis server |
| [jai-postgres](https://github.com/rluba/jai-postgres) | ⚠️ | Checks; running needs libpq and a database |
| [jai-tracy](https://github.com/rluba/jai-tracy) | ✅ | `-plug tracy` instruments and builds a profiled program |
| [sgpu](https://github.com/roeyb1/sgpu) | ✅ | All examples build on macOS; mesh shaders need a driver MoltenVK lacks |
| [The Way to Jai](https://github.com/Ivo-Balbaert/The_Way_to_Jai) | ✅ | 316 of 343 programs run; the rest check (windowed, interactive, Windows-only or deliberately failing) |
| [Vk-Engine](https://github.com/ostef/Vk-Engine) | ⚠️ | Checks for Linux; its Vulkan, ImGui and Jolt modules have no macOS support |

The exact revisions are pinned in `corpus/upstreams.json`. Notes per project:

- **focus-editor**: `jaic build first.jai` produces a working native editor on macOS (renders, takes input).
  Needs `python3 tools/build_native_libs.py` (stb libraries) and its own
  `modules/Objective_C/LightweightRenderingView/build.jai` run once (`jaic build build.jai` there). Debug builds
  need `~/Library/Application Support/dev.focus-editor` to exist (upstream creates `.../debug` non-recursively).
- **Vk-Engine** (+ Linalg, Jolt-Jai): `jaic check Build.jai -I Modules -I Source -os linux - Core|Renderer|Game|Editor`
  passes (the ImGui and Vulkan generators run on the real headers). Native macOS stops in about 3 s: the upstream
  `Vulkan`, `ImGui` and `JoltPhysics` modules and their `generate.jai` have no macOS branch (editing the upstream
  project is out of scope). `python3 tools/build_vk_engine_libs.py` builds `libImGui`, `libVkMemAlloc` and `libJoltC`
  for macOS. See [Vk-Engine](../native/vk-engine.md).
- **jaison**: tests and example run, also natively (`jaic build tests.jai`). **sgpu**: all examples check (host, linux, windows) and build natively on macOS
  after `python3 tools/build_slang.py` (Slang 2025.24.2, VMA and the Vulkan loader built from source); with MoltenVK
  all run except 04_mesh_shaders (MoltenVK has no `VK_EXT_mesh_shader`). Run commands: [native libraries](native-libs.md).
- **Jails**: `jaic build build.jai` produces a native `bin/jails` that answers LSP requests.
  Jails `-os windows` needs a Windows host (compile-time `MultiByteToWideChar`).
- **The_Way_to_Jai**: 343 entry points are sweep cases (`tools/upstream-cases.json`): 316 run to completion,
  the rest check (windowed Simp programs, interactive or endless ones, deliberate crashes, user-built libraries).
  The files that fail `check` are not compiler bugs: Windows-only APIs (19.8, 33.2C, 33.6, 50.1), the Windows-only
  raylib module (35.1, 52.2, 30/jai_raylib), intentional failures (20.2, 30.9, exercises/22), APIs older Jai versions
  had (6.6 `random_seed` result, 26.27 and exercises/30 `builder_to_string(allocator=)`, 33.10 `Sound_Player`
  struct, 51.2 GetRect `dropdown`), missing command-line arguments (30.14, 8.2, 12.8), a missing
  `cpp_library.cpp`, and 31.2, which calls GL at compile time without a context (crashes in libGL). 19.5 frees an
  advanced pointer and 27/foldera writes through null (both upstream bugs).

- **Metaprogramming libraries** (12 sweep cases): match-jai `examples/first.jai`; yield-jai `constant`, `defer`,
  `first`, `if`, `if_case`, `while` (`expand` and `for` need per-call `macro_expansion_block` export, not done);
  AST_Utils `build.jai` (rewrites a call through `compiler_modify_procedure`) and `astTests.jai`;
  Jai-Shader-Transpiler `build.jai` (GLSL from `@glsl` procedures); jai-utils `closure.jai`; unotest.
  Checked by hand but not cases: epic-fail (a `-plug` plugin, see [metaprogram plugins](../metaprogramming/metaprogram-plugins.md); its
  `assert` works when imported directly), MetaThreadSafe (its examples fail on purpose; the diagnostics match, except that the
  untaken `#if` branch of a baked instance is still checked), jai-control-flow (upstream uses `%%` as an escaped
  percent, which newer Jai reads as two arguments; a corrected copy passes all its tests).

- **rluba's libraries**: all compile; what runs depends on the services they talk to. Pinned commits are the
  full SHAs in `corpus/upstreams.json`; "Sweep cases" are ids in `tools/upstream-cases.json`.

  | Library | What it is | Pinned | Status | Sweep cases |
  | --- | --- | --- | --- | --- |
  | [jaison](https://github.com/rluba/jaison) | JSON parse/print, typed and generic | `2009cdb5895d` | run, also native | `jaison-tests`, `jaison-example`, `jaison-native-build` |
  | [uniform](https://github.com/rluba/uniform) | RE2-style regular expressions | `d624b6d6c77c` | run (test suite) | `uniform-tests` |
  | [stubborn](https://github.com/rluba/stubborn) | Compile-time test runner and matchers | `bad3d44895c9` | run (drives uniform's tests) | `stubborn-module`, `uniform-tests` |
  | [jai-date](https://github.com/rluba/jai-date) | Date parsing, formatting, arithmetic | `6037e7519934` | run (`#run` self-tests) | `jai-date-module` |
  | [jai-csv](https://github.com/rluba/jai-csv) | CSV parsing into typed arrays, escaping | `df76cdc354c8` | check (no tests upstream) | `jai-csv-module` |
  | [wait_group](https://github.com/rluba/wait_group) | kqueue/epoll event loop | `411f0f350cd8` | run | `wait-group-example` |
  | [cluster](https://github.com/rluba/cluster) | Process clustering with a shared listen socket | `455607c129ee` | native build | `cluster-build`, `cluster-crashing-example` |
  | [hyperserve](https://github.com/rluba/hyperserve) | HTTP server framework | `5286acf11791` | native build | `hyperserve-example-build`, `hyperserve-datastar-build` |
  | [jai-redis](https://github.com/rluba/jai-redis) | Redis (RESP3) client | `253391ac1df1` | native build; running needs a Redis server | `jai-redis-test-build` |
  | [jai-postgres](https://github.com/rluba/jai-postgres) | libpq bindings and typed queries | `1fec19889899` | check; running needs libpq and a database | `jai-postgres-module`, `jai-postgres-pgvector` |
  | [jai-tracy](https://github.com/rluba/jai-tracy) | Tracy profiler bindings and instrumenting plugin | `53f1e8aa4efe` (Tracy `30997d5ca6bb`) | native build with `-plug tracy` | `jai-tracy-plugin-check`, `jai-tracy-plugin-build` |

  Notes per library:
  - uniform (regex): `jaic run first.jai - test` runs its stubborn test suite at compile time (all pass) — case
    `uniform-tests`, which also covers **stubborn** (plus `stubborn-module`).
  - jai-date: `module.jai`'s `#run` self-tests pass (`jai-date-module`, `-I ../_modules` for uniform). Its
    `example.jai` passes `allocator =` as an ordinary named argument, which current Jai rejects (it needs `,,`):
    an upstream bug, not a case.
  - jai-csv: `jai-csv-module` checks; a scratch program parsing quoted fields and range-checked integers ran
    correctly (it has no tests of its own).
  - wait_group: `examples/example.jai` runs (`wait-group-example`).
  - cluster: `cluster.jai` builds natively (`cluster-build`); `cluster -n 2 -- crashing` starts, watches and
    reaps instances (needs `Process` to give children a socket stdin). `examples/crashing.jai` checks.
    `examples/http_server.jai` calls `cluster_accept(socket)`, but the module now wants an address type first:
    upstream bug.
  - hyperserve: both examples build (`hyperserve-example-build`, `hyperserve-datastar-build`); the built
    servers answered GET routes with path/query parameters and the datastar SSE stream when tried by hand.
  - jai-redis: `test.jai` builds (`jai-redis-test-build`); it needs a Redis server to run. Against a scripted
    RESP3 server it ran to completion (pipelining, pushes, pub/sub).
  - jai-postgres: `first.jai` (compiles the module, `jai-postgres-module`) and `examples/Jaipgvector.jai` check.
    Running needs libpq and a database (neither installed). `examples/example.jai` refers to `Uuid` but
    declares `UUID`: upstream bug.
  - jai-tracy: `tests/test.jai -plug tracy` checks and builds (`jai-tracy-plugin-*`; the build case's setup
    compiles `macos/libtracy.a` from `tracy/public/TracyClient.cpp`). With `-min_size 1` the plugin wraps
    `main` and `do_something` in Tracy zones and the program runs. Its `generate.jai` also runs under jaic
    (builds both macOS archs with `BuildCpp`, regenerates `bindings.jai`); not a case because it rewrites the
    pinned `bindings.jai`, and the regenerated enums differ from upstream's (no `TracyPlotFormat` prefix
    stripping, `u32` instead of `s32`).

Vk-Engine needs a `--release` build (about 45 s per module):
`cd corpus/upstream/ostef--Vk-Engine && jaic check Build.jai -I Modules -I Source -os linux - Core|Renderer|Game|Editor`.

## How to change it

Add a repository to `REPOSITORIES`/`DEPENDENCIES`/`LIBRARIES` in `fetch_upstreams.py`, run it, review the manifest diff and commit `corpus/upstreams.json` (never `corpus/upstream/`). Add entry points that compile to `tools/upstream-cases.json` so [jaic-sweep](jaic-sweep.md) keeps them green. Current per-project status is under [Project status](#project-status).

## Configuration

`--since YYYY-MM-DD`. Network access to GitHub. Tests: `tools/test_upstreams.py`.

## Dependencies

Git, Python 3.11+. Native libraries the projects need are built by [native-libs](native-libs.md).
