//! Authored source must reach checked virtual POSIX declarations, never native libc.
use jai_driver::CompilationUnit;
use jai_modules::{BootstrapOptions, GraphOptions};
use jai_sema::{ProcessAbiBindingContext, ResolveOptions};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{NoEffects, Outcome, host_effects::HostRequestKey};
use std::{fs, path::PathBuf};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-process-source-{}-{:?}",
            std::process::id(),
            HostRequestKey::allocate()
        ));
        fs::create_dir_all(path.join("modules/POSIX/bindings/macos/arm64")).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn checked_source_pipe_partial_reads_eof_close_and_errno_use_virtual_control_state() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("modules/POSIX/module.jai"),
        r#"
OS_Error_Code :: #type,isa s32;
#load "bindings/macos/arm64/base.jai";
"#,
    )
    .unwrap();
    fs::write(
        fixture
            .0
            .join("modules/POSIX/bindings/macos/arm64/base.jai"),
        r#"
pipe :: (fds:*[2]s32)->s32 #foreign libc;
read :: (fd:s32,buffer:*void,count:u64)->s64 #foreign libc;
write :: (fd:s32,buffer:*void,count:u64)->s64 #foreign libc;
close :: (fd:s32)->s32 #foreign libc;
__errno_location :: ()->*OS_Error_Code #foreign libc "__error";
libc :: #system_library "libc";
"#,
    )
    .unwrap();
    fs::write(
        fixture.0.join("main.jai"),
        r#"
#import "POSIX";
consume :: ()->s32 #no_context {
    fds: [2]s32;
    if pipe(*fds) != 0 return -1;
    input: [2]u8 = .[65,66];
    if write(fds[1], *input[0], 2) != 2 return -2;
    output: [2]u8;
    if read(fds[0], *output[0], 1) != 1 return -3;
    if output[0] != 65 return -4;
    if close(fds[1]) != 0 return -5;
    if read(fds[0], *output[1], 1) != 1 return -6;
    if output[1] != 66 return -7;
    if read(fds[0], null, 0) != 0 return -8;
    if read(fds[0], *output[0], 1) != 0 return -9;
    if close(fds[0]) != 0 return -10;
    if close(fds[0]) != -1 return -11;
    if <<__errno_location() != cast(OS_Error_Code)9 return -12;
    return cast(s32)output[0] + cast(s32)output[1];
}
MEASURED :: #run consume();
main :: ()->int #no_context { return cast(int)MEASURED; }
"#,
    )
    .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let modules = vec![fixture.0.join("modules")];
    let unit = CompilationUnit::load_with_bootstrap(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: modules.clone(),
        },
        BootstrapOptions::disabled(),
        Some(target.clone()),
    )
    .unwrap();
    let context =
        ProcessAbiBindingContext::from_graph(unit.graph(), &modules, target.clone()).unwrap();
    let options = ResolveOptions {
        target: Some(target),
        process_abi: Some(context),
        ..Default::default()
    };
    let program =
        jai_sema::resolve_graph_with_options(unit.graph(), &options, &mut NoEffects).unwrap();
    match jai_vm::execute(&program, Default::default()).outcome {
        Outcome::Complete(values) => assert_eq!(values[0].integer().unwrap().value(), 131),
        outcome => panic!("{outcome:?}"),
    }
}
