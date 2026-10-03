//! Run only self-written source and freshly emitted objects for caller defaults.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::target::NativeTarget;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

const LOCATION: &str = "Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }\n";
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execute(header: &str, body: &str, call: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-caller-native-é🦀-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    let fixture = Fixture(fs::canonicalize(root).unwrap());
    let input = fixture.0.join("café.jai");
    fs::write(&input, "").unwrap();
    let input = fs::canonicalize(input).unwrap();
    let filename = input.to_str().unwrap();
    let source = format!("{LOCATION}{header}\nmain :: ()->int {{ {body} }}")
        .replace("FILENAME", filename)
        .replace("DIRECTORY", fixture.0.to_str().unwrap());
    let offset = source.rfind(call).unwrap();
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
    let source = source.replace("EXPECTED", &(line * 1000 + column).to_string());
    fs::write(&input, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    let target = NativeTarget::new().unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(target.layout_policy().unwrap()),
            ..jai_sema::ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let object = fixture.0.join("program.o");
    let executable = fixture.0.join("program");
    target.write_object(&module, &object).unwrap();
    let linked = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut child = Command::new(executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42));
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("caller fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
const PROBE_BODY: &str = "expected := \"FILENAME\"; if loc.fully_pathed_filename.count != expected.count return 0; for i: 0..expected.count-1 { if loc.fully_pathed_filename[i] != expected[i] return 0; } return loc.line_number*1000+loc.character_number;";

#[test]
fn direct_and_named_callback_defaults_keep_exact_filename_and_unicode_columns() {
    execute(
        &format!("probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }}"),
        "text := \"é🦀\"; return ifx probe() == EXPECTED then 42 else 1;",
        "probe()",
    );
    execute(
        &format!("probe :: (value: int=1, loc := #caller_location)->int {{ {PROBE_BODY} }}"),
        "callback := probe; return ifx callback(value=9) == EXPECTED then 42 else 1;",
        "callback(value=9)",
    );
}

#[test]
fn generic_and_local_defaults_materialize_at_each_source_call() {
    execute(
        &format!("probe :: (value: $T, loc := #caller_location)->int {{ {PROBE_BODY} }}"),
        "return ifx probe(9) == EXPECTED then 42 else 1;",
        "probe(9)",
    );
    execute(
        "",
        &format!(
            "probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }} return ifx probe() == EXPECTED then 42 else 1;"
        ),
        "probe()",
    );
}

#[test]
fn compile_time_and_quoted_calls_keep_source_phase_and_original_location() {
    execute(
        &format!("probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }}"),
        "return ifx (#run probe()) == EXPECTED then 42 else 1;",
        "probe()",
    );
    for insertion in ["#insert quoted", "#insert,scope() quoted"] {
        execute(
            &format!("probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }}"),
            &format!(
                "quoted :: #code probe();\nreturn ifx ({insertion}) == EXPECTED then 42 else 1;"
            ),
            "probe()",
        );
    }
}

#[test]
fn explicit_location_arguments_and_defaults_keep_the_supplied_record() {
    let literal = "Source_Code_Location.{fully_pathed_filename=\"explicit.jai\", line_number=42, character_number=9}";
    let body = "if loc.fully_pathed_filename.count != 12 return 0; if loc.fully_pathed_filename[0] != #char \"e\" return 0; if loc.character_number != 9 return 0; return loc.line_number;";
    execute(
        &format!("probe :: (loc := #caller_location)->int {{ {body} }}"),
        &format!("return probe({literal});"),
        "probe(",
    );
    execute(
        &format!("probe :: (loc: Source_Code_Location = {literal})->int {{ {body} }}"),
        "callback := probe; return callback();",
        "callback()",
    );
    execute(
        &format!("probe :: (value: $T, loc: Source_Code_Location = {literal})->int {{ {body} }}"),
        "return probe(9);",
        "probe(9)",
    );
}

#[test]
fn literal_source_directives_use_exact_native_source_records() {
    execute(
        "",
        "text := \"é🦀\"; loc := #location(); expected := \"FILENAME\"; filename := #file; if filename.count != expected.count return 0; for i: 0..expected.count-1 { if filename[i] != expected[i] return 0; if loc.fully_pathed_filename[i] != expected[i] return 0; } if #line != loc.line_number return 0; return ifx loc.line_number*1000+loc.character_number == EXPECTED then 42 else 1;",
        "#location()",
    );
    execute(
        "",
        "loc := #run #location(); return ifx loc.line_number*1000+loc.character_number == EXPECTED then 42 else 1;",
        "#location()",
    );
    execute(
        &format!("probe :: (loc: Source_Code_Location)->int {{ {PROBE_BODY} }}"),
        "return ifx probe(#location()) == EXPECTED then 42 else 1;",
        "#location()",
    );
    execute(
        &format!("probe :: (value: $T, loc := #location())->int {{ {PROBE_BODY} }}"),
        "return ifx probe(9) == EXPECTED then 42 else 1;",
        "#location()",
    );
}

#[test]
fn macro_generated_default_reports_the_actual_invocation() {
    execute(
        &format!(
            "probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }}\nemit :: (target: Code) #expand {{ (#insert target) = probe(); }}"
        ),
        "value := 0; emit(value); return ifx value == EXPECTED then 42 else 1;",
        "emit(value)",
    );
}

#[test]
fn filepath_is_the_actual_directory_across_constant_and_runtime_phases() {
    execute(
        "directory :: #filepath; probe :: (value: $T, path := #filepath)->string { return path; }",
        "expected := \"DIRECTORY\"; local_directory :: #filepath; runtime := #filepath; baked := #run #filepath; supplied := probe(9, #filepath); defaulted := probe(9); if directory.count != expected.count || local_directory.count != expected.count || runtime.count != expected.count || baked.count != expected.count || supplied.count != expected.count || defaulted.count != expected.count return 0; for i: 0..expected.count-1 { if directory[i] != expected[i] || local_directory[i] != expected[i] || runtime[i] != expected[i] || baked[i] != expected[i] || supplied[i] != expected[i] || defaulted[i] != expected[i] return 0; } return 42;",
        "#filepath",
    );
    for insertion in ["#insert quoted", "#insert,scope() quoted"] {
        execute(
            "",
            &format!(
                "quoted :: #code #filepath; expected := \"DIRECTORY\"; actual := {insertion}; if actual.count != expected.count return 0; for i: 0..expected.count-1 {{ if actual[i] != expected[i] return 0; }} return 42;"
            ),
            "#filepath",
        );
    }
}

#[test]
fn nested_macro_call_chain_keeps_immediate_and_forwarded_locations() {
    execute(
        &format!(
            "probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }}\ninner :: (target: Code) #expand {{ (#insert target) = probe(); }}\nouter :: (target: Code) #expand {{ inner(target); }}"
        ),
        "value := 0; outer(value); return ifx value == EXPECTED then 42 else 1;",
        "inner(target)",
    );
    for invocation in ["inner(target)", "inner(target, loc)"] {
        execute(
            &format!(
                "probe :: (loc := #caller_location)->int {{ {PROBE_BODY} }}\ninner :: (target: Code, loc := #caller_location) #expand {{ (#insert target) = probe(loc); }}\nouter :: (target: Code, loc := #caller_location) #expand {{ {invocation}; }}"
            ),
            "value := 0; outer(value); return ifx value == EXPECTED then 42 else 1;",
            "outer(value)",
        );
    }
}
