# Developer documentation

Current frozen native ARM64 measurement: [machine-readable current baseline](status/mac-current-baseline.json), with exact candidate path/ref/snapshot and separate coordinator gates.
- [Dynamic import factory registration](compiler/plugin-registration.md):
  reference-confirmed placeholder/indirect-call boundary, retained compile-time
  frames, scoped source dependencies, tested factories and exact limitations.

- [Required native library initialization](compiler/native-required-libraries.md):
  real constructor effects in interpreted/linked programs, precise load failures
  and compile-time/SandboxHost capability boundaries.

- [Combined Mac integration commands and corpus boundaries](tools/cleanroom-integration.md):
  exact frozen workspace/catalogue counts, production helper checks, and permitted
  application staging with manifest provenance and exclusions before access.
- [Basic shared print context and public arithmetic](features/basic-public-context.md):
  one field across configurations, preserved builder layouts, supported free/hash and Vector2 APIs;
  [measured results and remaining dependencies](status/basic-core-public.md).

- [Rejected-overload lint defect](compiler/rejected-overload-lint-request.md):
  independent reduction and compiler-owner request for the required Http client context.

- [Mac Extensions dependency repairs](extensions/mac-dependency-repairs.md):
  Json ownership/numerics, Http time/atomics, and test-only dependency migrations;
  [measured results and remaining lint-context failure](status/extensions-mac-repairs.md).
- [Focused Mac compiler catalogue classifications and repairs](compiler/mac-catalogue-repairs.md),
  including reviewed decimal inference and narrowing rejection regressions.
- [Rejected generic overload lint audit roots](compiler/rejected-overload-audit.md):
  cached instance identity, selected-body diagnostics and independent regressions.
- [Module-root context/import ordering](compiler/module-root-context-ordering.md):
  configured bootstrap reuse before execution and context layout, with strict lint regressions.
- [Canonical called-library interception](compiler/canonical-library-interception.md):
  ordinary/generic body scheduling, canonical headers, filters and completion.

- [Extensions Io/Args dependency migration](extensions/dependency-migration.md):
  allocator ownership, fiber scratch lifecycle and actual WASI termination;
  [measured status and remaining dependency handoff](status/extensions-dependencies.md).
