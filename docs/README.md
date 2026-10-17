# Developer documentation

- [Reviewed runtime instrumentation defaults](portable-runtime-instrumentation.md):
  real system-entry calls, no-entry configurations and
  [27 scoped checks](status/shared-runtime-hooks-checkpoint.json).

- [ABI8 allocator metadata reservation](native/rpmalloc-metadata-reservations.md):
  transactional SDK ownership, allocation-free operation scopes and
  [scoped verification](status/shared-rpmalloc-metadata-checkpoint.json).
  ABI8 source requires rebuilt ABI8 SDK/callback assets; ABI7 candidates remain historical.

- [Basic underscore classification repair](portable-ascii-predicate-fix.md):
  exhaustive reviewed ASCII byte-domain assertions and
  [focused verification](status/shared-ascii-checkpoint.json).

- [Signed Apollo fractional conversion repair](status/shared-time-float-checkpoint.json):
  nonzero positive/negative femtoseconds, checked whole-second range and
  retained time/calendar assertions; final combined snapshot follows active fixes.

- [Canonical bindings and bridge generation](stdlib-tools-shared-emission.md):
  one model/run lifecycle, returned-buffer ownership, strict native type matching,
  declaration emission and remaining unsupported domains.

- [Portable collection checkpoint](status/shared-portable-collections-checkpoint.json):
  physical-slot [Bucket_Array removal](portable-bucket-array-removal.md),
  canonical [Soa/Xar operators](portable-soa-xar-audit.md),
  [sorting contracts](portable-sorting-audit.md) and
  [relative pointers](portable-relative-pointer-audit.md), with focused Mac evidence.

- [Canonical provider logical library spelling](generator-platform-logical.md):
  physical image identity versus generated Jai names, SDK/native consumers and
  current shared immutable verification.

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

- [Shared rpmalloc verification](tools/shared-rpmalloc-verification.md): mandatory diagnostic domains, real SDK instrumentation and explicit remaining configurations.

- [Shared MPEG macro verification](status/shared-mpeg-macros-verification.json): three reviewed public macros, real caller effects and enabled/disabled assertion-default controls; Walk_Table remains compiler-dependent.

- [Developer runtime packaging](tools/developer-runtime-packaging.md): real relocated LLVM consumers, dynamic dependency notices and explicit resource/distribution limits.
- [Pinned shared candidate06e8ea96](status/shared-candidate-06e8ea96.json): exact compiler hashes, committed stdlib tree, native asset search order and coordinator replay environment.

- [Unmapping allocator](portable-unmapping-allocator.md): real protection-failure ownership retention, Windows VM backend and [scoped integration verification](status/shared-unmapping-verification.json).

- [Shared-memory channel audit](portable-shared-channel.md) and [CRC/Base64/PCG contracts](portable-codec-contracts.md): real separate-process IPC, codec/RNG vectors and [corrected runner verification](status/shared-channel-codec-verification.json), with cleanup and native-platform limits.

- [Native debugger startup isolation](native/debug-info.md): unchanged breakpoint/backtrace/value assertions with user startup files disabled; [exact Mac verification](status/debugger-hermetic-launch.json).

- [Workspace object targets](compiler/workspace-object-targets.md): actual seven-target object headers, explicit target precedence and frontend ABI boundaries.

- [Live GL projection verification](status/shared-gl-projection-verification.json): public unqualified procedures retain the canonical mutable table's storage and loader updates.

- [Canonical generator runner](generator-run-shared.md), [model](generator-model.md) and [session](generator-session.md): coherent canonical ownership, semantic rejection and dependency cleanup, with [proposed-commit integration verification](status/shared-generator-run-model-verification.json).

- [Combined compiler checkpoint](compiler/shared-mac-checkpoint.md): independent producer-chain adoption, host-clock object proof, code-record defaults and current native debugger limitation.

- [Pinned combined candidate10d9b3f8](status/shared-candidate-10d9b3f8.json) and [fresh baseline status](status/shared-baseline-10d9b3f8.json): committed compiler/library identity, complete development native assets and serialized native ARM64 verification.

- [Canonical public operator facades](compiler/public-operator-facades.md): reviewed overload reexports, preserved identities/visibility and exact compile-time fixture classification.

- [Protected formatter inputs](tools/formatter-protected-inputs.md): real token/idempotence checks on mutable copies while original bytes and permissions stay protected.

