//! Embedded source execution, managed snapshots and measured ABI protocol.
use crate::Error;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
const RECIPE: &str = "vma-virtual-3.3.0";

pub(super) struct Session {
    pub(super) path: PathBuf,
    published: bool,
}
impl Session {
    pub(super) fn new() -> Result<Self, Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/native-dependencies");
        let project = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(Error::BackendIo)?;
        let ancestor = base
            .ancestors()
            .find(|path| path.exists())
            .ok_or_else(|| Error::Source("native build parent is unavailable".into()))?
            .canonicalize()
            .map_err(Error::BackendIo)?;
        if !ancestor.starts_with(&project)
            || ["reference", "vendor", "corpus/upstream"]
                .iter()
                .any(|name| ancestor.starts_with(project.join(name)))
        {
            return Err(Error::Source(
                "native build output escaped its managed source-build root".into(),
            ));
        }
        fs::create_dir_all(&base).map_err(Error::BackendIo)?;
        let base = base.canonicalize().map_err(Error::BackendIo)?;
        if base != project.join("artifacts/native-dependencies") {
            return Err(Error::Source(
                "native build output must not traverse directory symlinks".into(),
            ));
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::Source("native build clock precedes epoch".into()))?
            .as_nanos();
        let path = base.join(format!(
            "cli-proof-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).map_err(Error::BackendIo)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
                .map_err(Error::BackendIo)?;
        }
        Ok(Self {
            path,
            published: false,
        })
    }
    pub(super) fn into_path(mut self) -> PathBuf {
        self.published = true;
        self.path.clone()
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// The executable helper and its imports must match the source compiled into this driver.
pub(super) fn helper() -> Result<Command, Error> {
    let tools = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools");
    let sources = [
        (
            "check_corpus",
            include_bytes!("../../../../tools/check_corpus.py").as_slice(),
        ),
        (
            "inventory_corpus_features",
            include_bytes!("../../../../tools/inventory_corpus_features.py").as_slice(),
        ),
        (
            "native_dependencies",
            include_bytes!("../../../../tools/native_dependencies.py").as_slice(),
        ),
        (
            "native_dependency_proofs",
            include_bytes!("../../../../tools/native_dependency_proofs.py").as_slice(),
        ),
    ];
    let mut command = Command::new("/usr/bin/python3");
    // Compile the embedded bytes directly. Neither a source replacement between
    // validation and spawn nor an inherited/repository .pyc can supply code.
    command.arg("-I").arg("-c").arg(
        "import sys,types\nargs=sys.argv[1:]\nfor i in range(0,12,3):\n name,path,source=args[i:i+3]\n if name=='__main__': sys.argv=[path]+args[12:]\n module=types.ModuleType(name)\n module.__file__=path\n sys.modules[name]=module\n exec(compile(bytes.fromhex(source),path,'exec'),module.__dict__)\n",
    );
    for (name, expected) in sources {
        let path = tools.join(format!("{name}.py"));
        if fs::read(&path).map_err(Error::BackendIo)? != expected {
            return Err(Error::Source(
                "native source proof helper changed; rebuild this driver before using it".into(),
            ));
        }
        use std::fmt::Write;
        let mut source = String::with_capacity(expected.len() * 2);
        for byte in expected {
            write!(source, "{byte:02x}").expect("writing hexadecimal bytes to a String");
        }
        command
            .arg(if name == "native_dependency_proofs" {
                "__main__"
            } else {
                name
            })
            .arg(path)
            .arg(source);
    }
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/opt/homebrew/bin");
    if let Some(value) = env::var_os("TMPDIR") {
        command.env("TMPDIR", value);
    }
    Ok(command)
}

pub(super) struct Proof {
    pub(super) artifact: PathBuf,
    pub(super) digest: String,
}
impl Proof {
    pub(super) fn parse(bytes: &[u8], target: &str, directory: &Path) -> Result<Self, Error> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Error::Source("native ABI proof protocol is not UTF-8".into()))?;
        let rows: Vec<_> = text.lines().collect();
        if rows.len() != 8
            || rows[0] != "JAI_NATIVE_SOURCE_ABI_V1"
            || rows[1] != RECIPE
            || rows[2] != target
        {
            return Err(Error::Source(
                "native ABI proof recipe or exact target differs".into(),
            ));
        }
        let artifact = PathBuf::from(rows[3]);
        let expected = directory.join("libVkMemAlloc.a");
        if artifact != expected || artifact.canonicalize().map_err(Error::BackendIo)? != expected {
            return Err(Error::Source(
                "native ABI proof artifact escaped its fresh build session".into(),
            ));
        }
        if rows[4].len() != 64
            || !rows[4]
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Error::Source("invalid fresh dependency fingerprint".into()));
        }
        if rows[5..]
            != [
                "block 24 8 0 8 16",
                "allocation 32 8 0 8 16 24",
                "info 24 8 0 8 16",
            ]
        {
            return Err(Error::Source(
                "measured SDK record layout differs from reviewed 64-bit VMA ABI".into(),
            ));
        }
        Ok(Self {
            artifact,
            digest: rows[4].into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_python_source_verifies_an_inert_snapshot() {
        let session = Session::new().unwrap();
        let artifact = session.path.join("empty-authored-fixture");
        fs::write(&artifact, []).unwrap();
        let digest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let output = helper()
            .unwrap()
            .arg("verify-snapshot")
            .arg("--artifact")
            .arg(artifact)
            .arg("--sha256")
            .arg(digest)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, format!("{digest}\n").as_bytes());
    }

    #[test]
    fn measured_proof_requires_exact_target_digest_layout_and_fresh_artifact() {
        let session = Session::new().unwrap();
        let build = session.path.join("build");
        fs::create_dir(&build).unwrap();
        let artifact = build.join("libVkMemAlloc.a");
        fs::write(&artifact, b"authored inert proof fixture; never linked").unwrap();
        let valid = format!(
            "JAI_NATIVE_SOURCE_ABI_V1\nvma-virtual-3.3.0\narm64-apple-darwin\n{}\n{}\nblock 24 8 0 8 16\nallocation 32 8 0 8 16 24\ninfo 24 8 0 8 16\n",
            artifact.display(),
            "a".repeat(64)
        );
        assert!(Proof::parse(valid.as_bytes(), "arm64-apple-darwin", &build).is_ok());
        for invalid in [
            valid.replace("arm64-apple-darwin", "x86_64-unknown-linux-gnu"),
            valid.replace("vma-virtual-3.3.0", "unreviewed-recipe"),
            valid.replace(&"a".repeat(64), &"Z".repeat(64)),
            valid.replace("block 24 8 0 8 16", "block 24 4 0 8 16"),
            valid.replace("allocation 32 8 0 8 16 24", "allocation 32 8 0 8 16 20"),
            valid.replace("libVkMemAlloc.a", "existing-input.a"),
            format!("{valid}extra\n"),
        ] {
            assert!(Proof::parse(invalid.as_bytes(), "arm64-apple-darwin", &build).is_err());
        }
    }
}
