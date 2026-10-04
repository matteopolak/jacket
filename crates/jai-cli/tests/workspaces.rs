//! Execute only freshly compiled, independently authored workspace fixtures.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const API: &str = r#"
compiler_create_workspace :: (name: string) -> s64 #compiler;
add_build_string :: (data: string, w: s64) #compiler;
add_build_file :: (filename: string, w: s64) #compiler;
Build_Options :: struct { output_path: string; }
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
"#;
const OUTPUT_OPTIONS: &str = r#"
Output_Type :: enum u8 { NO_OUTPUT :: 0; EXECUTABLE :: 1; DYNAMIC_LIBRARY :: 2; STATIC_LIBRARY :: 3; OBJECT_FILE :: 4; }
Runtime_Mode :: enum u8 { AUTO :: 0; ENTRY_POINT_AND_INIT :: 1; ONLY_INIT :: 2; OMIT :: 3; }
Backtrace :: enum u8 { OFF :: 0; ON :: 1; }
Build_Options :: struct {
    output_type: Output_Type = .EXECUTABLE;
    runtime_support_definitions: Runtime_Mode = .AUTO;
    backtrace_on_crash: Backtrace = .ON;
    output_path: string;
}
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
"#;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-cli-workspaces-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn command(&self, action: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jai-rs"));
        for variable in [
            "JAI_RS_MODULE_PATH",
            "JAI_RS_STDLIB",
            "JAI_RS_PRELOAD",
            "JAI_RS_RUNTIME_SUPPORT",
            "JAI_RS_RUNTIME_ENTRY",
            "JAI_RS_RUNTIME_INITIALIZATION",
            "JAI_RS_RUNTIME_BACKTRACE",
            "JAI_RS_TARGET",
            "JAI_RS_CPU",
            "JAI_RS_FEATURES",
            "JAI_RS_OPT",
            "JAI_RS_DEBUG",
            "JAI_RS_CLANG",
            "JAI_RS_AR",
        ] {
            command.env_remove(variable);
        }
        command.arg(action).arg(self.0.join("main.jai"));
        command
    }
    fn build(&self) -> Output {
        self.command("build")
            .arg(self.0.join("root-program"))
            .output()
            .unwrap()
    }
}

