//! Link arguments derive from resolved metadata, never source identifier spellings.
use crate::Error;
use jai_sema::{ForeignLibrary, ForeignLibraryKind};

pub(crate) fn arguments(library: &ForeignLibrary, triple: &str) -> Result<Vec<String>, Error> {
    library
        .validate()
        .map_err(|error| Error::Source(error.to_string()))?;
    let ForeignLibraryKind::System { name } = &library.kind else {
        return Err(Error::Source(
            "source-declared local native library linking is unsupported; object emission retains its metadata without accessing native bytes".into(),
        ));
    };
    let linux = triple.contains("linux");
    if (library.options.no_dll || library.options.no_static_library) && !linux {
        return Err(Error::Source(
            "explicit system library linkage selection is unsupported on this host target".into(),
        ));
    }
    if triple.contains("apple")
        && std::path::Path::new("/System/Library/Frameworks")
            .join(format!("{name}.framework"))
            .is_dir()
    {
        return Ok(vec!["-framework".into(), name.clone()]);
    }
    let exact = linux && (name.ends_with(".a") || name.contains(".so"));
    let argument = if exact {
        format!("-l:{name}")
    } else {
        let name = name.strip_prefix("lib").unwrap_or(name);
        let name = name.strip_suffix(".dylib").unwrap_or(name);
        format!("-l{name}")
    };
    Ok(if library.options.no_dll {
        vec!["-Wl,-Bstatic".into(), argument, "-Wl,-Bdynamic".into()]
    } else if library.options.no_static_library && linux {
        vec!["-Wl,-Bdynamic".into(), argument]
    } else {
        vec![argument]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_sema::{ForeignLibraryId, ForeignLibraryOptions};
    use jai_source::Identities;
    fn system(name: &str) -> ForeignLibrary {
        ForeignLibrary {
            id: ForeignLibraryId::new(Identities::default().declaration()),
            kind: ForeignLibraryKind::System { name: name.into() },
            options: ForeignLibraryOptions::default(),
        }
    }
    #[test]
    fn actual_system_basename_has_target_aware_linker_spelling() {
        assert_eq!(
            arguments(&system("libc"), "aarch64-apple-darwin").unwrap(),
            ["-lc"]
        );
        assert_eq!(
            arguments(&system("libstdc++.so.6"), "x86_64-unknown-linux-gnu").unwrap(),
            ["-l:libstdc++.so.6"]
        );
        assert_eq!(
            arguments(&system("kernel32"), "x86_64-pc-windows-msvc").unwrap(),
            ["-lkernel32"]
        );
    }
    #[test]
    fn source_paths_and_options_cannot_become_linker_searches() {
            assert!(arguments(&system(name), "aarch64-apple-darwin").is_err());
        }
        let mut local = system("libc");
        local.kind = ForeignLibraryKind::Local {
        };
        assert!(arguments(&local, "aarch64-apple-darwin").is_err());
    }
}
