//! Record compiler-read source inputs plus conservatively included library/guide configuration.
use super::contract::{InputDigest, SourceState};
use crate::doc::site::Site;
use jacket_config::schema::Doc;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn sources(
    root: &Path,
    seen: &mut BTreeSet<PathBuf>,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    let canonical = std::fs::canonicalize(root)
        .map_err(|e| format!("source dependency {}: {e}", root.display()))?;
    if !seen.insert(canonical) {
        return Ok(());
    }
    for entry in std::fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            sources(&path, seen, files)?;
        } else if path.extension().is_some_and(|e| e == "jai") {
            files.insert(path);
        }
    }
    Ok(())
}

/// Stable public dependency names are rooted in supplied source trees, never developer paths.
pub(super) fn logical_path(
    file: &Path,
    root: &Path,
    stdlib: &Path,
    config: &Path,
    hash: &str,
) -> Result<String, String> {
    let key = if let Ok(path) = file.strip_prefix(root) {
        format!("library/{}", jacket_config::slashed(path))
    } else if let Ok(path) = file.strip_prefix(stdlib) {
        format!("stdlib/{}", jacket_config::slashed(path))
    } else if let Ok(path) = file.strip_prefix(config) {
        format!("configuration/{}", jacket_config::slashed(path))
    } else {
        format!(
            "external/{}-{}",
            &hash[..24],
            file.file_name().unwrap_or_default().to_string_lossy()
        )
    };
    super::contract::validate_path(&key)?;
    Ok(key)
}

/// Full exports record actual resolver reads and conservatively cover target-probe dependencies.
/// No per-module semantic result is reused until target/import dependency tracking is complete.
pub(in crate::doc) fn source_state(
    site: &Site,
    doc: &Doc,
    publication: &super::publication::Publication,
    assets: &[super::InputAsset],
    config: &Path,
    root: &Path,
    stdlib: &Path,
) -> Result<SourceState, String> {
    let mut files = site
        .modules
        .iter()
        .flat_map(|m| m.dependencies.iter().map(|(path, _)| path.clone()))
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for module in &site.modules {
        sources(&root.join(&module.id), &mut seen, &mut files)?;
    }
    sources(stdlib, &mut seen, &mut files)?;
    let configuration = serde_json::to_vec(&serde_json::json!({
        "doc":doc, "pageBase":site.base, "name":site.name, "version":site.version,
        "sourceRef":site.source_ref, "repository":site.repo, "playground":site.playground,
        "runtime":site.runtime, "breadcrumb":site.breadcrumb, "logo":site.logo,
        "publication": {"origin":publication.origin,"base":publication.base,"name":publication.name,
            "image":publication.image,"version":publication.version,"canonicalVersion":publication.canonical_version},
        "assets": assets.iter().map(|asset|(&asset.path,&asset.bytes)).chain(publication.image_file.iter().map(|(path,bytes)|(path,bytes))).map(|(path,bytes)|(path,digest(bytes))).collect::<Vec<_>>(),
        "modules":site.modules.iter().map(|m|m.id.as_str()).collect::<Vec<_>>(),
    }))
    .map_err(|e| e.to_string())?;
    for name in doc
        .home
        .iter()
        .chain(doc.since.iter())
        .chain(doc.page.iter().map(|p| &p.file))
        .chain(doc.section.iter().map(|p| &p.file))
    {
        files.insert(config.join(name));
    }
    if config.join("jacket.toml").is_file() {
        files.insert(config.join("jacket.toml"));
    }
    let mut loaded = BTreeMap::new();
    for module in &site.modules {
        for (file, hash) in &module.dependencies {
            if let Some(previous) = loaded.insert(file.clone(), hash.clone())
                && previous != *hash
            {
                return Err(format!(
                    "source changed while documentation modules were extracted: {}",
                    file.display()
                ));
            }
        }
    }
    let mut inputs = BTreeMap::new();
    for file in files {
        let bytes = std::fs::read(&file)
            .map_err(|e| format!("read documentation dependency {}: {e}", file.display()))?;
        let hash = digest(&bytes);
        if let Some(expected) = loaded.get(&file)
            && expected != &hash
        {
            return Err(format!(
                "source changed after compiler documentation extraction: {}",
                file.display()
            ));
        }
        let key = logical_path(&file, root, stdlib, config, &hash)?;
        if let Some(previous) = inputs.insert(key.clone(), hash.clone())
            && previous != hash
        {
            return Err(format!("ambiguous source dependency identity {key}"));
        }
    }
    for (path, bytes) in assets
        .iter()
        .map(|asset| (&asset.path, &asset.bytes))
        .chain(
            publication
                .image_file
                .iter()
                .map(|(path, bytes)| (path, bytes)),
        )
    {
        inputs.insert(format!("asset/{path}"), digest(bytes));
    }
    let state = crate::doc::git_state(config);
    Ok(SourceState {
        revision: state.as_ref().map(|s| s.commit.clone()),
        dirty: state.is_some_and(|s| !s.dirty.is_empty()),
        configuration_digest: digest(&configuration),
        inputs: inputs
            .into_iter()
            .map(|(path, sha256)| InputDigest {
                path,
                sha256,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_publication_flags_and_asset_bytes_are_pinned() {
        let dir = test_support::TempDir::new("doc-effective-publication");
        let site = Site::new(
            vec![],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            false,
        );
        let doc = Doc::default();
        let mut publication = super::super::publication::Publication {
            origin: "https://one.invalid".into(),
            base: "/".into(),
            name: "Fixture".into(),
            image: String::new(),
            version: String::new(),
            canonical_version: None,
            image_file: None,
        };
        let first = source_state(&site, &doc, &publication, &[], &dir, &dir, &dir).unwrap();
        publication.origin = "https://two.invalid".into();
        let second = source_state(&site, &doc, &publication, &[], &dir, &dir, &dir).unwrap();
        assert_ne!(first.configuration_digest, second.configuration_digest);
        let assets = vec![super::super::InputAsset {
            path: "runtime/engine.mjs".into(),
            bytes: b"owned runtime".to_vec(),
            provenance: super::super::contract::AssetProvenance::BrowserRuntime,
        }];
        let third = source_state(&site, &doc, &publication, &assets, &dir, &dir, &dir).unwrap();
        assert_ne!(second.configuration_digest, third.configuration_digest);
        assert!(
            third
                .inputs
                .iter()
                .any(|input| input.path == "asset/runtime/engine.mjs"
                    && input.sha256 == digest(b"owned runtime"))
        );
    }

    #[test]
    fn effective_logo_selection_changes_the_configuration_identity() {
        let dir = test_support::TempDir::new("doc-effective-assets");
        let mut site = Site::new(
            vec![],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            false,
        );
        let doc = Doc::default();
        let publication = super::super::publication::Publication {
            origin: String::new(),
            base: "/".into(),
            name: "Fixture".into(),
            image: String::new(),
            version: String::new(),
            canonical_version: None,
            image_file: None,
        };
        site.logo = Some("first.svg".into());
        let first = source_state(&site, &doc, &publication, &[], &dir, &dir, &dir).unwrap();
        site.logo = Some("second.svg".into());
        let second = source_state(&site, &doc, &publication, &[], &dir, &dir, &dir).unwrap();
        assert_ne!(first.configuration_digest, second.configuration_digest);
    }
}
