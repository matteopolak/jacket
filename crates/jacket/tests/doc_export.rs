//! Real compiler-to-export regressions. Build `jaic-cli` beside Jacket and expose the matching
//! declaration-capable `jaifmt` beside it or on PATH before running this suite.
//! Rendering URL checks use `tools/ci/check_doc_resolver.py`; relocated installations use
//! `tools/ci/check_installed_docs.py`.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

static RUN: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Extract an authored library without Node or example execution; inspect the actual bundle.
fn export(temp: &TempDir, base: &str, version: &str) -> Vec<Value> {
    let jacket = Path::new(env!("CARGO_BIN_EXE_jacket"));
    let jaic = jacket
        .parent()
        .unwrap()
        .join(format!("jaic{}", std::env::consts::EXE_SUFFIX));
    assert!(
        jaic.is_file(),
        "build jaic-cli beside Jacket before doc_export tests: {}",
        jaic.display()
    );
    let output = Command::new(jacket)
        .args([
            "doc",
            "--export-only",
            "--export",
            "bundle",
            "--base-url",
            base,
            "--version",
            version,
        ])
        .env("JACKET_DOCS_BUILDER", temp.join("uninstalled-builder"))
        .env("JAIC_STDLIB", repo().join("stdlib"))
        .env("XDG_CACHE_HOME", temp.join("cache"))
        .current_dir(temp)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let root = temp.join("bundle");
    let envelope: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let manifest = &envelope["data"];
    assert_eq!(manifest["schema"], "jacket-doc-site");
    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["partial"], false);
    manifest["shards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            serde_json::from_slice(
                &std::fs::read(root.join(entry["path"].as_str().unwrap())).unwrap(),
            )
            .unwrap()
        })
        .collect()
}

fn payload<'a>(values: &'a [Value], kind: &str) -> &'a Value {
    &values
        .iter()
        .find(|value| value["payload"] == kind)
        .unwrap()["data"]
}

fn module<'a>(values: &'a [Value], id: &str) -> &'a Value {
    &values
        .iter()
        .find(|value| value["payload"] == "module" && value["data"]["id"] == id)
        .unwrap()["data"]
}

fn named<'a>(values: &'a Value, name: &str) -> &'a Value {
    values
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["name"] == name)
        .unwrap()
}

fn targets(value: &Value) -> Vec<&str> {
    let mut found = Vec::new();
    if value["kind"] == "page" {
        found.push(value["id"].as_str().unwrap());
    }
    match value {
        Value::Array(values) => {
            for value in values {
                found.extend(targets(value));
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                found.extend(targets(value));
            }
        }
        _ => {}
    }
    found
}

#[test]
fn implicit_context_guide_and_user_types_keep_distinct_semantic_targets() {
    let _serial = RUN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp = TempDir::new("doc-context-export");
    temp.write("jacket.toml","[doc]\npath='lib'\nname='Tiny'\nhome='docs/home.md'\n[[doc.page]]\nfile='docs/language/context.md'\nname='Implicit context'\n");
    temp.write(
        "docs/home.md",
        "# Tiny\n\nRead the [implicit context](language/context.md).\n",
    );
    temp.write(
        "docs/language/context.md",
        "# Implicit context\n\n`#Context` is synthesized; `context` is active.\n",
    );
    temp.write("lib/Tiny/module.jai","// An owned library.\n\n// An ordinary user type.\nContext :: struct { field: int; }\n\n// An alias to the program type.\nActive :: #Context;\n\n// Uses the implicit type.\nuse :: (active: *#Context, alias: *Active, ordinary: *Context) {}\n");
    for base in ["/", "/reference/"] {
        let values = export(&temp, base, "current");
        let site = payload(&values, "site");
        assert_eq!(site["pageBase"], base);
        let declarations = &module(&values, "Tiny")["declarations"];
        assert_eq!(named(declarations, "Context")["kind"], "struct");
        assert_eq!(named(declarations, "Active")["kind"], "type");
        assert_eq!(named(declarations, "Active")["value"]["source"], "#Context");
        let parameters = &named(declarations, "use")["overloads"][0]["parameters"];
        assert_eq!(named(parameters, "active")["ty"]["source"], "*#Context");
        assert!(targets(&named(parameters, "active")["ty"]).is_empty());
        assert_eq!(
            targets(&named(parameters, "alias")["ty"]),
            ["api/Tiny/Active"]
        );
        assert_eq!(
            targets(&named(parameters, "ordinary")["ty"]),
            ["api/Tiny/Context"]
        );
        let home = values
            .iter()
            .find(|value| value["payload"] == "document" && value["data"]["kind"] == "home")
            .unwrap();
        assert!(targets(home).contains(&"guides/context"));
        let entries = &payload(&values, "search")["entries"];
        let alias = entries
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == "Active")
            .unwrap();
        assert_eq!(alias["kind"], "declaration");
        assert_eq!(alias["target"]["id"], "api/Tiny/Active");
        assert_eq!(alias["preview"]["kind"], "value");
        assert_eq!(alias["preview"]["value"], "#Context");
    }
}

