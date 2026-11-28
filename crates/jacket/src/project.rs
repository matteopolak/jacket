//! The project a command runs in: the nearest `jacket.toml` and the notes the user should see
//! (a pinned toolchain that is not this one).
use std::path::Path;

use jacket_config::{ConfigError, Disk, Loaded};

/// The configuration above `cwd`, with notes printed. A broken file is an error.
pub(crate) fn load(cwd: &Path) -> Result<Option<Loaded>, ConfigError> {
    load_with(cwd, true)
}

/// [`load`], printing the notes only when `notes` is set: a command that starts a tool which
/// reads the same file and prints them itself (`jailint`) leaves them to it.
pub(crate) fn load_with(cwd: &Path, notes: bool) -> Result<Option<Loaded>, ConfigError> {
    let loaded = jacket_config::load(&Disk, cwd)?;

    if notes && let Some(loaded) = &loaded {
        for note in &loaded.notes {
            eprintln!("jacket: {note}");
        }
    }

    Ok(loaded)
}

/// `[toolchain] version` against this jacket: a project pinned to another release is an error
/// with the command that installs it.
pub(crate) fn check_toolchain(loaded: &Loaded) -> Result<(), String> {
    pinned_toolchain(
        loaded.config.toolchain.version.as_deref(),
        &loaded.location.file(),
        env!("CARGO_PKG_VERSION"),
    )
}

fn pinned_toolchain(wanted: Option<&str>, file: &Path, running: &str) -> Result<(), String> {
    match wanted {
        Some(wanted) if wanted != running => Err(format!(
            "{} asks for toolchain {wanted}, but this is jacket {running}\nhelp: run `jacket upgrade --version {wanted}`, or change `[toolchain] version`",
            file.display()
        )),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_project_pinned_to_another_release_is_an_error_naming_the_upgrade() {
        let file = Path::new("/p/jacket.toml");

        assert!(pinned_toolchain(None, file, "1.0.0").is_ok());
        assert!(pinned_toolchain(Some("1.0.0"), file, "1.0.0").is_ok());
        let error = pinned_toolchain(Some("0.9.0"), file, "1.0.0").unwrap_err();

        assert_eq!(
            error,
            "/p/jacket.toml asks for toolchain 0.9.0, but this is jacket 1.0.0\nhelp: run `jacket upgrade --version 0.9.0`, or change `[toolchain] version`"
        );
    }
}
