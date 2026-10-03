//! Installed LLVM tools for freshly generated native fixtures.
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
    time::{Duration, Instant},
};

pub fn clang() -> PathBuf {
    static TOOL: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    TOOL.get_or_init(|| {
        let explicit = env::var_os("JAI_RS_CLANG");
        let prefix = env::var_os("LLVM_SYS_221_PREFIX")
            .or_else(|| option_env!("LLVM_SYS_221_PREFIX").map(Into::into))
            .map(PathBuf::from);
        let paths = env::var_os("PATH");
        resolve_clang(
            explicit.as_deref(),
            prefix.as_deref(),
            paths.as_deref(),
            &repository_root(),
        )
    })
    .as_ref()
    .unwrap_or_else(|error| panic!("native test Clang selection failed: {error}"))
    .clone()
}

fn repository_root() -> PathBuf {
    // An isolated test manifest may include this helper through an absolute path.
    let source = Path::new(file!());
    let source = if source.is_absolute() {
        source.to_owned()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(source)
    };
    source
        .canonicalize()
        .ok()
        .and_then(|source| {
            source
                .ancestors()
                .find(|path| path.file_name() == Some(OsStr::new("crates")))
                .and_then(Path::parent)
                .map(Path::to_owned)
        })
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

pub fn clang_command() -> Command {
    let mut command = Command::new(clang());
    scrub_environment(&mut command);
    command
}

/// Inputs are explicit so selection tests need not mutate process-wide variables.
pub fn resolve_clang(
    explicit: Option<&OsStr>,
    prefix: Option<&Path>,
    search_path: Option<&OsStr>,
    repository: &Path,
) -> Result<PathBuf, String> {
    let candidate = if let Some(explicit) = explicit {
        if explicit.is_empty() {
            return Err("JAI_RS_CLANG is empty".into());
        }
        locate(Path::new(explicit), search_path).ok_or_else(|| {
            format!(
                "JAI_RS_CLANG tool was not found: {}",
                Path::new(explicit).display()
            )
        })?
    } else if let Some(prefix) = prefix {
        if prefix.as_os_str().is_empty() {
            return Err("LLVM_SYS_221_PREFIX is empty".into());
        }
        prefix.join("bin/clang")
    } else {
        ["clang-22", "clang"]
            .into_iter()
            .find_map(|name| locate(Path::new(name), search_path))
            .ok_or_else(|| {
                "LLVM 22 Clang was not found; set JAI_RS_CLANG or LLVM_SYS_221_PREFIX".to_owned()
            })?
    };
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "cannot resolve native compiler {}: {error}",
            candidate.display()
        )
    })?;
    for protected in ["reference", "corpus", "vendor", ".git"] {
        let root = repository.join(protected);
        let root = root.canonicalize().unwrap_or(root);
        if canonical.starts_with(&root) {
            return Err(format!(
                "native compiler is inside protected {protected} inputs: {}",
                canonical.display()
            ));
        }
    }
    let metadata = canonical.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err(format!(
            "native compiler is not a file: {}",
            canonical.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!(
                "native compiler is not executable: {}",
                canonical.display()
            ));
        }
    }
    verify_version(&canonical)?;
    Ok(canonical)
}

fn locate(name: &Path, search_path: Option<&OsStr>) -> Option<PathBuf> {
    if name.is_absolute() || name.components().count() > 1 {
        return Some(name.to_owned());
    }
    search_path.and_then(|paths| {
        env::split_paths(paths)
            .map(|root| root.join(name))
            .find(|path| path.is_file())
    })
}

fn verify_version(path: &Path) -> Result<(), String> {
    let mut command = Command::new(path);
    scrub_environment(&mut command);
    let mut child = command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot inspect native compiler {}: {error}", path.display()))?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "native compiler version probe exceeded five seconds: {}",
                path.display()
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    let version = String::from_utf8_lossy(&output.stdout);
    let major = version
        .split("clang version ")
        .nth(1)
        .and_then(|suffix| suffix.split('.').next())
        .and_then(|major| major.parse::<u32>().ok());
    if !output.status.success() || major != Some(22) {
        return Err(format!(
            "native compiler must report Clang 22: {} reported {}",
            path.display(),
            version.trim()
        ));
    }
    Ok(())
}

fn scrub_environment(command: &mut Command) {
    for variable in [
        "LIBRARY_PATH",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "LD_AUDIT",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_ROOT_PATH",
        "DYLD_VERSIONED_LIBRARY_PATH",
        "DYLD_VERSIONED_FRAMEWORK_PATH",
        "CCC_OVERRIDE_OPTIONS",
        "SDKROOT",
    ] {
        command.env_remove(variable);
    }
}
