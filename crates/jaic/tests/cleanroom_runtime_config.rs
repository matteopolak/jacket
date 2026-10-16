//! Private compiler/runtime configuration has target storage, not a public module parameter.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

const SOURCE: &str = r#"
    __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
    #program_export "configured_size"
    configured_size :: () -> s64 #c_call { return __jaic_runtime_temporary_storage_size(); }
    #program_export "wrapped_size"
    wrapped_size :: () -> s64 #c_call {
        getter: () -> s64 #no_context = __jaic_runtime_temporary_storage_size;
        return getter();
    }
    main :: () {}
"#;

#[test]
fn private_runtime_size_is_real_target_data_with_workspace_isolation() {
    for size in [0, 65536, 131072] {
        let mut options = Options::host();
        options.runtime_support = false;
        options.temporary_storage_size = size;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        if let Err(error) = c.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
            panic!("{}", error.render(&c.sources));
        }
        let index = c
            .program
            .funcs
            .iter()
            .position(|f| {
                f.as_ref().is_some_and(|f| {
                    f.linkage == jaic::ir::Linkage::Export("configured_size".into())
                })
            })
            .unwrap();
        let func = jaic::ir::FuncId(index as u32);
        assert_eq!(
            c.interp.call(&c.program, func, &[]).unwrap(),
            vec![size as u64]
        );
    }
}

#[test]
fn final_runtime_size_replaces_previously_interpreted_configuration_data() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source = format!("{SOURCE}\n#assert(configured_size()==32768);");
    if let Err(error) = c.begin_sources(&[ProgramSource::String(source)]) {
        panic!("{}", error.render(&c.sources));
    }
    c.options.temporary_storage_size = 65536;
    if let Err(error) = c.finish_program() {
        panic!("{}", error.render(&c.sources));
    }
    for name in ["configured_size", "wrapped_size"] {
        let index = c
            .program
            .funcs
            .iter()
            .position(|f| {
                f.as_ref()
                    .is_some_and(|f| f.linkage == jaic::ir::Linkage::Export(name.into()))
            })
            .unwrap();
        assert_eq!(
            c.interp
                .call(&c.program, jaic::ir::FuncId(index as u32), &[])
                .unwrap(),
            vec![65536]
        );
    }
}

