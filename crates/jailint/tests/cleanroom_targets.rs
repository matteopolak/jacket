//! Original target-aware source checking; no native SDK execution or replaced stdlib oracle.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn fixture(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cleanroom-lint-targets")
        .join(name);
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    for dir in ["stdlib/prelude", "stdlib/Runtime_Support", "app"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("stdlib/prelude/module.jai"), "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);").unwrap();
    root
}

fn lint(root: &Path, flags: &[&str], path: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jailint"))
        .current_dir(root)
        .env("JAIC_STDLIB", root.join("stdlib"))
        .args(flags)
        .arg(path)
        .output()
        .unwrap()
}

fn expect(output: Output, code: i32) -> (String, String) {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(code), "{stdout}\n{stderr}");
    (stdout, stderr)
}

#[test]
fn relative_include_roots_are_relative_to_invocation_not_each_source() {
    let root = fixture("relative-includes");
    std::fs::create_dir_all(root.join("external/Probe")).unwrap();
    std::fs::write(
        root.join("external/Probe/module.jai"),
        "original :: () -> int {return 42;}",
    )
    .unwrap();
    std::fs::write(
        root.join("app/fragment.jai"),
        "M :: #import \"Probe\"; verify :: () -> int {return M.original();}",
    )
    .unwrap();
    std::fs::write(
        root.join("app/main.jai"),
        "#load \"fragment.jai\"; #assert verify()==42; main :: () {} ",
    )
    .unwrap();
    expect(
        lint(
            &root,
            &[
                "-I",
                root.join("external").to_str().unwrap(),
                "-D",
                "warnings",
            ],
            "app/main.jai",
        ),
        0,
    );
    let (_, stderr) = expect(lint(&root, &[], "app/main.jai"), 1);
    assert!(stderr.contains("Probe"), "{stderr}");
    for include in ["external", "./external", "app/../external"] {
        expect(
            lint(&root, &["-I", include, "-D", "warnings"], "app/main.jai"),
            0,
        );
        expect(
            lint(
                &root,
                &["-I", include, "-D", "warnings"],
                "app/fragment.jai",
            ),
            0,
        );
    }
    // A missing invocation-relative root must not resolve a source-relative shadow.
    std::fs::create_dir_all(root.join("app/missing/Probe")).unwrap();
    std::fs::write(
        root.join("app/missing/Probe/module.jai"),
        "original :: () -> int {return 42;}",
    )
    .unwrap();
    let (_, stderr) = expect(lint(&root, &["-I", "missing"], "app/main.jai"), 1);
    assert!(stderr.contains("Probe"), "{stderr}");
    std::fs::write(
        root.join("external/Probe/module.jai"),
        "original :: () -> int {return unresolved_body();}",
    )
    .unwrap();
    let (_, stderr) = expect(lint(&root, &["-I", "external"], "app/main.jai"), 1);
    assert!(stderr.contains("unresolved_body"), "{stderr}");
}

