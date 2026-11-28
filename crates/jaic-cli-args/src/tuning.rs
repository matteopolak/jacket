//! The `jaic` flags that tune a compile or study the compiler, apart from the language's own
//! options: code module counts, how files are parsed ahead of the checker, the interpreter's
//! counters, and the unlisted `-dev_*` flags. `jacket` can set the first group for a project from
//! `[experimental]` (`docs/tools/jacket.md`). They end up in `jaic::knobs` or in the backend's
//! `CodegenSwitches`.
use argdef::{Args, Error};

use crate::cli::Cli;

/// What the flags here set, in `Cli::tuning`.
// The doc comments are `-help` text: names and variables stay unquoted.
#[allow(clippy::doc_markdown, clippy::duplicated_attributes)]
#[derive(Default, Args)]
pub struct Tuning {
    /// generate code as exactly <n> modules (1 is -no_split; only a build uses it)
    #[arg(
        name = "-codegen_units",
        value = "n",
        needs = "a number of code modules",
        section = "Output (building only)"
    )]
    pub codegen_units: Option<usize>,
    // The size of the shared thread pool, which `-j` sets as well (`-j 0` leaves it at one thread
    // per core).
    /// threads in the pool that code generation and other parallel work share (default: one per core)
    #[arg(name = "-threads", value = "n", needs = "a number of threads")]
    pub threads: Option<usize>,
    // `-parse_threads N`, `-serial_parse`: how files are parsed ahead of the checker.
    /// threads that parse files ahead of the checker (0 parses serially)
    #[arg(name = "-parse_threads", value = "n", needs = "a number of threads")]
    pub parse_threads: Option<usize>,
    /// parse every file on the checking thread
    #[arg(name = "-serial_parse")]
    pub serial_parse: bool,
    // `-profile`, `-profile_ops`: the interpreter's counters.
    /// count what the interpreter runs per procedure and print the busiest
    #[arg(name = "-profile", section = "Diagnostics and output")]
    pub profile_counts: bool,
    #[arg(name = "-profile_ops", hidden)]
    pub profile_ops: bool,
    // `-profile_runs`: the cost of each compile-time run.
    /// print the compile-time runs (#run, constants, metaprograms) by the work they did
    #[arg(name = "-profile_runs")]
    pub profile_runs: bool,
    /// steps (loop iterations and calls) one compile-time run may execute before it fails (default 1000000000; 0: no limit)
    #[arg(name = "-run_budget", value = "steps", needs = "a number of steps", parse = read_run_budget)]
    pub run_budget: Option<u64>,
    // The `-dev_*` flags, for studying the compiler (unlisted in `-help`).
    #[arg(name = "-dev_body_probe", hidden)]
    pub body_probe: bool,
    #[arg(name = "-dev_body_fifo", hidden)]
    pub body_fifo: bool,
    #[arg(name = "-dev_speculative_bodies", hidden)]
    pub speculative_bodies: bool,
    #[arg(name = "-dev_no_sink", hidden)]
    pub no_sink: bool,
    #[arg(name = "-dev_no_flow", hidden)]
    pub no_flow: bool,
    #[arg(name = "-dev_dump_ops", needs = "a procedure name", hidden)]
    pub dump_ops: Option<String>,
    #[arg(name = "-dev_verify_ir", hidden)]
    pub verify_ir: bool,
    #[arg(name = "-dev_o0_passes", needs = "an LLVM pass pipeline", hidden)]
    pub o0_passes: Option<String>,
    #[arg(name = "-dev_split_units", needs = "a number of objects", hidden)]
    pub split_units: Option<usize>,
}

fn read_run_budget(name: &str, text: &str) -> Result<u64, Error> {
    text.parse().map_err(|_| {
        Error::invalid(format!("`{name}` takes a whole number, not `{text}`")).help(format!(
            "for example `{name} 5000000000`, or `{name} 0` for no limit"
        ))
    })
}

/// The process-wide switches these flags set.
pub fn knobs(cli: &Cli) -> jaic::knobs::Knobs {
    let t = &cli.tuning;

    jaic::knobs::Knobs {
        threads: t.threads.or(cli.shared.jobs()),
        parse_threads: t.parse_threads,
        serial_parse: t.serial_parse,
        profile: match (t.profile_ops, t.profile_counts) {
            (true, _) => jaic::knobs::Profile::Ops,
            (_, true) => jaic::knobs::Profile::Counts,
            _ => jaic::knobs::Profile::Off,
        },
        run_costs: t.profile_runs,
        // Native builds run generators and #run code in the interpreter too. Record those
        // lines alongside the executable's coverage instead of reporting them as never run.
        coverage: cli.coverage,
        body_probe: t.body_probe,
        body_fifo: t.body_fifo,
        speculative_bodies: t.speculative_bodies,
        no_sink: t.no_sink,
        no_flow: t.no_flow,
        dump_ops: t.dump_ops.clone(),
    }
}