#[test]
fn private_foreign_helpers_do_not_change_own_bindings_or_navigation_order() {
    let _serial = RUN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp = TempDir::new("doc-private-export");
    temp.write("jacket.toml","[doc]\npath='lib'\nname='Tiny'\n[[doc.category]]\nname='Standard library'\nmodules=['lib/**']\n[[doc.category]]\nname='Extensions'\nmodules=['lib/Ext']\n[[doc.page]]\nfile='guide.md'\nname='Guide'\n[[doc.section]]\nfile='tools.md'\nname='Tools'\n");
    temp.write("guide.md", "# Guide\n\nA guide.\n");
    temp.write("tools.md", "# Tools\n\nTool documentation.\n");
    temp.write("lib/Core/module.jai", "// Core.\n\nVALUE :: 1;\n");
    temp.write("lib/Ext/module.jai","// An owned API with private native helpers.\n\nCase :: struct { value: int; }\npublic_call :: native_call;\n#scope_file\nlib :: #library \"native\";\nnative_call :: () #foreign lib;\n");
    for base in ["/", "/reference/"] {
        let values = export(&temp, base, "current");
        let site = payload(&values, "site");
        assert_eq!(
            site["navigation"]
                .as_array()
                .unwrap()
                .iter()
                .map(|group| group["label"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["Guides", "Tools", "Standard library", "Extensions"]
        );
        let declarations = &module(&values, "Ext")["declarations"];
        assert_eq!(named(declarations, "Case")["binding"], false);
        assert_eq!(named(declarations, "public_call")["binding"], true);
        let entries = &payload(&values, "search")["entries"];
        assert_eq!(named(entries, "Case")["binding"], false);
        assert_eq!(named(entries, "public_call")["binding"], true);
        assert!(
            entries
                .as_array()
                .unwrap()
                .iter()
                .all(|entry| entry["name"] != "native_call")
        );
        assert!(targets(&site["navigation"]).contains(&"guides/guide"));
    }
}

#[test]
fn publication_metadata_reaches_real_exported_page_kinds_and_assets() {
    let _serial = RUN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp = TempDir::new("doc-publication-export");
    temp.write("jacket.toml","[doc]\npath='lib'\nname='Tiny'\nbreadcrumb='std'\nurl='https://example.com'\nsite_name='Social docs'\nog_image='https://example.com/og.png'\ncanonical_version='preferred'\n[[doc.page]]\nfile='guide.md'\nname='Guide'\n[[doc.section]]\nfile='rules.md'\nname='Rules'\n");
    temp.write(
        "lib/String/module.jai",
        "// Strings.\n\n// Trim & compare.\ntrim :: 42;\n",
    );
    temp.write("guide.md", "# Guide\n\nA short guide.\n");
    temp.write(
        "rules.md",
        "# Rules\n\nEvery rule.\n\n## Group\n\n### one\n\nThe first rule.\n",
    );
    for version in ["preferred", "archived"] {
        let values = export(&temp, "/", version);
        let site = payload(&values, "site");
        assert_eq!(site["name"], "Tiny");
        assert_eq!(site["breadcrumb"], "std");
        assert_eq!(site["socialSiteName"], "Social docs");
        let module = module(&values, "String");
        let item = named(&module["declarations"], "trim");
        assert_eq!(module["metadata"]["title"], "String — Tiny");
        assert_eq!(item["metadata"]["title"], "String.trim — Tiny");
        assert_eq!(item["metadata"]["noindex"], version != "preferred");
        assert_eq!(
            item["metadata"]["canonical"],
            if version == "preferred" {
                "https://example.com/api/String/trim"
            } else {
                "https://example.com/versions/preferred/api/String/trim"
            }
        );
        assert_eq!(item["metadata"]["structured"][0]["kind"], "tech_article");
        assert_eq!(
            item["metadata"]["structured"][1]["entries"][0]["name"],
            "std"
        );
        let documents = values
            .iter()
            .filter(|value| value["payload"] == "document")
            .map(|value| &value["data"])
            .collect::<Vec<_>>();
        for title in [
            "Guide — Tiny",
            "Rules — Tiny",
            "one — Tiny",
            "Social docs",
            "Page not found — Tiny",
        ] {
            assert!(
                documents
                    .iter()
                    .any(|page| page["metadata"]["title"] == title),
                "missing {title}"
            );
        }
        let missing = documents
            .iter()
            .find(|page| page["kind"] == "not_found")
            .unwrap();
        assert!(missing["metadata"]["url"].is_null() && missing["metadata"]["canonical"].is_null());
        assert_eq!(missing["metadata"]["noindex"], true);
        let sitemap = std::fs::read_to_string(temp.join("bundle/sitemap.xml")).unwrap();
        assert!(sitemap.contains("https://example.com/api/String/trim"));
        assert!(!sitemap.contains("404"));
        assert_eq!(
            std::fs::read_to_string(temp.join("bundle/robots.txt")).unwrap(),
            "Sitemap: https://example.com/sitemap.xml\n"
        );
    }
}

#[test]
fn conditional_fields_with_the_same_name_keep_distinct_formatted_anchors() {
    let _serial = RUN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp = TempDir::new("doc-conditional-field-export");
    temp.write("jacket.toml", "[doc]\npath='lib'\nname='Tiny'\n");
    temp.write("lib/Tiny/module.jai","Library_Info :: struct {\n    identifier: string;\n    #if OS == .WINDOWS { handle: u64; } else { handle: *void; }\n}\n");
    let values = export(&temp, "/", "current");
    let declaration = named(&module(&values, "Tiny")["declarations"], "Library_Info");
    let definition = &declaration["definitions"][0];
    let anchors = definition["anchors"].as_array().unwrap();
    assert_eq!(
        anchors
            .iter()
            .filter(|anchor| anchor["name"] == "handle")
            .count(),
        2,
        "both conditional field occurrences survive"
    );
    let ids = anchors
        .iter()
        .map(|anchor| anchor["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ids.iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        ids.len(),
        "conditional occurrences need distinct fragment IDs"
    );
    let code = definition["code"]["source"].as_str().unwrap();
    for anchor in anchors {
        assert_eq!(
            &code[anchor["start"].as_u64().unwrap() as usize
                ..anchor["end"].as_u64().unwrap() as usize],
            anchor["name"].as_str().unwrap()
        );
    }
    assert!(code.contains("handle: u64;") && code.contains("handle: *void;"));
    assert!(
        declaration["members"].as_array().unwrap().is_empty(),
        "formatted definitions already own field content; do not duplicate a misleading canonical field table"
    );
    let route = payload(&values, "site")["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["id"] == "api/Tiny/Library_Info")
        .unwrap();
    let ids = route["anchors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ids.iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        ids.len()
    );
}


#[test]
fn retired_renderer_options_explain_migration_without_creating_output() {
    let _serial = RUN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp = TempDir::new("doc-retired-renderer-options");
    temp.write("jacket.toml", "[doc]\npath='lib'\nname='Tiny'\n");
    temp.write("lib/Tiny/module.jai", "VALUE :: 1;\n");
    let jacket = Path::new(env!("CARGO_BIN_EXE_jacket"));
    for (flag, guidance) in [("--theme", "Astro"), ("--emit-input", "--export")] {
        let output = Command::new(jacket)
            .args([
                "doc",
                flag,
                "legacy",
                "--export-only",
                "--export",
                "bundle",
                "--out",
                "site",
            ])
            .env("JACKET_DOCS_BUILDER", temp.join("uninstalled-builder"))
            .env("JAIC_STDLIB", repo().join("stdlib"))
            .current_dir(&temp)
            .output()
            .unwrap();
        let message = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{flag}: {message}");
        assert!(
            message.contains("retired") && message.contains(guidance),
            "{flag}: {message}"
        );
        assert!(
            !temp.join("bundle").exists()
                && !temp.join("site").exists()
                && !temp.join("legacy").exists()
        );
    }
}
