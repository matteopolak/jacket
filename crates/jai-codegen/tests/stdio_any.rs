//! Fresh source uses the trusted system's real POSIX write, with captured streams.
#![cfg(unix)]
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-stdio-any-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn boxed_payloads_reach_real_stdout_and_stderr_through_the_c_write_abi() {
    let fixture = Fixture::new();
    let input = fixture.0.join("main.jai");
    fs::write(
        &input,
        r#"
        write :: (fd:s32, data:*u8, count:u64) -> s64 #c_call #foreign;
        send :: (text:string, fd:s32) -> int #no_debug {
            written:int=0;
            while written<text.count {
                result:=write(fd,text.data+written,cast(u64)(text.count-written));
                if result<=0 return -1;
                written+=result;
            }
            return written;
        } @WritesBytes
        main :: () -> int {
            original:s32=1;
            boxed:Any=original;
            original=42;
            number:=(cast(*s32)boxed.value_pointer).*;
            bytes:[3]u8;
            bytes[0]=cast(u8)(number/10+48);
            bytes[1]=cast(u8)(number%10+48);
            bytes[2]=10;
            text:string;
            text.data=*bytes[0];
            text.count=3;
            if send(text,1)!=3 return 1;
            if send("Any\n",2)!=4 return 2;
            return 42;
        }
    "#,
    )
    .unwrap();
    let graph = ModuleGraph::load(&input, GraphOptions::default()).unwrap();
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
    .unwrap();
    let sources = program.library().debug_sources().unwrap();
    let (send, _) = sources
        .procedures()
        .find(|(_, source)| source.name == "send")
        .unwrap();
    assert_eq!(sources.procedure_notes(send)[0].text(), b"WritesBytes");
    assert!(!sources.procedure_policy(send).emits());
    assert!(
        matches!(
            jai_vm::execute(&program, Limits::default()).outcome,
            Outcome::Failed(jai_vm::Error::UnsupportedForeignProcedure(_))
        ),
        "VM must not claim to execute unbound host I/O"
    );
    let llvm = jai_codegen::emit(&program).unwrap();
    let llvm_path = fixture.0.join("main.ll");
    let executable = fixture.0.join("main");
    fs::write(&llvm_path, &llvm).unwrap();
    let compiled = native_tools::clang_command()
        .arg(&llvm_path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{llvm}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut child = Command::new(&executable)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("fresh stdio fixture exceeded its deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(42));
    assert_eq!(output.stdout, b"42\n");
    assert_eq!(output.stderr, b"Any\n");
}
