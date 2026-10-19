//! The `jaic` flags that tune a compile or study the compiler, apart from the language's own
//! options: code module counts, how files are parsed ahead of the checker, the interpreter's
//! counters, and the unlisted `-dev_*` flags. `jacket` can set the first group for a project from
//! `[experimental]` (`docs/tools/jacket.md`). They end up in `jaic::knobs` or in the backend's
//! `CodegenSwitches`.
use super::{
    Action, Cli, CliError, CliOption,
    Section::{Diagnostics, Output},
};
use crate::options::store;
use std::path::PathBuf;

/// What the flags here set, in `Cli::tuning`.
#[derive(Default)]
pub(super) struct Tuning {
    /// `-codegen_units N`: exactly this many code modules.
    pub codegen_units: Option<usize>,
    /// `-threads N`: the size of the shared thread pool.
    pub threads: Option<usize>,
    /// `-parse_threads N`, `-serial_parse`: how files are parsed ahead of the checker.
    pub parse_threads: Option<usize>,
    pub serial_parse: bool,
    /// `-profile`, `-profile_ops`, `-coverage_file`: the interpreter's counters.
    pub profile_counts: bool,
    pub profile_ops: bool,
    pub coverage_file: Option<PathBuf>,
    /// The `-dev_*` flags, for studying the compiler (unlisted in `-help`).
    pub body_probe: bool,
    pub body_fifo: bool,
    pub no_sink: bool,
    pub no_flow: bool,
    pub dump_ops: Option<String>,
    pub verify_ir: bool,
    pub o0_passes: Option<String>,
    pub split_units: Option<usize>,
}

/// Code generation switches that hold for every workspace a backend writes.
#[cfg(feature = "llvm")]
#[derive(Clone, Default)]
pub(super) struct CodegenSwitches {
    pub units: Option<usize>,
    pub split_units: Option<usize>,
    pub verify_ir: bool,
    pub o0_passes: Option<String>,
}

#[cfg(feature = "llvm")]
impl CodegenSwitches {
    /// The `jaic_llvm::Codegen` fields these set; the rest is the default.
    pub(super) fn codegen(&self) -> jaic_llvm::Codegen {
        jaic_llvm::Codegen {
            units: self.units,
            split_units: self.split_units,
            verify_ir: self.verify_ir.then_some(true),
            o0_passes: self.o0_passes.clone(),
            ..Default::default()
        }
    }
}

impl Tuning {
    #[cfg(feature = "llvm")]
    pub(super) fn switches(&self) -> CodegenSwitches {
        CodegenSwitches {
            units: self.codegen_units,
            split_units: self.split_units,
            verify_ir: self.verify_ir,
            o0_passes: self.o0_passes.clone(),
        }
    }
}

/// The process-wide switches these flags set.
pub(super) fn knobs(cli: &Cli) -> jaic::knobs::Knobs {
    let t = &cli.tuning;
    jaic::knobs::Knobs {
        threads: t.threads,
        parse_threads: t.parse_threads,
        serial_parse: t.serial_parse,
        profile: match (t.profile_ops, t.profile_counts) {
            (true, _) => jaic::knobs::Profile::Ops,
            (_, true) => jaic::knobs::Profile::Counts,
            _ => jaic::knobs::Profile::Off,
        },
        coverage_file: t.coverage_file.clone(),
        body_probe: t.body_probe,
        body_fifo: t.body_fifo,
        no_sink: t.no_sink,
        no_flow: t.no_flow,
        dump_ops: t.dump_ops.clone(),
    }
}

fn parse_count(text: &str, flag: &str) -> Result<usize, CliError> {
    text.parse().map_err(|_| {
        CliError::new(format!("`{flag}` takes a whole number, not `{text}`"))
            .help(format!("for example `{flag} 4`"))
    })
}

/// Every option of jaic's own: the main table of `main.rs`, then these.
pub(super) fn all_options() -> impl Iterator<Item = &'static CliOption> {
    super::OPTIONS.iter().chain(OPTIONS)
}

const OPTIONS: &[CliOption] = &[
    CliOption::new(
        &["-codegen_units"],
        store!("a number of code modules", |cli, n| cli
            .tuning
            .codegen_units =
            Some(parse_count(&n, "-codegen_units")?)),
    )
    .help(
        Output,
        "-codegen_units <n>",
        "generate code as exactly <n> modules (1 is -no_split; only a build uses it)",
    ),
    CliOption::new(
        &["-threads"],
        store!("a number of threads", |cli, n| cli.tuning.threads =
            Some(parse_count(&n, "-threads")?)),
    )
    .help(
        Output,
        "-threads <n>",
        "threads in the pool that code generation and other parallel work share (default: one per core)",
    ),
    CliOption::new(
        &["-parse_threads"],
        store!("a number of threads", |cli, n| cli.tuning.parse_threads =
            Some(parse_count(&n, "-parse_threads")?)),
    )
    .help(
        Output,
        "-parse_threads <n>",
        "threads that parse files ahead of the checker (0 parses serially)",
    ),
    CliOption::new(
        &["-serial_parse"],
        Action::Switch(|cli| &mut cli.tuning.serial_parse),
    )
    .help(
        Output,
        "-serial_parse",
        "parse every file on the checking thread",
    ),
    CliOption::new(
        &["-profile"],
        Action::Switch(|cli| &mut cli.tuning.profile_counts),
    )
    .help(
        Diagnostics,
        "-profile",
        "count what the interpreter runs per procedure and print the busiest",
    ),
    CliOption::new(
        &["-profile_ops"],
        Action::Switch(|cli| &mut cli.tuning.profile_ops),
    ),
    CliOption::new(
        &["-coverage_file"],
        store!("a file", |cli, file| cli.tuning.coverage_file =
            Some(PathBuf::from(file))),
    )
    .help(
        Diagnostics,
        "-coverage_file <file>",
        "(-interp) append the procedures that ran to <file>",
    ),
    CliOption::new(
        &["-dev_body_probe"],
        Action::Switch(|cli| &mut cli.tuning.body_probe),
    ),
    CliOption::new(
        &["-dev_body_fifo"],
        Action::Switch(|cli| &mut cli.tuning.body_fifo),
    ),
    CliOption::new(
        &["-dev_no_sink"],
        Action::Switch(|cli| &mut cli.tuning.no_sink),
    ),
    CliOption::new(
        &["-dev_no_flow"],
        Action::Switch(|cli| &mut cli.tuning.no_flow),
    ),
    CliOption::new(
        &["-dev_dump_ops"],
        store!("a procedure name", |cli, name| cli.tuning.dump_ops =
            Some(name)),
    ),
    CliOption::new(
        &["-dev_verify_ir"],
        Action::Switch(|cli| &mut cli.tuning.verify_ir),
    )
    .build_only(),
    CliOption::new(
        &["-dev_o0_passes"],
        store!("an LLVM pass pipeline", |cli, passes| cli
            .tuning
            .o0_passes =
            Some(passes)),
    )
    .build_only(),
    CliOption::new(
        &["-dev_split_units"],
        store!("a number of objects", |cli, n| cli.tuning.split_units =
            Some(parse_count(&n, "-dev_split_units")?)),
    )
    .build_only(),
];
