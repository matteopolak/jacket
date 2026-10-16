//! Workspace snapshots must survive independent compilers and real child emission.
#![cfg(feature = "llvm")]
use std::{path::PathBuf, process::Command};

fn directory(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("T7numerics-options-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn checked(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn selected_workspace_options_survive_child_and_grandchild_compilation() {
    let dir = directory("projection");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("tests/compiler-semantics/workspace-options/projection.jai");
    checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source)
            .arg("-interp"),
    );
    for mode in ["-interp", "-O0", "-O2"] {
        let grandchild = r#"C :: #import "Compiler"; #import "Basic";
            #run {
                options := C.get_build_options();
                assert(options.llvm_options.bitcode_optimization_setting == .O1);
                assert(options.stack_trace);
                assert(options.user_data_u64 == 911);
            }
            main :: () {
                storage := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;
                assert(storage.original_size == 65536);
                storage.data[65535] = 71;
                assert(storage.data[65535] == 71);
                print("grandchild\n");
            }
        "#;
        let child = format!(
            r#"C :: #import "Compiler"; #import "Basic";
            #run {{
                options := C.get_build_options();
                assert(options.llvm_options.bitcode_optimization_setting == .O3);
                assert(!options.stack_trace);
                assert(options.user_data_u64 == 713);
                assert(options.temporary_storage_size == 131072);
                assert(options.user_data_string == "owned snapshot");
                assert(options.compile_time_command_line.count == 2);
                assert(options.compile_time_command_line[0] == "alpha");
                assert(options.compile_time_command_line[1] == "beta");
                assert(options.llvm_options.target_system_features == "");
                assert(options.llvm_options.target_system_cpu == "generic");
                assert(options.context_size_max == 8192);
                assert(options.machine_options[91] == 123);
                fresh := C.compiler_create_workspace("grandchild");
                next := C.get_build_options(fresh);
                assert(next.llvm_options.bitcode_optimization_setting == .UNSET);
                assert(next.stack_trace);
                next.output_path = {path:?};
                next.output_executable_name = "grandchild";
                next.llvm_options.bitcode_optimization_setting = .O1;
                next.user_data_u64 = 911;
                next.temporary_storage_size = 65536;
                C.set_build_options(next, fresh);
                C.add_build_string({grandchild:?}, fresh);
                assert(C.get_build_options().user_data_u64 == 713);
            }}
            main :: () {{
                storage := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;
                assert(storage.original_size == 131072);
                storage.data[131071] = 83;
                assert(storage.data[131071] == 83);
                print("child\n");
            }}
        "#,
            path = dir.to_str().unwrap()
        );
        let parent = format!(
            r#"C :: #import "Compiler"; #import "Basic";
            #run {{
                child := C.compiler_create_workspace("child");
                sibling := C.compiler_create_workspace("sibling");
                options := C.get_build_options(child);
                options.output_path = {path:?};
                options.output_executable_name = "child";
                options.llvm_options.bitcode_optimization_setting = .O3;
                options.stack_trace = false;
                options.user_data_u64 = 713;
                options.temporary_storage_size = 131072;
                options.llvm_options.target_system_cpu = "generic";
                owned := copy_string("owned snapshot");
                options.user_data_string = owned;
                args := string.["alpha", "beta"];
                options.compile_time_command_line = args;
                options.context_size_max = 8192;
                options.machine_options[91] = 123;
                C.set_build_options(options, child);
                owned[0] = #char "X";
                free(owned);
                args[0] = "changed after set";
                assert(C.get_build_options(child).user_data_u64 == 713);
                assert(C.get_build_options(child).user_data_string == "owned snapshot");
                assert(C.get_build_options(sibling).stack_trace);
                assert(C.get_build_options(sibling).llvm_options.bitcode_optimization_setting == .UNSET);
                assert(C.get_build_options().stack_trace);
                C.compiler_destroy_workspace(sibling);
                C.add_build_string({child:?}, child);
            }}
            main :: () {{ print("parent\n"); }}
        "#,
            path = dir.to_str().unwrap()
        );
        let builder = dir.join("builder.jai");
        std::fs::write(&builder, parent).unwrap();
        for name in ["child", "grandchild"] {
            let output = dir.join(name);
            if output.exists() {
                std::fs::remove_file(output).unwrap();
            }
        }
        let exe = dir.join("parent");
        let mut command = Command::new(env!("CARGO_BIN_EXE_jaic"));
        command.arg(&builder).arg(mode);
        if mode != "-interp" {
            command.arg("-exe").arg(&exe);
        }
        let result = checked(&mut command);
        if mode == "-interp" {
            assert_eq!(result.stdout, b"parent\n");
        } else {
            assert_eq!(checked(&mut Command::new(&exe)).stdout, b"parent\n");
        }
        for name in ["child", "grandchild"] {
            assert_eq!(
                checked(&mut Command::new(dir.join(name))).stdout,
                format!("{name}\n").as_bytes()
            );
        }
    }
}

