//! Resolved native dependency metadata, independent of executable expressions.
use crate::ProcedureId;
use jai_source::DeclarationId;
use std::{fmt, path::PathBuf};

/// Identity of a source library declaration, preserved across imports/reexports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForeignLibraryId {
    File(DeclarationId),
    Local {
        procedure: ProcedureId,
        index: usize,
    },
}
impl ForeignLibraryId {
    pub const fn new(declaration: DeclarationId) -> Self {
        Self::File(declaration)
    }
    pub const fn local(procedure: ProcedureId, index: usize) -> Self {
        Self::Local {
            procedure,
            index,
        }
    }
    pub const fn declaration(self) -> Option<DeclarationId> {
        match self {
            Self::File(id) => Some(id),
            Self::Local {
                ..
            } => None,
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::File(id) => id.index(),
            Self::Local {
                index, ..
            } => index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForeignLibraryKind {
    /// A basename searched only in the installed toolchain's system directories.
    System { name: String },
    /// A source-relative path. Metadata alone never authorizes opening its bytes.
    Local { path: PathBuf },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForeignLibraryOptions {
    pub link_always: bool,
    pub no_dll: bool,
    pub no_static_library: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignLibrary {
    pub id: ForeignLibraryId,
    pub kind: ForeignLibraryKind,
    pub options: ForeignLibraryOptions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignLibraryError {
    InvalidSystemName,
    InvalidLocalPath,
    NoLinkage,
    IncompatibleLinkage,
}
impl fmt::Display for ForeignLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSystemName => "system library name must be a plain nonempty basename",
            Self::InvalidLocalPath => "local library path must be nonempty and contain no NUL",
            Self::NoLinkage => "library cannot disable both dynamic and static linking",
            Self::IncompatibleLinkage => {
                "library filename conflicts with its requested linkage kind"
            }
        })
    }
}
impl std::error::Error for ForeignLibraryError {
}

impl ForeignLibrary {
    pub fn validate(&self) -> Result<(), ForeignLibraryError> {
        if self.options.no_dll && self.options.no_static_library {
            return Err(ForeignLibraryError::NoLinkage);
        }
        match &self.kind {
            ForeignLibraryKind::System {
                name,
            } => {
                if name.is_empty()
                    || name.starts_with('-')
                    || name == "."
                    || name == ".."
                    || !name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_-.+".contains(&byte))
                {
                    return Err(ForeignLibraryError::InvalidSystemName);
                }
                if (self.options.no_static_library && name.ends_with(".a"))
                    || (self.options.no_dll && (name.contains(".so") || name.ends_with(".dylib")))
                {
                    return Err(ForeignLibraryError::IncompatibleLinkage);
                }
            }
            ForeignLibraryKind::Local {
                path,
            } => {
                if path.as_os_str().is_empty() || path.as_os_str().as_encoded_bytes().contains(&0) {
                    return Err(ForeignLibraryError::InvalidLocalPath);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::Identities;

    fn system(name: &str) -> ForeignLibrary {
        ForeignLibrary {
            id: ForeignLibraryId::new(Identities::default().declaration()),
            kind: ForeignLibraryKind::System {
                name: name.into(),
            },
            options: ForeignLibraryOptions::default(),
        }
    }
    #[test]
    fn system_names_cannot_redirect_search_or_inject_options() {
        for name in ["libc", "libstdc++.so.6", "kernel32", "m"] {
            system(name).validate().unwrap();
        }
        for name in [
            "",
            "..",
            "/tmp/libc",
            "../libc",
            "-Lreference",
            "x y",
            "x\\y",
            "x\0y",
        ] {
            assert_eq!(
                system(name).validate(),
                Err(ForeignLibraryError::InvalidSystemName)
            );
        }
    }

    #[test]
    fn explicit_filenames_cannot_override_linkage_constraints() {
        let mut library = system("libowned.a");
        library.options.no_static_library = true;
        assert_eq!(
            library.validate(),
            Err(ForeignLibraryError::IncompatibleLinkage)
        );
        for name in ["libowned.so.6", "libowned.dylib"] {
            let mut library = system(name);
            library.options.no_dll = true;
            assert_eq!(
                library.validate(),
                Err(ForeignLibraryError::IncompatibleLinkage)
            );
        }
    }
}
