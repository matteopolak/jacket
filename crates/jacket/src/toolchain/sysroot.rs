//! `jacket toolchain add <linux target>`: a sysroot of pinned Debian packages, for linking a
//! Linux program on a machine whose own C library is not the target's.
use std::path::{Path, PathBuf};

use jaic::target::Target;
use jaic_toolchain::{Host, cache, pins};

/// Downloads the target's packages (each checked against its pinned sha256), unpacks them into
/// `<cache>/<triple>/sysroot` and returns that folder. `say` is told about each package.
///
/// The folder appears only when every package unpacked: the work is done in a sibling that is
/// renamed at the end, so an interrupted run leaves no half sysroot for a build to find.
pub(super) fn add(target: Target, host: &dyn Host, say: &dyn Fn(&str)) -> Result<PathBuf, String> {
    let pin = pins::sysroot(target)
        .ok_or_else(|| format!("there is no sysroot to fetch for {target}"))?;
    let entry = cache::entry_dir(host, target).ok_or("no cache folder: HOME is not set")?;
    let packages = pins::debian_packages(pin.debian_arch);
    let downloads = cache::root(host)
        .ok_or("no cache folder: HOME is not set")?
        .join("downloads");
    let work = entry.join("sysroot.partial");
    let _ = std::fs::remove_dir_all(&work);

    std::fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    let result = unpack_all(&packages, &downloads, &work, say);

    if let Err(error) = result {
        let _ = std::fs::remove_dir_all(&work);
        return Err(error);
    }

    let sysroot = entry.join("sysroot");
    let _ = std::fs::remove_dir_all(&sysroot);

    std::fs::rename(&work, &sysroot).map_err(|e| format!("{}: {e}", sysroot.display()))?;
    Ok(sysroot)
}

fn unpack_all(
    packages: &pins::DebianPins,
    downloads: &Path,
    work: &Path,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    for (index, pin) in packages.packages.iter().enumerate() {
        say(&format!(
            "{} {} ({}/{})",
            pin.package,
            pin.version,
            index + 1,
            packages.packages.len()
        ));
        let deb = downloads.join(format!("{}.deb", pin.sha256));

        super::fetch::pinned(
            &format!("{}/{}", packages.mirror, pin.path),
            pin.sha256,
            &deb,
        )?;
        super::fetch::unpack_deb(&deb, &work.join(".unpack"), work)?;
    }

    let _ = std::fs::remove_dir_all(work.join(".unpack"));

    if !work.join("usr").is_dir() {
        return Err("the packages unpacked no `usr` folder".into());
    }

    Ok(())
}
