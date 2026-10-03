//! Process-local authority for reviewed, freshly rebuilt dependency artifacts.
mod abi;
mod proofs;
use crate::Error;
use jai_codegen::{native_reachability::Reachable, target::NativeTarget};
use jai_sema::{ForeignLibraryId, ForeignLibraryKind, Library};
use proofs::{Proof, Session, helper};
use std::{env, fs, path::PathBuf, process::Command};

/// The constructor is private: paths or persisted JSON cannot create link authority.
pub(crate) struct VerifiedDependency {
    library: ForeignLibraryId,
    snapshot: PathBuf,
    session: PathBuf,
    runtime: CxxRuntime,
}
enum CxxRuntime {
    LibCpp,
    LibStdCpp,
}

impl Drop for VerifiedDependency {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.session);
    }
}

pub(crate) fn prepare(
    library: &Library,
    target: &NativeTarget,
    reachable: &Reachable,
) -> Result<Vec<VerifiedDependency>, Error> {
    let receipt = env::var_os("JAI_RS_NATIVE_VMA_RECEIPT");
    let name = env::var_os("JAI_RS_NATIVE_VMA_LIBRARY");
    let (receipt, name) = match (receipt, name) {
        (None, None) => return Ok(vec![]),
        (Some(receipt), Some(name)) => (PathBuf::from(receipt), PathBuf::from(name)),
        _ => return Err(Error::Source("reviewed VMA linking requires both JAI_RS_NATIVE_VMA_RECEIPT and JAI_RS_NATIVE_VMA_LIBRARY".into())),
    };
    let mut selected = None;
    for foreign in library.foreign_libraries() {
        if !matches!(&foreign.kind, ForeignLibraryKind::Local {path} if *path == name) {
            continue;
        }
        let used = foreign.options.link_always || library.prototypes().iter().any(|prototype| {
            reachable.contains(prototype.id) && matches!(&prototype.origin,
                jai_sema::PrototypeOrigin::Foreign{library:Some(owner),..} if owner.id == foreign.id)
        }) || library.globals().iter().any(|global| {
            reachable.contains_global(global.id()) && matches!(global.initializer(),
                jai_sema::GlobalInitializer::External(data) if matches!(data.source(),
                    jai_sema::ExternalDataSource::Library(owner) if owner.id == foreign.id))
        });
        if !used {
            continue;
        }
        if crate::native_tools::protected_input(&name)? {
            return Err(Error::Source(
                "reviewed native library identity is inside protected source inputs; no native dependency was linked".into(),
            ));
        }
        if selected.is_some() {
            return Err(Error::Source("reviewed dependency configuration matches multiple source library declarations; qualify the source library identity".into()));
        }
        if foreign.options.no_static_library {
            return Err(Error::Source("reviewed VMA recipe emits a static archive; source declaration forbids static linking".into()));
        }
        if name
            .extension()
            .is_some_and(|value| matches!(value.to_str(), Some("dll" | "so" | "dylib")))
        {
            return Err(Error::Source(
                "reviewed VMA recipe cannot replace an explicitly named dynamic library".into(),
            ));
        }
        selected = Some(foreign.id);
    }
    let Some(id) = selected else {
        return Ok(vec![]);
    };
    if !target.is_host() {
        return Err(Error::Source(
            "reviewed dependency ABI oracles require the actual host target".into(),
        ));
    }
    // Reject unreviewed symbols and mismatched source layouts before native execution.
    abi::validate(library, target, reachable, id)?;
    let triple = target
        .triple
        .as_str()
        .to_str()
        .map_err(|_| Error::Source("native target triple is not UTF-8".into()))?;
    let runtime = if triple.contains("apple") {
        CxxRuntime::LibCpp
    } else if triple.contains("linux") {
        CxxRuntime::LibStdCpp
    } else {
        return Err(Error::Source(
            "reviewed VMA dependency supports macOS and Linux hosts only".into(),
        ));
    };
    let session = Session::new()?;
    let build = session.path.join("build");
    let mut command = helper()?;
    command
        .arg("rebuild-vma-virtual")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--target")
        .arg(triple)
        .arg("--output")
        .arg(&build);
    let output = command.output().map_err(Error::BackendIo)?;
    if !output.status.success() {
        return Err(Error::Source(format!(
            "reviewed source dependency rebuild/ABI proof failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let parsed = Proof::parse(&output.stdout, triple, &build)?;
    let snapshot = session.path.join("verified-vma.a");
    let mut input = fs::File::open(&parsed.artifact).map_err(Error::BackendIo)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&snapshot)
        .map_err(Error::BackendIo)?;
    std::io::copy(&mut input, &mut file).map_err(Error::BackendIo)?;
    drop(file);
    let mut permissions = fs::metadata(&snapshot)
        .map_err(Error::BackendIo)?
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&snapshot, permissions).map_err(Error::BackendIo)?;
    let status = helper()?
        .arg("verify-snapshot")
        .arg("--artifact")
        .arg(&snapshot)
        .arg("--sha256")
        .arg(&parsed.digest)
        .output()
        .map_err(Error::BackendIo)?;
    if !status.status.success() || status.stdout != format!("{}\n", parsed.digest).as_bytes() {
        return Err(Error::Source(
            "fresh dependency snapshot fingerprint changed before linking".into(),
        ));
    }
    let session = session.into_path();
    Ok(vec![VerifiedDependency {
        library: id,
        snapshot,
        session,
        runtime,
    }])
}

pub(crate) fn apply(
    command: &mut Command,
    dependencies: &[VerifiedDependency],
    id: ForeignLibraryId,
) -> bool {
    let Some(dependency) = dependencies
        .iter()
        .find(|dependency| dependency.library == id)
    else {
        return false;
    };
    command.arg(&dependency.snapshot); // Keep the OS path; never stringify a linker input.
    command.arg(match dependency.runtime {
        CxxRuntime::LibCpp => "-lc++",
        CxxRuntime::LibStdCpp => "-lstdc++",
    });
    true
}
