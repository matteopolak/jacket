//! Self-written check-policy fixtures executed only by the Rust VM and new native output.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_vm::{ArithmeticError, Error, Limits, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitStatus},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-safety-checks-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn graph(&self) -> jai_modules::ModuleGraph {
        jai_modules::ModuleGraph::load(&self.0.join("main.jai"), Default::default()).unwrap()
    }
    fn program(&self) -> jai_ir::Program {
        jai_sema::resolve_graph(&self.graph()).unwrap()
    }
    fn native(&self, program: &jai_ir::Program) -> (ExitStatus, String) {
        let context = jai_codegen::Context::create();
        let target = jai_codegen::target::NativeTarget::new().unwrap();
        let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
        let ir = module.print_to_string().to_string();
        let object = self.0.join("program.o");
        target.write_object(&module, &object).unwrap();
        let executable = self.0.join("program");
        let linked = native_tools::clang_command()
            .arg(object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{}\n{ir}",
            String::from_utf8_lossy(&linked.stderr)
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                return (status, ir);
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated safety fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn complete(source: &str, expected: i32) -> String {
    let fixture = Fixture::new(source);
    let program = fixture.program();
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    let Outcome::Complete(values) = outcome else {
        panic!("source fixture did not complete in the VM: {outcome:?}");
    };
    assert!(
        matches!(values.as_slice(), [Value::Int(value)] if value.value() == i128::from(expected)),
        "{values:?}"
    );
    let (status, ir) = fixture.native(&program);
    assert_eq!(status.code(), Some(expected));
    ir
}

fn arithmetic_failure(source: &str, error: ArithmeticError) {
    let fixture = Fixture::new(source);
    let program = fixture.program();
    assert_eq!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(Error::Arithmetic(error))
    );
    let (status, _) = fixture.native(&program);
    assert!(
        !status.success(),
        "checked arithmetic must trap in generated code"
    );
}

#[test]
fn procedure_and_nested_scope_checks_wrap_only_the_marked_operations() {
    complete(
        "wrap :: (value:u8)->u8 #no_aoc { return value+3; } main :: ()->int { result:u8=254; #no_aoc { result+=3; #no_abc { result+=2; } } return cast(int)wrap(254)+cast(int)result+38; }",
        42,
    );
    complete(
        "main :: ()->int #no_aoc { nested :: (value:u8)->u8 { return value+3; } return cast(int)nested(254)+41; }",
        42,
    );
    complete(
        "negate :: (value:s8)->s8 #no_aoc { return -value; } main :: ()->int { return cast(int)negate(-128)+170; }",
        42,
    );
    complete(
        "wrap :: (value:u8)->u8 #c_call #no_aoc { return value+3; } main :: ()->int { return cast(int)wrap(254)+41; }",
        42,
    );
    complete(
        "main :: ()->int { value:u8=255; for 0..0 #no_aoc { value+=1; } while value==0 #no_aoc { value=255; value+=2; } return cast(int)value+41; }",
        42,
    );
    complete(
        "main :: ()->int #no_aoc { values:[1]int=.[42]; return values[cast(u8)255+1]; }",
        42,
    );
}

#[test]
fn body_policy_preserves_callable_type_identity_and_indirect_dispatch() {
    let source = "wrap :: (value:u8)->u8 #no_aoc { return value+3; } checked :: (value:u8)->u8 { return value; } main :: ()->int { function:(value:u8)->u8=wrap; return cast(int)function(254)+41; }";
    let fixture = Fixture::new(source);
    let program = fixture.program();
    assert_eq!(
        program.procedures()[0].signature,
        program.procedures()[1].signature
    );
    complete(source, 42);
}

#[test]
fn captured_code_expansions_and_deferred_bodies_keep_their_policy() {
    complete(
        "main :: ()->int { value:u8=255; update :: #code value+=1; #no_aoc { #insert,scope() update; } return cast(int)value+42; }",
        42,
    );
    arithmetic_failure(
        "main :: ()->int { value:u8=255; update :: #code value+=1; #no_aoc { #insert update; } return 42; }",
        ArithmeticError::IntegerOverflow,
    );
    complete(
        "main :: ()->int { value:u8=255; #no_aoc { defer value+=1; } return cast(int)value+42; }",
        42,
    );
    complete(
        "increment :: (value:*u8) #expand #no_aoc { value.*+=1; } main :: ()->int { value:u8=255; increment(*value); return cast(int)value+42; }",
        42,
    );
}

#[test]
fn typed_overflow_checks_cover_every_integer_width_and_core_operator() {
    for (ty, max, wrapped) in [
        ("u8", "255", "0"),
        ("u16", "65535", "0"),
        ("u32", "4294967295", "0"),
        ("u64", "18446744073709551615", "0"),
        ("s8", "127", "-128"),
        ("s16", "32767", "-32768"),
        ("s32", "2147483647", "-2147483648"),
        ("s64", "9223372036854775807", "-9223372036854775808"),
    ] {
        arithmetic_failure(
            &format!("main :: ()->int {{ value:{ty}={max}; value+=1; return 42; }}"),
            ArithmeticError::IntegerOverflow,
        );
        complete(
            &format!(
                "main :: ()->int #no_aoc {{ value:{ty}={max}; value+=1; if value=={wrapped} return 42; return 0; }}"
            ),
            42,
        );
    }
    for body in [
        "value:u8=0; value-=1;",
        "value:u64=9223372036854775808; value*=2;",
        "value:s8=-128; result := -value;",
    ] {
        arithmetic_failure(
            &format!("main :: ()->int {{ {body} return 42; }}"),
            ArithmeticError::IntegerOverflow,
        );
    }
    complete(
        "main :: ()->int #no_aoc { value:u64=9223372036854775808; value*=9223372036854775808; if value==0 return 42; return 0; }",
        42,
    );
}

#[test]
fn disabled_overflow_scope_restores_and_does_not_disable_callee_or_other_checks() {
    arithmetic_failure(
        "main :: ()->int { value:u8=255; #no_aoc { value+=1; } value=255; value+=1; return 42; }",
        ArithmeticError::IntegerOverflow,
    );
    arithmetic_failure(
        "plain :: () { value:u8=255; value+=1; } main :: ()->int #no_aoc { plain(); return 42; }",
        ArithmeticError::IntegerOverflow,
    );
    arithmetic_failure(
        "main :: ()->int #no_abc { value:u8=255; value+=1; return 42; }",
        ArithmeticError::IntegerOverflow,
    );
    arithmetic_failure(
        "main :: ()->int #no_aoc { value:int=1; zero:int=0; return value/zero; }",
        ArithmeticError::ZeroDivisor,
    );
    arithmetic_failure(
        "main :: ()->int #no_aoc { value:u8=1; count:u8=8; return cast(int)(value<<count); }",
        ArithmeticError::ShiftCount,
    );
    arithmetic_failure(
        "main :: ()->int { value:int=-9223372036854775808; divisor:int=-1; return value/divisor; }",
        ArithmeticError::SignedDivisionOverflow,
    );
    complete(
        "main :: ()->int #no_aoc { value:s8=-128; divisor:s8=-1; return cast(int)(value/divisor)+170; }",
        42,
    );
    complete(
        "main :: ()->int #no_aoc { value:s8=-128; divisor:s8=-1; return cast(int)(value%divisor)+42; }",
        42,
    );
}

#[test]
fn source_check_permissions_do_not_bypass_vm_resource_limits() {
    let fixture = Fixture::new("main :: () #no_aoc #no_abc { while true {} }");
    let program = fixture.program();
    let limits = Limits {
        fuel: 30,
        ..Limits::default()
    };
    assert_eq!(
        jai_vm::execute(&program, limits).outcome,
        Outcome::Failed(Error::Limit(jai_vm::LimitKind::Fuel))
    );
}

#[test]
fn disabled_bounds_allow_valid_backing_access_beyond_descriptor_count() {
    let ir = complete(
        "main :: ()->int #no_abc { backing:[2]int=.[1,42]; view:[]int=.{count=1,data=backing.data}; view[1]=40; return view[1]+2; }",
        42,
    );
    assert!(
        !ir.contains("index.in.bounds"),
        "source count checks must be omitted\n{ir}"
    );
    assert!(ir.contains("index.nonnull"), "null guard must remain\n{ir}");
}

#[test]
fn disabled_bounds_restore_and_keep_vm_allocation_and_null_guards() {
    for source in [
        "main :: ()->int { backing:[2]int=.[1,42]; view:[]int=.{count=1,data=backing.data}; #no_abc { value:=view[1]; } return view[1]; }",
        "main :: ()->int #no_abc { backing:[2]int=.[1,42]; view:[]int=.{count=1,data=backing.data}; return view[2]; }",
    ] {
        let fixture = Fixture::new(source);
        let program = fixture.program();
        assert!(matches!(
            jai_vm::execute(&program, Limits::default()).outcome,
            Outcome::Failed(Error::OutOfBounds { .. })
        ));
        if source.contains("value:=view[1]") {
            assert!(!fixture.native(&program).0.success());
        }
    }
    let fixture = Fixture::new(
        "main :: ()->int #no_abc { view:[]int=.{count=0,data=null}; return view[0]; }",
    );
    let program = fixture.program();
    assert_eq!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(Error::NullPointer)
    );
    assert!(!fixture.native(&program).0.success());
}

