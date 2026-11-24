//! The overrides that come first in every lookup. They are for CI and scripts: a `jacket.toml`
//! never names a machine's tools.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

/// What the command line named, which wins over the environment.
static FLAGS: OnceLock<Overrides> = OnceLock::new();

/// Records `-linker`, `-sysroot` and `-compiler_rt`. Called once, when the command line is read.
pub fn set_flags(linker: Option<&str>, sysroot: Option<&Path>, compiler_rt: Option<&Path>) {
    let _ = FLAGS.set(Overrides {
        linker: linker.map(|value| Given {
            value: value.into(),
            from: "-linker",
        }),
        archiver: None,
        sysroot: sysroot.map(|value| Given {
            value: value.to_path_buf(),
            from: "-sysroot",
        }),
        compiler_rt: compiler_rt.map(|value| Given {
            value: value.to_path_buf(),
            from: "-compiler_rt",
        }),
    });
}

impl Overrides {
    /// The command line's flags, else `JAIC_LINKER`, `JAIC_AR`, `JAIC_SYSROOT` and
    /// `JAIC_COMPILER_RT`.
    pub fn from_env() -> Overrides {
        let mut overrides = Overrides::from_variables();
        if let Some(flags) = FLAGS.get() {
            overrides.linker = flags.linker.clone().or(overrides.linker);
            overrides.sysroot = flags.sysroot.clone().or(overrides.sysroot);
            overrides.compiler_rt = flags.compiler_rt.clone().or(overrides.compiler_rt);
        }
        overrides
    }

    fn from_variables() -> Overrides {
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
