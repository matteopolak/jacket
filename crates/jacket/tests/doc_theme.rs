//! These tests share the integration suite's gate: jaic must be built beside jacket.
mod doc {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use test_support::TempDir;

    static RUN: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn tools() -> Option<(PathBuf, PathBuf)> {
        let jacket = PathBuf::from(env!("CARGO_BIN_EXE_jacket"));
        let jaic = jacket
            .parent()?
            .join(format!("jaic{}", std::env::consts::EXE_SUFFIX));
        if !jaic.is_file() {
            eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
            return None;
        }
        Some((jacket, jaic))
    }

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn theme_markup_tests_run_in_jai() {
        let _serial = RUN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((jacket, _)) = tools() else {
            return;
        };
        let output = Command::new(jacket)
            .args(["test", "-j", "2", "--path"])
            .arg(repo().join("crates/jacket/src/doc/site"))
            .env("JAIC_STDLIB", repo().join("stdlib"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn tiny_library_writes_pages_and_checks_fragment_links() {
        let _serial = RUN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((jacket, _)) = tools() else {
            return;
        };
        let temp = TempDir::new("doc-theme-e2e");
        temp.write("jacket.toml", "[doc]\npath = 'lib'\nname = 'Tiny'\n");
        temp.write(
            "lib/Tiny/module.jai",
            "// Tiny module.\n//\n// See [answer].\n\n// The answer.\nanswer :: 42;\n",
        );
        let run = |extra: &[&str]| {
            Command::new(&jacket)
                .arg("doc")
                .args(extra)
                .arg("--out")
                .arg(temp.join("site"))
                .env("JAIC_STDLIB", repo().join("stdlib"))
                .env("XDG_CACHE_HOME", temp.join("cache"))
                .current_dir(&temp)
                .output()
                .unwrap()
        };
        let output = run(&["--emit-input", "input"]);
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let input_file = temp.join("input/site.json");
        let mut bad_input: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&input_file).unwrap()).unwrap();
        bad_input["format"] = 999.into();
        std::fs::write(&input_file, serde_json::to_vec(&bad_input).unwrap()).unwrap();
        let cache = std::fs::read_dir(temp.join("cache/jacket/doc-site"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let native = cache.join(format!("doc-site{}", std::env::consts::EXE_SUFFIX));
        assert!(native.is_file(), "the theme is always compiled natively");
        let bad = Command::new(native)
            .env("JAIC_STDLIB", repo().join("stdlib"))
            .arg(temp.join("input"))
            .arg(temp.join("unused"))
            .arg("--check")
            .output()
            .unwrap();
        assert_eq!(bad.status.code(), Some(2));
        assert!(!temp.join("unused").exists());
        assert!(temp.join("site/.jacket-doc").is_file());
        assert!(temp.join("site/search-index.json").is_file());
        assert!(
            std::fs::read_to_string(temp.join("site/index.html"))
                .unwrap()
                .contains("Tiny")
        );
        let module = std::fs::read_to_string(temp.join("site/api/Tiny.html")).unwrap();
        assert!(module.contains("href=\"/api/Tiny/answer\""));
        let item = std::fs::read_to_string(temp.join("site/api/Tiny/answer.html")).unwrap();
        assert!(item.contains("The answer."));
        let icons = std::fs::read(temp.join("site/assets/icons.svg")).unwrap();
        let stamp = icons.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        }) >> 32;
        for icon in ["search", "system"] {
            assert!(item.contains(&format!("href=\"/assets/icons.svg?v={stamp:08x}#{icon}\"")));
        }
        assert!(!item.contains("assets/icons.svg#"));
        assert!(run(&["--check"]).status.success());
        temp.write("theme/assets/site.css", "body{color:green}");
        temp.write("theme/assets/extra.txt", "extra");
        temp.write(
            "jacket.toml",
            "[doc]\npath = 'lib'\nname = 'Tiny'\ntheme = 'theme'\n",
        );
        assert!(run(&[]).status.success());
        assert_eq!(
            std::fs::read_to_string(temp.join("site/assets/site.css")).unwrap(),
            "body{color:green}\n"
        );
        assert_eq!(
            std::fs::read_to_string(temp.join("site/assets/extra.txt")).unwrap(),
            "extra"
        );
        let overridden = std::fs::read_to_string(temp.join("site/index.html")).unwrap();
        let original_css = item
            .split("assets/site.css?v=")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(!overridden.contains(&format!("assets/site.css?v={original_css}")));
        temp.write("flag-theme/assets/site.css", "body{color:blue}");
        assert!(run(&["--theme", "flag-theme"]).status.success());
        assert_eq!(
            std::fs::read_to_string(temp.join("site/assets/site.css")).unwrap(),
            "body{color:blue}\n"
        );
        temp.write("bad-theme/parts.jai", "this is not valid Jai");
        let compile_error = run(&["--theme", "bad-theme"]);
        assert_eq!(compile_error.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&compile_error.stderr).contains("parts.jai"));
        // A link in trusted prose must be checked by the Jai writer as well.
        temp.write("broken.md", "# Guide\n\n[Missing](/missing)\n");
        temp.write(
            "jacket.toml",
            "[doc]\npath = 'lib'\n[[doc.page]]\nfile = 'broken.md'\nname = 'Broken'\n",
        );
        let broken = run(&["--check"]);
        assert_eq!(broken.status.code(), Some(1));
        let message = format!(
            "{}{}",
            String::from_utf8_lossy(&broken.stdout),
            String::from_utf8_lossy(&broken.stderr)
        );
        assert!(
            message.contains("/missing` leads to no written file"),
            "{message}"
        );
    }

