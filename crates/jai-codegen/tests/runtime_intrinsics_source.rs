//! Independently authored source, checked VM calls, and newly emitted native code.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::PrototypeOrigin;
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-runtime-source-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn parity(&self, source: &str, expected: i32) {
        self.parity_with_imports(source, expected, vec![]);
    }
    fn parity_with_imports(&self, source: &str, expected: i32, import_dirs: Vec<PathBuf>) {
        let source_path = self.0.join("main.jai");
        fs::write(&source_path, source).unwrap();
        let graph = jai_modules::ModuleGraph::load(
            &source_path,
            jai_modules::GraphOptions {
                import_dirs,
            },
        )
        .unwrap();
        let target = jai_codegen::target::NativeTarget::new().unwrap();
        let options = ResolveOptions {
            target: Some(target.build_target().unwrap()),
            ..ResolveOptions::default()
        };
        let program = resolve_graph_with_options(&graph, &options, &mut NoEffects).unwrap();
        let vm = jai_vm::execute(&program, Limits::default());
        assert!(
            matches!(&vm.outcome,Outcome::Complete(values) if matches!(values.as_slice(),[Value::Int(value)] if value.value() == i128::from(expected))),
            "VM fixture result disagreed: {:?}",
            vm.outcome
        );
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        for prototype in program.library().prototypes() {
            if matches!(prototype.origin, PrototypeOrigin::Intrinsic(_))
                && let Some(wrapper) =
                    module.get_function(&format!("jai.p{}", prototype.id.index()))
            {
                assert!(
                    wrapper.count_basic_blocks() > 0,
                    "intrinsic must have a real native implementation"
                );
            }
        }
        let object = self.0.join("program.o");
        let executable = self.0.join("program");
        target.write_object(&module, &object).unwrap();
        let output = native_tools::clang_command()
            .arg(&object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            module.print_to_string()
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected));
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                let _ = child.wait();
                panic!("native runtime fixture did not finish within ten seconds");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn open_jai_modules() -> Vec<PathBuf> {
    vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../corpus/upstream/withlang-dev--open-jai/modules"),
    ]
}

#[test]
#[ignore = "requires the supplied OpenJai source corpus"]
fn unchanged_pool_module_source_runs_real_blocks_and_lifetime_operations() {
    Fixture::new().parity_with_imports(
        r#"
        P :: #import "Pool";
        main :: () -> int {
            pool:P.Pool; P.set_allocators(*pool); pool.memblock_size=128;
            first:=cast(*u8) P.get(*pool,64); first[0]=42;
            second:=cast(*u8) P.get(*pool,64);
            if first==second || pool.bytes_left!=56 return 1;
            P.reset(*pool);
            reused:=cast(*u8) P.get(*pool,64);
            if reused!=first || first[0]!=42 return 2;
            P.release(*pool);
            if pool.bytes_left!=0 || pool.current_block!=null return 3;
            return 42;
        }
    "#,
        42,
        open_jai_modules(),
    );
}

