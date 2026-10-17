//! `+Plugin` arguments drive the Compiler hooks of a child workspace.
use std::{path::PathBuf, process::Command};

const PLUGIN: &str = include_str!("../../../tests/compiler/semantics/plugin-lifecycle.jai");

#[test]
fn plugin_hooks_build_and_execute_a_real_native_child_workspace() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-plugins");
    std::fs::create_dir_all(dir.join("modules/Lifecycle")).unwrap();
    std::fs::write(dir.join("modules/Lifecycle/module.jai"), PLUGIN).unwrap();
    let source = dir.join("program.jai");
    std::fs::write(
        &source,
        "#import \"Basic\"; main :: () { assert(generated==42); }\n",
    )
    .unwrap();
    let exe = dir.join("program");
    for check in [true, false] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jaic"));
        command.arg(&source);
        if check {
            command.arg("-check");
        } else {
            command.arg("-exe").arg(&exe);
        }
        let output = command.args(["+Lifecycle", "accept"]).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if !check {
            assert!(Command::new(&exe).status().unwrap().success());
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(&source)
        .args(["-check", "+Lifecycle", "refuse"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("plugin initialization failed"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn runtime_storage_options_distinguish_metadata_from_target_initialization() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-runtime-settings");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("settings.jai");
    for no_output in [false, true] {
        let output_setting = if no_output {
            "options.output_type=.NO_OUTPUT;"
        } else {
            ""
        };
        let text = format!(
            r#"
            C :: #import "Compiler";
            #run {{
                options:=C.get_build_options();
                options.temporary_storage_size=65536;
                {output_setting}
                C.set_build_options(options);
                got:=C.get_build_options();
                if got.temporary_storage_size!=65536 C.compiler_report("",0,0,"option was lost");
            }}
            main :: () {{}}
        "#
        );
        std::fs::write(&source, text).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&source)
            .arg("-check")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
        // Both metadata-only and initialized production runtimes support this
        // setting. Real allocation, zero storage and late updates are checked
        // by runtime_config; the core suite still rejects old consumers.
    }
}