#[test]
fn pure_check_binds_known_compiler_declarations_without_committing_effects() {
    let fixture = Fixture::new(
        "write_string :: (text: string, error: bool) #no_context #compiler; main :: () {}",
    );
    succeeded(&fixture.command("check").output().unwrap());
    assert!(!fixture.0.join("root-program").exists());

    let fixture = Fixture::new(
        "add_build_string :: (data: string, w: s64) #compiler; #run add_build_string(\"generated :: 42;\", -1); main :: () {} ",
    );
    let output = fixture.command("check").output().unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("unrecognized #compiler"));
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn pure_checks_resume_a_typed_run_condition_before_loading_its_source_branch() {
    let fixture = Fixture::new(
        "choose :: () -> bool { return true; } #if #run choose() { #load \"active.jai\"; } else { #load \"must-not-load.jai\"; } main :: () -> int { return answer; }",
    );
    fs::write(fixture.0.join("active.jai"), "answer :: 42;").unwrap();
    for command in ["check", "check-library"] {
        succeeded(&fixture.command(command).output().unwrap());
    }
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn final_source_warnings_keep_both_sites_and_do_not_repeat_across_rebuilds() {
    let fixture = Fixture::new(
        "old :: () -> int #deprecated \"use fresh\" { return 42; }\nmain :: () -> int { return old(); }",
    );
    for command in ["check", "check-library", "build"] {
        let mut invocation = fixture.command(command);
        if command == "build" {
            invocation.arg(fixture.0.join("root-program"));
        }
        let output = invocation.output().unwrap();
        succeeded(&output);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            stderr
                .matches("warning: procedure 'old' is deprecated: use fresh")
                .count(),
            1,
            "{stderr}"
        );
        assert!(stderr.contains("main.jai:2:"), "{stderr}");
        assert!(stderr.contains("main.jai:1:"), "{stderr}");
        assert!(stderr.contains("declared deprecated here"), "{stderr}");
    }
    assert_eq!(execute(&fixture.0.join("root-program")), 42);

    fs::write(fixture.0.join("main.jai"), format!("{API}\nold :: () -> int #deprecated \"use fresh\" {{ return 42; }}\n#run add_build_string(\"answer :: 42;\", -1);\nmain :: () -> int {{ return old(); }}")).unwrap();
    let output = fixture.build();
    succeeded(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr
            .matches("warning: procedure 'old' is deprecated: use fresh")
            .count(),
        1,
        "{stderr}"
    );
    assert!(stderr.contains("use fresh"), "{stderr}");
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compiler_prelude_with_helper(helper: &str) -> String {
    let mut source = jai_driver::modules::compiler_prelude_source().to_owned();
    source.push('\n');
    source.push_str(helper);
    source
}
fn authored_preload_with_helper(helper: &str) -> String {
    format!(
        "{}\nFIRST_ADD_CONTEXT :: #code #add_context #as using base: Context_Base;\n{helper}\n",
        include_str!("../../../stdlib/prelude/module.jai")
    )
}
fn succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn execute(path: &Path) -> i32 {
    let mut child = Command::new(path).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status.code().unwrap();
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("freshly authored fixture exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn generated_root_source_is_compiled_into_a_real_executable() {
    let fixture = Fixture::new(&format!(
        "{API}\n#run add_build_string(\"answer :: 42;\", -1); main :: () -> int {{ return answer; }}"
    ));
    succeeded(&fixture.build());
    assert_eq!(execute(&fixture.0.join("root-program")), 42);
}

#[test]
fn recipe_without_root_main_builds_its_child_at_the_requested_path() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    child := compiler_create_workspace("child");
    add_build_string("main :: () -> int {{ return 37; }}", child);
    set_build_options(Build_Options.{{output_path = "child-program"}}, child);
}}
#run recipe();"#
    ));
    let output = fixture.build();
    succeeded(&output);
    assert_eq!(execute(&fixture.0.join("child-program")), 37);
    assert!(!fixture.0.join("root-program").exists());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("child-program"), "{stdout}");
}

#[test]
fn root_and_file_based_child_keep_independent_entries_and_outputs() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    child := compiler_create_workspace("child");
    add_build_file("child.jai", child);
    set_build_options(Build_Options.{{output_path = "child-program"}}, child);
}}
#run recipe();
main :: () -> int {{ return 11; }}"#
    ));
    fs::write(
        fixture.0.join("child.jai"),
        "#load \"value.jai\"; main :: () -> int { return value; }",
    )
    .unwrap();
    fs::write(fixture.0.join("value.jai"), "value :: 29;").unwrap();
    succeeded(&fixture.build());
    assert_eq!(execute(&fixture.0.join("root-program")), 11);
    assert_eq!(execute(&fixture.0.join("child-program")), 29);
}

#[test]
fn incomplete_child_and_invalid_generated_entries_preserve_existing_outputs() {
    for (source, diagnostic) in [
        (
            format!("{API}\n#run compiler_create_workspace(\"empty\"); main :: () {{}}"),
            "awaiting source inputs",
        ),
        (
            format!(
                "{API}\nrecipe :: () {{ child := compiler_create_workspace(\"child\"); add_build_string(\"main :: (n: int) {{}}\", child); }} #run recipe(); main :: () {{}}"
            ),
            "main cannot take parameters",
        ),
        (
            format!(
                "{API}\n#run add_build_string(\"bad :: () -> int {{ return missing; }}\", -1); main :: () {{}}"
            ),
            "unknown name 'missing'",
        ),
    ] {
        let fixture = Fixture::new(&source);
        let root_output = fixture.0.join("root-program");
        fs::write(&root_output, "preserved").unwrap();
        let output = fixture.build();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
        if diagnostic != "awaiting source inputs" {
            assert!(stderr.contains(".jai-generated-"), "{stderr}");
            assert!(stderr.contains(".jai:1:"), "{stderr}");
        }
        assert_eq!(fs::read(&root_output).unwrap(), b"preserved");
    }
}

