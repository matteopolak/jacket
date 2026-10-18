//! Optimisation levels, code generation options and sanitizer selection.
use inkwell::OptimizationLevel;
use jaic::target::Target as BuildTarget;
use std::path::PathBuf;

/// Optimization pipeline selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OptLevel {
    #[default]
    O0,
    O1,
    O2,
    O3,
    /// `default<O2>` tuned for size (`.OS`).
    Os,
    /// Smaller still (`.OZ`).
    Oz,
}

impl OptLevel {
    pub(super) fn llvm(self) -> OptimizationLevel {
        match self {
            OptLevel::O0 => OptimizationLevel::None,
            OptLevel::O1 => OptimizationLevel::Less,
            OptLevel::O2 | OptLevel::Os | OptLevel::Oz => OptimizationLevel::Default,
            OptLevel::O3 => OptimizationLevel::Aggressive,
        }
    }

    /// New-pass-manager pipeline string, `None` when no optimization runs.
    pub(super) fn pipeline(self) -> Option<&'static str> {
        match self {
            OptLevel::O0 => None,
            OptLevel::O1 => Some("default<O1>"),
            OptLevel::O2 => Some("default<O2>"),
            OptLevel::O3 => Some("default<O3>"),
            OptLevel::Os => Some("default<Os>"),
            OptLevel::Oz => Some("default<Oz>"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub opt_level: OptLevel,
    /// What to build for; `None` selects the host.
    pub target: Option<BuildTarget>,
    /// Also write the textual LLVM IR (before optimization) to this path.
    pub emit_ir: Option<PathBuf>,
    /// Emit native debug information (DWARF; see `docs/native/debug-info.md`).
    pub debug_info: bool,
    /// Sanitizer instrumentation (`docs/native/sanitizers.md`).
    pub sanitize: Sanitize,
    /// LLVM CPU name (`llvm_options.target_system_cpu`); `None`: the oldest CPU the triple runs
    /// on (`baseline_cpu`), `"native"`: the build machine's CPU and features.
    pub cpu: Option<String>,
    /// LLVM feature string (`llvm_options.target_system_features`, such as `+simd128`);
    /// `None`: the CPU's own. wasm always gets `+bulk-memory` added.
    pub features: Option<String>,
    /// What a metaprogram's `Build_Options` asked of code generation (`None`: jaic's default).
    pub codegen: Codegen,
}

/// Code generation choices from `Build_Options` and its `llvm_options`
/// (`docs/metaprogramming/build-options.md`). Every `None` keeps jaic's default.
#[derive(Clone, Debug, Default)]
pub struct Codegen {
    /// `machine_code_optimization_setting`: the target machine's level, apart from the IR's.
    pub machine_level: Option<OptLevel>,
    /// `enable_loop_unrolling`, `enable_loop_vectorization`, `enable_slp_vectorization`,
    /// `merge_functions`: the optimization pipeline's switches.
    pub loop_unrolling: Option<bool>,
    pub loop_vectorization: Option<bool>,
    pub slp_vectorization: Option<bool>,
    pub merge_functions: Option<bool>,
    /// `disable_inlining`: every function not marked `inline` gets `noinline`.
    pub disable_inlining: Option<bool>,
    /// `enable_tail_calls = false`: `"disable-tail-calls"` on every function.
    pub tail_calls: Option<bool>,
    /// `enable_split_modules`: `Some(false)` keeps unoptimized codegen in one module.
    pub split_modules: Option<bool>,
    /// `-no_split`: one codegen unit at every optimization level, with machine code generation
    /// unsplit as well (what `-codegen_units 1` does).
    pub single_unit: bool,
    /// `-codegen_units N` (`[experimental] codegen_units`): exactly this many code modules.
    pub units: Option<usize>,
    /// `-dev_split_units N`: split an optimized module into this many objects however small it
    /// is (tests force a split of programs far below the threshold).
    pub split_units: Option<usize>,
    /// `-dev_verify_ir`: `Some(true)` runs the LLVM verifier on each module (a release jaic skips
    /// it otherwise), `Some(false)` skips it even in a debug jaic.
    pub verify_ir: Option<bool>,
    /// `-dev_o0_passes <pipeline>`: an LLVM pass pipeline to run on unoptimized builds.
    pub o0_passes: Option<String>,
    /// `enable_frame_pointers`: `Some(true)` keeps a frame record in every function,
    /// `Some(false)` in none (Apple targets keep their ABI's frame records regardless).
    pub frame_pointers: Option<bool>,
    /// `disable_redzone`: `noredzone` on every function.
    pub no_red_zone: bool,
    /// `output_llvm_ir`, `output_bitcode` (after optimization) and the `_before_optimizations`
    /// variants: where to write them.
    pub ir_after: Option<PathBuf>,
    pub bitcode_after: Option<PathBuf>,
    pub bitcode_before: Option<PathBuf>,
    /// `minimum_os_version` for a macOS target, as `major.minor` (the triple's version).
    pub macos_version: Option<String>,
}

/// Which sanitizers instrument a native build. [`link`] links the runtime they call.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Sanitize {
    /// `AddressSanitizer`: heap, stack and global out-of-bounds, use-after-free, use-after-return.
    pub address: bool,
    /// The `UBSan` checks that exist at the LLVM IR level and match Jai semantics: accesses
    /// outside an object of known size (`bounds-checking`, Clang's `local-bounds`).
    pub undefined: bool,
    /// `SanitizerCoverage` `trace-pc-guard` and `pc-table` for line and function coverage; the
    /// link adds `runtime/coverage.c`, which writes the blocks that ran at exit
    /// (`docs/tools/coverage.md`).
    pub coverage: bool,
    /// Coverage instrumentation for libFuzzer (`SanitizerCoverage`: inline 8-bit counters, a PC
    /// table and comparison tracing). [`link`] adds libFuzzer without its `main`, so the program
    /// starts the engine itself with `LLVMFuzzerRunDriver`.
    pub fuzzer: bool,
}

impl Sanitize {
    pub fn any(self) -> bool {
        self.address || self.undefined || self.coverage || self.fuzzer
    }