#[test]
fn selected_target_branches_check_with_unchanged_compiler_host() {
    let root = fixture("matrix");
    let host = jaic::sema::Options::host();
    let host_os = match host.os {
        jaic::sema::TargetOs::Windows => "WINDOWS",
        jaic::sema::TargetOs::Linux => "LINUX",
        jaic::sema::TargetOs::MacOS => "MACOS",
        jaic::sema::TargetOs::Wasm => "WASM",
    };
    let host_cpu = match host.cpu {
        jaic::sema::TargetCpu::X64 => "X64",
        jaic::sema::TargetCpu::Arm64 => "ARM64",
        jaic::sema::TargetCpu::Wasm => "CUSTOM",
    };
    for (flags, os, cpu) in [
        (vec!["-os", "windows", "-cpu", "x64"], "WINDOWS", "X64"),
        (vec!["-os", "windows", "-cpu", "arm64"], "WINDOWS", "ARM64"),
        (vec!["-os", "linux", "-cpu", "x86_64"], "LINUX", "X64"),
        (vec!["-os", "linux", "-cpu", "aarch64"], "LINUX", "ARM64"),
        (vec!["-os", "macos", "-cpu", "x64"], "MACOS", "X64"),
        (vec!["-os", "macos", "-cpu", "arm64"], "MACOS", "ARM64"),
        (vec!["-os", "wasm"], "WASM", "CUSTOM"),
        (vec!["-target", "wasm64-unknown-wasi"], "WASM", "CUSTOM"),
    ] {
        std::fs::write(root.join("app/main.jai"),format!("#assert BUILD_OS == .{host_os}; #assert BUILD_CPU == .{host_cpu}; #if OS == .{os} && CPU == .{cpu} {{ main :: () {{}} }} else {{ #assert false; }}")).unwrap();
        expect(lint(&root, &flags, "app/main.jai"), 0);
        std::fs::write(
            root.join("app/main.jai"),
            format!(
                "main :: () {{ #if OS == .{os} && CPU == .{cpu} {{ missing_target_body(); }} }}"
            ),
        )
        .unwrap();
        let (_, stderr) = expect(lint(&root, &flags, "app/main.jai"), 1);
        assert!(stderr.contains("missing_target_body"), "{stderr}");
    }
}

#[test]
fn active_branch_produces_real_lints_and_inactive_invalid_body_stays_inactive() {
    let root = fixture("findings");
    std::fs::write(root.join("app/main.jai"), "main :: () { #if OS == .WINDOWS { target_unused := 1; } else { non_windows_unused := 2; } } #if OS != .WINDOWS { broken :: () { unavailable_other_target(); } }").unwrap();
    let (stdout, stderr) = expect(
        lint(
            &root,
            &["-os", "windows", "-cpu", "arm64", "-D", "warnings"],
            "app/main.jai",
        ),
        1,
    );
    assert!(
        stdout.contains("unused variable `target_unused`"),
        "{stdout}{stderr}"
    );
    assert!(
        !stdout.contains("unused variable `non_windows_unused`"),
        "{stdout}"
    );
    assert!(!stderr.contains("did not compile completely"), "{stderr}");
    let (_, stderr) = expect(lint(&root, &["-os", "linux"], "app/main.jai"), 1);
    assert!(stderr.contains("unavailable_other_target"), "{stderr}");
}

#[test]
fn triples_override_flags_and_invalid_targets_fail_as_usage_errors() {
    let root = fixture("triples");
    std::fs::write(
        root.join("app/main.jai"),
        "#assert OS == .WINDOWS; #assert CPU == .ARM64; main :: () {}",
    )
    .unwrap();
    expect(
        lint(
            &root,
            &[
                "-os",
                "linux",
                "-cpu",
                "x64",
                "-target",
                "aarch64-pc-windows-msvc",
            ],
            "app/main.jai",
        ),
        0,
    );
    for flags in [
        vec!["-os", "linxu"],
        vec!["-cpu", "sparc"],
        vec!["-target", "sparc-unknown-linux-gnu"],
        vec!["-target", "x86_64-unknown-mystery"],
        vec!["-target", "wasm64-unknown-mystery"],
        vec!["-os"],
        vec!["-cpu"],
        vec!["-target"],
        vec!["--context", "app/missing.jai"],
        vec!["--context", "app"],
        vec!["--manifest", "app"],
    ] {
        expect(lint(&root, &flags, "app/main.jai"), 2);
    }
    for flag in ["-os", "-cpu", "-target", "--context", "--manifest"] {
        // No source argument: otherwise it is legitimately consumed as the option value.
        let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
            .current_dir(&root)
            .env("JAIC_STDLIB", root.join("stdlib"))
            .arg(flag)
            .output()
            .unwrap();
        let (_, stderr) = expect(output, 2);
        assert!(stderr.contains("needs a value"), "{stderr}");
    }
}

