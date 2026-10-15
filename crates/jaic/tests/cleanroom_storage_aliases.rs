//! Original constant immutability and documented foreign storage reexport regressions.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use std::rc::Rc;

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

fn compile(source: &str, os: TargetOs, cpu: TargetCpu) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    options.os = os;
    options.cpu = cpu;
    options.import_paths.push("/lib".into());
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", format!("{} Type_Info_Variant :: struct {{using #as info:Type_Info; name:string; variant_of:*Type_Info; variant_flags:u32;}}", bootstrap::BOOTSTRAP).as_bytes());
    fs.insert(
        "/lib/Backing/module.jai",
        b"Handle :: #type,distinct u64; Secret :: u32; cleanroom_unavailable_data: Handle #elsewhere;".as_slice(),
    );
    fs.insert(
        "/lib/Facade/module.jai",
        b"using,only(Handle,cleanroom_unavailable_data) #import \"Backing\";".as_slice(),
    );
    fs.insert("/lib/Mutable/module.jai", b"original:s64=7;".as_slice());
    fs.insert(
        "/lib/Forward/module.jai",
        b"using,except(omitted_name) #import \"Facade\";".as_slice(),
    );
    let mut c = Compiler::new(options, Rc::new(fs));
    c.interp.host = Box::new(jaic::interp::SandboxHost::default());
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

#[test]
fn initialized_constants_are_never_assignable_storage_aliases() {
    for source in [
        "original:s64=7; Saved :: original; main :: () {Saved=9;}",
        "original:s64=7; Saved:s64:original; main :: () {Saved=9;}",
        "M :: #import \"Mutable\"; Saved :: M.original; main :: () {Saved=9;}",
        "Record :: struct {n:s64;} original:Record; Saved :: original; main :: () {Saved.n=9;}",
    ] {
        let (c, result) = compile(source, TargetOs::MacOS, TargetCpu::Arm64);
        let error = result.expect_err("initialized :: bindings must remain immutable");
        assert!(
            error.message.contains("constant"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn filtered_storage_reexports_do_not_publish_excluded_declarations() {
    let (c, result) = compile(
        r#"R :: #import "Backing"; #assert R.Secret==u32; F :: #import "Forward"; #assert F.Secret==u32; main :: () {}"#,
        TargetOs::MacOS,
        TargetCpu::Arm64,
    );
    let error = result.expect_err("a reexport must retain its name filters");
    assert!(
        error.message.contains("no exported member `Secret`"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn foreign_storage_type_queries_and_reexports_need_no_runtime_symbol() {
    for (os, cpu) in [
        (TargetOs::MacOS, TargetCpu::Arm64),
        (TargetOs::Windows, TargetCpu::X64),
        (TargetOs::Linux, TargetCpu::Arm64),
        (TargetOs::Wasm, TargetCpu::Wasm),
    ] {
        let (c, result) = compile(
            r#"
            F :: #import "Forward";
            R :: #import "Backing";
            #assert type_of(F.cleanroom_unavailable_data)==R.Handle && type_of(R.cleanroom_unavailable_data)==F.Handle;
            #assert size_of(type_of(F.cleanroom_unavailable_data))==8;
            #assert type_info(type_of(F.cleanroom_unavailable_data))==type_info(F.Handle);
            Holder :: struct(T:Type=type_of(F.cleanroom_unavailable_data)) { item:T; }
            #assert Holder()==Holder(F.Handle);
            use_default :: (value:F.Handle=F.cleanroom_unavailable_data)->F.Handle {return value;}
            main :: () { x:Holder(); if false {x.item=use_default();} }
        "#,
            os,
            cpu,
        );
        if let Err(e) = result {
            panic!("{}", e.render(&c.sources));
        }
        assert_eq!(
            c.program.foreigns.iter().filter(|f| f.is_data).count(),
            1,
            "reexports must reuse the foreign storage descriptor"
        );
    }
}

#[test]
fn unavailable_foreign_value_expressions_still_require_runtime_storage() {
    let (c, result) = compile(
        r#"missing:s64 #elsewhere; computed :: missing+1; #assert computed==1; main :: () {}"#,
        TargetOs::MacOS,
        TargetCpu::Arm64,
    );
    let error = result.expect_err("actual value computation needs available foreign storage");
    assert!(
        error.message.contains("not available"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn native_c_storage_reads_and_mutations_share_reexported_declarations() {
    use std::{path::PathBuf, process::Command};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("tests/compiler-semantics/storage-aliases");
    let dir = root.join("target/storage-aliases-interpreter");
    std::fs::create_dir_all(&dir).unwrap();
    let library = dir.join(format!(
        "{}cleanroom-alias-provider{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    let mut command = Command::new("clang");
    command.args(["-std=c11", "-Wall", "-Wextra", "-Werror"]);
    if cfg!(target_os = "macos") {
        command.arg("-dynamiclib");
    } else {
        command.arg("-shared");
    }
    if !cfg!(windows) {
        command.arg("-fPIC");
    }
    let output = command
        .arg(fixture.join("provider.c"))
        .arg("-o")
        .arg(&library)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    jaic::interp::add_library_dir(dir);
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/Backing/module.jai",
        include_bytes!("../../../tests/compiler-semantics/storage-aliases/Backing/module.jai")
            .as_slice(),
    );
    fs.insert(
        "/lib/Facade/module.jai",
        include_bytes!("../../../tests/compiler-semantics/storage-aliases/Facade/module.jai")
            .as_slice(),
    );
    fs.insert(
        "/lib/Forward/module.jai",
        include_bytes!("../../../tests/compiler-semantics/storage-aliases/Forward/module.jai")
            .as_slice(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths.push("/lib".into());
    let mut c = Compiler::new(options, Rc::new(fs));
    let source = format!(
        "F :: #import \"Forward\"; R :: #import \"Backing\"; {} #assert storage_alias_check()==0;",
        include_str!("../../../tests/compiler-semantics/storage-aliases/behavior.jai")
    );
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{}", e.render(&c.sources));
    }
    assert_eq!(c.program.foreigns.iter().filter(|f| f.is_data).count(), 2);
}

#[test]
fn unavailable_foreign_storage_still_diagnoses_real_reads() {
    let (c, result) = compile(
        r#"F :: #import "Facade"; read :: ()->F.Handle {return F.cleanroom_unavailable_data;} #run read(); main :: () {}"#,
        TargetOs::MacOS,
        TargetCpu::Arm64,
    );
    let error = result.expect_err("a real read still requires available foreign storage");
    assert!(
        error.message.contains("cleanroom_unavailable_data")
            && error.message.contains("not available"),
        "{}",
        error.render(&c.sources)
    );
}