#[test]
fn duplicate_source_output_paths_fail_before_writing_any_artifact() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    set_build_options(Build_Options.{{output_path = "same"}});
    child := compiler_create_workspace("child");
    add_build_string("main :: () {{}}", child);
    set_build_options(Build_Options.{{output_path = "same"}}, child);
}}
#run recipe(); main :: () {{}}"#
    ));
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("multiple workspaces request output"));
    assert!(!fixture.0.join("same").exists());
}

#[test]
fn imported_main_is_not_an_application_entry() {
    let fixture = Fixture::new("#import,file \"imported.jai\";");
    fs::write(fixture.0.join("imported.jai"), "main :: () {}").unwrap();
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no application main procedure"));
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn source_optimization_and_output_settings_reach_llvm_emission() {
    let fixture = Fixture::new(
        r#"
Llvm_Bitcode_Optimization_Setting :: enum u8 { UNSET :: 0; O0 :: 1; O1 :: 2; O2 :: 3; O3 :: 4; OS :: 5; OZ :: 6; }
Llvm_Machine_Code_Optimization_Setting :: enum u8 { UNSET :: 0; NONE :: 1; LESS :: 2; DEFAULT :: 3; AGGRESSIVE :: 4; }
Llvm_Options :: struct { bitcode_optimization_setting: Llvm_Bitcode_Optimization_Setting; machine_code_optimization_setting: Llvm_Machine_Code_Optimization_Setting; }
Build_Options :: struct { output_path: string; llvm_options: Llvm_Options; }
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
#run set_build_options(Build_Options.{output_path = "optimized.ll", llvm_options = Llvm_Options.{bitcode_optimization_setting = Llvm_Bitcode_Optimization_Setting.O3, machine_code_optimization_setting = Llvm_Machine_Code_Optimization_Setting.AGGRESSIVE}});
answer :: (n: int) -> int { return n * 7; }
main :: () -> int { return answer(6); }
"#,
    );
    let output = fixture
        .command("emit-llvm")
        .arg(fixture.0.join("ignored.ll"))
        .arg("-O0")
        .output()
        .unwrap();
    succeeded(&output);
    let ir = fs::read_to_string(fixture.0.join("optimized.ll")).unwrap();
    assert!(
        ir.contains("ret i32 42") || ir.contains("ret i64 42"),
        "{ir}"
    );
    assert!(!fixture.0.join("ignored.ll").exists());
}

#[test]
fn source_target_override_is_explicitly_rejected_before_output() {
    let fixture = Fixture::new(
        r#"
Llvm_Options :: struct { target_system_triple: string; }
Build_Options :: struct { llvm_options: Llvm_Options; }
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
#run set_build_options(Build_Options.{llvm_options = Llvm_Options.{target_system_triple = "wasm32-unknown-unknown"}});
main :: () {}
"#,
    );
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requests target wasm32-unknown-unknown")
    );
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn selected_compiler_prelude_source_survives_generated_source_rebuilds() {
    let preload = compiler_prelude_with_helper("bootstrap_value :: 43;");
    let fixture = Fixture::new(&format!(
        "{API}\n#run add_build_string(\"answer :: bootstrap_value;\", -1); main :: () -> int {{ return answer; }}"
    ));
    let modules = fixture.0.join("modules");
    fs::create_dir(&modules).unwrap();
    fs::write(modules.join("module.jai"), preload).unwrap();
    let output = fixture
        .command("build")
        .env("JAI_RS_STDLIB", &modules)
        .arg(fixture.0.join("root-program"))
        .output()
        .unwrap();
    succeeded(&output);
    assert_eq!(execute(&fixture.0.join("root-program")), 43);
    fs::remove_file(modules.join("module.jai")).unwrap();
    let output = fixture
        .command("build")
        .env("JAI_RS_STDLIB", &modules)
        .arg(fixture.0.join("missing-program"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Preload source not found"));
    assert!(!fixture.0.join("missing-program").exists());
}

#[test]
fn incomplete_runtime_source_reports_required_preload_context_without_an_artifact() {
    let fixture = Fixture::new("main :: () -> int { return bootstrap_value + runtime_value; }");
    let modules = fixture.0.join("modules");
    fs::create_dir(&modules).unwrap();
    fs::write(
        modules.join("module.jai"),
        authored_preload_with_helper("bootstrap_value :: 30;"),
    )
    .unwrap();
    fs::write(modules.join("Runtime_Support.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool, TEMPORARY_STORAGE_SIZE: s32 = 32768); #if DEFINE_INITIALIZATION { runtime_value :: 17; } else { runtime_value :: 3; }").unwrap();
    let output = fixture
        .command("build")
        .env("JAI_RS_STDLIB", &modules)
        .env("JAI_RS_RUNTIME_SUPPORT", "search")
        .env("JAI_RS_RUNTIME_ENTRY", "0")
        .env("JAI_RS_RUNTIME_INITIALIZATION", "1")
        .env("JAI_RS_RUNTIME_BACKTRACE", "0")
        .arg(fixture.0.join("root-program"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Context_Base"), "{stderr}");
    assert!(stderr.contains("module.jai:"), "{stderr}");
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn authored_runtime_context_and_typed_bootstrap_parameters_reach_native_execution() {
    let fixture = Fixture::new(
        "main :: () -> int { return bootstrap_value + runtime_value + context.runtime_marker; }",
    );
    let modules = fixture.0.join("modules");
    fs::create_dir(&modules).unwrap();
    fs::write(
        modules.join("module.jai"),
        authored_preload_with_helper("bootstrap_value :: 30;"),
    )
    .unwrap();
    fs::write(modules.join("Runtime_Support.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool, TEMPORARY_STORAGE_SIZE: s32 = 32768); Context_Base :: struct { runtime_marker: int = 1; } #if DEFINE_INITIALIZATION { runtime_value :: 17; } else { runtime_value :: 3; }").unwrap();
    let output = fixture
        .command("build")
        .env("JAI_RS_STDLIB", &modules)
        .env("JAI_RS_RUNTIME_SUPPORT", "search")
        .env("JAI_RS_RUNTIME_ENTRY", "0")
        .env("JAI_RS_RUNTIME_INITIALIZATION", "1")
        .env("JAI_RS_RUNTIME_BACKTRACE", "0")
        .arg(fixture.0.join("root-program"))
        .output()
        .unwrap();
    succeeded(&output);
    assert_eq!(execute(&fixture.0.join("root-program")), 48);
}

#[test]
fn recipe_child_can_emit_an_actual_native_object() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    child := compiler_create_workspace("child");
    add_build_string("main :: () -> int {{ return 37; }}", child);
    set_build_options(Build_Options.{{output_path = "child.o"}}, child);
}}
#run recipe();"#
    ));
    let output = fixture
        .command("emit-object")
        .arg(fixture.0.join("unused.o"))
        .output()
        .unwrap();
    succeeded(&output);
    let object = fs::read(fixture.0.join("child.o")).unwrap();
    assert!(object.len() > 100);
    assert!(!fixture.0.join("unused.o").exists());
}

#[test]
fn source_object_output_does_not_require_or_fabricate_main() {
    let fixture = Fixture::new(&format!(
        r#"{OUTPUT_OPTIONS}
#run set_build_options(Build_Options.{{output_type = .OBJECT_FILE, output_path = "library.o"}});
answer :: () -> int {{ return 37; }}"#
    ));
    succeeded(&fixture.build());
    let object = fs::read(fixture.0.join("library.o")).unwrap();
    assert!(object.len() > 100);
    assert!(!fixture.0.join("root-program").exists());
    let output = fixture.command("emit-llvm").output().unwrap();
    succeeded(&output);
    let ir = fs::read_to_string(fixture.0.join("library.o")).unwrap();
    assert!(!ir.contains("@main("), "{ir}");
    assert!(ir.contains("define "), "{ir}");
}

#[test]
fn source_no_output_is_a_checked_status_without_an_artifact() {
    let fixture = Fixture::new(&format!(
        r#"{OUTPUT_OPTIONS}
#run set_build_options(Build_Options.{{output_type = .NO_OUTPUT}});
answer :: () -> int {{ return 37; }}"#
    ));
    let output = fixture.build();
    succeeded(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("native output disabled"));
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn library_output_requires_explicit_exports_before_touching_outputs() {
    for kind in ["DYNAMIC_LIBRARY", "STATIC_LIBRARY"] {
        let fixture = Fixture::new(&format!(
            r#"{OUTPUT_OPTIONS}
#run set_build_options(Build_Options.{{output_type = .{kind}}});
main :: () {{}}"#
        ));
        let path = fixture.0.join("root-program");
        fs::write(&path, "preserved").unwrap();
        let output = fixture.build();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("native library output requires explicit #program_export")
        );
        assert_eq!(fs::read(path).unwrap(), b"preserved");
    }
}

fn installed_clang() -> PathBuf {
    let candidate = std::env::var_os("LLVM_SYS_221_PREFIX")
        .or_else(|| option_env!("LLVM_SYS_221_PREFIX").map(Into::into))
        .map(|prefix| PathBuf::from(prefix).join("bin/clang"))
        .or_else(|| {
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|root| root.join("clang"))
                    .find(|path| path.is_file())
            })
        })
        .expect("an installed Clang is required for authored C consumers");
    let tool = candidate
        .canonicalize()
        .expect("the selected installed Clang must be readable");
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for name in ["reference", "vendor", "corpus/upstream"] {
        let root = project.join(name);
        let root = root.canonicalize().unwrap_or(root);
        assert!(
            !tool.starts_with(root),
            "supplied native inputs are protected"
        );
    }
    tool
}