#[test]
fn driver_preserves_supplied_source_runtime_and_target_options() {
    let root = fixture("compiler-options");
    std::fs::create_dir_all(root.join("configured/prelude")).unwrap();
    let preload = root.join("configured/prelude/module.jai");
    std::fs::write(&preload, "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; } selected_bootstrap :: 73;").unwrap();
    std::fs::write(root.join("app/main.jai"), "#assert selected_bootstrap == 73; #assert OS == .LINUX; #assert CPU == .ARM64; main :: () {}").unwrap();
    let mut compiler_options = jaic::sema::Options::host();
    compiler_options.preload = Some(preload);
    compiler_options.runtime_support = false;
    jailint::target::Selection {
        os: Some(jaic::sema::TargetOs::Linux),
        cpu: Some(jaic::sema::TargetCpu::Arm64),
        triple: None,
    }
    .apply(&mut compiler_options)
    .unwrap();
    let options = jailint::driver::Options {
        compiler_options,
        source: None,
        paths: vec![root.join("app/main.jai")],
        import_dirs: Vec::new(),
        stdlib: root.join("missing-default-library"),
        config: Default::default(),
        jobs: 2,
        verbose: false,
        add_sources: Vec::new(),
    };
    let result = jailint::driver::run(&options);
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    assert_eq!(result.contexts.len(), 1);
    assert!(result.contexts[0].complete);
    assert_eq!(result.contexts[0].os, jaic::sema::TargetOs::Linux);
    assert_eq!(result.contexts[0].cpu, jaic::sema::TargetCpu::Arm64);
}

#[test]
fn target_manifest_retains_config_source_and_required_parameter_failure() {
    let root = fixture("config-required");
    std::fs::create_dir_all(root.join("Required")).unwrap();
    std::fs::write(
        root.join("Required/module.jai"),
        "#module_parameters(MODE:int); #if OS == .LINUX { value :: MODE; } else { #assert false; }",
    )
    .unwrap();
    std::fs::write(
        root.join("custom.toml"),
        "[rules]\nunused_variable = \"deny\"\n",
    )
    .unwrap();
    let (_, stderr) = expect(
        lint(
            &root,
            &[
                "-os",
                "linux",
                "--config",
                "custom.toml",
                "--manifest",
                "required.json",
            ],
            "Required/module.jai",
        ),
        1,
    );
    assert!(stderr.contains("MODE"), "{stderr}");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("required.json")).unwrap()).unwrap();
    assert!(
        manifest["config_source"]
            .as_str()
            .unwrap()
            .ends_with("custom.toml")
    );
    assert!(!manifest["incomplete"].as_array().unwrap().is_empty());
}

#[test]
fn parallel_roots_and_fix_passes_keep_the_selected_target() {
    let root = fixture("parallel-fix");
    for name in ["one", "two"] {
        std::fs::write(root.join(format!("app/{name}.jai")), "#assert OS == .WINDOWS; #assert CPU == .ARM64; main :: () { #if OS == .WINDOWS { target_unused := 1; } else { unavailable_host_branch(); } }").unwrap();
    }
    expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "-j",
                "2",
                "-D",
                "warnings",
                "--manifest",
                "parallel.json",
            ],
            "app",
        ),
        1,
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("parallel.json")).unwrap()).unwrap();
    assert_eq!(manifest["roots"], 2);
    assert_eq!(manifest["contexts"].as_array().unwrap().len(), 2);
    for context in manifest["contexts"].as_array().unwrap() {
        assert_eq!(context["os"], "windows");
        assert_eq!(context["cpu"], "arm64");
        assert_eq!(context["complete"], true);
    }
    expect(
        lint(
            &root,
            &["-os", "windows", "-cpu", "arm64", "--fix", "-D", "warnings"],
            "app",
        ),
        1,
    );
    expect(
        lint(
            &root,
            &["-os", "windows", "-cpu", "arm64", "-D", "warnings"],
            "app",
        ),
        0,
    );
    let fixed = std::fs::read_to_string(root.join("app/one.jai")).unwrap();
    assert!(!fixed.contains("target_unused"));
    assert!(
        fixed.contains("unavailable_host_branch"),
        "inactive source must remain intact"
    );
}

