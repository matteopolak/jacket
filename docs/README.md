# Developer documentation

- Sound_Player: [plan](status/audio-plan.md), [progress](status/audio-progress.md),
  [deviations](status/audio-deviations.md), [questions](specs/questions/sound-player.md)

Current replacement integration: [commands and results](tools/cleanroom-integration.md), [deviations](status/integration-deviations.md), [module inventory](status/integration-module-inventory.json).111/135 reviewed module facades are present; presence does not establish complete functionality. Native CI and opaque public corpus/Focus remain coordinator work; publication is held.

Start with [compiler architecture](compiler/architecture.md). Real projects that compile are listed in [upstream corpus](tools/upstream-corpus.md#project-status).

- [Features](features.md): what works, performance and what is missing
- [Window/render progress](status/window-render-progress.md): scoped native window,
  native input, Windows pointer tracking and keyboard layouts, Keymap, window lifecycle, original native-font
  feature regressions, real font-atlas/text rendering and texture color-space transitions,
  textures, depth/stencil and asynchronous readback evidence;
  [remaining deviations](status/window-render-deviations.md).
- [Window/render clean-room incident](status/window-render-cleanroom-incident-2026-10-14.md):
  reviewed metadata-only exposure, clearance and implementation resumption.
- [License](license.md): AGPL-3.0-or-later with a runtime library exception for compiled programs
- [SDL native ABI and prefix safety](native/sdl-native-abi.md): exact public layouts, complete private C records, native pointers, field probes and canaries
- [Portable MPEG](portable-mpeg.md): MPEG-1 streams/video/CRC-checked audio, independent fixtures and remaining gaps
- [File watcher execution and ABI blocker](portable-file-watcher.md): real macOS event queues, sanitizer coverage and preserved FSEvents binding regression
- [File watcher and ABI dependency](portable-file-watcher.md): real native events and preserved FSEvents binding regression
- [macOS bundles and plist helpers](portable-macos-bundler.md): native bundle/icon tools, validated XML and new formatting dependency
- [Portable sound](portable-sound-player.md): canonical audio records, real sample arithmetic and shared mutable topology
- [Executable containers](portable-executable-formats.md): bounded ar parsing, symbol indexes, ownership and native format inputs
- [Portable SIMD](portable-simd.md): reviewed scalar kernels, byte gathers/scatters and complete reductions
- [Portable Meow](portable-meow.md): canonical register macros, real calico hashing and native C vector provenance

## Language

Jai language behavior as implemented by `jaic`.

- [Casts and conversions](language/casts-and-conversions.md)
- [Windows ABI unwind tables](compiler/windows-unwind.md)
- [Compiler host and output target constants](compiler/host-target-constants.md)
- [The implicit context](language/context.md)
- [Control flow: loops, cases and defer](language/control-flow.md)
- [Dead-code elimination: what is checked when unused](language/dead-code-elimination.md)
- [Declarations, constants and globals](language/declarations-and-constants.md)
- [Statement directives, flags and notes](language/directives-and-notes.md)
- [Enums](language/enums.md)
- [External data](language/external-data.md)
- [Compiler intrinsics](language/intrinsics.md)
- [Lambdas and anonymous procedures](language/lambdas.md)
- [Macros and custom iteration](language/macros-and-custom-iteration.md)
- [`#must` results and `#discard` parameters](language/must-and-discard.md)
- [Module parameters](language/module-parameters.md)
- [Modules and imports](language/modules-and-imports.md)
- [Arithmetic overflow checks and `#no_aoc`](language/arithmetic-overflow-checks.md)
- [Integers, floats and numeric literals](language/numbers.md)
- [`Long_Double`: C's `long double` (jaic extension)](language/long-double.md)
- [Operator overloading](language/operator-overloading.md)
- [Operator precedence](language/operators.md)
- [Pointers, arrays and bounds checks](language/pointers-and-arrays.md)
- [Polymorphism and baking](language/polymorphism.md)
- [Procedures and calls](language/procedures.md)
- [Scoping: visibility, using and conditional declarations](language/scoping.md)
- [SIMD and `#asm`](language/simd-asm.md)
- [Strings and literals](language/strings-and-literals.md)
- [Structs and aggregate literals](language/structs.md)
- [Type values, identity and type info](language/type-values-and-info.md)
- [Unions and tagged unions](language/unions.md)
- [using](language/using.md)
- [Compiler compatibility questions](specs/stdlib/compiler-questions.md): unresolved API contracts and implementation limits

## Metaprogramming

Compile-time execution, the `Compiler` module, reflection and code values.

- [Build options and the metaprogram API: what jaic honours](metaprogramming/build-options.md)
- [Code values and #insert](metaprogramming/code-values-and-insertion.md)
- [Compile-time values, globals and runtime info](metaprogramming/compile-time-data-and-state.md)
- [Compile-time execution (`#run`)](metaprogramming/compile-time-execution.md)
- [Compiler module (Jai side)](metaprogramming/compiler-module.md)
- [Independent compiler facades](compiler/compiler-facades.md): current supported helpers, traversal/printing and focused test overlay
- [Compiler-managed global data](compiler/compiler-global-data.md): registered storage identity, segment handles and compile-time/target runtime records
- [Bootstrap atomics](compiler/bootstrap-atomics.md): scalar validation, host/native/WASM atomic semantics and contention regressions
- [Ordered variadics and interfaces](compiler/mixed-variadics-and-interfaces.md): mixed packs, continuation lifetime, binding-time structural checks and loaded-file parameter scope
- [Register operands](compiler/register-operands.md): canonical SIMD operand identity, generic expansion and scalar lane results
- [Context cleanup](compiler/context-cleanup.md): conditional caller-scope macro context and lexical exit behavior
- [Generic default reflection](compiler/generic-default-reflection.md): source identity, evaluated instance defaults and storage contract
- [Measured performance messages](compiler/performance-messages.md): reviewed run records, actual counters and host timing boundary
- [Compiler progress](status/compiler-progress.md) and [deviations](status/compiler-deviations.md): producer checkpoints, verification evidence, gaps and required long-term fixes
- [Completeness follow-up](compiler/completeness-follow-up.md): qualified constant writes, runtime configuration, suspended threads and alias documentation
- [Private runtime configuration](compiler/runtime-configuration.md): settled workspace sizes, real target data and the staged producer allocation contract
- [Complete library lint context](tools/linter-library-context.md): private roots, real bootstrap bindings, alias identity and failure on incomplete checks
- [Literal precision](compiler/literal-precision.md): contextual defaults, required precision, aliases, folding, overloads and module arguments
- [Browser integration status](compiler/browser-integration-status.md): explicit source-library embedding, supported Compiler regression and remaining integration failures
- [Using-filter propagation](compiler/using-filter-propagation.md): parameter/macro bindings, embedded filters, mapped storage and safe Code reconstruction
- [Independent runtime Jai lexer](compiler/runtime-jai-lexer.md): implementation, supported recovery and interpreter/native verification
- [Compiler records (messages, syntax trees, type descriptors)](metaprogramming/compiler-records.md)
- [Metaprogram plugins (`+Name`)](metaprogramming/metaprogram-plugins.md)
- [Bootstrap and Runtime_Support](specs/jai/README.md)
- [Reflection and Type_Info](metaprogramming/reflection-and-type-info.md)
- [Workspaces and metaprograms](metaprogramming/workspaces.md)

## Compiler internals

How `crates/jaic`, the interpreter and the language server work.

- [Compiler architecture](compiler/architecture.md)
- [Owned compiler formatting verification](compiler/formatting-verification.md): source-built formatter/linter and generated-source idempotence
- [Compiler integration handoff](compiler-integration-handoff.md): source build, canonical bootstrap and runtime contract for independent library workers
- [Producer integration requests](compiler/producer-integration-requests.md): remaining Autorun path repair and independent safety/reflection review
- [Clean-room compiler implementation plan](compiler/cleanroom-implementation-plan.md)
- [Diagnostics](compiler/diagnostics.md): error message style guide, rendering, runtime failures, exit statuses
- [Format string check](compiler/format-string-check.md): `print` format strings against their argument count
- [jaic `#asm` blocks](compiler/asm.md)
- [SIMD lowering](compiler/simd-lowering.md): target features, per-ISA backends and how to add an intrinsic or instruction set
- [Threads under `jaic -interp`](compiler/interpreter-threads.md)
- [Fiber intrinsics: switching stacks in every backend](compiler/fibers.md)
- [jaic interpreter](compiler/interpreter.md)
- [Low-level IR](compiler/ir.md)
- [Compile speed](compiler/compile-speed.md): where an `-O0` build spends its time, what was done about it, current numbers
- [Memory limit (`JAIC_MEMORY_LIMIT`)](compiler/memory-limit.md)
- [Shared Jai language server](compiler/language-server.md) (front page: [jailsp README](../crates/jai-language-server/README.md), with the comparison with Jails): feature list, expansions, inlay hints, format strings, lints and quick fixes, `#import` links
- [Doc comments](compiler/doc-comments.md): the Markdown comment convention (parameters, intra-doc links) and how the server and the VS Code grammar use it
- [Language server refactorings](compiler/language-server-refactorings.md): extract, inline, fill in cases and fields, `ifx` rewrites, call hierarchy, selection ranges
- [Parser](compiler/parser.md)
- [Module instance identity](compiler/module-instance-identity.md): ordered module arguments and module specialization identity
- [Private standard-library source loading](compiler/internal-source-policy.md): canonical import/load provenance and cache checks
- [Sema: module loading and top-level expansion](compiler/sema-modules.md)
- [Sema: polymorphism and declarations](compiler/sema-polymorphism-and-declarations.md)
- [Stack traces (`context.stack_trace`)](compiler/stack-traces.md)

## Native builds

The LLVM backend, C ABI and linking.

- [C ABI, callbacks and C++ methods](native/c-abi.md)
- [C ABI coverage matrix](native/c-abi-coverage.md)
- [jaic LLVM backend](native/llvm-backend.md)
- [Native debug information](native/debug-info.md)
- [Native build and linking](native/native-linking.md)
- [Sanitizers for native builds](native/sanitizers.md)
- [Vk-Engine corpus project](native/vk-engine.md)
- [Native Windows executables](native/windows.md)
- [WebAssembly target (wasm64, WASI, `Wasi_Runtime`)](native/wasm-target.md)
- [Threads in WASI builds (green threads)](native/wasm-threads.md)

## Standard library

- [Independent integration commands and limits](tools/cleanroom-integration.md)
- [Integration deviations audit](status/integration-deviations.md) and [requests](integration-requests.md)
- [Integration contract questions](specs/questions/cleanroom-integration.md): unresolved runtime instance and ABI labels, distinct from specified implementation gaps
- [Legacy contract audit](tools/legacy-contract-audit.md): exact API evidence, reasoned fixture migrations and genuine failures retained for repair

- [Core foundations implementation handoff](foundations-handoff.md)
- [Basic public surface audit and outstanding exports](foundations-api-audit.md)
- [Remaining Basic implementation plan](status/foundations-completion-plan.md)
- [Basic producer dependency requests](status/foundations-dependencies.md)
- [General cross-compilation predicate request](status/foundations-preamble-request.md)
- [Strict lint target and program-context request](status/foundations-lint-request.md)
- [Public alias completion signature request](status/foundations-lsp-request.md)
- [Core foundations native ABI notes](foundations-native-abi.md)
- [Core foundations deviations and remaining limits](status/foundations-deviations.md)
- [Core foundations live progress](status/foundations-progress.md)
- [Exact shortest-float boundary audit](../tools/check_float_power_sweep.py): independent rational checks of every normal binary32/binary64 power-of-two interval
- [Core foundations contract questions](specs/stdlib/foundations-questions.md)
- [Numerics implementation status](status/numerics.md)
- [Numerics current progress](status/numerics-progress.md)
- [Numerics deviations register](status/numerics-deviations.md)
- [Numerics corrective review and tool evidence](status/numerics-corrective-plan.md)
- [Numerics compiler restriction handoff](status/numerics-compiler-handoff.md)
- [Numerics current canonical dependency reproducers](status/numerics-dependency-handoff.md)
- [Numerics implementation and verification](numerics.md)
- [Numerics implementation plan](status/numerics-plan.md)
- [Numerics contract questions](specs/stdlib/numerics-questions.md)
- [Window/render worker coverage and verification](window-render.md)
- [Window/render API coverage ledger](window-render-coverage.md)
- [Window/render worker handoff and integration gates](window-render-handoff.md)
- [Native GL format mapping ledger](window-render-gl-formats.json)
- [Focused window/render verification runner](../tools/check_window_render.py)
- [Window/render contract questions](specs/stdlib/window-render-questions.md)
- [Platform I/O backends and native/WASI runner](platform-io.md), [worker progress](status/platform-io-progress.md), and [deviations](status/platform-io-deviations.md)
- [Platform I/O compiler and lint handoff](platform-io-compiler-handoff.md)
- [Windows startup CI regressions and default-policy review](platform-io-startup-regressions.md)
- [Windows device notifications](native/window-render-device-notifications.md): real
  interface registration, bounded payload decoding, callbacks and native verification limits.

- [Windows startup CI regressions and native verification](platform-io-startup-regressions.md)
- [Interpreter/native Windows section queries and 0100a1f CI](platform-io-section-query-regression.md)
- [e8c312d native crash/thread follow-up and SDK regressions](platform-io-e8c312d-follow-up.md)
- [Private ARM64 exception backend design and checks](platform-io-arm64-crash-plan.md)
- [Native WGL investigation](native/window-render-wgl-investigation.md): filtered CI
  failure evidence, production diagnostics, actual driver probe and explicit real-driver CI deployment.
- [Native WGL investigation](native/window-render-wgl-investigation.md): filtered CI
  failure evidence, real retirement-surface lifetime, native diagnostics and pinned
  official-source ARM64 Mesa build/deployment with explicit runtime verification limits.
- [jaic Extensions](extensions/README.md)
- [Portable implementation module ledger](portable-stdlib-ledger.md)
- [Portable implementation and verification](portable-stdlib.md): includes process/thread, context remapping, text input and Unix System coverage
- [Portable implementation gaps](portable-stdlib-gaps.md): focused pending reproducers and unresolved behavior
- [Runtime configuration adoption](portable-runtime-configuration.md): actual settled allocations, CAPS and ownership handoff
- [Owned source format/lint runner](../tools/format_portable_stdlib.py): explicit scope and source-built jaifmt/jailint
- [WAV parser design and validation](portable-wav.md)
- [SMTP/MIME transport and loopback validation](portable-mail.md)
- [POSIX crash diagnostics and native startup verification](portable-crash-handler.md)
- [Classic ZIP directory parsing and ownership](portable-zip-directory.md)
- [Atomics and verified bootstrap aliases](portable-atomics.md)
- [Deep_Copy verified graphs and remaining contracts](portable-deep-copy.md)
- [Unix BuildCpp artifacts, cleanup and toolchain limits](portable-buildcpp.md)
- [Treemap weights, stable nodes and rectangle layout](portable-treemap.md)
- [POSIX shared-memory channels and mixed-process verification](portable-shared-memory.md)
- [Autorun lifecycle and compiler integration gaps](portable-autorun.md)
- [Portable worker progress](status/stdlib-progress.md) and [deviations](status/stdlib-deviations.md)
- [Socket native declaration ledger](socket-native-ledger.json)
- [Focused portable regression runner](../tools/check_portable_stdlib.py)
- [Portable implementation plan](plans/portable-stdlib.md)
- [Unresolved portable contracts](specs/stdlib/questions.md)
- [SDL3 desktop bindings, generation and native packaging](extensions/sdl3.md)
- [SDL3 comprehensive header/GPU coverage, organization and verification](extensions/sdl3-coverage.md)

Implementation work uses the API specification as its compatibility contract. Implementers do not read the source of a Jai distribution; specification review supplies public declarations and behavior through the documented clean-room workflow.

- [Standard-library public-contract fixtures](../tests/stdlib-contract/README.md)
- [Native binding implementation coverage](native/bindings-coverage.md)
- [Native binding generation and tool verification](native/bindings-generation.md)
- [Native SDL2 runtime input identities](native/sdl-sdk-inputs.json)
- [Native binding verified snapshot evidence](native/bindings-verified-snapshot.json)
- [Native binding contract questions](specs/stdlib/native-bindings-questions.md)
- [Native binding implementation deviations](status/bindings-deviations.md)
- [Bindings worker progress and remaining scope](status/bindings-progress.md)
- [Native WGL context implementation and verification boundary](native/wgl-context.md)
- [Windows render integration fixed-array literal blocker](native/compiler-fixed-array-literal.md)
- [iOS helper availability and private native host verification](native/apple-toolchain.md)
- [Android EGL native header input and verification boundary](native/egl-sdk-inputs.json)
- [SDL native RWops pointer forwarding and contract limits](native/sdl-rwops.md)
- [Explicit Android POSIX C header delegates](native/android-posix-delegates.md)
- [ImGui nested member scopes and real native C++ calls](native/imgui-nested-methods.md)
- [Safe Linux ARM64 stat syscall and record marshalling](native/linux-arm64-stat.md)
- [Generic GL current-context wrapper](native/gl-dispatch.md)
- [Native binding lint target/root context blockers](native/lint-target-context.md)
- [X11 modal message box and real-server verification boundary](native/x11-message-box.md)
- [DXC reflection forwarding and SDK/source verification limits](native/dxc-reflection.md)
- [Real GL SDK validation and reviewable integration fixes](native/gl-sdk-validation.md)
- [Current compiler context-probing regression](native/compiler-context-probe.md)
- [Jai language guide](specs/jai/README.md)

## Browser

The WebAssembly compiler bundle and its releases. The hosted playground UI lives in the portfolio site (https://matteopolak.com/playground/jai).

- [Browser compiler (WebAssembly bundle)](browser/playground.md)
- [Browser compiler releases](browser/compiler-releases.md)
- [Language tour (playground default workspace)](browser/tour.md)

## Tools and workflow

- [Developer tooling](developer-tools.md): reflected C header generation, original C/Jai round trips, API audit and outstanding tooling scope; [progress](status/developer-tools-progress.md) and [deviations](status/developer-tools-deviations.md).

Scripts, CI and project policies.

- [The `jaic` command line](tools/cli.md): modes, options, plugins, and what changed from 0.16
- [Benchmarks and profiling](tools/benchmarks.md)
- [Build storage and target directories](tools/build-storage.md)
- [Compile-time benchmark](tools/compile-time-benchmark.md): wall time, peak RSS and phases of real projects, `-timings`
- [Regex benchmark](tools/regex-benchmark.md): `Extensions/Regex` against Rust `regex`, RE2, PCRE2, CTRE and Hyperscan on a generated corpus
- [Edit benchmark](tools/edit-bench.md): how long `jailsp` takes to answer after an edit inside a body, compiles per edit
- [LSP partial-edit implementation plan](plans/lsp-partial-edits.md) and [open questions](specs/lsp-questions.md)
- [LSP partial-edit regression harness](tools/lsp-partial-edits.md): IDE recovery, custom `for` navigation, UTF-16 suffix edits and corpus checks; [0.17.1 worker validation](tools/lsp-validation-0171.md)
- [IDE coverage](tools/ide-coverage.md): which identifiers of a project get no hover or go to definition from `jailsp`
- [Language server benchmark](tools/lsp-benchmark.md): latency, CPU time and RSS of `jailsp` (or another server) on real and generated projects and very large files
- [Code formatting](tools/code-formatting.md): rustfmt and jaifmt checks, format-only commits and `.git-blame-ignore-revs`
- [Continuous integration](tools/continuous-integration.md)
- [Dependency policy](tools/dependency-policy.md)
- [Justfile](tools/justfile.md): `just build|fmt|lint|test|check|hooks`, the recipes CI also runs
- [Pre-commit hook](tools/pre-commit-hook.md): `.githooks/pre-commit`, jaifmt, jailint, rustfmt and item spacing on the staged files
- [Benchmark generator](tools/benchmark-generator.md): `benchgen.py` corpus-shaped programs of any size, `jaistats.py` corpus statistics
- [Differential testing](tools/differential-testing.md): interpreter, native, `-O2` and wasm must agree; `jaigen` random programs; `jaic-reduce`
- [GitHub Action](tools/github-action.md): `setup-jai`, installing the released toolchain in CI
- [Fuzzing](tools/fuzzing.md)
- [jaic regression sweep](tools/jaic-sweep.md)
- [Stdlib target check](tools/stdlib-target-check.md): every stdlib module type-checked for every target, unreferenced code included
- [Standard library API auditor](tools/stdlib-api-audit.md): compare documented public module names with importer-visible source declarations
- [Public-language contract fixtures](../tests/cleanroom-language/README.md): original self-checking cases from the language guide, run against an explicit local source build
- [Clean-room implementation workspaces](tools/cleanroom-workspaces.md): prepare independent source snapshots and enforce implementer filesystem isolation
- [Standard library integration decisions](tools/stdlib-integration-decisions.md): bootstrap layout, internal imports, public facade identity, and compatibility test corrections
- [Stdlib runtime tests](tools/stdlib-runtime-tests.md): every stdlib test run in the interpreter, natively and on wasm on every platform; skip list; per-module coverage ratchet
- [Objective-C selector check](tools/objc-selector-check.md): every selector the stdlib sends matches its arguments and exists in the runtime
- [jaifmt (Jai formatter)](tools/jaifmt.md) (front page: [jaifmt README](../jaifmt/README.md)): the `jaifmt/` program, its `build.jai` metaprogram and `jaifmt.wasm`
- [jailint (Jai linter)](tools/jailint.md) (front page: [jailint README](../crates/jailint/README.md)): rules, `jailint.toml`, suppression, adding a rule
- [LLVM setup](tools/llvm-setup.md)
- [LLVM backends and binary size](tools/llvm-backends.md)
- [Nix flake](tools/nix.md)
- [Package managers and installers](tools/package-managers.md): Homebrew tap, `install.sh`/`install.ps1`, the release jobs that publish them and their one-time setup
- [PGO and BOLT](tools/pgo-and-bolt.md): profile-guided and post-link optimisation of the released binaries, `tools/build_pgo.py`
- [Third-party native libraries](tools/native-libs.md)
- [Regex Unicode tables](tools/regex-unicode-tables.md): `tools/gen_regex_unicode.py`, the generated `unicode_data.jai` and the lookup API of `Extensions/Regex`
- [Third-party smoke test](tools/third-party-smoke-test.md): finding public Jai repositories and trying them
- [open-jai expectation harness](tools/openjai-expectations.md)
- [Releases](tools/releases.md)
- [Rule coverage](tools/rule-coverage.md): rule IDs in the language docs and the tests that cite them
- [Upstream corpus](tools/upstream-corpus.md)
- [VS Code extension](tools/vscode-extension.md): `editors/vscode`, grammar, language client, jaifmt formatting, toolchain download, packaging and publishing
- [Neovim plugin](tools/neovim-plugin.md): `editors/nvim`, the `jailsp` config for `vim.lsp.enable`, jaifmt formatting, syntax file, toolchain download, tests and the `jai.nvim` mirror

## Plans

- [Window/render progress](status/window-render-progress.md) and [deviations](status/window-render-deviations.md): production gaps, staged evidence and next verification steps.
- [Window/render compiler build](../tools/window_render_compiler.py): builds the integrated compiler/linter directly; the reflection correction is production compiler code.
- [Window/render coverage](window-render-coverage.md): public native windows, Keymap and renderer contracts, with verification limits.

Agreed designs that are not built yet.

- [Async IO, HTTP and JSON](plans/async-io.md): fibers over io_uring/epoll/kqueue/IOCP, typed JSON, and an HTTP server and client

- [Native integration CI](tools/native-integration-ci.md): temporary branch checks and real Windows File execution

- [Sound_Player implementation and verification](features/sound-player.md)