#[test]
fn source_dynamic_and_static_libraries_link_with_an_authored_c_consumer() {
    let dynamic_filename = if cfg!(target_os = "macos") {
        "libfixture.dylib"
    } else {
        "libfixture.so"
    };
    for (kind, filename) in [
        ("DYNAMIC_LIBRARY", dynamic_filename),
        ("STATIC_LIBRARY", "libfixture.a"),
    ] {
        let fixture = Fixture::new(&format!(
            r#"{OUTPUT_OPTIONS}
recipe :: () {{ set_build_options(Build_Options.{{output_type = .{kind}, output_path = "{filename}"}}); }}
#run recipe();
#scope_module
#program_export "fixture_counter"
counter: s32 = 7;
#program_export "fixture_increment"
increment :: (delta: s32) -> s32 #c_call {{ counter += delta; return counter; }}"#
        ));
        succeeded(&fixture.build());
        let library = fixture.0.join(filename);
        assert!(fs::metadata(&library).unwrap().len() > 100);
        assert!(!fixture.0.join("root-program").exists());
        let consumer = fixture.0.join("consumer.c");
        fs::write(&consumer, "extern int fixture_counter; extern int fixture_increment(int); int main(void) { int value = fixture_increment(30); return value + fixture_counter - 32; }").unwrap();
        let executable = fixture.0.join("consumer");
        let output = Command::new(installed_clang())
            .arg(&consumer)
            .arg(&library)
            .arg("-Xlinker")
            .arg("-rpath")
            .arg("-Xlinker")
            .arg(&fixture.0)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        succeeded(&output);
        assert_eq!(execute(&executable), 42);
    }
}