#[test]
fn real_child_workspace_inherits_target_and_records_its_body_check() {
    let root = fixture("workspace-target");
    // These are our private, implemented compiler primitives, not facade stubs.
    std::fs::write(
        root.join("app/build.jai"),
        r#"
        __jaic_workspace_create :: (name:string) -> int #compiler;
        __jaic_workspace_set_option :: (w:int,key:string,value:string) #compiler;
        __jaic_workspace_add_file :: (w:int,filename:string) #compiler;
        #run {
            w := __jaic_workspace_create("lint target child");
            __jaic_workspace_set_option(w,"do_output","false");
            __jaic_workspace_add_file(w,"child.jai");
        }
        main :: () {}
    "#,
    )
    .unwrap();
    std::fs::write(
        root.join("app/child.jai"),
        "#assert OS == .WINDOWS; #assert CPU == .ARM64; main :: () {}",
    )
    .unwrap();
    expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "--manifest",
                "child.json",
            ],
            "app",
        ),
        0,
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("child.json")).unwrap()).unwrap();
    let contexts = manifest["contexts"].as_array().unwrap();
    let child = contexts
        .iter()
        .find(|context| {
            context["root"].as_str().unwrap().ends_with("build.jai")
                && context["visited_files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|path| path.as_str().unwrap().ends_with("child.jai"))
        })
        .expect("real child workspace must be visited under the builder root");
    assert_eq!(child["os"], "windows");
    assert_eq!(child["cpu"], "arm64");
    assert_eq!(child["complete"], true);
    std::fs::write(
        root.join("app/child.jai"),
        "main :: () { missing_child_body(); }",
    )
    .unwrap();
    let (_, stderr) = expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "--manifest",
                "child-failed.json",
            ],
            "app",
        ),
        1,
    );
    assert!(stderr.contains("missing_child_body"), "{stderr}");
}

#[test]
fn explicit_program_context_supplies_real_module_arguments_without_changing_privacy() {
    let root = fixture("explicit-context");
    std::fs::create_dir_all(root.join("stdlib/Public")).unwrap();
    std::fs::write(
        root.join("stdlib/Public/module.jai"),
        "#module_parameters(MODE:int); #assert MODE == 7; #load \"part.jai\";",
    )
    .unwrap();
    std::fs::write(root.join("stdlib/Public/part.jai"), "#assert OS == .WINDOWS; #assert CPU == .ARM64; read :: () -> int { context_unused := 1; return MODE; }").unwrap();
    std::fs::write(
        root.join("app/context.jai"),
        "P :: #import \"Public\"(MODE=7); main :: () { P.read(); }",
    )
    .unwrap();
    let (stdout, stderr) = expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "--context",
                "app/context.jai",
                "-D",
                "warnings",
            ],
            "stdlib/Public/part.jai",
        ),
        1,
    );
    assert!(
        stdout.contains("unused variable `context_unused`"),
        "{stdout}{stderr}"
    );
    assert!(!stderr.contains("did not compile completely"), "{stderr}");
    std::fs::create_dir_all(root.join("stdlib/internal/Denied")).unwrap();
    std::fs::write(
        root.join("stdlib/internal/Denied/module.jai"),
        "read :: () -> int { return 7; }",
    )
    .unwrap();
    std::fs::write(
        root.join("app/context.jai"),
        "P :: #import \"internal/Denied\"; main :: () { P.read(); }",
    )
    .unwrap();
    let (_, stderr) = expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "--context",
                "app/context.jai",
            ],
            "stdlib/internal/Denied/module.jai",
        ),
        1,
    );
    assert!(stderr.contains("private"), "{stderr}");
    std::fs::write(root.join("app/context.jai"), "main :: () {}").unwrap();
    let (_, stderr) = expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "--context",
                "app/context.jai",
            ],
            "stdlib/Public/part.jai",
        ),
        1,
    );
    assert!(stderr.contains("did not visit requested file"), "{stderr}");
}

