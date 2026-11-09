//! Materialize, compile and run the embedded Jai documentation theme.
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

include!(concat!(env!("OUT_DIR"), "/doc_theme.rs"));

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Unique scratch names also distinguish concurrent runs in one process (unit tests).
fn nonce() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

pub struct Input {
    pub path: PathBuf,
    temporary: bool,
}

impl Input {
    pub fn new(keep: Option<&Path>) -> Result<Self, String> {
        let path = keep.map_or_else(
            || std::env::temp_dir().join(format!("jacket-doc-input-{}", nonce())),
            Path::to_path_buf,
        );
        Ok(Self {
            path,
            temporary: keep.is_none(),
        })
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        if self.temporary {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

pub struct Theme {
    files: BTreeMap<String, Vec<u8>>,
    overrides: BTreeMap<String, Vec<u8>>,
}

fn read_tree(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for entry in
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        // Hidden files are skipped as build.rs skips them in the embedded theme.
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "theme symlinks are not supported: {}",
                path.display()
            ));
        }
        if meta.is_dir() {
            read_tree(root, &path, files)?;
        } else if meta.is_file() {
            let name = jacket_config::slashed(path.strip_prefix(root).unwrap());
            files.insert(
                name,
                std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?,
            );
        }
    }
    Ok(())
}

fn cache_root() -> Result<PathBuf, String> {
    if cfg!(windows) {
        return jacket_config::env::LOCALAPPDATA
            .os()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| "LOCALAPPDATA is not set".into());
    }
    if let Some(root) = jacket_config::env::XDG_CACHE_HOME
        .os()
        .filter(|v| !v.is_empty())
    {
        return Ok(PathBuf::from(root));
    }
    let home = jacket_config::env::HOME
        .os()
        .filter(|v| !v.is_empty())
        .ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join(if cfg!(target_os = "macos") {
        "Library/Caches"
    } else {
        ".cache"
    }))
}

