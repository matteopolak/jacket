//! The project a command runs in: the nearest `jacket.toml` and the notes the user should see
//! (a pinned toolchain that is not this one).
use jacket_config::{ConfigError, Disk, Loaded};
use std::path::Path;

/// The configuration above `cwd`, with notes printed. A broken file is an error.
pub fn load(cwd: &Path) -> Result<Option<Loaded>, ConfigError> {
    load_with(cwd, true)
}

/// [`load`], printing the notes only when `notes` is set: a command that starts a tool which
/// reads the same file and prints them itself (`jailint`) leaves them to it.
pub fn load_with(cwd: &Path, notes: bool) -> Result<Option<Loaded>, ConfigError> {
    let loaded = jacket_config::load(&Disk, cwd)?;
    if notes && let Some(loaded) = &loaded {
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
