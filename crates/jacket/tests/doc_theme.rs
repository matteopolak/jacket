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
            .args(["test", "-j", "3", "--path"])
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
        let Some((jacket, jaic)) = tools() else {
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
        let mut command = if native.is_file() {
            Command::new(native)
        } else {
            let mut command = Command::new(&jaic);
            command
                .arg(cache.join("src/main.jai"))
                .args(["-interp", "--"]);
            command
        };
        let bad = command
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
        let module = std::fs::read_to_string(temp.join("site/api/Tiny/index.html")).unwrap();
        assert!(module.contains("href=\"/api/Tiny/answer/\""));
        let item = std::fs::read_to_string(temp.join("site/api/Tiny/answer/index.html")).unwrap();
        assert!(item.contains("The answer."));
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
            "body{color:green}"
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
            "body{color:blue}"
        );
        temp.write("bad-theme/parts.jai", "this is not valid Jai");
        let compile_error = run(&["--theme", "bad-theme"]);
        assert_eq!(compile_error.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&compile_error.stderr).contains("parts.jai"));
        // A link in trusted prose must be checked by the Jai writer as well.
        temp.write("broken.md", "# Guide\n\n[Missing](/missing/)\n");
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
            message.contains("/missing/` leads to no written file"),
            "{message}"
        );
    }
}