fn hash_part(hash: &mut Sha256, name: &str, bytes: &[u8]) {
    hash.update((name.len() as u64).to_le_bytes());
    hash.update(name);
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

pub struct Prepared {
    binary: PathBuf,
}

impl Theme {
    pub fn new(overrides: Option<&Path>) -> Result<Self, String> {
        let mut files: BTreeMap<String, Vec<u8>> = FILES
            .iter()
            .map(|(p, b)| ((*p).to_owned(), b.to_vec()))
            .collect();
        let mut extra = BTreeMap::new();
        if let Some(root) = overrides {
            read_tree(root, root, &mut extra)?;
        }
        files.extend(extra.clone());
        Ok(Self {
            files,
            overrides: extra,
        })
    }

    pub fn assets(&self) -> Vec<String> {
        self.files
            .keys()
            .filter(|p| p.starts_with("assets/"))
            .cloned()
            .collect()
    }

    fn key(&self, compiler_version: &[u8], web: &BTreeMap<String, Vec<u8>>) -> String {
        let mut hash = Sha256::new();
        hash_part(&mut hash, "jacket", env!("CARGO_PKG_VERSION").as_bytes());
        hash_part(&mut hash, "jaic", compiler_version);
        // Hash the original and the merged tree: even an override identical to an embedded
        // file is represented, and removed/added files cannot collide by concatenation.
        for (name, bytes) in FILES {
            hash_part(&mut hash, name, bytes);
        }
        for (name, bytes) in &self.files {
            hash_part(&mut hash, name, bytes);
        }
        for (name, bytes) in &self.overrides {
            hash_part(&mut hash, &format!("override/{name}"), bytes);
        }
        for (name, bytes) in web {
            hash_part(&mut hash, &format!("Web/{name}"), bytes);
        }
        format!("{:x}", hash.finalize())
    }

    pub fn prepare(&self, jaic: &Path, stdlib: &Path, timings: bool) -> Result<Prepared, String> {
        self.prepare_at(jaic, stdlib, timings, &cache_root()?)
    }

    fn prepare_at(
        &self,
        jaic: &Path,
        stdlib: &Path,
        timings: bool,
        cache: &Path,
    ) -> Result<Prepared, String> {
        let start = Instant::now();
        let version = Command::new(jaic)
            .arg("-version")
            .output()
            .map_err(|e| e.to_string())?;
        if !version.status.success() {
            return Err("jaic -version failed".into());
        }
        let mut web = BTreeMap::new();
        let web_root = stdlib.join("Extensions/Web");
        read_tree(&web_root, &web_root, &mut web)?;
        let mut identity = version.stdout;
        // Backend features may differ between two compilers with the same -version string.
        let compiler =
            std::fs::read(jaic).map_err(|e| format!("cannot read {}: {e}", jaic.display()))?;
        identity.extend_from_slice(&Sha256::digest(compiler));
        let key = self.key(&identity, &web);
        let dir = cache.join("jacket/doc-site").join(key);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("cannot create theme cache {}: {e}", dir.display()))?;
        // Install sources before compiling: #filepath is also the theme's asset root, so
        // the compiler must see the final path rather than a staging directory's name.
        let source = dir.join("src");
        if !source.is_dir() {
            let stage = dir.join(format!(".src-{}", nonce()));
            std::fs::create_dir(&stage).map_err(|e| e.to_string())?;
            let result = (|| {
                for (name, bytes) in &self.files {
                    let file = stage.join(name);
                    std::fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
                    std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
                }
                match std::fs::rename(&stage, &source) {
                    Ok(()) => Ok(()),
                    Err(_) if source.is_dir() => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            })();
            let _ = std::fs::remove_dir_all(&stage);
            result?;
        }
        let binary = dir.join(format!("doc-site{}", std::env::consts::EXE_SUFFIX));
        if binary.is_file() {
            if timings {
                eprintln!(
                    "jacket doc timings: theme compile: cached ({:.3}s)",
                    start.elapsed().as_secs_f64()
                );
            }
            return Ok(Prepared {
                binary,
            });
        }
        let temp = dir.join(format!(
            ".doc-site-{}{}",
            nonce(),
            std::env::consts::EXE_SUFFIX
        ));
        let output = Command::new(jaic)
            .env(jacket_config::env::STDLIB.name, stdlib)
            .arg(source.join("main.jai"))
            .args(["-o", "-no_debug_info", "-exe"])
            .arg(&temp)
            .output()
            .map_err(|e| format!("cannot start jaic: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let _ = std::fs::remove_file(&temp);
            if stderr.contains("built without the `llvm` feature and cannot write native output")
                || stdout
                    .contains("built without the `llvm` feature and cannot write native output")
            {
                return Err(format!(
                    "{} cannot write native output, which the documentation theme needs: install a jaic with the native backend",
                    jaic.display()
                ));
            }
            eprint!("{stderr}");
            print!("{stdout}");
            return Err("documentation theme did not compile".into());
        } else if let Err(e) = std::fs::rename(&temp, &binary) {
            if !binary.is_file() {
                return Err(format!("cannot install theme binary: {e}"));
            }
            let _ = std::fs::remove_file(&temp);
        }
        if timings {
            eprintln!(
                "jacket doc timings: theme compile: {:.3}s",
                start.elapsed().as_secs_f64()
            );
        }
        Ok(Prepared {
            binary,
        })
    }
}

impl Prepared {
    pub fn run(
        &self,
        stdlib: &Path,
        input: &Path,
        out: &Path,
        check: bool,
    ) -> Result<ExitCode, String> {
        let mut command = Command::new(&self.binary);
        command
            .env(jacket_config::env::STDLIB.name, stdlib)
            .arg(input)
            .arg(out);
        if check {
            command.arg("--check");
        }
        let status = command
            .status()
            .map_err(|e| format!("cannot run documentation theme: {e}"))?;
        if status.code().is_none() {
            eprintln!("jacket: error: documentation theme terminated: {status}");
        }
        Ok(ExitCode::from(
            status.code().unwrap_or(1).clamp(0, 255) as u8
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_support::TempDir;

    #[test]
    fn embedded_files_match_the_complete_theme_tree() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/doc/site");
        let mut disk = BTreeMap::new();
        read_tree(&root, &root, &mut disk).unwrap();
        assert_eq!(Theme::new(None).unwrap().files, disk);
    }

    #[test]
    fn hidden_files_are_not_theme_files() {
        assert!(
            FILES
                .iter()
                .all(|(name, _)| name.split('/').all(|part| !part.starts_with('.')))
        );
        let temp = TempDir::new("doc-hidden");
        std::fs::create_dir_all(temp.path().join(".vscode")).unwrap();
        std::fs::write(temp.path().join(".vscode/settings.json"), "{}").unwrap();
        std::fs::write(temp.path().join(".DS_Store"), "x").unwrap();
        std::fs::write(temp.path().join("extra.jai"), "").unwrap();
        let mut files = BTreeMap::new();
        read_tree(temp.path(), temp.path(), &mut files).unwrap();
        assert_eq!(files.keys().collect::<Vec<_>>(), ["extra.jai"]);
    }

    #[test]
    fn overrides_replace_extend_and_invalidate_the_cache() {
        let original = Theme::new(None).unwrap();
        let web = BTreeMap::from([("module.jai".into(), b"Web".to_vec())]);
        let key = original.key(b"jaic 1", &web);
        let temp = TempDir::new("doc-overrides");
        // Identical overrides still have their own presence in the key.
        temp.write("main.jai", &original.files["main.jai"]);
        let same = Theme::new(Some(&temp)).unwrap();
        assert_ne!(key, same.key(b"jaic 1", &web));
        temp.write("assets/site.css", "body{color:green}");
        temp.write("assets/extra.txt", "extra");
        let theme = Theme::new(Some(&temp)).unwrap();
        assert_eq!(theme.files["assets/site.css"], b"body{color:green}");
        assert!(theme.assets().contains(&"assets/extra.txt".into()));
        assert_ne!(key, theme.key(b"jaic 1", &web));
        assert_ne!(key, original.key(b"jaic 2", &web));
        assert_ne!(key, original.key(b"jaic 1", &BTreeMap::new()));
    }

    #[test]
    fn temporary_inputs_are_removed_and_explicit_inputs_are_kept() {
        let temporary = Input::new(None).unwrap();
        std::fs::create_dir(&temporary.path).unwrap();
        let path = temporary.path.clone();
        drop(temporary);
        assert!(!path.exists());
        let kept = TempDir::new("doc-input-kept");
        drop(Input::new(Some(&kept)).unwrap());
        assert!(kept.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn concurrent_compiles_publish_complete_sources_and_an_atomic_binary() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new("doc-concurrent");
        let jaic = temp.write(
            "jaic",
            r#"#!/bin/sh
if [ "$1" = '-version' ]; then echo 'fake jaic'; exit 0; fi
for last do :; done
printf '#!/bin/sh\nexit 0\n' > "$last"
chmod +x "$last"
"#,
        );
        std::fs::set_permissions(&jaic, std::fs::Permissions::from_mode(0o755)).unwrap();
        temp.write("stdlib/Extensions/Web/module.jai", "Web");
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..2)
                .map(|_| {
                    scope.spawn(|| {
                        let theme = Theme::new(None).unwrap();
                        let prepared = theme
                            .prepare_at(&jaic, &temp.join("stdlib"), false, &temp)
                            .unwrap();
                        assert!(prepared.binary.is_file());
                        assert_eq!(
                            std::fs::read(prepared.binary.parent().unwrap().join("src/main.jai"))
                                .unwrap(),
                            theme.files["main.jai"]
                        );
                    })
                })
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }

    #[cfg(unix)]
    #[test]
    fn a_compiler_without_a_native_backend_is_an_error_not_a_slower_path() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new("doc-interpreted");
        let jaic = temp.write(
            "jaic",
            r#"#!/bin/sh
if [ "$1" = '-version' ]; then echo 'fake jaic'; exit 0; fi
echo 'this jaic was built without the `llvm` feature and cannot write native output' >&2
exit 1
"#,
        );
        std::fs::set_permissions(&jaic, std::fs::Permissions::from_mode(0o755)).unwrap();
        temp.write("stdlib/Extensions/Web/module.jai", "Web");
        let theme = Theme::new(None).unwrap();
        let error = theme
            .prepare_at(&jaic, &temp.join("stdlib"), false, &temp)
            .err()
            .expect("the theme needs the native backend");
        assert!(error.contains("native backend"), "{error}");
        assert!(error.contains("install"), "{error}");
        // Nothing is cached, so a later run with a good compiler is not shadowed.
        let again = theme.prepare_at(&jaic, &temp.join("stdlib"), false, &temp);
        assert!(again.is_err());
    }
}
