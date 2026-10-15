//! Real allocation through the canonical runtime, or an explicit isolated candidate.
use std::{path::PathBuf, process::Command};

fn checked(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap_or_else(|error| {
        panic!("required source-built integration command {command:?}: {error}")
    });
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn finalized_workspace_sizes_allocate_real_runtime_storage() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    // Isolate fixture staging while checking the production library when available.
    let dir = root.join(format!(
        "target/runtime-configuration-cli/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let library = if root.join("stdlib/prelude/module.jai").is_file() {
        root.join("stdlib")
    } else {
        let library = dir.join("stdlib");
        checked(
            Command::new("python3")
                .arg(root.join("tests/compiler-semantics/runtime-configuration/stage-candidate.py"))
                .arg("--destination")
                .arg(&library),
        );
        library
    };
    let source = root.join("tests/compiler-semantics/runtime-configuration/allocation.jai");
    let interpreted = checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", &library)
            .arg(&source)
            .arg("-interp"),
    );
    assert_eq!(interpreted.stdout, b"ok\n");
    for optimization in ["-O0", "-O2"] {
        let executable = dir.join(format!(
            "allocation{optimization}{}",
            std::env::consts::EXE_SUFFIX
        ));
        checked(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .env("JAIC_STDLIB", &library)
                .arg(&source)
                .arg(optimization)
                .arg("-exe")
                .arg(&executable),
        );
        assert_eq!(checked(&mut Command::new(executable)).stdout, b"ok\n");
    }

    let mut build = String::from("C :: #import \"Compiler\"; #run {\n");
    for (name, size) in [("zero", 0), ("large", 131072)] {
        let child = format!(
            r#"
            #import "Basic";
            main :: () {{
                storage:=cast(*Temporary_Storage_Paged_Alloc)context.temporary_storage;
                assert(storage!=null && storage.original_size=={size});
                assert((storage.data==null)==({size}==0));
                if {size}>0 {{ storage.data[{size}-1]=91; assert(storage.data[{size}-1]==91); }}
                print("ok\n");
            }}
        "#
        );
        let path = dir.to_string_lossy().replace('\\', "/");
        build.push_str(&format!(
            r#"
            {{
                workspace:=C.compiler_create_workspace("{name}");
                options:=C.get_build_options(workspace);
                options.output_type=.EXECUTABLE;
                options.output_path={path:?};
                options.output_executable_name="{name}";
                options.temporary_storage_size={size};
                C.set_build_options(options,workspace);
                C.add_build_string({child:?},workspace);
            }}
        "#
        ));
    }
    build.push_str("} main :: () {}\n");
    let builder = dir.join("build.jai");
    std::fs::write(&builder, build).unwrap();
    checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", &library)
            .arg(builder)
            .arg("-interp"),
    );
    for name in ["zero", "large"] {
        let executable = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        assert_eq!(checked(&mut Command::new(executable)).stdout, b"ok\n");
    }

    // Unintercepted child compilation runs after the root's front end. Its
    // explicitly selected root setting must still reach root execution/output.
    let original =
        include_str!("../../../tests/compiler-semantics/runtime-configuration/allocation.jai");
    let (setup, body) = original.split_once("main ::").unwrap();
    let root_id = jaic::build::TOP_LEVEL_WORKSPACE;
    let change = format!(
        r#"C :: #import "Compiler"; #run {{ options:=C.get_build_options({root_id}); options.temporary_storage_size=131072; C.set_build_options(options,{root_id}); }} main :: () {{}}"#
    );
    let late = format!(
        r#"{setup}
        #run {{
            child:=C.compiler_create_workspace("late root setting");
            options:=C.get_build_options(child);
            options.output_type=.NO_OUTPUT;
            C.set_build_options(options,child);
            C.add_build_string({change:?},child);
        }}
        main ::{body}
    "#,
        body = body.replace("65536", "131072").replace("65535", "131071")
    );
    let late_file = dir.join("late-root.jai");
    std::fs::write(&late_file, late).unwrap();
    let interpreted = checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", &library)
            .arg(&late_file)
            .arg("-interp"),
    );
    assert_eq!(interpreted.stdout, b"ok\n");
    let executable = dir.join(format!("late-root{}", std::env::consts::EXE_SUFFIX));
    checked(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", &library)
            .arg(late_file)
            .arg("-O2")
            .arg("-exe")
            .arg(&executable),
    );
    assert_eq!(checked(&mut Command::new(executable)).stdout, b"ok\n");
}