#[test]
#[ignore = "requires the supplied OpenJai source corpus"]
fn unchanged_flat_pool_module_source_poison_reset_reuses_live_backing() {
    Fixture::new().parity_with_imports(
        r#"
        P :: #import "Flat_Pool";
        main :: () -> int {
            pool:P.Flat_Pool; pool.memblock_size=128;
            first:=cast(*u8) P.get(*pool,16); first[0]=42;
            P.reset(*pool,true);
            if first[0]!=204 return 1;
            reused:=cast(*u8) P.get(*pool,16);
            if reused!=first return 2;
            P.fini(*pool);
            if pool.bytes_left!=0 || pool.current_block!=null return 3;
            return 42;
        }
    "#,
        42,
        open_jai_modules(),
    );
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn source_memory_direct_and_indirect_calls_agree_between_vm_and_native_code() {
    Fixture::new().parity(
        r#"
        memcpy :: (dest:*void,source:*void,count:s64) #intrinsic;
        memcmp :: (a:*void,b:*void,count:s64)->s16 #must #intrinsic;
        memset :: (dest:*void,value:u8,count:s64) #intrinsic;
        main :: () -> int {
            value:u64=0; fill:=memset; fill(cast(*void) *value,42,8);
            copy:u64=0; memcpy(cast(*void) *copy,cast(*void) *value,8);
            if memcmp(cast(*void) *value,cast(*void) *copy,8)!=0 return 1;
            smaller:u8=127; larger:u8=128;
            if memcmp(cast(*void) *smaller,cast(*void) *larger,1)>=0 return 2;
            return cast(int) (copy & 255);
        }
    "#,
        42,
    );
}

#[test]
fn source_generic_bool_and_integer_cas_keep_multiresults_and_observed_values() {
    Fixture::new().parity(
        r#"
        compare_and_swap :: (pointer:*$T,old:T,new:T)->(success:bool,old_value:T) #intrinsic;
        main :: () -> int {
            flag:bool=false; ok,old_flag:=compare_and_swap(*flag,false,true);
            if !ok || old_flag || !flag return 1;
            number:s64=7; changed,old_number:=compare_and_swap(*number,7,42);
            if !changed || old_number!=7 || number!=42 return 2;
            unchanged,observed:=compare_and_swap(*number,7,1);
            if unchanged || observed!=42 || number!=42 return 3;
            return number;
        }
    "#,
        42,
    );
}

#[test]
fn ordinary_same_named_source_procedure_keeps_its_native_body() {
    Fixture::new().parity(
        "memset :: () -> int { return 42; } main :: () -> int { return memset(); }",
        42,
    );
}

#[test]
fn source_run_memory_result_is_embedded_before_native_runtime_reachability() {
    Fixture::new().parity(r#"
        memset :: (dest:*void,value:u8,count:s64) #intrinsic;
        build :: () -> int { value:u64=0; memset(cast(*void) *value,42,8); return cast(int) (value & 255); }
        main :: () -> int { return #run build(); }
    "#,42);
}

#[test]
fn destination_returning_memory_contracts_keep_actual_pointers_and_signed_fill_bytes() {
    Fixture::new().parity(
        r#"
        memcpy :: (dest:*void,source:*void,count:s64)->*void #intrinsic;
        memset :: (dest:*void,value:s64,count:s64)->*void #intrinsic;
        main :: () -> int {
            value:u64=0; fill:=memset; filled:=fill(cast(*void) *value,-214,8);
            if filled!=cast(*void) *value return 1;
            copy:u64=0; copied:=memcpy(cast(*void) *copy,cast(*void) *value,8);
            if copied!=cast(*void) *copy return 2;
            if memcpy(null,null,0)!=null return 3;
            return cast(int) (copy & 255);
        }
    "#,
        42,
    );
}

#[test]
fn specialized_swap_preserves_scalar_and_record_pointer_values_and_indirect_calls() {
    Fixture::new().parity(
        r#"
        swap :: (a:*$T,b:*T) #intrinsic;
        Box :: struct { value:s64; target:*s64; }
        main :: () -> int {
            first:s64=1; second:s64=42; swap(*first,*second);
            if first!=42 || second!=1 return 1;
            a:Box=.{value=7,target=*first}; b:Box=.{value=42,target=*second};
            swap(*a,*b); swap(*a,*a);
            if a.value!=42 || a.target!=*second || b.value!=7 || b.target!=*first return 2;
            flag:bool=false; other:bool=true; swap(*flag,*other);
            if !flag || other return 3;
            byte:=cast(*u8) *flag; byte.*=165; swap(*flag,*flag);
            if byte.*!=165 return 5;
            swap_boxes :: (a:*Box,b:*Box) #intrinsic "swap";
            exchange:=swap_boxes; exchange(*a,*b);
            if b.value!=42 return 4;
            return first;
        }
    "#,
        42,
    );
}