#[test]
fn known_fixed_array_bounds_fail_during_semantic_resolution() {
    for source in [
        "main :: ()->int { values:[2]int=.[1,2]; return values[2]; }",
        "main :: ()->int { values:[2]int=.[1,2]; return values[cast(u8)1+cast(u8)1]; }",
        "main :: ()->int { values:[2]int=.[1,2]; N::1; return values[cast(int)N+N]; }",
        "main :: ()->int { values:[2]int=.[1,2]; values[-1]=42; return 42; }",
        "main :: ()->int { values:[0]int; return values[0]; }",
    ] {
        let fixture = Fixture::new(source);
        let diagnostic = jai_sema::resolve_graph(&fixture.graph())
            .unwrap_err()
            .to_string();
        assert!(
            diagnostic.contains("constant array index is outside"),
            "{diagnostic}"
        );
    }
    let fixture =
        Fixture::new("main :: ()->int #no_abc { values:[2]int=.[1,2]; return values[2]; }");
    let program = fixture.program();
    assert!(matches!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(Error::OutOfBounds { .. })
    ));
}

#[test]
fn constant_and_run_execution_use_the_same_source_policy() {
    complete(
        "main :: ()->int #no_aoc { wrapped :: cast(u8)255+cast(u8)3; answer :: #run -> int { value:u8=255; value+=3; return cast(int)value; }; return cast(int)wrapped+answer+38; }",
        42,
    );
    for source in [
        "main :: ()->int { overflow :: cast(u8)255+cast(u8)1; return 42; }",
        "main :: ()->int { overflow :: #run -> u8 { value:u8=255; value+=1; return value; }; return 42; }",
        "main :: ()->int #no_aoc { value:int=300; return cast(u8)value; }",
    ] {
        let fixture = Fixture::new(source);
        if source.contains("value:int=300") {
            let program = fixture.program();
            assert_eq!(
                jai_vm::execute(&program, Limits::default()).outcome,
                Outcome::Failed(Error::CheckedCast)
            );
            assert!(!fixture.native(&program).0.success());
        } else {
            let diagnostic = jai_sema::resolve_graph(&fixture.graph())
                .unwrap_err()
                .to_string();
            assert!(
                diagnostic.contains("overflow") || diagnostic.contains("IntegerOverflow"),
                "{diagnostic}"
            );
        }
    }
}
