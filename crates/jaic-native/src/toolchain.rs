//! The machine's tools, found by `jaic-toolchain`, as commands to run.
use jaic_toolchain::{Found, Lookup, System};
use std::process::Command;

static SYSTEM: System = System;

/// The lookup over this machine. The LLVM installs are searched only when an LLVM program is.
pub(crate) fn lookup() -> Lookup<'static> {
    Lookup::new(&SYSTEM).with_llvm_bins(|| jacket_config::llvm::bin_dirs(crate::BAKED_LLVM_PREFIX))
}

/// A command that starts `found` with the arguments and variables its lookup chose.
pub(crate) fn command(found: &Found) -> Command {
    let mut command = Command::new(&found.program);
    command.args(&found.args).envs(found.env.iter().cloned());
    command
}