- [Io_Uring canonical atomics](extensions/io-uring.md#canonical-atomics-and-protocol-verification):
  preserved ring ownership, stronger ordering cost, host protocol tests and Linux evidence limits.

- Sound_Player: [plan](status/audio-plan.md), [progress](status/audio-progress.md),
  [deviations](status/audio-deviations.md), [questions](specs/questions/sound-player.md),
  [playback and independent entity phases](features/sound-player.md)
- [Audio native integration](features/audio-native-integration.md): relocatable
  static linkage, six desktop SDK/CI recipes, focus and device recovery evidence.
- [Audio testing](features/audio-testing.md): assertion-disabled and real-resource
  close-failure tests, NEW spec provenance, independent
  review request, hardware/source distinctions and legacy-gate migration.

Current replacement integration: [commands and results](tools/cleanroom-integration.md), [deviations](status/integration-deviations.md), [module inventory](status/integration-module-inventory.json).132/135 reviewed module facades are available (including canonical Preload); three modules and one constant-table supplement remain absent. The [native ARM64 baseline report](status/mac-integration-baseline.json) records the exact combined snapshot, commands and outstanding failures. Presence does not establish complete functionality. Native CI and opaque public corpus/Focus remain coordinator work; publication is held.

[macOS resolver candidate](portable-socket-resolver.md) records native C ABI
evidence and the genuine forward-enum compiler gate, without a Jai success claim.
[Deep_Copy timer operations](portable-deep-copy-timing.md) records the real
disabled-assertion defect, explicit clock/error fix and Mac regression evidence.

[macOS resolver and network database bindings](portable-socket-resolver.md) record
active C/Jai ABI and native behavior evidence, with the remaining full-lint compiler gate.

Android host [ADB operations](portable-adb.md) now include app lifecycle status,
force-stop, installation queries and reviewed SDK discovery/cache/startup, with
protocol/assertion-configuration tests and device gaps recorded.
[JAVA_HOME discovery](portable-java-discovery.md) validates real launcher status
and records its interpreter/native protocol coverage and Windows dependency.
[Android configuration flags](portable-android-values.md) covers actual manifest
mask parsing, diagnosed invalid input and independent target checks.
[ASN.1 readers](portable-asn1.md) now follow refreshed value/TLV, known-array,
OID equality/text and length-prefix contracts, with separate test corrections.
The same document covers all three diagnostic printer overloads, iterative
nesting, supported types, actual tests and finer output/platform policy questions.
[X.509 key-info extraction](portable-x509.md) matches genuine newly generated
native C certificate vectors and preserves caller-backed encoded/content views.
[ZIP wire utilities](portable-zip-wire.md) covers real checksum/version/offset/unit
operations, shared producer staging and the precise remaining archive contracts.

- [Reassigned generator emission helpers](generator-emission.md): disjoint helper
  ownership, all ten reviewed contracts, canonical options protocol and native evidence.

- [Generator platform helpers](generator-platform.md): owned native-library/object
  APIs, reviewed cache/context/path contracts, official LLVM delegation and canonical model integration.
  [Copy-contract revalidation](generator-platform.md#copy-contract-revalidation)
  identifies the existing f6909c62 repair and fresh filesystem/native evidence.
- [String-macro diagnostic reduction](generator-platform-macro-cause.md): original
  SDK/FFI evidence and the exact callback-model cause of the combined test failure.
- [Module-root context ordering](compiler/module-root-context-ordering.md): the
  compiler-owned general lint fix integrated for generator context registration.

Start with [compiler architecture](compiler/architecture.md). Real projects that compile are listed in [upstream corpus](tools/upstream-corpus.md#project-status).

- [Features](features.md): what works, performance and what is missing
- [Generator model helpers](generator-model.md): canonical model operations, type parameters, aggregate defaults, ownership, verification and integration dependencies
- [Generator configured-SDK path check](status/generator-sdk-path.md): native test repair for symlinked T7 build storage
- [Private generator protocol](generator-session.md): the single active slot, ownership and nested callback restoration
- [Module Context lint repair](status/generator-lint-context.md): independent reduction and general finalization fix
- [Generator Mac integration repairs](status/generator-integration-repairs.md): real macro failure diagnostics and framework metadata hygiene
- [Generator modifier helpers](generator-modifiers.md): all eight integrated public helpers, canonical shared context protocol, and measured public-import host/target verification
- [Generator Mac integration repairs](status/generator-integration-repairs.md): real macro failure diagnostics, framework metadata hygiene and timestamp fixture storage
- [Generator lint file identity reduction](status/generator-lint-path-alias.md): canonical path aliases in compile-time sandbox file overlays
- [Window/render progress](status/window-render-progress.md): scoped native window,
  native input, Windows pointer tracking and keyboard layouts, Keymap, window lifecycle, original native-font
  feature regressions, real font-atlas/text rendering and texture color-space transitions,
  textures, depth/stencil and asynchronous readback evidence;
  [remaining deviations](status/window-render-deviations.md).

- [Shader compiler helpers](native/shader-compiler-helpers.md): reviewed D3D
  string ownership and DXC version text, with separate native/source evidence
- [X11 clipboard ownership](native/x11-clipboard-ownership.md): real cleanup,
  allocator tracking and separate native/source verification
- [Reviewed X11/Android helpers](native/x11-android-helpers.md): ten source/API
  implementations, native command/policy tests and explicit device limitations
- [Current combined worker Mac evidence](native/bindings-mac-current.md):
  current producer staging, zero failures and explicit remaining gates
- [Objective-C runtime helpers](native/objc-runtime-helpers.md): real encoding,
  runtime class/method installation, member declarations and native ABI tests
- [FSEvents context adapter](native/fsevents-context.md): exact public order,
  private SDK marshalling, callback lifetime and native canary/event checks
- [Direct3D canonical interface dependencies and compiler repros](native/d3d-source-dependencies.md)
- [Private rpmalloc SDK backend](native/rpmalloc-native-backend.md): complete
  native forwarding, checked alignment boundary and remaining public work
- [Public rpmalloc adapter progress](native/rpmalloc-public-adapter.md): actual
  raw/rewrite allocation, callback/config lifetimes, real statistics profiles
  and remaining public implementation
- [Canonical rpmalloc heap adapter design](native/rpmalloc-heap-adapter-design.md):
  implemented typed ownership, native field/graph marshalling, atomic probes,
  callback-safe publication, the pinned C page-granularity fix and remaining resource policies
- [Explicit raw heap build profile plan](native/rpmalloc-raw-heap-plan.md): private
  build selection and separate canonical raw record marshalling
- [rpmalloc allocator protocol and diagnostic snapshots](native/rpmalloc-allocator-protocol.md):
  complete reviewed mode dispatch, raw-base records, transactional capture and output gates
- [rpmalloc native cache selections](native/rpmalloc-global-cache-plan.md):
  real SDK domains, conditional public layouts and cache-option verification.
- [Transactional rpmalloc heap handle reservation](native/rpmalloc-heap-reservation.md):
  preflight OOM/null handling, actual SDK failure rollback and remaining graph allocation limits
- [rpmalloc heap ABI evidence](native/rpmalloc-heap-abi.md): complete pinned
  native storage probes and required separation from canonical public records
- [License](license.md): AGPL-3.0-or-later with a runtime library exception for compiled programs
- [SDL native ABI and prefix safety](native/sdl-native-abi.md): exact public layouts, complete private C records, native pointers, field probes and canaries
- [Portable MPEG](portable-mpeg.md): MPEG-1 streams/video/CRC-checked audio, independent fixtures and remaining gaps
- [File watcher and ABI dependency](portable-file-watcher.md): real native events, retryable close ownership and preserved FSEvents binding regression
- [macOS bundles and plist helpers](portable-macos-bundler.md): native bundle/icon tools, validated XML and new formatting dependency
- [Portable sound](portable-sound-player.md): canonical audio records, real sample arithmetic and shared mutable topology
- [Executable objects and containers](portable-executable-formats.md): canonical ABI, bounded Mach-O/ELF/COFF/PE/universal/ar workflows, ownership, native checks and [exact remaining rows](implementation-scope.json)
- [Executable lexer and TBD](executable-text-formats.md): real token workflows, owned Apple text-stub parsing, SDK tapi symbol equality and error/allocation regressions
- [Executable formats final handoff](executable-formats-handoff.md): zero remaining reviewed rows, canonical Guid dependency provenance, CodeView ABI/oracle checks and integration status
- [Portable SIMD](portable-simd.md): reviewed scalar kernels, byte gathers/scatters and complete reductions
- [Portable Meow](portable-meow.md): canonical register macros, real calico hashing and native C vector provenance
- [Debug_Info public parsers](debug-info-public-parsers.md): DWARF4 units, CodeView types/symbols/inline overlays, MSF/PDB/TPI/IPI workflows and SDK oracles
  ([isolated implementation checkpoint](status/debug-info-impl.md))
- [Debug public runtime](debug-public-runtime.md): final conditional APIs, assertion/dialog/attachment/minidump workflows and platform evidence limits
  ([integration handoff](status/debug-integration-handoff.md), [combined local evidence](native/debug-public-evidence.json))

## Language

Jai language behavior as implemented by `jaic`.

- [Casts and conversions](language/casts-and-conversions.md)
- [Full available compiler suite and triage](compiler/full-suite-triage.md)
- [Native interpreter thread entry and ownership](compiler/windows-thread-entry-lifecycle.md)
- [Windows SDK runtime resolution and native build gate](compiler/windows-sdk-runtime-resolution.md)
- [Windows compiler CRT selection and tool build gate](compiler/windows-crt-linkage.md)
- [Independent cast/shift reduction and open grammar contract](compiler/cast-shift-widths.md)
- [Checked expression result metadata](compiler/typed-expression-metadata.md)
- [Real x64 CPUID execution](compiler/cpuid.md)
- [Architectural x64 timestamps, random instructions and memory hints](compiler/x86-machine-instructions.md)
- [Default generic pointee construction and constraints](compiler/default-generic-pointers.md)
- [Opaque pointer constants and compile-time capture](compiler/raw-pointer-constants.md)
- [Forward overlay declaration and layout resolution](compiler/forward-overlay-layout.md)
- [Forward enum values, numbering and reflection order](compiler/enum-dependencies.md)
- [Interpreter Windows thread handles and naming](compiler/windows-thread-handles.md)
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
- [Target-aware jailint and coverage manifests](tools/jailint-targets.md): real compiler targets, invocation-relative imports, checked contexts, conditional loads and integration-runner handoff
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

- [Data/resource handoff](portable-data-handoff.md).
- [Data/resource tests and dependencies](portable-data-testing.md);
  [Windows resources and native gate](portable-windows-resources.md).
- [Subtitles implementation and per-API evidence](portable-subtitles.md).
- Data/resource [plan](portable-data-plan.md), [public API evidence](portable-data-api.md),
  [progress](status/portable-data-progress.md), [deviations](status/portable-data-deviations.md),
  and [questions](specs/questions/portable-data.md).

- [Independent integration commands and limits](tools/cleanroom-integration.md)
- [Integration deviations audit](status/integration-deviations.md) and [requests](integration-requests.md)
- [Integration contract questions](specs/questions/cleanroom-integration.md): unresolved runtime instance and ABI labels, distinct from specified implementation gaps
- [Legacy contract audit](tools/legacy-contract-audit.md): exact API evidence, reasoned fixture migrations and genuine failures retained for repair

- [Core foundations implementation handoff](foundations-handoff.md)
- [Basic/String/Hash/File_Utilities public surface audit](foundations-api-audit.md)
- [Wide conversions, division exercise and scalar FNV](foundations-wide-integers.md)
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
- [e223538 fault-test oracle review and native SDK trigger](platform-io-e223538-fault-oracles.md)
- [Windows Thread assertion side effects and configuration coverage](platform-io-windows-thread-assertions.md)
- [Private ARM64 exception backend design and checks](platform-io-arm64-crash-plan.md)
- [Native WGL investigation](native/window-render-wgl-investigation.md): filtered CI
  failure evidence, production diagnostics, actual driver probe and explicit real-driver CI deployment.
- [Native WGL investigation](native/window-render-wgl-investigation.md): filtered CI
  failure evidence, real retirement-surface lifetime, native diagnostics and pinned
  official-source ARM64 Mesa build/deployment, deterministic prepared child Python,
  verified LLVM-only cache, pinned Windows WGL target/install validation and SDK
  dependency evidence, optional VS search-path fingerprinting and required ARM64
  SDK inputs, with explicit runtime verification limits.
  SDK inputs, transient-only pinned-download retries and explicit runtime
  verification limits.

- [jaic Extensions](extensions/README.md)
- [Portable implementation module ledger](portable-stdlib-ledger.md)
- [Portable implementation and verification](portable-stdlib.md): includes process/thread, context remapping, text input and Unix System coverage
- [System raw counter platform split](portable-system-counter.md): real RDTSC and Unix raw-clock nanosecond-member behavior
- [Portable implementation gaps](portable-stdlib-gaps.md): focused pending reproducers and unresolved behavior
- [Runtime configuration adoption](portable-runtime-configuration.md): actual settled allocations, CAPS and ownership handoff
- [Time and calendar API inventory](portable-time.md): real clocks, calendar conversion, caller-return duration macro and remaining contracts
- [Machine_X64 compiler gaps](portable-machine-gaps.md): original interpreter/native CPUID failure and genuine native C comparison
- [Flat_Pool core and container audit](portable-flat-pool.md): real initialization, lazy page commitment, parameterized overwrite and lifecycle
- [Pool behavior and checked allocation extents](portable-pool.md)
- [Unix Thread assertion configuration and lifecycle ownership](portable-thread-lifecycles.md)
- [Windows calendar verification](../tools/check_portable_windows_calendar.py): source/ABI checks, explicit native-host requirement
- [Windows shared-memory backend and remaining gates](portable-windows-channel.md)
- [Resolved opaque pointer compiler handoff and Registry verification limits](portable-opaque-pointer-constants.md)
- [Windows Socket wrappers, native ABI evidence and remaining public gaps](portable-windows-socket.md)
- [Native Socket field accessors and independent SDK record oracles](portable-socket-fields.md)
- [Socket ancillary helpers and actual native SDK vectors](portable-socket-cmsg.md)
- [Windows Registry native APIs and unresolved pseudo-handle ABI](portable-windows-registry.md)
- [Android toolchain ABI naming, OID identity, digests and remaining APIs](portable-android-toolchain.md)
- [Default metaprogram build workflow](portable-default-metaprogram.md): real workspaces and plugin lifecycle, focused Mac evidence and explicit Check/compiler dependency requests
- [APK file views and iteration](portable-apk.md): real ZIP32 inputs, ownership, local payload offsets and malformed-boundary regressions
- [Project generator workflow](portable-project-generator.md): real target builds and deterministic Visual Studio artifacts; precise linter-host/resource gaps
- [Memory visualizer protocol comparison](portable-memory-protocol.md): opaque pointer-pair equality, packet layouts and genuine compile-time/native regressions
- [Portable owned implementation inventory](status/implementation-remaining.json): reviewed API denominator, conservative complete/partial/missing source counts, exact names, draft state and transfer estimates
- [Runtime pending-output discard handoff](portable-runtime-output-discard.md): canonical private hook, exact binding patch and real byte-I/O tests
- [Canonical signing model handoff](portable-signing-model-handoff.md): shared record/enum defaults, union storage and generator ownership boundary
- [Android SDK ADB command subsystem](portable-adb.md): native host CLI protocol tests, canonical records and live configuration
- [ASN.1 primitive parsing and byte cursors](portable-asn1.md): canonical tags, signed integers, full-width OIDs and unresolved value/printer contracts
- [Exact missing owned public declarations](portable-missing-surface.md)
- [Windows UTF-8 helpers](portable-windows-utf8.md): real Kernel32 conversion code, ABI checks and outstanding native execution
- [Assigned implementation remainder](status/implementation-remaining.json): fresh exact owned names, committed/draft status, effort and verification gaps
- [Canonical generator model dependency](generator-model.md): staged NEW producer records/helpers and explicit ownership policies
- [Public generator value-emission workflow](../tools/check_developer_value_emission.py): typed integer/float/string/enum values, expressions and native default calls
- [Public generator cursor/type-query workflow](../tools/check_developer_cursor_source.py): copied SDK source tokens, compiled C++ effects, borrowed model identity and cycle diagnostics
- [Multiple-header generation workflow](../tools/check_developer_multi_input.py): shared types/macros, callback renaming, C/C++ bridges and native calls
- [Pointer-default generation workflow](../tools/check_developer_pointer_defaults.py): shared-literal-safe null normalization and real omitted/explicit foreign arguments
- [Byte-string macro workflow](../tools/check_developer_string_macros.py): SDK initializer evaluation, copied callback payloads and real native byte/count comparisons
- [Public omission visitor workflow](../tools/check_developer_declaration_visitors.py): canonical closure storage, callback ordering and stopped traversal
- [Signed integer-range workflow](../tools/check_developer_integer_range.py):8/16/32/64/128-bit native oracle, aliases/enums and cross-target source evidence
- [Public generator type-emission workflow](../tools/check_developer_type_emission.py): C++ declarators, generated Jai callbacks, borrowed type identity and diagnosed failures
- [Shared generator context protocol](developer-tools-context-protocol.md): internal canonical-owner handoff, supplied options and lifetime requirements
- [Check factory workflow](../tools/check_developer_check_plugin.py): persistent plugin storage and real configured compiler-message effects
- [Check mangled-signature workflow](../tools/check_developer_check.py): official LLVM AST parsing, staged Mac effects and real pool statistics
- [Debug signal/assertion lifecycle tests](../tools/check_developer_debug_lifecycle.py): actual handler restoration and native fatal callback evidence
- [Customized C++ binding workflow](../tools/check_developer_cpp_custom.py): visitors, overloads, argument callbacks and native POD/reference effects
- [Owned source format/lint runner](../tools/format_portable_stdlib.py): explicit scope and source-built jaifmt/jailint
- [Bindings generator naming helpers](generator-names.md): eight reviewed helpers, integrated public imports, canonical options and native/source verification
- [WAV parser design and validation](portable-wav.md)
- [SMTP/MIME transport and loopback validation](portable-mail.md)
- [Mail scramble numeric-alphabet review question](portable-mail-questions.md)
- [Default allocator heaps, ownership and performance tradeoffs](portable-default-allocator.md)
- [Real direct Android signing keys and crypto-tool dependency](portable-direct-signing-key.md)
- [Java SDK JKS generation source and honest verification limits](portable-jks-generation.md)
- [Canonical Android hexadecimal formatter aliases](portable-android-format-helpers.md)
- [Relocated native crash-helper package lookup and override audit](portable-native-package-lookup.md)
- [Remaining exact portable contract requests and owned scope](portable-remaining-contract-requests.md)
- [Reviewed S128/Apollo arithmetic foundation handoff](portable-wide-arithmetic-handoff.md)
- [Metaprogram plugin public-contract/compiler handoff](metaprogram-plugins-dependency-request.md)
- [Real compile-time plugin parser/initializer and required compiler capability](portable-metaprogram-plugins.md)
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
- [Portable test provenance and current verification gaps](portable-verification.md)
- [Focused real x64 CPU-query verification](../tools/check_portable_machine.py)
- [Portable implementation plan](plans/portable-stdlib.md)
- [Unresolved portable contracts](specs/stdlib/questions.md)
- [SDL3 desktop bindings, generation and native packaging](extensions/sdl3.md)
- [SDL3 comprehensive header/GPU coverage, organization and verification](extensions/sdl3-coverage.md)
- [SDL3 current native CI corrections and verification](status/sdl3-progress.md)
- [SDL3 MSVC header profiles, canonical ledger hashes and clean CI repairs](extensions/sdl3-ci-repairs.md)
- [SDL3 deterministic generated bytes and Windows CRLF checkout regression](extensions/sdl3-generated-bytes.md)
- [SDL3 official installed SDK packaging and relocated C/C++ consumer checks](extensions/sdl3-sdk.md)
- [SDL3 implementation deviations and remaining limits](status/sdl3-deviations.md)

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
- [SDL native pointers, exact public prefixes and complete private SDK records](native/sdl-rwops.md):
  ownership, adapter extension/probe instructions and corrected layout oracle
- [Explicit Android POSIX C header delegates](native/android-posix-delegates.md)
- [ImGui nested member scopes and real native C++ calls](native/imgui-nested-methods.md)
- [Safe Linux ARM64 stat syscall and record marshalling](native/linux-arm64-stat.md)
- [Generic GL current-context wrapper](native/gl-dispatch.md)
- [Native binding lint target/root context blockers](native/lint-target-context.md)
- [X11 modal message box and real-server verification boundary](native/x11-message-box.md)
- [DXC reflection forwarding and SDK/source verification limits](native/dxc-reflection.md)
- [DXC compilation, argument ownership and verification limits](native/dxc-compilation.md)
- [Real GL SDK validation and reviewable integration fixes](native/gl-sdk-validation.md)
- [Current compiler context-probing regression](native/compiler-context-probe.md)
- [Jai language guide](specs/jai/README.md)

## Browser

The WebAssembly compiler bundle and its releases. The hosted playground UI lives in the portfolio site (https://matteopolak.com/playground/jai).

- [Browser compiler (WebAssembly bundle)](browser/playground.md)
- [Independent browser scheduler/filesystem verification](browser/sandbox-runtime-verification.md): private runtime provider coverage with strict public module target guards
- [Browser compiler releases](browser/compiler-releases.md)
- [Language tour (playground default workspace)](browser/tour.md)

## Tools and workflow

- [Profiler/Codex reduced compiler requests](profiler-codex-compiler-handoff.md): passing9b3d4932 dependency replay, owned startup/zone-entry fixes, and actual native/interpreted instrumentation.
- [Iprof/Codex workflow progress](profiler-codex-workflows.md): complete reviewed owned APIs, real frame/history/report and Codex v2 workflows, passing Mac verification and platform limits.
- [Iprof/Codex owned implementation and per-API audit](profiler-codex.md): canonical records, helper lifetime, real profiling traversal, focused verification and exact remaining gaps.

- [Developer tooling](developer-tools.md): reflected C headers, configured-SDK C/C++ bindings, measured manual profiler, typed Codex metrics, native ABI probes, DWARF/CodeView queries, staged DWARF byte decoding and canonical MSF readers/page bits; [test provenance](developer-tools-testing.md), [native integration plan](developer-tools-native-integration.md), [remaining functions](status/developer-tools-remaining.md), [progress](status/developer-tools-progress.md) and [deviations](status/developer-tools-deviations.md).

Scripts, CI and project policies.

- [The `jaic` command line](tools/cli.md): modes, options, plugins, and what changed from 0.16
- [Benchmarks and profiling](tools/benchmarks.md)
- [Build storage and target directories](tools/build-storage.md)
- [Compile-time benchmark](tools/compile-time-benchmark.md): wall time, peak RSS and phases of real projects, `-timings`
- [Regex benchmark](tools/regex-benchmark.md): `Extensions/Regex` against Rust `regex`, RE2, PCRE2, CTRE and Hyperscan on a generated corpus
- [Edit benchmark](tools/edit-bench.md): how long `jailsp` takes to answer after an edit inside a body, compiles per edit
- [LSP partial-edit implementation plan](plans/lsp-partial-edits.md) and [open questions](specs/lsp-questions.md)
- [LSP partial-edit regression harness](tools/lsp-partial-edits.md): IDE recovery, custom `for` navigation, UTF-16 suffix edits and corpus checks; [0.17.1 worker validation](tools/lsp-validation-0171.md)
- [Basic temp / exit LSP investigation](tools/lsp-basic-resolution-verification.md): actual native candidate requests, general qualified-value identity fix and mandatory final merged verification
- [LSP progress](status/lsp-progress.md) and [implementation deviations](status/lsp-deviations.md): current isolated-worker evidence, ownership overlaps and remaining integration limits
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
- [Standard library integration decisions](tools/stdlib-integration-decisions.md): bootstrap layout, distribution native libraries, internal imports, public facade identity, and compatibility test corrections
- [Stdlib runtime tests](tools/stdlib-runtime-tests.md): reviewed public contracts, producer suites and preserved compiler/ABI/Extensions regressions; explicit platform limits and migration provenance
- [Objective-C selector check](tools/objc-selector-check.md): every selector the stdlib sends matches its arguments and exists in the runtime
- [jaifmt (Jai formatter)](tools/jaifmt.md) (front page: [jaifmt README](../jaifmt/README.md)): the `jaifmt/` program, its `build.jai` metaprogram and `jaifmt.wasm`
- [jailint (Jai linter)](tools/jailint.md) (front page: [jailint README](../crates/jailint/README.md)): rules, `jailint.toml`, suppression, adding a rule
- [LLVM setup](tools/llvm-setup.md)
- [LLVM backends and binary size](tools/llvm-backends.md)
- [Nix flake](tools/nix.md)
- [Package managers and installers](tools/package-managers.md): Homebrew tap, `install.sh`/`install.ps1`, the release jobs that publish them and their one-time setup
- [PGO and BOLT](tools/pgo-and-bolt.md): profile-guided and post-link optimisation of the released binaries, `tools/build_pgo.py`
- [Third-party native libraries](tools/native-libs.md)
- [Pinned xxhash070 Windows ARM64 build repair and native verification](tools/xxhash-arm64-build.md)
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

- [Private native check transport and safety regressions](compiler/private-native-checks.md)

- [Stdlib test migration manifest](status/stdlib-test-migration.json): per-file retirement/move decisions and preserved regression provenance.

- Sound_Player [per-public-API inventory](status/audio-api-inventory.md) and
  [minimal dependency requests](status/audio-dependency-requests.md)

- [Integration target/context lint plan](plans/2026-10-15-quality-contexts.md): actual-source coverage, incomplete contexts and remaining gates.

- [Partial-edit probe boundary verification](tools/cleanroom-integration.md#partial-edit-probe-boundary-correction): original edit coverage and two reproduced directive cases.

- [Production-source lifecycle integration](tools/cleanroom-integration.md#refresh20-merged-production-verification): Pool movement, real Thread imports and enabled/disabled assertion execution without library overlays.

- [Refresh21 compiler integration requests](compiler/integration-refresh21-requests.md): original enum dependency reduction, repaired bootstrap primitive and classified full-suite gaps.

- [Catalogue output and compile-only verification](tools/stdlib-runtime-tests.md#independent-catalogue-channel-and-compile-only-checks): separate exact diagnostic channels and preserved failure status.

- [Refresh22 combined verification](tools/cleanroom-integration.md#refresh22-combined-checkpoint): workspace counts, production allocator artifacts and remaining gates.

- [Foreign native storage reexports](compiler/foreign-data-reexports.md): reviewed immutable-binding boundary, general filtered export fix and actual Socket identity verification.

- [Extensions assertion-profile runner](../tests/extensions/run-assert-profiles.py): actual Io worker preparation/completion under enabled and disabled assertions, with unchanged production source hashes.
# Native Mac integration measurement

See [Code source reconstruction](compiler/code-source-reconstruction.md) for
general AST serialization, public edit handling, original regressions and
explicit remaining syntax/ownership boundaries.
See [custom iteration controls](compiler/custom-iteration-controls.md) for array
removal forwarding and independent operator-name diagnostic coverage.
See [retained body ownership](compiler/retained-body-ownership.md) for canonical
member/callback editing, contextual enum members and native/WASI evidence.
See [type-instantiation flags](compiler/type-instantiation-flags.md) for reviewed
array/directive mappings, variadic composition and mutable type round trips.

The [machine-readable baseline](status/mac-integration-baseline.json) records the
exact candidate snapshot, commands, environment, counts and failure names.
Snapshot `90fea8ab` has 822 workspace passes, seven failures and one ignored test;
the 408-case catalogue remains red. This is a measurement, not completion.
Focused follow-up: seven reviewed Thread fixtures pass interpreter/native ARM64
O0 and strict lint; original thread and native backtrace regressions pass. See
[integration evidence](tools/cleanroom-integration.md) for the coverage changes
and unresolved old negative assumptions. Full baseline remains red.

The three original sandbox deadlock/timed-wake scenarios also pass through the
compiler's actual private provider; native-only public Thread remains guarded.

The full CLI suite now reports17 passes/one missing-Default_Metaprogram failure.
Unsupported Natvis emission is explicitly diagnosed; it remains unimplemented.

Catalogue prepared inputs now execute real file/resource/font cases. Native SDL
ABI/runtime tests pass with source-built probes; interpreted WM initialization
still fails and remains reported. OS-specific unavailable cases are not passes.

Corrected compiler storage/immutability regressions pass58 core/module checks.
The unsupported live-constant-alias implementation and history remain excluded.

System's reviewed x64 Machine_X64 alias now resolves canonically on Windows;
host behavior and dependent unused-code source checks pass, without a Windows
runtime claim.

Unix Socket.select now uses canonical POSIX.timeval through private ABI glue;
real host select/canary, strict unused-code and original C ABI checks pass.

- [Verified Mac graphics catalogue repairs](native/mac-graphics-catalogue-repairs.md): real clipboard, notification, fullscreen and context lifecycle fixes with explicit native limits.

Merged Extensions:28 native ARM64 O0 passes;27 interpreter passes and one Json
numbers timeout, recorded as failure. Native clipboard fixtures now require
restoration and bounded child cleanup; no clipboard substitute is used.

- [Mac compiler catalogue repairs](compiler/mac-catalogue-repairs.md): reviewed fixture migrations, live AST links and correct union alignment.
- [Rejected-overload lint audit](compiler/rejected-overload-audit.md): speculative instances stay outside declaration audit roots; selected and invalid concrete bodies remain checked.

- [Developer tools status and remaining APIs](status/developer-tools-remaining.md): canonical model emission, native Check/Debug subsets and unresolved integration gates.

- [Generator model helpers](generator-model.md) and [native platform helpers](generator-platform.md): shared canonical Generator/Token_Info/Library_Symbols, supported subsets and ownership limits.

Shared candidate7467b49d combines the new committed producer snapshots. All19
CLI tests and original SDL WM now pass after general Mac foreign-call affinity
repair. Json's unchanged interpreter numbers suite completes in149.449 seconds;
the earlier120-second timeout is retained. Full aggregate replay remains required.
- [Android project generation](android-project.md): canonical project models, escaped Gradle/manifest generation and explicit build dependencies.
- [Android host orchestration](android-orchestration.md): SDK/NDK discovery, launcher/device helpers, integrated verification and exact signing-dependent drafts.
- [Android APK build orchestration](android-build.md): Gradle and SDK command orchestration, ZIP packaging, signing dependencies and verification limits.

- [Android binary manifests](android-manifest.md): valid AXML writer, UTF-8/UTF-16 reader, canonical APK parsing and system-zlib decompression.

- [Basic LSP resolution verification](tools/lsp-basic-resolution-verification.md): reported Basic.temp and exit(1) checks on the merged server

- [Shared stdlib worktree](tools/shared-stdlib-worktree.md): source ownership, shared commits and independent build/test outputs
- [Shared portable verification](tools/shared-portable-verification.md): combined ASN.1, Socket, JKS and APK source checks and native baseline limits
- [Generator model shared audit](status/generator-model-shared-audit.json), [tools shared audit](status/stdlib-tools-shared-audit.json), and [bindings shared audit](status/bindings-shared-audit.json): pre-adoption producer inventories; these are not combined passing results


ASN1/Socket shared-tree verification now uses current production dependencies and
a fresh source-built compiler, with separate worker outputs and no old overlays.
[The scoped handoff](portable-asn1-socket-handoff.md) records 54 actual host
executions, 34 supported-platform source/lint gates, and the blocked Android
compiler gate. [The API inventory](status/asn1-socket-remaining.json) keeps six
Android rows partial and other native platforms unverified.

- [Retained portable shared audit](status/stdlib-shared-audit.json): per-API current audit distinct from 929 historical bookkeeping rows
- [Current shared integration baseline](status/shared-integration-baseline.json): exact candidate, commands, measured counts and unresolved failures
- [Shared IDE verification](tools/shared-ide-verification.md): strict scanner, independent definition regressions and separate actual-server protocol gate
- [Shared generator checkpoint](tools/shared-generator-checkpoint.md): canonical lifecycle, dependent arrays, native macro evaluation and honest remaining emission domains
- [Generator macro helpers](generator-macro-shared.md): seven reviewed public helpers and guarded native evidence
- [C variadic header validation](compiler/c-variadic-header-validation.md): ordinary diagnostics replace invalid-header pack-lowering panic while valid native ABI calls remain checked


[Essential operations inside optional assertions](optional-assert-side-effects.md)
records a finite read-only handwritten stdlib audit, independent disabled-default
text/keymap reproductions, precise owner routing, and explicit pure/diagnostic
non-defect classifications. [Its scoped findings](status/optional-assert-side-effects.json)
include exact file/line/source hashes and required platform limits.

- [Stdlib completion backlog](tools/stdlib-completion-backlog.md): exact owner/API gaps, dependencies and acceptance evidence