#[test]
fn library_abi_and_archive_tool_rejections_preserve_existing_outputs() {
    let fixture = Fixture::new(&format!(
        r#"{OUTPUT_OPTIONS}
#run set_build_options(Build_Options.{{output_type = .STATIC_LIBRARY}});
#program_export "wrong_abi"
answer :: () -> int {{ return 42; }}"#
    ));
    let path = fixture.0.join("root-program");
    fs::write(&path, "preserved").unwrap();
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must use #c_call"));
    assert_eq!(fs::read(&path).unwrap(), b"preserved");

    let fixture = Fixture::new(&format!(
        r#"{OUTPUT_OPTIONS}
#run set_build_options(Build_Options.{{output_type = .STATIC_LIBRARY}});
#program_export "valid_abi"
answer :: () -> s32 #c_call {{ return 42; }}"#
    ));
    let path = fixture.0.join("root-program");
    fs::write(&path, "preserved").unwrap();
    let output = fixture
        .command("build")
        .env("JAI_RS_AR", reference)
        .arg(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("refusing to execute reference tool"));
    assert_eq!(fs::read(&path).unwrap(), b"preserved");
}

#[test]
fn output_cannot_overwrite_an_imported_source() {
    let fixture = Fixture::new("#load \"helper.jai\"; main :: () -> int { return answer; }");
    let source = fixture.0.join("helper.jai");
    fs::write(&source, "answer :: 42;").unwrap();
    let output = fixture
        .command("emit-object")
        .arg(&source)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("overwrite its source input"));
    assert_eq!(fs::read(source).unwrap(), b"answer :: 42;");
}

