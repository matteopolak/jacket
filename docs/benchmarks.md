# Compiler benchmarks

## What it is

`jai-bench` measures compiler-stage time, source throughput, allocation counts/bytes, reallocations and maximum live allocation footprint. It provides baselines for later work on allocations and compiler speed.

## How it works

Generated workloads contain 4, 64 or 1,024 procedures, with locals, loops, scalar arguments and short-circuit logic. Benchmarks cover lexing, parsing (including lexing), semantic resolution, LLVM module construction/verification, module construction plus text serialization, and the full source-to-IR pipeline. Backend stages reuse prepared inputs outside timing. `lower_llvm` prepares its LLVM context outside timing; `emit_llvm` includes context creation and serialization.

Two additional benchmarks lex all local reference sources and all pinned recent upstream sources. Corpus loading and decoding happen before timing. These measure lexical work only; no upstream or reference program executes. Corpus parsing, module resolution, native linking and compile-time execution benchmarks must be added as those stages become available.

Divan's allocation profiler wraps the system allocator only in the benchmark executable. Counts distinguish allocation, deallocation and grow operations; do not confuse maximum live bytes with total allocated bytes. Profiling adds timing overhead. These single-thread measurements do not track allocations from unmanaged threads. LLVM allocates through its native allocator, so Rust allocator counters exclude LLVM's internal heap usage. Native heap profiling must be added before making whole-backend allocation claims.

Nested integer ranges have separate `range_lower_llvm` and `range_pipeline` cases at 4, 64 and 1,024 procedures. They exercise reverse traversal, named outer continue/break and verified LLVM construction without changing the original baseline workload. They benchmark compilation, not generated program runtime.

Deferred nested loops have `cleanup_lower_llvm` and `cleanup_pipeline` cases at the same sizes. They include cleanup on named outer exits and track both IR resolution allocations and LLVM construction costs. Original workloads stay unchanged for comparison.

Constant dependency workloads have `constants_resolve` and `constants_pipeline` cases at 4, 64 and 1,024 declarations. They exercise forward dependency worklists, literal evaluation and initialized global storage without changing the earlier generated procedure workload.

Conditional expression workloads have `conditional_lower_llvm` and `conditional_pipeline` cases at 4, 64 and 1,024 procedures. They combine integer and Boolean `ifx` results with nested conditionals and short-circuit control flow, including PHI verification.

Mixed-width workloads have `integer_lower_llvm` and `integer_pipeline` cases at the same sizes. They exercise `u64` arithmetic, checked and intentional narrowing, signed extension, scalar defaults, named argument reordering, and exhaustive Boolean cases. These are separate fixtures; the original comparison workloads remain unchanged.

## How to change it

Shared type cases use 4, 64 and 1,024 graph levels. `type_intern_create` includes registry initialization and new pointer/array/slice/dynamic-array interning; `type_intern_reuse` repeats those lookups in a prepared registry. `nominal_graph_build_freeze` includes nominal reservation, recursive pointer/descriptor definitions, and freeze's value-cycle validation. `type_layout_cold` computes explicit LP64 layouts for nested records in a fresh engine; `type_layout_cached` queries a prewarmed root. Fixture construction and freeze happen outside layout timing. Preflight assertions check canonical IDs, graph cardinality, root size/alignment and field offsets before measurements. These measure shared type infrastructure, not source syntax support or a verified target ABI.

Add a representative, correct workload to `crates/jai-bench/benches/compiler.rs` and a behavior test for the feature it exercises. Avoid timing filesystem reads, source generation, fixture preparation or failed compilations inside a successful-stage benchmark. Keep rejection-path measurements separately named.

Run `cargo bench -p jai-bench --bench compiler --locked -- --test` for smoke checks. After fetching upstream sources, run with `--test --ignored` for the optional upstream benchmark. CI performs smoke checks, without timing gates on a shared runner.

## Configuration

Run `python3 tools/benchmark.py --samples 100 --upstream` using Python 3.11+. It saves raw results, build output, environment/command metadata, input hashes and a compiler-source archive under `artifacts/benchmarks/<timestamp>/`. Omitting `--upstream` uses only the checked-in local reference and generated workloads. The script uses the repository target directory and disables an inherited compiler wrapper.

The initial 50-sample shared-host run recorded a 5.07 ms median source-to-IR pipeline for 1,024 generated procedures, 31,784 allocation operations plus 1,081 grows, and about 4.16 MB maximum live allocation. Reference and upstream lexical medians were 137.6 ms and 181.2 ms. Raw measurements are in `artifacts/benchmarks/20261001T221256.105252Z/results.txt`.

A later run after scalar-cast/compound-update work recorded 11.88 ms for the same generated workload, with identical allocation counts. Its lexical medians rose to 305.9 ms and 440.9 ms while other builds/tests shared the host. This demonstrates why these timings cannot establish a regression or improvement without controlled measurement. `artifacts/benchmarks/20261001T222905.259878Z/` also includes a compiler-source archive and corpus fingerprints. Both recorded runs used the earlier handwritten text backend. The LLVM API replacement changes the workload and memory accounting, so those runs are historical baselines rather than performance comparisons for the current backend. These are instrumented baselines, not optimized results or Jai compiler parity.

A 25-sample LLVM API baseline at `artifacts/benchmarks/20261001T231909.958818Z/` records LLVM 22.1.1 and explicit Rust-only allocation scope. For 1,024 procedures, medians were 5.62 ms for module construction/verification, 17.27 ms including serialization, and 22.69 ms for the complete pipeline. The pipeline recorded 58,419 Rust allocation operations plus 1,065 grows and 4.35 MB peak live Rust allocations; native LLVM heap use is excluded. The shared host was active, so these remain descriptive baselines without a performance gate.

## Dependencies

Divan 0.1.21, selected with the native 14-day Cargo policy and verified with all 33 locked external packages before compilation. Compiler crates remain independent of Divan. See [dependency policy](dependency-policy.md).
