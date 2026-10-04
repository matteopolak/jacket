//! Authored #run calls retain the exact source C-vararg descriptor-flag protocol.
use jai_driver::CompilationUnit;
use jai_modules::{BootstrapOptions, GraphOptions};
use jai_sema::{ProcessAbiBindingContext, ResolveOptions};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{NoEffects, Outcome, host_effects::HostRequestKey};
use std::{fs, path::PathBuf};

#[test]
fn checked_source_nested_fcntl_flags_and_errno_complete_without_native_effects() {
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let directory = Cleanup(std::env::temp_dir().join(format!(
        "jai-fcntl-source-{}-{:?}",
        std::process::id(),
        HostRequestKey::allocate()
    )));
    let bindings = directory.0.join("modules/POSIX/bindings/macos/arm64");
    fs::create_dir_all(&bindings).unwrap();
    fs::write(directory.0.join("modules/POSIX/module.jai"), "OS_Error_Code::#type,isa s32; #load \"bindings/macos/arm64/base.jai\"; #load \"bindings/macos/arm64/stdio.jai\";").unwrap();
    fs::write(
        bindings.join("base.jai"),
        r#"
pipe::(fds:*[2]s32)->s32 #foreign libc;
close::(fd:s32)->s32 #foreign libc;
__errno_location::()->*OS_Error_Code #foreign libc "__error";
#scope_file
libc::#system_library "libc";
"#,
    )
    .unwrap();
    fs::write(
        bindings.join("stdio.jai"),
        r#"
fcntl::(fd:s32,command:s32,__args:..Any)->s32 #foreign libc;
#scope_file
libc::#system_library "libc";
"#,
    )
    .unwrap();
    let input = directory.0.join("main.jai");
    fs::write(
        &input,
        r#"
#import "POSIX";
add_fd_flags::(fd:s32,flags:s32)->s32 #no_context {
    return fcntl(fd,2,fcntl(fd,1)|flags);
}
consume::()->s32 #no_context {
    fds:[2]s32;
    if pipe(*fds)!=0 return -1;
    if fcntl(fds[0],1)!=0 return -2;
    if add_fd_flags(fds[0],1)!=0 return -3;
    if fcntl(fds[0],1)!=1 return -4;
    old:=fcntl(fds[1],3);
    if old!=1 return -5;
    if fcntl(fds[1],4,old|4)!=0 return -6;
    if fcntl(fds[1],3)!=5 return -7;
    if close(fds[0])!=0 return -8;
    if close(fds[1])!=0 return -9;
    if fcntl(fds[0],1)!=-1 return -10;
    if <<__errno_location()!=cast(OS_Error_Code)9 return -11;
    return 42;
}
MEASURED::#run consume();
main::()->s32 #no_context{return MEASURED;}
"#,
    )
    .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let modules = vec![directory.0.join("modules")];
    let unit = CompilationUnit::load_with_bootstrap(
        &input,
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
        Outcome::Complete(values) => assert_eq!(values[0].integer().unwrap().value(), 42),
        outcome => panic!("{outcome:?}"),
    }
}
