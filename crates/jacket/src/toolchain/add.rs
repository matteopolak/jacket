//! `jacket toolchain add|list|remove`: what no package manager provides, in jacket's cache.
//! A build only reads the cache; nothing else in jacket downloads.
use std::io::Write;

use jaic::target::Target;
use jaic_toolchain::{Host, Lookup, System, cache, pins, tilde};

use crate::status;

/// The target a name stands for: a triple, or a board or `[target.NAME]` table.
fn resolve(name: &str, config: Option<&jacket_config::Config>) -> Result<Target, String> {
    let known = crate::target::all(config);
    let triple = known
        .iter()
        .find(|target| target.name == name)
        .and_then(|target| target.effective_triple())
        .unwrap_or(name);

    Target::parse(triple).map_err(|error| {
        format!(
            "`{name}` is not a target jacket knows: {error}\nhelp: `jacket toolchain` lists them"
        )
    })
}

/// Whether lines are coloured and the live line is redrawn: a terminal.
/// Whether the progress line is redrawn in place: jaic's rule for its own status line.
fn live() -> bool {
    jaic::progress::mode(false, false) == jaic::progress::Mode::Live
}

/// Redraws the progress line in place on a terminal; elsewhere says nothing until the end.
fn live_line(text: &str) {
    if live() {
        let mut err = std::io::stderr().lock();
        let _ = write!(err, "\r\x1b[K{text}");
        let _ = err.flush();
    }
}

fn clear_live() {
    if live() {
        let mut err = std::io::stderr().lock();
        let _ = write!(err, "\r\x1b[K");
        let _ = err.flush();
    }
}

/// `jacket toolchain add TARGET...`: fetches a sysroot for a Linux target, or builds the builtins
/// of a freestanding one. Returns whether every target worked.
pub fn add(names: &[String], config: Option<&jacket_config::Config>) -> Result<(), String> {
    let lookup = super::lookup();

    for name in names {
        let target = resolve(name, config)?;

        add_one(target, &lookup)?;
    }

    Ok(())
}

/// What a download needs, checked before the first byte: `curl` and `tar`, and a cache folder
/// that can be written.
fn downloads_possible(lookup: &Lookup) -> Result<(), String> {
    use jaic_toolchain::package::Need;
    let command = "jacket toolchain add";

    crate::prerequisite::require(&Need::Curl, "curl", command)?;
    crate::prerequisite::require(&Need::Tar, "tar", command)?;
    let root = cache::root(lookup.host).ok_or_else(|| {
        format!(
            "there is no cache folder, and `{command}` needs one\nnote: set HOME, or XDG_CACHE_HOME to a folder you can write to"
        )
    })?;
    crate::prerequisite::writable_dir(&root, "the cache folder", command)
}

fn add_one(target: Target, lookup: &Lookup) -> Result<(), String> {
    let color = jaic::render::detect(jaic::render::ColorChoice::Auto).color;

    if let Some(family) = pins::compiler_rt_family(target) {
        let pin = pins::compiler_rt(family);

        if let Some((_, source)) = cache::builtins(lookup.host, target)
            && cache::builtins_stamp(lookup.host, target).as_deref() == Some(pin.stamp().as_str())
        {
            status::print("Builtins", &format!("{source} (cached)"), color);
            return Ok(());
        }

        downloads_possible(lookup)?;
        let version = pin.version;
        let built = super::runtime::add(target, lookup, &|done, total| {
            live_line(&format!(
                "{:>12} compiler-rt {version} for {target} ({done}/{total})",
                "Building"
            ));
        });

        clear_live();
        let built = built?;

        status::print("Builtins", &super::runtime::finished(&built), color);

        return Ok(());
    }

    if pins::sysroot(target).is_some() {
        if let Some(dir) = cache::sysroot(lookup.host, target) {
            status::print("Sysroot", &format!("{} (cached)", tilde(&dir)), color);
            return Ok(());
        }

        downloads_possible(lookup)?;
        let sysroot = super::sysroot::add(target, lookup.host, &|what| {
            live_line(&format!("{:>12} {what}", "Fetching"));
        });

        clear_live();
        let sysroot = sysroot?;

        status::print(
            "Sysroot",
            &format!("{} (jacket toolchain)", tilde(&sysroot)),
            color,
        );

        return Ok(());
    }

    Err(format!(
        "there is nothing to add for {target}\nnote: jaic links it with the system's tools; `jacket toolchain check {target}` says which"
    ))
}

/// The table `jacket toolchain list` prints: what the cache holds for each target.
pub fn list() -> String {
    let host = System;
    let entries = cache::entries(&host);
    let Some(root) = cache::root(&host) else {
        return "no cache folder: HOME is not set\n".to_string();
    };

    if entries.is_empty() {
        return format!(
            "{} holds nothing yet\nnote: `jacket toolchain add TARGET` fetches a sysroot or builds builtins\n",
            tilde(&root)
        );
    }

    let mut text = format!("{}\n", tilde(&root));
    let width = entries
        .iter()
        .map(|e| e.target.len())
        .max()
        .unwrap_or(0)
        .max(6);

    text.push_str(&format!(
        "{:<width$}  {:<8}  BUILTINS\n",
        "TARGET", "SYSROOT"
    ));

    for entry in entries {
        text.push_str(&format!(
            "{:<width$}  {:<8}  {}\n",
            entry.target,
            if entry.sysroot {
                "yes"
            } else {
                "-"
            },
            entry.builtins.as_deref().unwrap_or("-"),
        ));
    }

    text
}

/// `jacket toolchain remove TARGET...` deletes what the cache holds for each target; with
/// `all`, the whole cache: every target, the downloads and the unpacked sources.
pub fn remove(
    names: &[String],
    all: bool,
    config: Option<&jacket_config::Config>,
) -> Result<Vec<String>, String> {
    let host = System;
    let root = cache::root(&host).ok_or("no cache folder: HOME is not set")?;
    let mut removed = Vec::new();

    if all {
        if root.is_dir() {
            std::fs::remove_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
            removed.push(tilde(&root));
        }

        return Ok(removed);
    }

    if names.is_empty() {
        return Err("name a target to remove, or `--all`\nhelp: `jacket toolchain list` shows what is cached".into());
    }

    for name in names {
        let target = resolve(name, config)?;
        let dir = cache::entry_dir(&host, target).ok_or("no cache folder: HOME is not set")?;

        if !host.is_dir(&dir) {
            return Err(format!(
                "nothing is cached for {target}\nhelp: `jacket toolchain list` shows what is"
            ));
        }

        std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        removed.push(tilde(&dir));
    }

    Ok(removed)
}
