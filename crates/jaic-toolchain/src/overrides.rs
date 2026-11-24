//! The overrides that come first in every lookup. They are for CI and scripts: a `jacket.toml`
//! never names a machine's tools.
use std::ffi::OsString;
use std::path::PathBuf;

/// A value and where it came from, for the status line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Given<T> {
    pub value: T,
    pub from: &'static str,
}

/// What the user named. An empty value counts as unset: PowerShell 7.5 passes `VAR=''` for an
/// unset variable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Overrides {
    pub linker: Option<Given<OsString>>,
    pub archiver: Option<Given<OsString>>,
    pub sysroot: Option<Given<PathBuf>>,
    pub compiler_rt: Option<Given<PathBuf>>,
}

impl Overrides {
    /// `JAIC_LINKER`, `JAIC_AR`, `JAIC_SYSROOT` and `JAIC_COMPILER_RT`.
    pub fn from_env() -> Overrides {
        use jacket_config::env;
        let given = |var: &env::Var| {
            var.non_empty().map(|value| Given {
                value,
                from: var.name,
            })
        };
        let path = |var: &env::Var| {
            given(var).map(|g| Given {
                value: PathBuf::from(g.value),
                from: g.from,
            })
        };
        Overrides {
            linker: given(&env::LINKER),
            archiver: given(&env::AR),
            sysroot: path(&env::SYSROOT),
            compiler_rt: path(&env::COMPILER_RT),
        }
    }
}
