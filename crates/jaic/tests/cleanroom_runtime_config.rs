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
