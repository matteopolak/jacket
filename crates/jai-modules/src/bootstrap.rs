//! Select actual source input for the shared implicit Preload module.
use crate::SourceProvider;
use std::{
    fmt, io,
    path::{Path, PathBuf},
};

/// Bootstrap selection is separate from explicit import search configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PreludeSource {
    /// Low-level frontend tests may deliberately compile without runtime declarations.
    Disabled,
    /// Require Preload in the configured, ordered module roots.
    #[default]
    Search,
    /// Require this exact source file; canonicalization still uses the source provider.
    File(PathBuf),
}

/// Explicit Runtime_Support policy derived from workspace build settings.
/// The source module also retains its 32768-byte fallback for direct imports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSupportParameters {
    pub define_system_entry_point: bool,
    pub define_initialization: bool,
    pub enable_backtrace_on_crash: bool,
    pub temporary_storage_size: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSupportSource {
    #[default]
    Search,
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSupportOptions {
    pub source: RuntimeSupportSource,
    pub parameters: RuntimeSupportParameters,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BootstrapOptions {
    pub prelude: PreludeSource,
    pub runtime_support: Option<RuntimeSupportOptions>,
}
impl BootstrapOptions {
    pub fn disabled() -> Self {
        Self {
            prelude: PreludeSource::Disabled,
            runtime_support: None,
        }
    }
}

#[derive(Debug)]
pub enum PreludeError {
    Missing { roots: Vec<PathBuf> },
    MissingRuntimeSupport { roots: Vec<PathBuf> },
    InvalidConfiguration(&'static str),
    Io { path: PathBuf, cause: io::Error },
}
impl fmt::Display for PreludeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing {
                roots,
            } => {
                f.write_str("Preload source not found in configured module roots")?;
                for root in roots {
                    write!(f, " {}", root.display())?;
                }
                Ok(())
            }
            Self::MissingRuntimeSupport {
                roots,
            } => {
                f.write_str("Runtime_Support source not found in configured module roots")?;
                for root in roots {
                    write!(f, " {}", root.display())?;
                }
                Ok(())
            }
            Self::InvalidConfiguration(message) => f.write_str(message),
            Self::Io {
                path,
                cause,
            } => write!(f, "{}: {cause}", path.display()),
        }
    }
}
impl std::error::Error for PreludeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io {
                cause, ..
            } => Some(cause),
            Self::Missing {
                ..
            }
            | Self::MissingRuntimeSupport {
                ..
            }
            | Self::InvalidConfiguration(_) => None,
        }
    }
}

impl RuntimeSupportSource {
    pub fn resolve(
        &self,
        roots: &[PathBuf],
        provider: &dyn SourceProvider,
    ) -> Result<PathBuf, PreludeError> {
        let candidate = match self {
            Self::File(path) => path.clone(),
            Self::Search => roots
                .iter()
                .flat_map(|root| {
                    [
                        root.join("Runtime_Support.jai"),
                        root.join("Runtime_Support").join("module.jai"),
                    ]
                })
                .find(|path| provider.is_file(path))
                .ok_or_else(|| PreludeError::MissingRuntimeSupport {
                    roots: roots.to_vec(),
                })?,
        };
        canonical(provider, &candidate)
    }
}

impl PreludeSource {
    /// Resolve bytes through the same provider as all user/imported sources.
    /// A present but unreadable candidate is an error, not a reason to change roots.
    pub fn resolve(
        &self,
        roots: &[PathBuf],
        provider: &dyn SourceProvider,
    ) -> Result<Option<PathBuf>, PreludeError> {
        let candidate = match self {
            Self::Disabled => return Ok(None),
            Self::File(path) => path.clone(),
            Self::Search => roots
                .iter()
                .flat_map(|root| {
                    [
                        root.join("module.jai"),
                        root.join("Preload").join("module.jai"),
                    ]
                })
                .find(|path| provider.is_file(path))
                .ok_or_else(|| PreludeError::Missing {
                    roots: roots.to_vec(),
                })?,
        };
        canonical(provider, &candidate).map(Some)
    }
}

fn canonical(provider: &dyn SourceProvider, path: &Path) -> Result<PathBuf, PreludeError> {
    provider
        .canonicalize(path)
        .map_err(|cause| PreludeError::Io {
            path: path.to_owned(),
            cause,
        })
}