#[test]
fn failed_host_link_preserves_the_previous_artifact() {
    let fixture = Fixture::new(
        "absent :: () -> s32 #foreign \"jai_fixture_missing_native_symbol\"; main :: () -> int { if absent() == 42 return 0; return 1; }",
    );
    let path = fixture.0.join("root-program");
    fs::write(&path, "preserved").unwrap();
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("LLVM compilation/linking failed"));
    assert_eq!(fs::read(path).unwrap(), b"preserved");
}

#[test]
fn source_build_option_locations_survive_into_artifact_policy_errors() {
    let options = OUTPUT_OPTIONS.replace(
        "set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;",
        "Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }\nset_build_options :: (options: Build_Options, w: s64 = -1, loc: Source_Code_Location = #caller_location) #compiler;",
    );
    let source = format!(
        "{options}\n#run set_build_options(Build_Options.{{output_path = \"main.jai\"}});\nmain :: () {{}}\n"
    );
    let fixture = Fixture::new(&source);
    let output = fixture.build();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("main.jai:"), "{stderr}");
    assert!(stderr.contains("overwrite its source input"), "{stderr}");
    assert_eq!(
        fs::read_to_string(fixture.0.join("main.jai")).unwrap(),
        source
    );
}

#[test]
fn committed_compiler_console_bytes_are_raw_and_rebuilds_do_not_repeat_them() {
    let fixture = Fixture::new(&format!(
        r#"{OUTPUT_OPTIONS}
write_string :: (text: string, error: bool) #no_context #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {{
    write_string("hello\0world", false);
    write_string("stderr-note\n", true);
    add_build_string("answer :: 42;", -1);
}}
#run recipe();
#run set_build_options(Build_Options.{{output_type = .NO_OUTPUT}});
main :: () -> int {{ return answer; }}"#
    ));
    let output = fixture.build();
    succeeded(&output);
    assert_eq!(output.stdout, b"hello\0world");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.matches("stderr-note").count(), 1, "{stderr}");
    assert!(stderr.contains("native output disabled"), "{stderr}");
    assert!(!fixture.0.join("root-program").exists());
}

#[test]
fn failed_compiler_transaction_cannot_publish_console_output() {
    let fixture = Fixture::new(
        r#"
write_string :: (text: string, error: bool) #no_context #compiler;
compiler_report :: (message: string) #compiler;
recipe :: () {
    write_string("uncommitted", false);
    compiler_report("stop transaction");
}
#run recipe(); main :: () {}
"#,
    );
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
    assert!(String::from_utf8_lossy(&output.stderr).contains("stop transaction"));
    assert!(!fixture.0.join("root-program").exists());
}
