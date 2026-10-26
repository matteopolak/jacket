//! A `+Plugin` sees the build options the command line asked for, and can still replace them.
use std::{path::PathBuf, process::Command};

/// Prints the options of the workspace it is attached to, then, with `-optimize`, switches the
/// optimization on the way a plugin does (`set_optimization`).
const PROBE: &str = r#"#import "Compiler";
#import "Basic";
Probe :: struct { #as using base: Metaprogram_Plugin; optimize: bool; }
get_plugin :: () -> *Metaprogram_Plugin {
    p := New(Probe);
    p.init = (plugin: *Metaprogram_Plugin, options: [] string) -> bool {
        (cast(*Probe) plugin).optimize = options.count == 1 && options[0] == "-optimize";
        return true;
    };
    p.add_source = (plugin: *Metaprogram_Plugin) {
        o := get_build_options(plugin.workspace);
        print("before: llvm=% debug=% stack=% array=% null=% cast=% crash=%\n",
            o.llvm_options.bitcode_optimization_setting, o.emit_debug_info, o.stack_trace,
            o.array_bounds_check, o.null_pointer_check, o.cast_bounds_check, o.backtrace_on_crash);
        if (cast(*Probe) plugin).optimize {
            set_optimization(*o, .OPTIMIZED, preserve_debug_info = false);
            o.stack_trace = false;
            set_build_options(o, plugin.workspace);
            o = get_build_options(plugin.workspace);
            print("after: llvm=% debug=% stack=% array=%\n",
                o.llvm_options.bitcode_optimization_setting, o.emit_debug_info, o.stack_trace,
                o.array_bounds_check);
        }
    };
    return p;
}
"#;

fn setup(name: &str) -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(dir.join("modules/Probe")).unwrap();
    std::fs::write(dir.join("modules/Probe/module.jai"), PROBE).unwrap();
    let source = dir.join("program.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\nmain :: () { print(\"hi\\n\"); }\n",
    )
    .unwrap();
    (dir, source)
}

fn probe(source: &PathBuf, flags: &[&str], plugin_options: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .args(["-check", "-quiet"])
        .args(flags)
        .arg("+Probe")
        .args(plugin_options)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn a_plugin_starts_from_the_options_of_the_command_line() {
    let (_dir, source) = setup("cli-plugin-build-options");
    let default =
        "before: llvm=UNSET debug=DEFAULT stack=true array=ON null=ON cast=FATAL crash=ON\n";
    let cases: [(&[&str], &str); 7] = [
        (&[], default),
        (
            &["-O2"],
            "before: llvm=O2 debug=DEFAULT stack=true array=ON null=ON cast=FATAL crash=ON\n",
        ),
        (
            &["-O0"],
            "before: llvm=O0 debug=DEFAULT stack=true array=ON null=ON cast=FATAL crash=ON\n",
        ),
        (
            &["-od"],
            "before: llvm=O1 debug=DEFAULT stack=true array=ON null=ON cast=FATAL crash=ON\n",
        ),
        (
            &["-optimized"],
            "before: llvm=O2 debug=DEFAULT stack=false array=OFF null=OFF cast=OFF crash=ON\n",
        ),
        (
            &["-very_debug", "-no_debug_info"],
            "before: llvm=O0 debug=NONE stack=true array=ON null=ON cast=FATAL crash=ON\n",
        ),
        (
            &["-no_backtrace_on_crash"],
            "before: llvm=UNSET debug=DEFAULT stack=true array=ON null=ON cast=FATAL crash=OFF\n",
        ),
    ];
    for (flags, expected) in cases {
        assert_eq!(probe(&source, flags, &[]), expected, "{flags:?}");
    }
}

#[test]
fn a_plugin_can_replace_the_options_of_the_command_line() {
    let (_dir, source) = setup("cli-plugin-build-options-override");
    let printed = probe(&source, &["-very_debug"], &["-optimize"]);
    assert_eq!(
        printed,
        "before: llvm=O0 debug=DEFAULT stack=true array=ON null=ON cast=FATAL crash=ON\n\
         after: llvm=O2 debug=NONE stack=false array=OFF\n"
    );
}

#[test]
fn the_optimization_of_the_command_line_reaches_the_plugin_workspace_build() {
    let (dir, source) = setup("cli-plugin-build-options-output");
    let ir = |level: &str| {
        let name = format!("program{}", level.trim_start_matches('-'));
        let path = dir.join(format!("{name}.ll"));
        let status = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&source)
            .args(["-quiet", level, "-exe"])
            .arg(dir.join(&name))
            .arg("-emit_ir")
            .arg(&path)
            .arg("+Probe")
            .status()
            .unwrap();
        assert!(status.success());
        std::fs::read_to_string(path).unwrap()
    };
    assert_ne!(ir("-O0"), ir("-O2"));
}