#[test]
fn invalid_workspace_option_reads_fail() {
    let dir = directory("negative");
    let source = dir.join("invalid.jai");
    std::fs::write(
        &source,
        "C :: #import \"Compiler\"; #run { C.get_build_options(987654); } main :: () {}\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .arg("-check")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid workspace 987654"));
}

#[test]
fn dynamic_updates_and_optimization_reset_are_visible_in_the_selected_workspace() {
    let dir = directory("dynamic");
    let source = dir.join("dynamic.jai");
    std::fs::write(&source, r#"
        C :: #import "Compiler"; #import "Basic";
        #run {
            C.set_build_options_dc(.{do_output=false});
            child := C.compiler_create_workspace("dynamic");
            options := C.get_build_options(child);
            options.output_type = .NO_OUTPUT;
            options.llvm_options.bitcode_optimization_setting = .O3;
            options.llvm_options.machine_code_optimization_setting = .AGGRESSIVE;
            C.set_build_options(options, child);
            options.llvm_options.bitcode_optimization_setting = .UNSET;
            options.llvm_options.machine_code_optimization_setting = .UNSET;
            C.set_build_options(options, child);
            args := string.["-lm"];
            C.set_build_options_dc(.{write_added_strings=false, output_executable_name="dynamic name", append_linker_arguments=args}, child);
            assert(C.get_build_options(child).additional_linker_arguments.count == 1);
            C.add_build_string("C :: #import \"Compiler\"; #import \"Basic\"; #run { o:=C.get_build_options(); assert(!o.write_added_strings); assert(o.output_executable_name==\"dynamic name\"); assert(o.additional_linker_arguments.count==1 && o.additional_linker_arguments[0]==\"-lm\"); assert(o.llvm_options.bitcode_optimization_setting==.UNSET); assert(o.llvm_options.machine_code_optimization_setting==.UNSET); } main :: () {}", child);
        }
        main :: () {}
    "#).unwrap();
    checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source)
            .arg("-interp"),
    );
}

#[test]
fn isolated_cross_target_host_keeps_a_separate_options_registry() {
    let dir = directory("host-isolation");
    let source = dir.join("host.jai");
    std::fs::write(
        &source,
        r#"
        C :: #import "Compiler"; #import "Basic";
        #run { o:=C.get_build_options(); o.user_data_u64=441; C.set_build_options(o); }
        ISOLATED :: #run,host -> u64 {
            Local :: #import "Compiler";
            o:=Local.get_build_options();
            if o.user_data_u64 != 0 return 777;
            o.user_data_u64=99;
            Local.set_build_options(o);
            return Local.get_build_options().user_data_u64;
        }
        #assert ISOLATED==99;
        #run { assert(C.get_build_options().user_data_u64==441); }
        main :: () {}
    "#,
    )
    .unwrap();
    let os = if cfg!(target_os = "linux") {
        "windows"
    } else {
        "linux"
    };
    checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source)
            .args(["-check", "-os", os, "-cpu", "x64"]),
    );
}

