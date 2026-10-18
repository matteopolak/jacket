//! The project a command runs in: the nearest `jacket.toml`, migration of legacy files on first
//! use, and the notes the user should see (a pinned toolchain that is not this one).
use jacket_config::{ConfigError, Disk, Loaded, migrate};
use std::path::Path;

/// Whether the run is on CI, where nothing is written without being asked.
pub fn on_ci() -> bool {
    std::env::var_os("CI").is_some_and(|v| !v.is_empty() && v != "false" && v != "0")
}

/// Writes a `jacket.toml` next to legacy config files when the directory chain above `cwd` has
/// them and no `jacket.toml`, and says so on stderr (stdout may be a protocol). Legacy files are
/// never deleted. On CI it only warns.
pub fn migrate_on_first_use(cwd: &Path) {
    let Some(location) = jacket_config::locate(&Disk, cwd) else {
        return;
    };
    if location.jacket || location.legacy.is_empty() {
        return;
    }
    if on_ci() {
        eprintln!(
            "jacket: warning: {} has legacy config ({}); run `jacket config migrate` to write {}",
            location.dir.display(),
            location.legacy.join(", "),
            jacket_config::FILE_NAME
        );
        return;
    }
    match migrate::run(&Disk, &location, false) {
        Ok((
            migrate::Outcome::Wrote {
                path, ..
            },
            Some(plan),
        )) => {
            eprintln!("{}", migrate::summary(&plan, &path));
            for warning in &plan.warnings {
                eprintln!("jacket: {warning}");
            }
        }
        Ok(_) => {}
        Err(error) => eprintln!("jacket: warning: could not migrate: {error}"),
    }
}

/// The configuration above `cwd`, with notes printed. A broken file is an error.
pub fn load(cwd: &Path) -> Result<Option<Loaded>, ConfigError> {
    let loaded = jacket_config::load(&Disk, cwd)?;
    if let Some(loaded) = &loaded {
        for note in &loaded.notes {
            eprintln!("jacket: {note}");
        }
    }
    Ok(loaded)
}

/// `[toolchain] version` against this jacket: warns on a mismatch.
pub fn check_toolchain(loaded: &Loaded) {
    if let Some(wanted) = &loaded.config.toolchain.version
        && wanted != env!("CARGO_PKG_VERSION")
    {
        eprintln!(
            "jacket: warning: {} asks for toolchain {wanted}, but this is jacket {}",
            loaded.location.file().display(),
            env!("CARGO_PKG_VERSION")
        );
    }
}