    #[test]
    fn heads_match_page_kinds_and_publication_version() {
        let _serial = RUN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let jacket = Path::new(env!("CARGO_BIN_EXE_jacket"));
        if !jacket
            .parent()
            .unwrap()
            .join(format!("jaic{}", std::env::consts::EXE_SUFFIX))
            .is_file()
        {
            return;
        }
        let temp = TempDir::new("doc-metadata");
        temp.write("jacket.toml", "[doc]\npath='lib'\nname='Tiny'\nbreadcrumb='std'\nurl='https://example.com'\nsite_name='Tiny std'\nog_image='https://example.com/og.png'\ncanonical_version='2'\n[[doc.page]]\nfile='guide.md'\nname='Guide'\n[[doc.section]]\nfile='rules.md'\nname='Rules'\n");
        temp.write(
            "lib/String/module.jai",
            "// Strings.\n\n// Trim & compare.\ntrim :: 42;\n",
        );
        temp.write("guide.md", "# Guide\n\nA short guide.\n");
        temp.write(
            "rules.md",
            "# Rules\n\nEvery rule.\n\n## Group\n\n### one\n\nThe first rule.\n",
        );
        let run = |version: &str| {
            Command::new(jacket)
                .args([
                    "doc",
                    "--version",
                    version,
                    "--out",
                    "site",
                    "--emit-input",
                    "input",
                ])
                .current_dir(&temp)
                .env("JAIC_STDLIB", repo.join("stdlib"))
                .env("XDG_CACHE_HOME", temp.join("cache"))
                .output()
                .unwrap()
        };
        for version in ["2", "1"] {
            let output = run(version);
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let pages = [
                ("index.html", "Tiny std"),
                ("api/String.html", "String — Tiny std"),
                ("api/String/trim.html", "String.trim — Tiny std"),
                ("rules.html", "Rules — Tiny std"),
                ("rules/one.html", "one — Tiny std"),
                ("guides/guide.html", "Guide — Tiny std"),
                ("404.html", "Page not found — Tiny std"),
            ];
            for (path, title) in pages {
                let page = std::fs::read_to_string(temp.join("site").join(path)).unwrap();
                let head = page.split_once("</head>").unwrap().0;
                assert!(head.contains(&format!("<title>{title}</title>")), "{head}");
                let description = head
                    .split_once("name=\"description\" content=\"")
                    .unwrap()
                    .1
                    .split('"')
                    .next()
                    .unwrap();
                assert!(description.chars().count() <= 155);
                assert!(!description.contains(['<', '>']));
                for property in [
                    "og:title",
                    "og:description",
                    "og:site_name",
                    "og:type",
                    "og:image",
                ] {
                    assert!(head.contains(property), "{path}: {property}");
                }
                assert_eq!(head.contains("og:url"), path != "404.html");
                assert!(head.contains("summary_large_image"));
                assert!(head.contains("rel=\"icon\""));
                assert_eq!(
                    head.contains("content=\"noindex\""),
                    version == "1" || path == "404.html"
                );
                let folder = path.strip_suffix(".html").unwrap_or(path);
                let folder = if folder == "index" {
                    ""
                } else {
                    folder
                };
                let target = if version == "1" {
                    format!("https://example.com/versions/2/{folder}")
                } else {
                    format!("https://example.com/{folder}")
                };
                // A 404 is served at many URLs, so it has no canonical one.
                assert_eq!(
                    head.contains(&format!("rel=\"canonical\" href=\"{target}\"")),
                    path != "404.html"
                );
                if let Some((_, json)) = head.split_once("application/ld+json\">") {
                    let data: serde_json::Value =
                        serde_json::from_str(json.split_once("</script>").unwrap().0).unwrap();
                    let breadcrumbs = data
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|d| d["@type"] == "BreadcrumbList")
                        .unwrap();
                    assert_eq!(breadcrumbs["itemListElement"][0]["name"], "std");
                    assert_eq!(breadcrumbs["itemListElement"][1]["name"], "String");
                    if path.contains("/trim") {
                        assert_eq!(data[0]["@type"], "TechArticle");
                        assert_eq!(breadcrumbs["itemListElement"][2]["name"], "trim");
                    }
                }
                if path == "404.html" {
                    // Absolute from the base URL, so it works at any depth on any host.
                    assert!(page.contains("href=\"/api/String\""));
                    assert!(!page.contains("href=\"https://example.com/api/"));
                }
            }
            let sitemap = std::fs::read_to_string(temp.join("site/sitemap.xml")).unwrap();
            assert!(sitemap.contains("https://example.com/api/String/trim"));
            assert!(!sitemap.contains("404.html"));
            assert_eq!(
                std::fs::read_to_string(temp.join("site/robots.txt")).unwrap(),
                "Sitemap: https://example.com/sitemap.xml\n"
            );
        }
    }

    #[test]
    fn implicit_context_guide_keeps_ordinary_context_declarations_distinct() {
        let _serial = RUN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((jacket, _)) = tools() else {
            return;
        };
        let temp = TempDir::new("doc-context");
        temp.write("jacket.toml", "[doc]\npath='lib'\nname='Tiny'\nhome='docs/home.md'\n[[doc.page]]\nfile='docs/language/context.md'\nname='Implicit context'\n");
        temp.write(
            "docs/home.md",
            "# Tiny\n\nRead the [implicit context](language/context.md).\n",
        );
        temp.write(
            "docs/language/context.md",
            "# Implicit context\n\n`#Context` is synthesized; `context` is active.\n",
        );
        temp.write("lib/Tiny/module.jai", "// An owned library.\n\n// An ordinary user type.\nContext :: struct { field: int; }\n\n// An alias to the program type.\nActive :: #Context;\n\n// Uses the implicit type.\nuse :: (active: *#Context, alias: *Active, ordinary: *Context) {}\n");
        for base in ["/", "/reference/"] {
            let output = Command::new(&jacket)
                .args(["doc", "--base-url", base, "--out", "site"])
                .current_dir(&temp)
                .env("JAIC_STDLIB", repo().join("stdlib"))
                .env("XDG_CACHE_HOME", temp.join("cache"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let checked = Command::new(&jacket)
                .args(["doc", "--check", "--base-url", base])
                .current_dir(&temp)
                .env("JAIC_STDLIB", repo().join("stdlib"))
                .env("XDG_CACHE_HOME", temp.join("cache"))
                .output()
                .unwrap();
            assert!(
                checked.status.success(),
                "{}{}",
                String::from_utf8_lossy(&checked.stdout),
                String::from_utf8_lossy(&checked.stderr)
            );
            let home = std::fs::read_to_string(temp.join("site/index.html")).unwrap();
            assert!(home.contains(&format!("href=\"{base}guides/context\"")));
            assert!(temp.join("site/guides/context.html").is_file());
            let ordinary =
                std::fs::read_to_string(temp.join("site/api/Tiny/Context.html")).unwrap();
            assert!(ordinary.contains("An ordinary user type."));
            let signature = std::fs::read_to_string(temp.join("site/api/Tiny/use.html")).unwrap();
            assert!(signature.contains(&format!("href=\"{base}api/Tiny/Context\"")));
            assert!(signature.contains("(active: *#Context, alias: *<a"));
            let search: serde_json::Value = serde_json::from_slice(
                &std::fs::read(temp.join("site/search-index.json")).unwrap(),
            )
            .unwrap();
            let alias = search["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row[1] == "Active")
                .unwrap();
            assert_eq!(alias[2], "type");
            assert_eq!(alias[4], ":: #Context");
        }
    }

    #[test]
    fn private_foreign_helpers_and_documentation_sidebar_order_reach_html() {
        let _serial = RUN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((jacket, _)) = tools() else {
            return;
        };
        let temp = TempDir::new("doc-private-foreign");
        temp.write("jacket.toml", "[doc]\npath='lib'\nname='Tiny'\n[[doc.category]]\nname='Standard library'\nmodules=['lib/**']\n[[doc.category]]\nname='Extensions'\nmodules=['lib/Ext']\n[[doc.page]]\nfile='guide.md'\nname='Guide'\n[[doc.section]]\nfile='tools.md'\nname='Tools'\n");
        temp.write("guide.md", "# Guide\n\nA guide.\n");
        temp.write("tools.md", "# Tools\n\nTool documentation.\n");
        temp.write("lib/Core/module.jai", "// Core.\n\nVALUE :: 1;\n");
        temp.write("lib/Ext/module.jai", "// An owned API with private native helpers.\n\nCase :: struct { value: int; }\npublic_call :: native_call;\n#scope_file\nlib :: #library \"native\";\nnative_call :: () #foreign lib;\n");
        for base in ["/", "/reference/"] {
            let output = Command::new(&jacket)
                .args(["doc", "--base-url", base, "--out", "site"])
                .current_dir(&temp)
                .env("JAIC_STDLIB", repo().join("stdlib"))
                .env("XDG_CACHE_HOME", temp.join("cache"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let home = std::fs::read_to_string(temp.join("site/index.html")).unwrap();
            let positions: Vec<_> = ["Guides", "Tools", "Standard library", "Extensions"]
                .iter()
                .map(|label| {
                    home.find(&format!("<div class=\"nav-h\">{label}</div>"))
                        .unwrap()
                })
                .collect();
            assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
            let case = std::fs::read_to_string(temp.join("site/api/Ext/Case.html")).unwrap();
            assert!(!case.contains("class=\"badge foreign\""));
            let foreign =
                std::fs::read_to_string(temp.join("site/api/Ext/public_call.html")).unwrap();
            assert!(foreign.contains("class=\"badge foreign\""));
            let search: serde_json::Value = serde_json::from_slice(
                &std::fs::read(temp.join("site/search-index.json")).unwrap(),
            )
            .unwrap();
            let items = search["items"].as_array().unwrap();
            let flags = |name| {
                items.iter().find(|row| row[1] == name).unwrap()[6]
                    .as_u64()
                    .unwrap()
            };
            assert_eq!(flags("Case") & 2, 0);
            assert_eq!(flags("public_call") & 2, 2);
            assert!(home.contains(&format!("href=\"{base}guides/guide\"")));
        }
    }
}
