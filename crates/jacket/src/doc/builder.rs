//! Locate and run the versioned, explicitly installed Astro source builder.
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    #[serde(rename = "jacketDocSiteSchema")]
    schema: u32,
}

pub(super) struct Builder {
    root: PathBuf,
}

impl Builder {
    /// Prefer an explicit installation, then the packaged sibling, then this source checkout.
    /// Example: `JACKET_DOCS_BUILDER=/opt/docs-builder jacket doc library/`.
    pub(super) fn locate() -> Result<Self, String> {
        let explicit = jacket_config::env::DOCS_BUILDER.os().map(PathBuf::from);
        let root = if let Some(root) = explicit {
            root
        } else {
            let executable = std::env::current_exe().map_err(|e| e.to_string())?;
            let executable = executable.canonicalize().map_err(|e| e.to_string())?;
            let packaged = executable
                .parent()
                .ok_or("Jacket executable has no parent")?
                .join("docs-builder");
            if packaged.is_dir() {
                packaged
            } else {
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../website/docs")
            }
        };
        Self::validate(&root)?;
        let node=Command::new("node").arg("--version").output().map_err(|_|"jacket doc needs Node 24.12 or newer in the Node24 line; install Node and run `npm ci` in the docs builder".to_string())?;
        let version = String::from_utf8_lossy(&node.stdout);
        let parsed = semver::Version::parse(version.trim().trim_start_matches('v'))
            .map_err(|_| "cannot determine installed Node version".to_string())?;
        if !node.status.success() || parsed.major != 24 || parsed < semver::Version::new(24, 12, 0)
        {
            return Err(
                "jacket doc needs Node24.12 or newer within Node24; see docs/tools/astro-docs.md"
                    .into(),
            );
        }
        Ok(Self {
            root: root.canonicalize().map_err(|e| e.to_string())?,
        })
    }

    fn validate(root: &Path) -> Result<(), String> {
        let missing = || {
            format!(
                "no compatible docs builder at {}; install the versioned docs-builder source package or set JACKET_DOCS_BUILDER (docs/tools/astro-docs.md)",
                root.display()
            )
        };
        let bytes = std::fs::read(root.join("package.json")).map_err(|_| missing())?;
        let package: Package = serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid docs builder package.json: {e}"))?;
        if package.name != "@jacket/docs-builder"
            || package.version != env!("CARGO_PKG_VERSION")
            || package.schema != 1
        {
            return Err(format!(
                "docs builder must match Jacket {} and support jacket-doc-site/1",
                env!("CARGO_PKG_VERSION")
            ));
        }
        if !root.join("scripts/build.mjs").is_file() {
            return Err(missing());
        }
        if !root.join("node_modules/astro/package.json").is_file() {
            return Err(format!(
                "docs builder dependencies are missing; run `npm ci` in {} before `jacket doc`",
                root.display()
            ));
        }
        Ok(())
    }

    /// The JavaScript wrapper stages and validates a complete site before replacing owned output.
    pub(super) fn run(&self, export: &Path, out: &Path, check: bool) -> Result<ExitCode, String> {
        let mut command = Command::new("node");
        command
            .arg(self.root.join("scripts/build.mjs"))
            .arg("--export")
            .arg(export)
            .arg("--out")
            .arg(out)
            .current_dir(&self.root);
        if check {
            command.arg("--check");
        }
        let status = command
            .status()
            .map_err(|e| format!("start Astro docs builder: {e}"))?;
        Ok(ExitCode::from(
            status.code().unwrap_or(2).try_into().unwrap_or(2),
        ))
    }
}

/// A semantic bundle retained by `--export`, or owned only for the lifetime of one build.
pub(super) struct Input {
    pub path: PathBuf,
    temporary: bool,
}

impl Input {
    /// Reserve a fresh directory without overwriting any existing temporary input.
    pub(super) fn new(keep: Option<PathBuf>) -> Result<Self, String> {
        if let Some(path) = keep {
            return Ok(Self {
                path,
                temporary: false,
            });
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jacket-doc-export-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&path)
            .map_err(|e| format!("reserve semantic export directory: {e}"))?;
        Ok(Self {
            path,
            temporary: true,
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_support::TempDir;

    #[test]
    fn missing_mismatched_or_uninstalled_builder_fails_before_output() {
        let dir = TempDir::new("doc-builder");
        assert!(
            Builder::validate(&dir)
                .unwrap_err()
                .contains("no compatible docs builder")
        );
        std::fs::write(dir.join("package.json"),serde_json::json!({"name":"@jacket/docs-builder","version":env!("CARGO_PKG_VERSION"),"jacketDocSiteSchema":2}).to_string()).unwrap();
        assert!(
            Builder::validate(&dir)
                .unwrap_err()
                .contains("support jacket-doc-site/1")
        );
        std::fs::write(dir.join("package.json"),serde_json::json!({"name":"@jacket/docs-builder","version":env!("CARGO_PKG_VERSION"),"jacketDocSiteSchema":1}).to_string()).unwrap();
        std::fs::create_dir(dir.join("scripts")).unwrap();
        std::fs::write(dir.join("scripts/build.mjs"), "").unwrap();
        assert!(Builder::validate(&dir).unwrap_err().contains("npm ci"));
    }
}
