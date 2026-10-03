use jai_modules::SourceProvider;
use jai_runtime::{Error, Options, Script, SourceBundle, browser_target};
use std::path::Path;

#[test]
fn source_uses_shared_engine_with_real_runtime_phase() {
    let script = Script::from_source("seed :: #run answer(); answer :: () -> int { if #compile_time return 40; return 900; } main :: () -> int { if #compile_time return 700; return seed + 2; }", Options::default()).unwrap();
    assert_eq!(script.run(&[]).unwrap().exit_code, 42);
}
#[test]
fn arguments_preserve_boundaries_and_real_string_storage() {
    let script = Script::from_source("main :: (args: []string) -> int { if args.count != 2 return 1; if args[0] != \"hello world\" return 2; if args[1] != \"\" return 3; return 42; }", Options::default()).unwrap();
    assert_eq!(
        script
            .run(&["hello world".into(), "".into()])
            .unwrap()
            .exit_code,
        42
    );
    assert_eq!(script.run(&[]).unwrap().exit_code, 1);
}
#[test]
fn bundle_resolves_supplied_loads_without_filesystem_fallback() {
    let mut bundle = SourceBundle::default();
    let path = bundle
        .insert(
            "main.jai",
            b"#load \"helper.jai\"; main :: () -> int { return answer; }".to_vec(),
        )
        .unwrap();
    bundle
        .insert("helper.jai", b"answer :: 42;".to_vec())
        .unwrap();
    assert_eq!(
        Script::prepare(&path, &bundle, Options::default())
            .unwrap()
            .run(&[])
            .unwrap()
            .exit_code,
        42
    );
    assert!(bundle.read(Path::new("/etc/passwd")).is_err());
    assert!(bundle.insert("../escape.jai", vec![]).is_err());
    assert!(!bundle.is_file(Path::new("/Users/matthew/projects/jai/README.md")));
}
#[test]
fn browser_target_uses_actual_32_bit_layout() {
    let options = Options {
        target: browser_target(),
        ..Options::default()
    };
    let script =
        Script::from_source("main :: () -> int { return size_of(*int) + 38; }", options).unwrap();
    assert_eq!(script.run(&[]).unwrap().exit_code, 42);
}
#[test]
fn runtime_limits_and_failure_are_real() {
    let options = Options {
        limits: jai_runtime::Limits {
            fuel: 100,
            ..Default::default()
        },
        ..Options::default()
    };
    let script = Script::from_source("main :: () { while true {} }", options).unwrap();
    assert!(matches!(
        script.run(&[]),
        Err(Error::Execution(jai_vm::Error::Limit(
            jai_vm::LimitKind::Fuel
        )))
    ));
    let script =
        Script::from_source("main :: () -> int { return 42; }", Options::default()).unwrap();
    assert!(matches!(
        script.run(&["extra".into()]),
        Err(Error::Arguments(_))
    ));
}
#[test]
fn globals_are_isolated_between_runs_and_void_returns_success() {
    let script = Script::from_source(
        "counter: int = 41; main :: () -> int { counter += 1; return counter; }",
        Options::default(),
    )
    .unwrap();
    assert_eq!(script.run(&[]).unwrap().exit_code, 42);
    assert_eq!(script.run(&[]).unwrap().exit_code, 42);
    assert_eq!(
        Script::from_source("main :: () {}", Options::default())
            .unwrap()
            .run(&[])
            .unwrap()
            .exit_code,
        0
    );
}
#[test]
fn bundle_limits_replace_existing_content_without_partial_write() {
    let mut bundle = SourceBundle::new(4);
    bundle.insert("main.jai", b"1234".to_vec()).unwrap();
    assert!(bundle.insert("other.jai", b"5".to_vec()).is_err());
    bundle.insert("main.jai", b"42".to_vec()).unwrap();
    assert_eq!(bundle.byte_count(), 2);
    assert_eq!(bundle.read(Path::new("main.jai")).unwrap(), b"42");
}
#[test]
fn context_defaults_and_shared_mutation_execute_normally() {
    let script = Script::from_source("#add_context number: int = 33; change :: () { context.number += 9; } main :: () -> int { change(); return context.number; }", Options::default()).unwrap();
    assert_eq!(script.run(&[]).unwrap().exit_code, 42);
    assert_eq!(script.run(&[]).unwrap().exit_code, 42);
}
#[test]
fn foreign_metadata_does_not_authorize_native_execution() {
    let script = Script::from_source("Crt :: #library,system \"libc\"; absolute :: (value: s32) -> s32 #foreign Crt \"abs\"; main :: () -> int { return absolute(-42); }", Options::default()).unwrap();
    assert!(matches!(
        script.run(&[]),
        Err(Error::HostBindingRequired(
            jai_runtime::MissingHostBinding::ForeignProcedure(_)
        ))
    ));
    assert!(script.host_capabilities().is_empty());
}
#[test]
fn runtime_refuses_compile_time_only_direct_and_indirect_calls() {
    for source in [
        "only :: () -> int #compile_time { return 42; } main :: () -> int { return only(); }",
        "only :: () -> int #compile_time { return 42; } main :: () -> int { callback := only; return callback(); }",
    ] {
        let script = Script::from_source(source, Options::default()).unwrap();
        assert!(matches!(
            script.run(&[]),
            Err(Error::Execution(jai_vm::Error::CompileTimeOnlyProcedure(_)))
        ));
    }
}

#[test]
fn virtual_bundle_names_keep_one_closed_posix_identity() {
    let mut bundle = SourceBundle::default();
    let path = bundle
        .insert(
            "sub/../main.jai",
            b"main :: () -> int { return 42; }".to_vec(),
        )
        .unwrap();
    assert_eq!(path, Path::new("/jai-script/main.jai"));
    assert_eq!(bundle.entry_path("./main.jai").unwrap(), path);
    assert_eq!(bundle.entry_path(&path).unwrap(), path);
    for name in [
        "/jai-script/extra.jai",
        "../escape.jai",
        "C:/extra.jai",
        "a\\b.jai",
        "a\0b.jai",
    ] {
        assert!(bundle.insert(name, vec![]).is_err(), "{name:?}");
    }
    assert!(
        bundle
            .read(Path::new("/jai-script/../elsewhere/main.jai"))
            .is_err()
    );
    assert!(
        bundle
            .read(Path::new("/jai-script-other/main.jai"))
            .is_err()
    );
}