- [GL assertion effects](portable-gl-assertion-effects.md) and [Cocoa assertion effects](portable-cocoa-assertion-effects.md): actual SDK operations, failure ownership and [scoped producer/integration verification](status/shared-rendering-cocoa-checkpoint.json).

- [Hash_Table and Walk_Table](portable-hash-table-audit.md): caller-visible slot/control semantics and [38 current integration checks](status/shared-hash-table-walk-verification.json), retiring the named-while dependency.

- [Allocator runtime domains](native/rpmalloc-runtime-policies.md) and [reviewed Objective-C helpers](native/objc-runtime-helpers.md): canonical ownership/live callbacks, actual ABI boundaries and [scoped native checkpoint review](status/shared-native-domain-checkpoint.json).

- [Immutable local captures](tools/immutable-local-captures.md): general input/output overlap checks replace a role-specific path exception.

- [Generator native helper relocation](tools/generator-helper-relocation.md): proper install identity and an actual moved-package native consumer.

- [Pinned ABI7 candidate a97d0f6f](status/shared-candidate-a97d0f6f.json): exact committed production source/tool identity and actual simultaneous-domain interpreter/native proof on assigned T7 storage.

[SIMD byte/lane contracts](portable-simd-audit.md) and [Basic time/calendar audit](portable-time-shared-audit.md) distinguish native host evidence, source-only targets and unresolved semantics. [Original fixture premise migrations](status/shared-portable-premises.json) preserve all assertions while using lexical operators and trusted private-unit placement.

[Canonical generator integration counts](status/shared-generator-checkpoint.json) pin producer manifests, exact focused commands, retained failed helper-identity assertions and the still-blocked Android target. [Reviewed AppKit contract](specs/stdlib/modules/appkit.md) is available to the bindings owner; its new surface is not yet included in the pinned a97 candidate.

[Available frozen553 build identity](status/shared-candidate-553afc98.json) records4439committed inputs,258paired ABI7native assets, actual source-built tools,69focused production checks and9relocated/license checks. Later signed-time/ASCII commits are separate; broad baseline remains deferred until active known fixes are combined.

[Compact current verification report](status/shared-current-verification.json) separates frozen553 checks, later committed fixes, deferred broad commands and unresolved fixture/platform gates.

Shared integration checkpoints: [typed struct arguments](status/shared-typed-generic-checkpoint.json), [named parameter completion](tools/lsp-named-arguments.md), [reviewed record defaults](compiler/derived-record-defaults.md), [Hash/Math scoped proof](status/shared-foundations-checkpoint.json), and [Linux link plugin](portable-linux-build.md). The [current batch ledger](status/shared-integration-batch.json) separates committed checks from active dependencies and coordinator-owned gates.

