//! The machine's tools, found by `jaic-toolchain`, as commands to run.
use std::process::Command;

use jaic::target::Target;
use jaic_toolchain::linker::host_target;
use jaic_toolchain::{Found, Lookup, System};

static SYSTEM: System = System;

/// This machine, for the lookups that take a [`jaic_toolchain::Host`].
#[cfg(target_os = "macos")]
pub(crate) fn system() -> &'static System {
    &SYSTEM
}

/// The lookup over this machine. The LLVM installs are searched only when an LLVM program is;
/// an `LLVM_PREFIX` (or `llvm-sys` variable) that names no install is an error first.
pub(crate) fn lookup() -> Result<Lookup<'static>, String> {
    jacket_config::llvm::check_configured()?;
    Ok(Lookup::new(&SYSTEM)
        .with_llvm_bins(|| jacket_config::llvm::bin_dirs(crate::BAKED_LLVM_PREFIX)))
}

/// A command that starts `found` with the arguments and variables its lookup chose.
pub(crate) fn command(found: &Found) -> Command {
    let mut command = Command::new(&found.program);

    command.args(&found.args).envs(found.env.iter().cloned());
    command
}

/// Prints the status line for a tool a build uses (`Linker`, `Sysroot`, `Builtins`): always for a
/// cross or bare-metal build, and for a host build with `-verbose`.
///
/// ```text
///       Linker /opt/homebrew/bin/ld.lld 21.1.8
/// ```
pub(crate) fn report(verb: &str, found: &Found, lookup: &Lookup, target: Option<Target>) {
    let cross = target.is_some_and(|target| target != host_target(lookup.host));

    jaic::progress::tool(verb, &found.describe(lookup), cross);

    if let Some(sysroot) = &found.sysroot {
        jaic::progress::tool("Sysroot", &sysroot.describe(lookup), cross);
    }
}
