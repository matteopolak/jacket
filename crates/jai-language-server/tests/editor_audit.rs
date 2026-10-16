//! Original causal IDE audit fixtures, independent of corpus and library implementations.
use jaic::sema::{Compiler, Options, VirtualFs, ide::IdeFacts};
use std::{path::Path, rc::Rc};

fn compiler(source: &str, dependency: &str) -> Compiler {
    let mut fs = VirtualFs::default();
    fs.insert("/audit/main.jai", source.as_bytes());
    fs.insert(
        "/audit/other.jai",
        "loaded :: () { loaded_missing; }".as_bytes(),
    );
    fs.insert("/modules/Dependency/module.jai", dependency.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    options.import_paths = vec!["/modules".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.ide = Some(Box::new(IdeFacts::new(vec!["/audit/".into()])));
    assert!(
        compiler
            .compile_program(Path::new("/audit/main.jai"))
            .is_err()
    );
    compiler
}

#[test]
fn audit_does_not_evaluate_unselected_dependency_headers() {
    let mut c = compiler(
        "#import \"Dependency\";\nbroken :: missing;\nalias :: unselected;\nanswer :: 42;\nmain :: () { value := answer; }",
        "Hidden :: [#run unneeded_size()] s64;\nunneeded_size :: () -> s64 { return 1; }\nunselected :: (value: Hidden) {}\n",
    );
    assert!(
        !c.program
            .func_names
            .iter()
            .any(|n| n.contains("unneeded_size"))
    );
    let errors = c.ide_check_all();
    assert!(
        errors.iter().any(|e| e.message.contains("missing")),
        "{errors:?}"
    );
    assert!(
        !c.program
            .func_names
            .iter()
            .any(|n| n.contains("unneeded_size")),
        "unselected headers outside the existing recording roots must not trigger evaluation"
    );
}

#[test]
fn bodiless_audit_does_not_evaluate_unselected_dependency_headers() {
    let mut c = compiler(
        "#import \"Dependency\";\nbroken :: missing;\nalias :: unselected;\nmain :: () {}",
        "Hidden :: [#run unneeded_size()] s64;\nunneeded_size :: () -> s64 { return 1; }\nunselected :: (value: Hidden) #foreign;\n",
    );
    assert!(
        !c.program
            .func_names
            .iter()
            .any(|n| n.contains("unneeded_size"))
    );
    let errors = c.ide_check_all();
    assert!(errors.iter().any(|e| e.message.contains("missing")));
    assert!(
        !c.program
            .func_names
            .iter()
            .any(|n| n.contains("unneeded_size"))
    );
}

#[test]
fn sketching_does_not_retry_unrelated_failed_bodies_after_compile_time_progress() {
    let mut c = compiler(
        "broken :: missing;\nfailed :: () { nested :: () {}; nested(); missing_body; }\nfirst :: (value: $T) { ignored := #run value_of(); result := value; }\nsecond :: (value: $T) { ignored := #run value_of(); result := value; }\nvalue_of :: () -> s64 { return 1; }\nmain :: () {}",
        "",
    );
    let errors = c.ide_check_all();
    assert!(errors.iter().any(|e| e.message.contains("missing_body")));
    let failed = c
        .program
        .func_names
        .iter()
        .position(|n| n.contains("failed"))
        .unwrap();
    assert!(c.program.funcs[failed].is_none());
    let nested_before = c
        .program
        .func_names
        .iter()
        .filter(|n| n.contains("nested"))
        .count();
    assert!(nested_before > 0);
    c.add_source(&jaic::sema::ProgramSource::String(
        "unrelated_progress :: 42;".into(),
    ))
    .unwrap();
    c.ide_sketch_polymorphs();
    let ide = c.ide.as_ref().unwrap();
    assert!(ide.refs.len() > ide.sketch_from.as_ref().unwrap().refs);
    assert_eq!(
        ide.expansions.len(),
        2,
        "both sketches must actually check their #run expressions"
    );
    for name in ["first#", "second#"] {
        let at = c
            .program
            .func_names
            .iter()
            .position(|n| n.starts_with(name))
            .unwrap();
        assert!(
            c.program.funcs[at].is_some(),
            "selected sketch {name} must finish"
        );
    }
    assert!(
        c.program.funcs[failed].is_none(),
        "a sketch must not recheck an unrelated failed body with stand-in recovery"
    );
    let nested_after = c
        .program
        .func_names
        .iter()
        .filter(|n| n.contains("nested"))
        .count();
    assert_eq!(
        nested_after, nested_before,
        "sketch work must follow its actual reachable bodies"
    );
}

#[test]
fn reachable_dependency_and_independent_recorded_errors_remain_checked() {
    let mut c = compiler(
        "#import \"Dependency\";\n#load \"other.jai\";\nbroken :: missing;\nindependent :: () { other_missing; }\nmain :: () { array: [1] s64; needed(array); }",
        "Hidden :: [#run required_size()] s64;\nrequired_size :: () -> s64 { return 1; }\nneeded :: (value: Hidden) { dependency_missing; }",
    );
    let errors = c.ide_check_all();
    for name in [
        "missing",
        "other_missing",
        "dependency_missing",
        "loaded_missing",
    ] {
        assert!(
            errors.iter().any(|e| e.message.contains(name)),
            "{name}: {errors:?}"
        );
    }
    assert!(
        c.program
            .func_names
            .iter()
            .any(|n| n.contains("required_size")),
        "reachable dependency headers must still evaluate their actual types"
    );
}

#[test]
fn audit_shared_failed_generator_is_not_relowered_without_semantic_progress() {
    let mut source = String::from(
        "broken :: missing;\nfailed_generator :: () -> string { nested_repeat :: () {}; nested_repeat(); return missing_generator; }\nindependent :: () { other_missing; }\nhealthy :: () { answer := 42; }\n",
    );
    for n in 0..16 {
        source.push_str(&format!(
            "caller_{n} :: () {{ #insert #run failed_generator(); }}\n"
        ));
    }
    source.push_str("main :: () {}\n");
    let mut c = compiler(&source, "");
    let errors = c.ide_check_all();
    for name in ["missing_generator", "other_missing"] {
        assert!(
            errors.iter().any(|e| e.message.contains(name)),
            "{name}: {errors:?}"
        );
    }
    assert_eq!(
        c.program
            .func_names
            .iter()
            .filter(|n| n.as_str() == "nested_repeat")
            .count(),
        1,
        "each compile-time caller must reuse an unchanged failed generator result"
    );
    let generator = c
        .program
        .func_names
        .iter()
        .position(|n| n == "failed_generator")
        .unwrap();
    assert!(c.program.funcs[generator].is_none());
    let healthy = c
        .program
        .func_names
        .iter()
        .position(|n| n == "healthy")
        .unwrap();
    assert!(c.program.funcs[healthy].is_some());
    c.add_source(&jaic::sema::ProgramSource::String(
        "missing_generator :: \"generated :: 1;\";".into(),
    ))
    .unwrap();
    let retried = c.ide_check_all();
    assert!(
        !retried
            .iter()
            .any(|e| e.message.contains("unknown identifier `missing_generator`")),
        "{retried:?}"
    );
    assert!(
        c.program.funcs[generator].is_some(),
        "new source must permit a real successful retry"
    );
}