#[test]
fn target_reaches_loaded_private_configured_module_and_preserves_privacy() {
    let root = fixture("private-loaded");
    std::fs::create_dir_all(root.join("stdlib/internal/Probe")).unwrap();
    std::fs::write(root.join("stdlib/internal/Probe/module.jai"), "#module_parameters(MODE:int); #assert MODE == 7; #if OS == .WINDOWS { #load \"windows.jai\"; } else { #assert false; }").unwrap();
    std::fs::write(root.join("stdlib/internal/Probe/windows.jai"), "#scope_file #assert CPU == .ARM64; hidden :: () -> int #no_context { return MODE; } #scope_export read :: () -> int #no_context { unused_platform_local := 1; return hidden(); }").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #scope_file H :: #import \"internal/Probe\"(MODE=7); #program_export \"marker\" marker :: () -> int #c_call { return H.read(); }").unwrap();
    let flags = ["-os", "windows", "-cpu", "arm64", "-D", "warnings"];
    let (stdout, stderr) = expect(lint(&root, &flags, "stdlib/internal/Probe/windows.jai"), 1);
    assert!(
        stdout.contains("unused variable `unused_platform_local`"),
        "{stdout}{stderr}"
    );
    assert!(!stderr.contains("did not compile completely"), "{stderr}");
    std::fs::write(
        root.join("app/main.jai"),
        "#import \"internal/Probe\"(MODE=7); main :: () {}",
    )
    .unwrap();
    let (_, stderr) = expect(lint(&root, &flags, "app/main.jai"), 1);
    assert!(stderr.contains("private"), "{stderr}");
}

#[test]
fn manifest_records_real_checked_contexts_and_unvisited_conditional_loads() {
    let root = fixture("manifest");
    std::fs::create_dir_all(root.join("Pkg")).unwrap();
    std::fs::write(
        root.join("Pkg/module.jai"),
        "#if OS == .WINDOWS { #load \"windows.jai\"; } else { #load \"other.jai\"; }",
    )
    .unwrap();
    std::fs::write(
        root.join("Pkg/windows.jai"),
        "read :: () -> int { return 1; }",
    )
    .unwrap();
    std::fs::write(
        root.join("Pkg/other.jai"),
        "read :: () -> int { return missing_other(); }",
    )
    .unwrap();
    expect(
        lint(
            &root,
            &[
                "-os",
                "windows",
                "-cpu",
                "arm64",
                "--manifest",
                "coverage.json",
            ],
            "Pkg",
        ),
        0,
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(manifest["target"]["os"], "windows");
    assert_eq!(manifest["target"]["cpu"], "arm64");
    assert_eq!(manifest["contexts"][0]["complete"], true);
    let visited = manifest["contexts"][0]["visited_files"].as_array().unwrap();
    assert!(
        visited
            .iter()
            .any(|path| path.as_str().unwrap().ends_with("windows.jai"))
    );
    assert!(
        !visited
            .iter()
            .any(|path| path.as_str().unwrap().ends_with("other.jai"))
    );
    assert!(
        manifest["unvisited_files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path.as_str().unwrap().ends_with("other.jai"))
    );
    expect(
        lint(
            &root,
            &["-os", "linux", "--manifest", "failure.json"],
            "Pkg",
        ),
        1,
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("failure.json")).unwrap()).unwrap();
    assert!(
        manifest["contexts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|context| context["complete"] == false)
    );
    assert!(!manifest["incomplete"].as_array().unwrap().is_empty());
}