Latest available immutable ABI8 compiler/library/native [candidate receipt](status/shared-candidate-8fa08957.json) and [integration commands/limits](tools/cleanroom-integration.md#immutable-abi8-shared-candidate-8fa08957). This current committed checkpoint excludes active producer repairs; whole baseline/corpus gates remain pending.

Current known cohort: [loaded anonymous imports](compiler/loaded-anonymous-imports.md), [canonical Table order](foundations-table-scope.md), [generator emission](stdlib-tools-shared-emission.md), [canonical model defaults/comparison review](status/shared-model-template-comparison-checkpoint.json), [Runtime audit](portable-runtime-core-audit.md), [Meow audit](portable-meow.md), and [partial Minimal workflow](portable-minimal-metaprogram.md). Current Minimal92/4 remains red; the [batch ledger](status/shared-integration-batch.json) tracks critical AppKit/projection handoffs before the final candidate and coordinator400.

Minimal’s supported LLVM Mac workflow now passes [96 guarded checks](status/shared-minimal-verified-checkpoint.json) after the [options-only compiler repair](compiler/workspace-options-projection.md). Historical80/6 and92/4 receipts remain unchanged. [Known-batch status](status/shared-integration-batch.json) currently waits AppKit/Cocoa for the final immutable pin; original400/runtime/Focus remain coordinator-owned.

Ready final cohort: [AppKit/Cocoa](status/shared-appkit-cocoa-checkpoint.json), [declaration-bound callable metadata](status/shared-callable-metadata-checkpoint.json), and [all14 reviewed matrix operators](foundations-math-matrix-operators.md). The [current ledger](status/shared-integration-batch.json) separates ownchecks, qualifiedproducerproof, heldURIwork androot400/Focus acceptance.

- [Current shared integration checkpoint](status/shared-integration-batch.json), [603 production receipts](status/shared-candidate-60321c1d.json), [constant evaluation repair](status/shared-callable-constant-checkpoint.json) and [paired public operator filters](status/shared-operator-filter-checkpoint.json). Full original400/Focus remain coordinator-owned; no release acceptance is inferred.

- [Pinned beec9491 compiler/library/native candidate](status/shared-candidate-beec9491.json) and [actual POSIX stream storage proof](status/shared-posix-stream-checkpoint.json): exact source/binary/environment receipt for coordinator original400 replay. Final focused gates/catalogue are recorded separately.

- [Exact beec9491 Mac baseline](status/mac-beec9491-baseline.json): scoped production gates pass; complete433-case catalogue failures remain classified with [reproducers and proper followups](integration-requests.md). Native/GUI lease released to coordinator; original400/Focus and Windows/Linux native verification remain open.

[Integrated check-only IR lifetime repair](status/shared-ir-lifetime-checkpoint.json) records the exact822 patch review and two passing backend-boundary regressions.

[Reviewed AppKit flags and Metal Render inheritance](native/appkit-metal-corpus-dispositions.md) records exact public shape, real SDK evidence and remaining protected application replays.

[Post-beec LSP integration](status/shared-lsp-followup-checkpoint.json) records exact patches,301 merged tests,52 overlapping followup checks,105 core regressions, reconciliation and open native/scanner gates.

[Exact debug poison and passive entry integration](status/shared-json-entry-checkpoint.json) records supported compiler changes, scanner ownership transfer, retained timeout and final replay requirement. [Full workspace handoff](status/post-beec-workspace-request.json) requires coordinator-owned opaque corpus inputs.

[Fresh60d2ff19 combined candidate](status/shared-candidate-60d2ff19.json) pins actual tools and ABI8 assets; [the ledger](status/shared-integration-batch.json) separates focused checks from required full workspace/original corpus/Focus/native CI acceptance.

[Exact60d final scoped baseline](status/mac-60d2ff19-baseline.json) records468passingRusttests and real native controls, retained Json timeout/widget failures, released native slot and outstanding coordinator gates.

[Reviewed Metal Color attachment embedding](native/metal-render-pass-attachments.md) has22verified source/link checks; the [integration receipt](status/shared-metal-attachment-checkpoint.json) keeps allthree native programs explicitly pending.

[Current pointer-subscript integration](status/shared-pointer-subscript-checkpoint.json) records exact fb7 adoption, preserved visibility,8/7 and14/1 failures,82passingfocusedchecks and the separate missing Bucket_Array implementation.
- [HTTP conformance native dependency setup and CI failure triage](tools/http-native-setup.md)

[Conditional readiness and error retention](compiler/conditional-readiness-errors.md) records the generic speculative-expansion repair, corrected official positive, preserved rejected controls and protected-replay boundary.
- [Bucket_Array physical pointer indexing](foundations-bucket-pointer.md): reviewed canonical operator, address/hole/privacy source proofs and pending native gates.

Current ready [30901830 candidate](status/shared-candidate-30901830.json) combines pointer/Color/HTTP/readiness and real Bucket_Array indexing; [focused receipt](status/mac-30901830-focused.json) distinguishes97Rust passes,12actualO0/O2programs, preserved transport failure and pending broad acceptance.

[Basic filesystem platform guards](basic-filesystem-platform-guards.md) select the reviewed four-API target surface without Runtime classifier cycles; exact two-line repair and qualified evidence retained.

[HTTP/TLS target repair](extensions/http-tls-target-repair.md) fixes Windows temporary allocation while preserving the four failed source-matrix gates and unresolved clock/provider questions.

[Required WASM tour thread provider](compiler/wasm-workspace-triage.md) uses the private scheduler with real workers; full-tour GPU/Window_Type remains red, and unchanged60d formatter replay passed after root metadata correction.

[Linux Clipboard](portable-clipboard-linux.md) implements native X11 selection transfer with exact helper pins; normal ephemeral-Xvfb and relocated-package CI are wired, Linux runtime remains unverified locally.

[Native HTTP date adapter](extensions/http-native-date-adapter.md) preserves the reviewed helper body/native API; portable WASM date support, real Mac3case proof and retained Windows/WASM failures are separate.

[NSWindow backing-scale alias](native/appkit-window-backing-scale.md) retains the canonical getter signature and procedure identity; real non-window SDK proof is distinct from pending GUI acceptance.

[File_Watcher platform backends](file-watcher-platforms.md) implement inotify and overlapped Windows ownership; actual platform CI is wired, source/decoder proof remains distinct from native notification acceptance.

- [S128 comparisons and inherited Apollo operators](specs/stdlib/README.md): signed word-boundary semantics, generic unary repair, exact proof and limits; [questions](specs/stdlib/s128-comparison-questions.md).

- [Typed polymorphic struct argument inference](compiler/typed-struct-arguments.md): parameter identity, preserved declared types, independent RED/green checks and [oracle questions](specs/typed-struct-argument-questions.md).

[HTTP WASM termination controls](extensions/http-wasm-test-termination.md) retain failure counters/nonzero exits through real WASI/browser hosts; portable codec assertions remain unconditional and live-clock checks remain native.

[Windows time-zone ABI](windows-time-zone-abi.md) restores the real SDK record with complete layout/field source checks and native MSVC comparison jobs; Windows execution remains pending.

[Metal texture resource promotion](native/metal-texture-resource.md) retains the exact unnamed using edge, canonical method signatures and qualified offscreen SDK proof; no implicit-pointer conversion or #as substitute.

- [HTTP CI case coverage](tools/http-ci-case-coverage.md): exact twelve-case dispatch; seven native Io cases remain unavailable on WASM, never passed.

- [Code insertion lexical scopes](compiler/insert-declaration-scope.md): corrected unbraced declaration publication and explicit-block privacy; [merged checks](status/shared-insert-scope-checkpoint.json).

- [Windows cross-build stream accounting](status/shared-windows-cross-streams.json): exact status/stdout/stderr contract and timeout controls; actual Windows execution pending.

- [Current merged repair ledger](status/shared-known-cohort-progress.json): exact scoped counts, remaining canonical Basic array closure and platform/runtime gates.

- [Basic program parameter identity](basic-program-parameter-identity.md): corrected reviewed header, nominal identity/configuration controls and retained negative; native proof qualified separately.

- [Canonical Basic array ownership](status/shared-array-owner-source-checkpoint.json): original shared array bodies declared in public Basic context, with private allocator dependency closure; remaining custom configuration failure retained.

- [Canonical Basic array declaration context](specs/stdlib/README.md): shared implementation ownership, overload injection, private allocation closure and [remaining questions](specs/stdlib/basic-array-context-questions.md).

- [Windows native provider environment](status/shared-windows-provider-environment.json): matching CPU helper, ordinary DLL loading and file-identity CI checks; native Windows acceptance pending.

- [User-deferred capability accounting](tools/user-deferred-capabilities.md) reports native Windows console-color consumers as unavailable, preserves ANSI/native-required checks and keeps failures separate from deferrals.

- [Frozen e0e287ea candidate](status/shared-candidate-e0e287ea.json) and [Mac focused results](status/mac-e0e287ea-focused.json): exact source/tool/native identity, corrected construction checks and retained genuine compatibility/platform failures.

- [Windows WGL consumer review](tools/windows-wgl-consumer-review.md) and [exact handoff status](status/shared-wgl-consumer-review.json): validated proposed DLL/lint/wheel scope, adoption held for matching producer receipt; no new native Windows proof.

- [Integrated Windows WGL consumer companion](status/shared-wgl-consumer-adoption.json): exact Mesa app-local deployment and explicit lint roots, 13 setup/provider tests, preserved issue1 gates; actual Windows replay pending.

- [Typed and retained runtime configuration integration](status/shared-typed-retained-adoption.json): exact stable compiler repairs,114focused tests, source hashes and pending matched-tool/public-constant/native verification.

- [Retained startup native replay](status/shared-typed-retained-native.json): committed combined compiler ARM64/bare-wasm64 O0/O2 execution and preserved harness environment failure.

- [Public runtime constant integration](status/shared-public-runtime-constant-adoption.json): canonical provenance, preserved negative contracts and292combined compiler tests.

- [Reviewed GL debug policy](native/gl-debug-output-policy.md): exact core/ARB/unsupported behavior and [integration proof boundaries](status/shared-gl-policy-adoption.json).

- [Final combined0c0d316e candidate](status/shared-candidate-0c0d316e.json), [focused verification](status/mac-0c0d316e-focused.json), [ready/active/gate ledger](status/shared-final-cohort-ledger.json) and [coordinator full-workspace request](status/shared-0c0d316e-workspace-request.json).

- **Current final candidate:** [f1a7e3a8 corrected reviewed-oracle receipt](status/shared-candidate-f1a7e3a8.json), [verified product/source equivalence](status/shared-final-source-equivalence.json), [coordinator workspace request](status/shared-final-workspace-request.json). Supersedes initial0c oracle capture; actual tools/native/product bytes unchanged.

- [Clipboard Linux toolchain provisioning](clipboard-linux-toolchain.md): isolated pinned components,9configuration tests, actualLinuxCI runtime/package rerun still required.

- [Normal CI orchestration closure](tools/ci-orchestration-closure.md): full enabled SDL Linux dependencies, runner ownership and retained failure classifications.

- [CI runner/provider readiness](status/ci-orchestration-ready.json): production-library checks, exact fixture platforms/inputs, actual ABI8 provider wiring and outstanding native/source verification.

- [CI diagnostic companion](tools/ci-diagnostic-companion.md): exact Clipboard adoption, bounded failures and focused verification; [Linux Clipboard diagnostics](clipboard-linux-build-diagnostics.md).

- [Windows cross CI triage](native/windows-cross-ci-triage.md): platform/provider selection, stale fixture contracts and eight retained source failures; [integration review](status/windows-cross-integration-review.json).

- [Io Linux accept-burst diagnosis](extensions/io-linux-accept-burst-diagnosis.md): failure-only stage evidence, unchanged load/assertions and [integration review](status/io-burst-integration-review.json).

- [Windows interpreter import providers](compiler/windows-import-providers.md): COFF metadata and shared SDK discovery; [integration review](status/windows-import-integration-review.json).

- [Windows D3DCompiler provider diagnosis](native/windows-d3dcompiler-provider.md): actual SDK selection and required native probes; [integration review](status/windows-d3dcompiler-integration-review.json).

- [LLVM-free WASM CI triage](compiler/wasm-ci-triage.md): real C-bridge I/O fixtures, native-only guards and required graphical tour gap; [integration review](status/wasm-ci-integration-review.json).

- [Basic null-format triage](basic-null-format.md): confirmed public-input controls and separate compiler default-Type/boxing defects; production formatter and invalid-descriptor assertions remain unchanged.

- [Default Type images](compiler/default-type-images.md): canonical void descriptor defaults and reviewed golden correction; [integration receipt](status/default-type-integration-review.json).

- [Editor audit integration](tools/lsp-editor-audit-integration.md): effectful-header ordering and selected sketch closure; exact merged hashes and verification limits.

- [Current CI cohort ledger](status/ci-known-cohort-ready.json) and [committed5b25 candidate](status/shared-candidate-5b25c83a.json): exact tools/assets, ready/active/blocker dispositions and deferred original acceptance.

- [Cross-source compiler phases](compiler/cross-source-phases.md): single-selection context loading, target-library isolation, preserved REDs and pending public contracts; [combined integration proof](status/numerics-cross-source-integration.json).

- [Combined c0c+5b candidate718](status/shared-candidate-7187647b.json): matching tools/native assets and exact focused results for root’s next original400 gate.

- [Linux Clipboard static dependency closure](clipboard-linux-static-link-closure.md): required private X11/pthread retention, causal link-argv evidence and real Linux runtime gates.

- [Conditional callable joins](compiler/ifx-callable-joins.md): official else-header rule, narrow metadata repair, causal tests and the separate Io workspace configuration reducer.

- [IDE preflight failure reuse](tools/lsp-second-preflight-integration.md): exact second-repair source guards, causal invalidation controls and pending unchanged-bound original timing gate.

- [Four-ready combined receipt](status/shared-candidate-2a29b054.json): exact source/tool/ABI8 identity, focused counts and remaining native/original gates.

- [Third-party native preparation](native/sdl-upstream-native-preparation.md): real source-built helper closure, strict XML result parsing and pending native acceptance.

- [Caller font and selector source fixtures](native/font-selector-input-fixtures.md): strict real inputs and metadata discovery; native SDK gates remain separate.

- [Real Windows resource fixture](native/windows-resource-fixture.md): target RC/PE construction, durable source hashes and actual Windows runtime still required.

- [Linux allocator feature/header closure](native/linux-rpmalloc-feature-headers.md): strict C11 declarations and all23 ABI8 profiles; actual Linux link/runtime still required.

- [CI companion with fresh native assets](status/shared-candidate-940fc8cf.json): exact940 source/tool identity,174 source-paired assets and remaining original/platform gates.

- [Linux pthread storage](native/thread-linux-pthread-storage.md): ARM64 SDK layout repair with target-native verification still required.

- [AppKit receiver repair](native/appkit-receiver-repair.md): preserved signatures with SDK-correct receivers and scoped runtime evidence.

- [Formatter fixture inputs](tools/formatter-fixture-inputs.md): strict source metadata handling and required CI readonly acceptance.

- [Platform-I/O target accounting and Windows path oracle](native/platform-io-target-accounting.md): explicit per-fixture capabilities and retained native checks.

- [Interpreter pthread names](compiler/interpreter-pthread-names.md): checked scheduler identity translation and explicit unsupported execution modes.

- [Required WASM tour providers](compiler/wasm-tour-providers.md): real private canvas/filesystem protocols and unchanged required tour assertions.

- [IDE import discovery](tools/lsp-import-discovery.md): guarded filesystem miss reuse, functional assertions and opt-in unchanged60s budget.

- [Linux observer capture retention](tools/io-observer-retention.md): exact three-file failed-run evidence, with workload and failure gates unchanged.

- [Isolated foundation configuration fixtures](foundation-fixture-configuration.md): valid single-program positives and separately retained origin-negative failures.

- [Foundation target capabilities](tools/foundations-target-capabilities.md): complete public lint contexts, per-case required targets and retained runtime evidence.

- [Formatter runtime fixture evidence](tools/formatter-fixture-runtime.md): producer staged original native/WASM acceptance with integrated replay still required.

- [SDL canonical fixtures and SDK header repair](status/sdl-current-cohort-integration.json): unchanged private ABI assertions, real generated SDK header and platform-qualified proof.

- [Program parameter origin](compiler/program-parameter-origin.md) and [Windows driver/linker options](native/windows-link-options.md): exact independent repairs; merged origin7/backend25 tests pass. Actual Windows runtime remains CI-owned.

- [GetRect target contracts](specs/stdlib/README.md): canonical Linux Clipboard dependency, explicit unavailable records, retained pure numerical WASM and private-Xvfb runtime wiring. [Integration qualification](status/numerics-target-contract-integration.json).

- [Audio native integration](features/audio-native-integration.md): retained strict runtime receipts, real ALSA CI provider configuration and T7-aware build roots. [Supplied producer evidence](status/audio-ci-provider-evidence.json) is Mac-only; Linux/Windows execution remains required. Windows environment repair transport is pending.

- [Audio Windows environment ownership](status/audio-windows-env-request.md): reviewed `_dupenv_s` copy/free and explicit acquisition errors; all3 source/test hashes and3 other nonindex postimages verified;2 status documents retain accepted shared context. Mac/MinGW supplied proof is distinct from required MSVC/Windows CI.

- [Cross-host ABI8 callback preparation](native/windows-cross-host-providers.md): complete real host provider closure before compile-time foreign calls; target PE and Windows runtime qualifications remain separate.

- [Generator native setup and reviewed C++ ABI fixtures](generator-runtime-ci.md): stable setup17/C++9 source cohorts hash-verified together; five orchestration controls pass. Supplied actual Mac C++ consumers pass, while the required original C-model enum conversion remains RED and is assigned separately. No whole generator CI acceptance claimed.

- [Http fixture target review](extensions/http-app-fixture-target-review.md): exact two App/Client positives retain native runtime assertions and explicit WASM unavailability; three metadata controls pass against adopted production selection, portable codec coverage unchanged. Native replay still required.

- Latest root-reviewed [Input](specs/stdlib/modules/input.md) and [Runtime_Support](specs/stdlib/modules/runtime-support.md) refinements are retained: platform hook availability/unknown external-event ownership and default context allocator capability boundaries. No new exports or guessed behavior added.

- [Windows x64 context alignment](native/windows-context-alignment.md): exact two record annotations, five ready source/doc hashes verified; real source/SDK evidence remains distinct from required native MSVC replay. Narrow formatter corrections cover the newly guarded WebGPU helpers and two ready fixtures only.

- [Windows target provider closure](native/windows-target-provider-plan.md): real23 ABI8 SDK/callback pairs plus4 context/counter archives, target-machine checks and all-object PE link; x64 MinGW compilation/link PASS, runtime/ARM64/MSVC pending. Direct stdio and scoped Windows SDK inline source repairs retained; no disabled profiles. [Exact scoped receipt](status/windows-target-binding-providers.json).

- [Canonical global enum lookup](generator-model-enum-scope.md): lexical-first search reaches the active Generator Block with owner/bounds checks; eight exact ready hashes verified. Supplied Mac SDK enum/callback controls pass; the separate required incomplete opaque-printer gate remains RED and assigned.

- [Nonheap numeric and tagged-union formatting](tools/toml-nonheap-formatting.md): all14 exact ready source/test/doc hashes verified against admitted patch; ten receipt-policy checks pass. Producer proof preserves all four original nonheap cases and reports33 actual Mac interpreter/native executions; combined platform replay remains required. Root-reviewed Basic refinement and sanitized oracle retained.

- [Import argument group preservation](compiler/import-argument-groups.md): exact follow-up on accepted origin validation, retaining both IDE discovery paths. Current merged93 core/101 LSP checks, Rustfmt, warnings-denied clippy and fresh tool build pass; [integration receipt](status/import-groups-integration.json). Root public child-workspace protocol replay remains required.

- [WGL public/private lint contexts](status/wgl-lint-public-context.md): complete required-file coverage, prior DLL/consumer checks and four existing fixtures retained. [Adaptation receipt](status/wgl-public-context-integration.json); eleven policy checks pass, actual Windows graphics replay required.

- [Upstream Linux clipboard closure](tools/upstream-linux-clipboard-closure.md): real C helper preparation plus mandatory source/artifact receipts before application links; eight policies pass, actual Linux consumer replay required.

- [Storage and native elapsed-clock selection](tools/foundations-storage-clock.md): original mixed fixture remains byte-identical/native-required; identical storage assertions retain separate WASM coverage. Four ready source/doc hashes verified; current catalogue regression passes. Producer runtime proof is separately attributed.

- [Portable-data public-context lint](tools/portable-data-context-lint.md): four exact ready hashes verified, four policy controls pass; unchanged runtime/SDK inventories and seven target privacy controls retained. Supplied71 complete lint contexts are separate producer evidence; current Windows native replay remains required.

- Host provider caches now require source-paired ABI8 receipts; filename-only/ABI7 reuse is rejected. SDK input guards include .inc/patch and callback-context sources. Eleven selection policies pass; real native CI remains required.

- [Canonical runtime allocator alias](runtime-default-allocator-alias.md): exact reviewed Default_Allocator entity, public visibility and default-context identity; obsolete disconnected allocator removed. Seven owned postimages/deletion and three reviewed contracts verified. Producer18 actual ARM/x64 program modes pass; original Windows x64 TOML remains required CI. No native C inputs changed.

- [Input original fixture adoption](tools/input-original-adoption.md): six exact source hashes, persistent production resize storage and signed-wheel/window/payload assertions retained. Four source checks pass; prior nine Mac runtime outcomes are separately pinned, current combined GUI smoke remains root-owned.

- [Complete Windows target provider receipt](status/windows-target-binding-providers-complete.json):52 real archives including retained SDK/storage probes, all-object PE link and target-machine verification; no Windows runtime claim.

- [Allocator capability fixture qualification](status/allocator-caps-integration.json): exact reviewed optimized/fallback target bits; all allocation/identity checks retained. Independent of the broader catalogue batch and its withdrawn emitted-WASM artifact evidence.

- [GetRect sanitizer selection](specs/stdlib/README.md): four reviewed module fixtures share the canonical target policy; six indexed-cohort dispatch controls pass. Scoped index contains only numerics hooks; foundation/Debug peer work remains unstaged. This is selection proof, not sanitizer/native execution.

- [IO child MAIN configuration](extensions/io-http-child-main-configuration.md): exact eight ready postimages verified against admitted packet; original workload/assertions retained. Supplied focused proof is distinct from current full native/HTTP/LSan replay.

- [HTTP route/App ownership](extensions/http-router-ownership.md): exact eight ready postimages verified against admitted packet; original workload/assertions retained. Supplied focused proof is distinct from current full native/HTTP/LSan replay.

- [Debug fixture target contracts](tools/debug-fixture-targets.md): six ready source/doc hashes verified; exact ownership fixture native provider stays required. Portable WASM controls retained; unconditional set_report_mode API repair remains separately active, never inferred absent.

- [Catalogue mode contracts](tools/foundations-catalogue-mode-contracts.md): explicit per-mode dispatch and required rejection accounting; emitted-WASM durability remains open.

- [Clean-room integration](tools/cleanroom-integration.md): real workspace helper prerequisites and exact source/native candidate accounting.

- [Generator opaque printer](generator-opaque-printer.md): canonical live-cursor completeness proof, pointer support, and required remaining C-model failures.

- [SDL Python CI setup](status/sdl-python-matrix-833.md): pinned real headers, explicit scratch, and 60 scoped checks; native acceptance separate.

- [WASI output boundary](tools/wasi-file-boundary.md): public SDK late-zero reproducer and withdrawn durability proof; production repair remains required.

- [POSIX crash provider executable lifetime and setup](native/portable-crash-executable-lifetime.md): real main-image/DSO lifetime handling; Linux replay required.

- [Workspace provider disposition](compiler/workspace-matrix-provider-disposition.md): unchanged child/grandchild tests and the required real crash-provider bootstrap.

- [Debug report mode](tools/debug-report-mode.md): unconditional canonical state setter; Windows native policy remains unresolved.

- [WASM C numeric conversion](native/wasm-c-numeric.md): C-locale/direct precision and general WASI exports; canonical output durability still required.

- [HTTP termination accounting](extensions/io-http-833-evidence.md): exact status/stdout/stderr protocols; Linux connection failures remain unresolved.

- [Integration deviations](status/integration-deviations.md): remaining semantic bounds, staging substitutes and skipped platform verification for the combined ready cohort.

- [Immutable combined candidate213fa751](status/shared-candidate-213fa751.json) and [ready/active/blocker ledger](status/current-ready-cohort-baseline.json): exact tools/assets, scoped counts and root gate commands.

- [Windows symbol import selection](compiler/windows-symbol-import-selection.md), [archive padding](compiler/windows-import-archive-padding.md), [atomic permissions](compiler/atomic-permission-checks.md), and [random parity](compiler/x86-random-parity.md): exact ready integration; Windows/Linux native CI remains required.

- [Browser request bundling](tools/playground-request-bundling.md) and [CI browser/path boundaries](tools/ci-browser-and-path-boundaries.md): canonical source identity, size limits, and required failure accounting.

- [Binding verifier private roots and SDK profiles](native/linux-833-binding-verifier.md): preserved type/privacy assertions; source40-to0 producer proof, native Linux/Windows replay required.

- [Reviewed target policies](tools/foundations-833-target-policy.md):21no-DCE checks and exact native dependencies; historical406failure rows are not replayed passes.

- [Installed OpenSSL SDK discovery](tools/openssl-sdk-discovery.md): real SDK probing and canonical X509 inputs; native Linux/Windows package support remains required.

- [Module default forwarding](compiler/module-default-forwarding-boundary.md): written-list identity and preserved canonical storage/type checks; root official verdict confirms distinction.

- [Basic wide-float presentation](basic-wide-float-presentation.md): reviewed float64 conversion with canonical metadata; original Linux native replay required.

- [Windows SDL Python contract repair](status/sdl-windows-policy-ae6.md): native-path ledger checks and explicit generated LF policy; actual Windows replay pending.

- [Linux failed-copy observer](extensions/io-failed-copy-observer.md): bounded numeric capture from original children; workload and failure gates retained, Linux correctness unresolved.

- [Intel Mac SDK development dependencies](tools/mac-intel-sdk-closure.md): matching clang-c/libclang provisioning; actual Intel CI remains required.

- [Linux clipboard consumer closure](tools/linux-clipboard-consumer-closure.md): real helper build/search paths across all general Linux native consumers; runtime replay required.
- [Current matching CI candidate](status/shared-candidate-d9f32f30.json): committed source, tools, ABI8 identities and explicit open gates.

- [Windows dynamic SDK packaging](tools/windows-dynamic-sdk-packaging.md): retain actual import libraries, deploy matching DLLs and reject collisions; native replay and CRT/ffi/Clang closure pending.

- [SDL window-manager failure evidence](tools/sdl-wm-failure-evidence.md): actual driver/provider/result diagnostics on failure, preserving every runtime assertion.

- [Current four-mode runtime report](status/runtime-d9f32f30.json): all451cases completed,1414passes25failures; required RED with exact classifications.