#[test]
fn build_file_paths_preserve_caller_location_and_explicit_absolute_controls() {
    let dir = directory("caller-relative");
    std::fs::create_dir_all(dir.join("helpers/nested")).unwrap();
    let child = dir.join("child.jai");
    std::fs::write(
        &child,
        "#import \"Basic\"; main :: () { print(\"main child\\n\"); }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("helpers/nested/inner.jai"),
        r#"
        build_child :: (name: string, anchor: Source_Code_Location, absolute: string) {
            child := C.compiler_create_workspace(name);
            options := C.get_build_options(child);
            options.output_executable_name = name;
            options.output_path = ".";
            C.set_build_options(options, child);
            if name == "explicit" C.add_build_file("child.jai", child, anchor);
            else if name == "absolute" C.add_build_file(absolute, child);
            else C.add_build_file("child.jai", child);
        }
    "#,
    )
    .unwrap();
    std::fs::write(dir.join("helpers/helper.jai"), "#load \"nested/inner.jai\"; helper :: (name: string, anchor: Source_Code_Location, absolute: string) { build_child(name, anchor, absolute); }\n").unwrap();
    let positive = format!(
        r#"
        C :: #import "Compiler";
        #load "helpers/helper.jai";
        #run {{
            C.set_build_options_dc(.{{do_output=false}});
            direct := C.compiler_create_workspace("direct");
            options := C.get_build_options(direct);
            options.output_executable_name = "direct";
            options.output_path = ".";
            C.set_build_options(options,direct);
            C.add_build_file("child.jai",direct);
            helper("explicit", #location, {absolute:?});
            helper("absolute", #location, {absolute:?});
        }}
        main :: () {{}}
    "#,
        absolute = child.to_str().unwrap()
    );
    let default = format!(
        r#"
        C :: #import "Compiler";
        #load "helpers/helper.jai";
        #run {{
            C.set_build_options_dc(.{{do_output=false}});
            helper("default", #location, {absolute:?});
        }}
        main :: () {{}}
    "#,
        absolute = child.to_str().unwrap()
    );
    for mode in ["-interp", "-O0", "-O2"] {
        std::fs::write(dir.join("main.jai"), &positive).unwrap();
        for name in ["direct", "explicit", "absolute"] {
            let output = dir.join(name);
            if output.exists() {
                std::fs::remove_file(output).unwrap();
            }
        }
        checked(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .current_dir(&dir)
                .arg("main.jai")
                .arg(mode),
        );
        for name in ["direct", "explicit", "absolute"] {
            assert_eq!(
                checked(&mut Command::new(dir.join(name))).stdout,
                b"main child\n"
            );
        }
        std::fs::write(dir.join("main.jai"), &default).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .current_dir(&dir)
            .arg("main.jai")
            .arg(mode)
            .output()
            .unwrap();
        assert_eq!(rejected.status.code(), Some(1));
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("helpers/nested/child.jai") && error.contains("does not exist"),
            "{error}"
        );
    }
    // With a real helper-local source, the omitted location must select that file.
    std::fs::write(
        dir.join("helpers/nested/child.jai"),
        "#import \"Basic\"; main :: () { print(\"helper child\\n\"); }\n",
    )
    .unwrap();
    for mode in ["-interp", "-O0", "-O2"] {
        let output = dir.join("default");
        if output.exists() {
            std::fs::remove_file(&output).unwrap();
        }
        checked(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .current_dir(&dir)
                .arg("main.jai")
                .arg(mode),
        );
        assert_eq!(checked(&mut Command::new(output)).stdout, b"helper child\n");
    }
}

#[test]
fn exact_independent_portable_projection_and_relative_controls() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pending = root.join("tests/portable-stdlib/pending");
    for mode in ["-check", "-O0", "-O2"] {
        checked(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(pending.join("child-build-options-projection.jai"))
                .arg(mode),
        );
        let direct = checked(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(pending.join("relative-add-file/main.jai"))
                .arg(mode)
                .args(["-add", "#run,stallable run_relative_file(\"direct\");"]),
        );
        assert_eq!(direct.stdout, b"child compiled\n");
        let helper = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(pending.join("relative-add-file/main.jai"))
            .arg(mode)
            .args(["-add", "#run,stallable run_relative_file(\"helper\");"])
            .output()
            .unwrap();
        assert_eq!(helper.status.code(), Some(1));
        let error = String::from_utf8_lossy(&helper.stderr);
        assert!(
            error.contains("helper/child.jai") && error.contains("does not exist"),
            "{error}"
        );
    }
}
