//! Promoted literals preserve captures through VM and actual O0/O2 native execution.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-promoted-literal-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check(source: &str, expected: i32) {
    let scratch = Scratch::new();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode: optimization,
                ..Optimization::default()
            },
            ..TargetOptions::default()
        })
        .unwrap();
        let program = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                target: Some(target.build_target().unwrap()),
                layout: Some(target.layout_policy().unwrap()),
                ..jai_sema::ResolveOptions::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        let execution = jai_vm::execute(&program, jai_vm::Limits::default());
        let jai_vm::Outcome::Complete(values) = execution.outcome else {
            panic!("VM failed: {:?}", execution.outcome);
        };
        let [jai_vm::Value::Int(value)] = values.as_slice() else {
            panic!("integer result required");
        };
        assert_eq!(value.value(), i128::from(expected));
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command.arg(&object);
        if cfg!(target_os = "macos") {
            command.arg("-Wl,-no_fixup_chains");
        }
        let output = command.arg("-o").arg(&executable).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected), "{optimization:?}\n{source}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated promoted literal fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn jaison_number() {
    check(
        r###"
JSON_Type::enum u8{NULL::0;BOOLEAN::1;NUMBER::3;STRING::2;}
JSON_Value::struct{type:JSON_Type;union{boolean:bool;number:float64;str:string;}}
main::()->int{value:=JSON_Value.{type=.NUMBER,number=42.0}; copy:=value; if copy.type!=.NUMBER return 1; return cast(int)copy.number;}
"###,
        42,
    );
}

#[test]
fn jaison_string() {
    check(
        r###"
JSON_Type::enum u8{NULL::0;STRING::2;}
JSON_Value::struct{type:JSON_Type;union{number:float64;str:string;}}
main::()->int{value:JSON_Value=.{type=.STRING,str="forty-two"};copy:=value;if copy.type!=.STRING || copy.str!="forty-two" return 1;return 42;}
"###,
        42,
    );
}

#[test]
fn interleaved_source_order() {
    check(
        r###"
Owner::struct{struct{x,y:int;}middle:int;}
tick::(state:*int,n:int)->int{state.*=state.* * 10+n;return n;}
main::()->int{state:=0;value:Owner=.{x=tick(*state,1),middle=tick(*state,2),y=tick(*state,3)};if state!=123 || value.x!=1 || value.middle!=2 || value.y!=3 return 1;return 42;}
"###,
        42,
    );
}

#[test]
fn lazy_selected_arm() {
    check(
        r###"
Owner::struct{struct{x,y:int;}}
tick::(state:*int)->int{state.*+=1;return 21;}
main::()->int{state:=0;value:Owner=ifx false then Owner.{x=tick(*state),y=tick(*state)} else Owner.{x=20,y=22};if state!=0 return 1;return value.x+value.y;}
"###,
        42,
    );
}

#[test]
fn selected_branch_default() {
    check(
        r###"
Owner::struct{union{struct{x:int=1;y:int=22;}other:int=99;}}
main::()->int{value:Owner=.{x=20};copy:=value;return copy.x+copy.y;}
"###,
        42,
    );
}

#[test]
fn named_using_overlay() {
    check(
        r###"
Child::struct{x:int=1;y:int=2;}
Owner::struct{using child:Child=.{x=7,y=22};}
main::()->int{value:Owner=.{x=20};return value.x+value.y;}
"###,
        42,
    );
}

#[test]
fn ordered_default_override() {
    check(
        r###"
Owner::struct{struct{x:int=1;y:int=2;}x=5;y=22;}
main::()->int{value:Owner=.{x=20};return value.x+value.y;}
"###,
        42,
    );
}

#[test]
fn generic_canonical_fields() {
    check(
        r###"
Box::struct(T:Type,Fill:T){struct{amount:T=Fill;}tail:T=Fill;}
main::()->int{value:Box(int,21)=.{amount=21,tail=21};copy:=value;return copy.amount+copy.tail;}
"###,
        42,
    );
}

#[test]
fn pointer_provenance() {
    check(
        r###"
Owner::struct{union{pointer:*int;number:int;}}
identity::(p:*int)->*int{return p;}
main::()->int{value:=42;box:Owner=.{pointer=identity(*value)};copy:=box;return copy.pointer.*;}
"###,
        42,
    );
}

#[test]
fn callback_provenance() {
    check(
        r###"
Callback::#type(x:int)->int #must;
Owner::struct{struct{callback:Callback;}}
add::(x:int)->int #must{return x+22;}
make::()->Callback{return add;}
main::()->int{value:Owner=.{callback=make()};copy:=value;return copy.callback(x=20);}
"###,
        42,
    );
}

#[test]
fn raw_float_bits() {
    check(
        r###"
Owner::struct{union{number:float64;bits:u64;}}
identity::(value:float64)->float64{return value;}
main::()->int{value:Owner=.{number=identity(0h7FF8_0000_0000_0042)};copy:=value;return ifx copy.bits==0x7ff8000000000042 then 42 else 1;}
"###,
        42,
    );
}
