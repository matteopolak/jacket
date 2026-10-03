//! Actual reference sources are parsed and evaluated by Rust only; native bytes are never loaded.
use jai_driver::{
    CompilationUnit, CompilerSession,
    host_io::{FileCompilerSession, HostIo, OriginalInputPolicy},
};
use jai_modules::{
    BootstrapOptions, GraphOptions, RuntimeSupportOptions, RuntimeSupportParameters,
    RuntimeSupportSource,
};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::host_effects::HostRequestKey;
use std::{
    fs,
    path::{Path, PathBuf},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-file-source-{}-{:?}",
            std::process::id(),
            HostRequestKey::allocate()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn protected_host() -> HostIo {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let policy = OriginalInputPolicy::from_trusted_inventory(&workspace)
        .expect("authenticate trusted original-input inventory");
    let mut host = HostIo::default();
    policy
        .apply(&mut host)
        .expect("install original-input deny policy before granting a temp root");
    host
}
#[test]
fn unchanged_file_wrappers_read_seek_write_and_close_under_explicit_temp_root() {
    match fs::metadata(reference_root.join("File/module.jai")) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "SKIP unchanged File source acceptance: optional reference checkout is absent"
            );
            return;
        }
        result => {
            result.expect("inspect optional reference File source");
        }
    }
    let reference = reference_root.canonicalize().unwrap();
    let fixture = Fixture::new();
    fs::write(fixture.0.join("input.txt"), b"abc").unwrap();
    fs::write(
        fixture.0.join("main.jai"),
        r#"
#import "Basic";
#import "File";
MEASURED :: #run () -> s64 {
    file, success := file_open("input.txt", log_errors=false);
    if !success return -1;
    defer file_close(*file);
    length, measured := file_length(file);
    if !measured return -2;
    if !file_set_position(file, 2) return -3;
    data: [1] u8;
    read_ok, count := file_read(file, *data[0], 1);
    if !read_ok || count != 1 || data[0] != 99 return -4;
    contents, loaded := read_entire_file("input.txt", log_errors=false);
    if !loaded return -6;
    defer free(contents.data);
    if contents.count != 3 || contents[0] != 97 || contents[1] != 98 || contents[2] != 99 return -7;
    if !write_entire_file("output.txt", "changed") return -5;
    return length;
};
main :: () -> s64 { return MEASURED; }
"#,
    )
    .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let unit = CompilationUnit::load_with_bootstrap(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: vec![reference],
        },
        BootstrapOptions {
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: false,
                    enable_backtrace_on_crash: false,
                    temporary_storage_size: 32768,
                },
            }),
            ..Default::default()
        },
        Some(target.clone()),
    )
    .unwrap();
    let mut host = protected_host();
    let root = host.register_root(&fixture.0, true).unwrap();
    let mut effects =
        FileCompilerSession::with_root(CompilerSession::new(), host, root, Path::new("")).unwrap();
    let program = unit
        .resolve_with_file_session(target, &mut effects)
        .unwrap();
    let result = jai_vm::execute(&program, Default::default());
    match result.outcome {
        jai_vm::Outcome::Complete(values) => assert_eq!(values[0].integer().unwrap().value(), 3),
        outcome => panic!("{outcome:?}"),
    }
    assert_eq!(fs::read(fixture.0.join("output.txt")).unwrap(), b"changed");
}
#[test]
fn checked_source_calls_exact_stdio_prototypes_with_transactional_file_capabilities() {
    let fixture = Fixture::new();
    let modules = fixture.0.join("modules");
    let posix = modules.join("POSIX");
    fs::create_dir_all(posix.join("bindings/macos/arm64")).unwrap();
    fs::write(
        posix.join("module.jai"),
        "#load \"bindings/macos/arm64/stdio.jai\";\n",
    )
    .unwrap();
    fs::write(
        posix.join("bindings/macos/arm64/stdio.jai"),
        r#"
FILE :: struct {};
fopen :: (path:*u8, mode:*u8)->*FILE #foreign libc;
fseek :: (stream:*FILE, offset:s64, origin:s32)->s32 #foreign libc;
ftello :: (stream:*FILE)->s64 #foreign libc;
fclose :: (stream:*FILE)->s32 #foreign libc;
libc :: #system_library "libc";
"#,
    )
    .unwrap();
    fs::write(fixture.0.join("input.txt"), b"abc").unwrap();
    fs::write(
        fixture.0.join("main.jai"),
        r#"
#import "POSIX";
measure :: () -> s64 #no_context {
    path: [10] u8 = .[105,110,112,117,116,46,116,120,116,0];
    mode: [3] u8 = .[114,98,0];
    stream := fopen(*path[0], *mode[0]);
    if stream == null return -1;
    fseek(stream, 0, 2);
    length := ftello(stream);
    fclose(stream);
    return length;
}
MEASURED :: #run measure();
main :: () -> s64 #no_context { return MEASURED; }
"#,
    )
    .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let unit = CompilationUnit::load_with_bootstrap(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: vec![modules],
        },
        BootstrapOptions::disabled(),
        Some(target.clone()),
    )
    .unwrap();
    let mut host = protected_host();
    let root = host.register_root(&fixture.0, false).unwrap();
    let mut effects =
        FileCompilerSession::with_root(CompilerSession::new(), host, root, Path::new("")).unwrap();
    let program = unit
        .resolve_with_file_session(target, &mut effects)
        .unwrap();
    match jai_vm::execute(&program, Default::default()).outcome {
        jai_vm::Outcome::Complete(values) => assert_eq!(values[0].integer().unwrap().value(), 3),
        other => panic!("{other:?}"),
    }
}
