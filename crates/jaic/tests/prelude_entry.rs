use jaic::sema::VirtualFs;
use std::path::Path;

#[test]
fn explicit_prelude_selection() {
    if jacket_config::env::PRELUDE_TEST_CHILD.os().is_some() {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/prelude/module.jai", b"".as_slice());
        fs.insert("/lib/Base.jai", b"".as_slice());
        fs.insert("/external/Base.jai", b"".as_slice());
        let expected = jacket_config::env::EXPECTED_PRELUDE.get().unwrap();
        assert_eq!(
            jaic::find_preload(&fs, Path::new("/lib")),
            (expected != "missing").then(|| expected.clone().into())
        );
        assert_eq!(
            jaic::enclosing_stdlib(&fs, Path::new("/lib/internal/file.jai"), Path::new("/lib")),
            (expected != "missing").then(|| "/lib".into())
        );
        let mut options = jaic::sema::Options::host();
        options.set_stdlib(&fs, Path::new("/lib"));
        assert_eq!(options.stdlib_root(), Some(Path::new("/lib")));
        let selected = if expected == "missing" {
            "/lib/missing.jai"
        } else {
            &expected
        };
        assert_eq!(options.preload.as_deref(), Some(Path::new(selected)));
        return;
    }
    for (setting, expected) in [
        ("", "/lib/prelude/module.jai"),
        ("Base.jai", "/lib/Base.jai"),
        ("/external/Base.jai", "/external/Base.jai"),
        ("missing.jai", "missing"),
    ] {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "explicit_prelude_selection", "--nocapture"])
            .env("JAIC_PRELUDE_TEST_CHILD", "1")
            .env("JAIC_PRELUDE", setting)
            .env("JAIC_STDLIB", "/lib")
            .env("EXPECTED_PRELUDE", expected)
            .status()
            .unwrap();
        assert!(status.success(), "JAIC_PRELUDE={setting}");
    }
}