#[test]
fn private_runtime_configuration_preserves_invalid_signature_and_size_diagnostics() {
    for declaration in [
        "__jaic_runtime_temporary_storage_size :: () -> u32 #intrinsic; main :: () { __jaic_runtime_temporary_storage_size(); }",
        "__jaic_runtime_temporary_storage_size :: (ignored:int) -> s64 #intrinsic; main :: () { __jaic_runtime_temporary_storage_size(1); }",
    ] {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        let error = c
            .compile_sources(&[ProgramSource::String(declaration.into())])
            .expect_err("incorrect private ABI must reject");
        assert!(
            error.message.contains("signature"),
            "{}",
            error.render(&c.sources)
        );
    }
    for size in [-1, i32::MAX as i64 + 1] {
        let mut options = Options::host();
        options.runtime_support = false;
        options.temporary_storage_size = size;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        let error = c
            .compile_sources(&[ProgramSource::String(SOURCE.into())])
            .expect_err("unallocatable runtime size must reject");
        assert!(
            error.message.contains("temporary_storage_size"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn compile_time_reads_and_unused_getters_cannot_opt_in_an_old_target_runtime() {
    for extra in [
        "#assert(__jaic_runtime_temporary_storage_size()==65536);",
        "unused :: () -> s64 { return __jaic_runtime_temporary_storage_size(); }",
    ] {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/Runtime_Support/module.jai","#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);".as_bytes());
        let mut options = Options::host();
        options.import_paths = vec!["/lib".into()];
        options.temporary_storage_size = 65536;
        let mut c = Compiler::new(options, Rc::new(fs));
        let source = format!(
            "__jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context; {extra} main :: () {{}}"
        );
        let error = c
            .compile_sources(&[ProgramSource::String(source)])
            .expect_err("old target runtime must keep its configuration diagnostic");
        assert!(
            error.message.contains("target runtime"),
            "{}",
            error.render(&c.sources)
        );
    }
}

fn startup_reader(runtime: &str, size: i64) -> (Compiler, jaic::sema::Result<()>) {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Runtime_Support/module.jai", format!(
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);
{runtime}"
    ).as_bytes());
    let mut options = Options::host();
    options.import_paths = vec!["/lib".into()];
    options.temporary_storage_size = size;
    let mut c = Compiler::new(options, Rc::new(fs));
    let result = c.compile_sources(&[ProgramSource::String("main::(){}".into())]);
    (c, result)
}

#[test]
fn ordinary_private_target_startup_reads_nondefault_configuration() {
    let (mut c, result) = startup_reader(
        r#"
        #scope_file
        __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
        read_size :: () -> s64 #no_context {return __jaic_runtime_temporary_storage_size();}
        #program_export "main"
        entry :: () -> s64 #c_call {return read_size();}
    "#,
        1048576,
    );
    result.map_err(|e| e.render(&c.sources)).unwrap();
    let entry = c.exported_func("main").unwrap();
    assert_eq!(
        c.interp.call(&c.program, entry, &[]).unwrap(),
        vec![1048576]
    );
}

#[test]
fn retained_startup_callback_relocation_reads_nondefault_configuration() {
    let (mut c, result) = startup_reader(
        r#"
        #scope_file
        __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
        read_size :: () -> s64 #no_context {return __jaic_runtime_temporary_storage_size();}
        #no_reset getter: () -> s64 #no_context;
        #run {getter=read_size;}
        #program_export "main"
        entry :: () -> s64 #c_call {return getter();}
    "#,
        1048576,
    );
    result.map_err(|e| e.render(&c.sources)).unwrap();
    let entry = c.exported_func("main").unwrap();
    assert_eq!(
        c.interp.call(&c.program, entry, &[]).unwrap(),
        vec![1048576]
    );
}

#[test]
fn unreachable_or_reset_callback_reads_do_not_opt_in_target_startup() {
    for (qualifier, call) in [("#no_reset", "return 0;"), ("", "return getter();")] {
        let runtime = format!(
            r#"
            #scope_file
            __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
            read_size :: () -> s64 #no_context {{return __jaic_runtime_temporary_storage_size();}}
            {qualifier} getter: () -> s64 #no_context;
            #run {{getter=read_size;}}
            #program_export "main"
            entry :: () -> s64 #c_call {{{call}}}
        "#
        );
        let (c, result) = startup_reader(&runtime, 1048576);
        let error = result
            .expect_err("compile-time-only callback storage must not authorize target startup");
        assert!(
            error.message.contains("target runtime"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn overwritten_retained_callback_cannot_claim_its_old_configuration_read() {
    let (c, result) = startup_reader(
        r#"
        #scope_file
        __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
        read_size :: () -> s64 #no_context {return __jaic_runtime_temporary_storage_size();}
        stale :: () -> s64 #no_context {return 32768;}
        #no_reset getter: () -> s64 #no_context = read_size;
        #run {getter=stale;}
        #program_export "main"
        entry :: () -> s64 #c_call {return getter();}
    "#,
        1048576,
    );
    let error = result.expect_err("final retained target state replaced the old reader");
    assert!(
        error.message.contains("target runtime"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn retained_startup_aggregate_and_global_cycles_preserve_reachability() {
    let (mut c, result) = startup_reader(
        r#"
        #scope_file
        __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
        read_size :: () -> s64 #no_context {return __jaic_runtime_temporary_storage_size();}
        Getter :: () -> s64 #no_context;
        Node :: struct {next:*Node; callbacks:[1]Getter;}
        #no_reset node:Node;
        #no_reset root:*Node;
        #run {node.next=*node; node.callbacks[0]=read_size; root=*node;}
        #program_export "main"
        entry :: () -> s64 #c_call {return root.next.callbacks[0]();}
    "#,
        1048576,
    );
    result.map_err(|e| e.render(&c.sources)).unwrap();
    c.finalize_runtime_configuration().unwrap();
    let entry = c.exported_func("main").unwrap();
    assert_eq!(
        c.interp.call(&c.program, entry, &[]).unwrap(),
        vec![1048576]
    );
}
