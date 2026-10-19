//! Process-wide performance and diagnostic switches, set once at startup.
//!
//! They come from `jaic` flags (`-threads`, `-parse_threads`, `-serial_parse`, `-profile`, `-coverage_file`,
//! and the `-dev_*` flags that study the compiler itself) and, through `jacket`, from the
//! `[experimental]` table of `jacket.toml`. Nothing here changes what a program means; the
//! settings are read in places far from the options a compile carries (the interpreter, the
//! parser's worker threads), so they live in one `OnceLock` rather than in every signature.
//!
//! Call [`set`] before the first compile; a later call is ignored. Without one, [`get`] is the
//! default (everything off), which is what the language server and tests see.
use std::path::PathBuf;
use std::sync::OnceLock;

/// What the interpreter counts per procedure (`interp/profile.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Profile {
    #[default]
    Off,
    /// Calls, blocks and IR instructions in the unoptimized control flow.
    Counts,
    /// The same on the straightened code, with the interpreter's own ops counted.
    Ops,
}

#[derive(Clone, Debug, Default)]
pub struct Knobs {
    /// Size of the shared thread pool (`-threads`, `jaic::pool`); `None`: one per core.
    pub threads: Option<usize>,
    /// Threads parsing files ahead of the checker (`-parse_threads`); `None`: by core count.
    pub parse_threads: Option<usize>,
    /// Parse on the checking thread (`-serial_parse`).
    pub serial_parse: bool,
    /// `-profile`, `-profile_ops`.
    pub profile: Profile,
    /// `-coverage_file`: append the procedures that ran under `-interp` to this file.
    pub coverage_file: Option<PathBuf>,
    /// `-dev_body_probe`: print which shared tables each body mutates (`sema/body_probe.rs`).
    pub body_probe: bool,
    /// `-dev_body_fifo`: lower queued bodies first-in-first-out.
    pub body_fifo: bool,
    /// `-dev_no_sink`, `-dev_no_flow`: turn off interpreter passes (`interp/code.rs`).
    pub no_sink: bool,
    pub no_flow: bool,
    /// `-dev_dump_ops <procedure>`: print one procedure's interpreter ops.
    pub dump_ops: Option<String>,
}

static KNOBS: OnceLock<Knobs> = OnceLock::new();

/// Sets the switches for this process; `false` when they were already set (or read).
pub fn set(knobs: Knobs) -> bool {
    KNOBS.set(knobs).is_ok()
}

/// The switches in force.
pub fn get() -> &'static Knobs {
    KNOBS.get_or_init(Knobs::default)
}