    /// Parse a `-sanitize` value: a comma-separated list of `address`, `undefined` and `fuzzer`.
    pub fn parse(list: &str) -> Result<Sanitize, String> {
        let mut sanitize = Sanitize::default();
        for name in list.split(',') {
            match name.trim() {
                "address" => sanitize.address = true,
                "undefined" => sanitize.undefined = true,
                "coverage" => sanitize.coverage = true,
                "fuzzer" => sanitize.fuzzer = true,
                other => {
                    return Err(format!(
                        "unknown sanitizer `{other}`\nhelp: `-sanitize` takes address, undefined, fuzzer or a list: `-sanitize address,fuzzer`"
                    ));
                }
            }
        }
        Ok(sanitize)
    }

    /// The Clang driver's `-fsanitize=` value; the driver then links the matching runtime.
    pub(super) fn driver_flag(self) -> Option<String> {
        // `fuzzer-no-link`: the instrumentation flags only. libFuzzer's own `main` would clash
        // with the program's; `fuzzer_runtime` names the engine archive without it.
        let names: Vec<&str> = [
            (self.address, "address"),
            (self.undefined, "undefined"),
            (self.fuzzer, "fuzzer-no-link"),
        ]
        .into_iter()
        .filter_map(|(on, name)| on.then_some(name))
        .collect();
        (!names.is_empty()).then(|| format!("-fsanitize={}", names.join(",")))
    }

    /// Passes appended to the optimization pipeline, where Clang runs its sanitizer passes.
    /// The bounds checks come first so that `ASan` does not instrument them.
    pub(super) fn passes(self, opt_level: OptLevel) -> Vec<&'static str> {
        let mut passes = Vec::new();
        if self.undefined {
            // `rt-abort`: report through the UBSan runtime, then stop the program. The check
            // needs to see which object a pointer came from; unoptimized code keeps every
            // value in a stack slot, so promote those first (`sroa`) or it finds almost none.
            passes.push(if opt_level == OptLevel::O0 {
                "function(sroa,bounds-checking<rt-abort>)"
            } else {
                "function(bounds-checking<rt-abort>)"
            });
        }
        if self.address {
            passes.push("asan");
        }
        if self.coverage || self.fuzzer {
            // Last, as Clang does: the pass numbers the blocks that are left (and, for the
            // fuzzer, after the other sanitizers so their checks are not instrumented).
            passes.push(SANCOV_PASS);
        }
        passes
    }
}

/// The `SanitizerCoverage` pass of `-coverage` and `-sanitize fuzzer` builds (options: [`set_coverage_options`]).
/// The fuzzer's counters and PC table are also what a coverage report reads; its traced
/// comparisons let libFuzzer solve `if x == 0x1234` style branches.
pub const SANCOV_PASS: &str = "sancov-module";
