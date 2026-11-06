//! Presentation-independent, versioned documentation interchange.
pub(super) mod build;
pub mod contract;
pub(super) use super::slug;
pub(super) mod document;
mod produce;
pub(super) mod publication;
pub(super) use produce::{InputAsset, Request, write};
mod inputs;
pub(super) use inputs::source_state;

#[cfg(test)]
mod tests {
    use super::contract::*;

    fn runtime_manifest() -> serde_json::Value {
        serde_json::json!({"schema":"jacket-doc-site","schemaVersion":1,"producer":{"jacket":"owned","compiler":"owned","formatter":null},"source":{"revision":null,"dirty":false,"configurationDigest":"a".repeat(64),"inputs":[]},"partial":false,"capabilities":["browser_runtime","editable_examples"],"shards":[],"assets":[]})
    }

    #[test]
    fn unknown_asset_provenance_is_not_an_open_string_contract() {
        let mut value = runtime_manifest();
        value["assets"] = serde_json::json!([{"path":"assets/logo.svg","mediaType":"image/svg+xml","sha256":"a".repeat(64),"bytes":1,"provenance":"unknown-origin"}]);
        assert!(serde_json::from_value::<Manifest>(value).is_err());
    }

    #[test]
    fn runtime_capability_requires_the_exact_complete_runtime_asset_pair() {
        let manifest: Manifest = serde_json::from_value(runtime_manifest()).unwrap();
        assert!(
            manifest.validate().is_err(),
            "a runtime capability without assets is not a runnable export"
        );
        let asset = |path: &str, media: &str| serde_json::json!({"path":path,"mediaType":media,"sha256":"a".repeat(64),"bytes":1,"provenance":"browser_runtime"});
        let mut valid = runtime_manifest();
        valid["assets"] = serde_json::json!([
            asset("runtime/engine.mjs", "text/javascript"),
            asset("runtime/jai_wasm.wasm", "application/wasm")
        ]);
        assert!(
            serde_json::from_value::<Manifest>(valid.clone())
                .unwrap()
                .validate()
                .is_ok()
        );
        let mut wrong_origin = valid.clone();
        wrong_origin["assets"][0]["provenance"] = serde_json::json!("configured_logo");
        let mut wrong_media = valid.clone();
        wrong_media["assets"][1]["mediaType"] = serde_json::json!("text/plain");
        let mut extra = valid.clone();
        extra["assets"]
            .as_array_mut()
            .unwrap()
            .push(asset("runtime/extra.wasm", "application/wasm"));
        let mut undeclared = valid.clone();
        undeclared["capabilities"] = serde_json::json!([]);
        let mut duplicate = valid.clone();
        duplicate["capabilities"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("browser_runtime"));
        for invalid in [wrong_origin, wrong_media, extra, undeclared, duplicate] {
            assert!(
                serde_json::from_value::<Manifest>(invalid)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
    }

    #[test]
    fn external_schemes_are_case_insensitive_without_accepting_executable_urls() {
        let site = crate::doc::site::Site::new(
            vec![],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            false,
        );
        for href in [
            "HTTPS://example.com/Path",
            "HtTp://example.com/",
            "MAILTO:reader@example.com",
        ] {
            assert!(
                matches!(super::build::target(&site,href,"index.html").unwrap(),Target::External {url} if url==href)
            );
        }
        for href in ["JaVaScRiPt:alert(1)", "FILE:///private/path"] {
            assert!(super::build::target(&site, href, "index.html").is_err());
        }
    }

    #[test]
    fn manifest_rejects_unsupported_versions_and_escaping_inventory_paths() {
        let manifest = Manifest {
            schema: Schema::JacketDocSite,
            schema_version: 1,
            producer: Producer {
                jacket: "test".into(),
                compiler: "test".into(),
                formatter: None,
            },
            source: SourceState {
                revision: None,
                dirty: false,
                configuration_digest: "a".repeat(64),
                inputs: vec![],
            },
            partial: false,
            capabilities: vec![],
            shards: vec![Inventory {
                path: "../escape.json".into(),
                sha256: "a".repeat(64),
                bytes: 1,
            }],
            assets: vec![],
        };
        assert!(manifest.validate().is_err());
        let mut valid = manifest;
        valid.shards[0].path = "site.json".into();
        assert!(valid.validate().is_ok());
        valid.schema_version = 2;
        assert!(valid.validate().is_err());
    }
}
