//! Verify VM/native parity using only newly generated native code.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_modules::{ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(std::path::PathBuf);

fn optional_original_source(relative: &str) -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    match std::fs::read_to_string(&path) {
        Ok(source) => Some(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "SKIP optional original-source operator gate: {} is absent",
                path.display()
            );
            None
        }
        Err(error) => panic!(
            "cannot read optional original source {}: {error}",
            path.display()
        ),
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn parity(source: &str, expected: i32) {
    parity_files(&[("/native-operators/main.jai", source)], expected);
}
fn parity_files(files: &[(&str, &str)], expected: i32) {
    parity_with_options(files, expected, &jai_sema::ResolveOptions::default());
}
fn parity_with_options(files: &[(&str, &str)], expected: i32, options: &jai_sema::ResolveOptions) {
    let mut overlay = SourceOverlay::new();
    let path = Path::new(files[0].0);
    for &(path, source) in files {
        overlay
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(path, Default::default(), &overlay).unwrap();
    let program =
        jai_sema::resolve_graph_with_options(&graph, options, &mut jai_vm::NoEffects).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("VM failed: {:?}", execution.outcome);
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected an integer VM result");
    };
    assert_eq!(value.value(), i128::from(expected));
    let ir = jai_codegen::emit(&program).unwrap();
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-native-operators-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&scratch.0).unwrap();
    for optimization in ["-O0", "-O2"] {
        let executable = scratch.0.join(format!("generated-program{optimization}"));
        let mut compiler = native_tools::clang_command()
            .args([optimization, "-x", "ir", "-", "-o"])
            .arg(&executable)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        compiler
            .stdin
            .take()
            .unwrap()
            .write_all(ir.as_bytes())
            .unwrap();
        let output = compiler.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{optimization}: {}\n{ir}",
            String::from_utf8_lossy(&output.stderr)
        );
        let fingerprint = std::fs::read(&executable)
            .unwrap()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
            });
        let mut process = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = process.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected), "{optimization} result");
                eprintln!(
                    "operator-native {optimization} vm={expected} native={expected} binary-fnv1a64={fingerprint:016x}"
                );
                break;
            }
            if Instant::now() >= deadline {
                process.kill().unwrap();
                process.wait().unwrap();
                panic!("generated {optimization} operator program exceeded five seconds");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn lexical_modifier_selection_has_vm_native_parity() {
    parity_with_options(
        &[(
            "/native-operators/main.jai",
            r#"
        Box::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:int)->$R #modify{R=T;return true;}
                {return .{value=a.value+b};}
            operator +::(a:$T,b:$U)->T #modify{return false,"excluded";}
                {return invalid_rejected_body;}
            return (Box.{value=4}+3).value;
        }
    "#,
        )],
        7,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
    );
}

#[test]
fn local_generic_mutation_loads_runtime_defaults_after_its_operands() {
    parity(
        r#"
        Box::struct{value:int;}
        factor:int=2;
        operand::()->int{factor=3;return 2;}
        main::()->int{
            operator +=::(a:*$T,b:int,scale:int=factor){a.value+=b*scale;}
            value:=Box.{value=1};value+=operand();return value.value;
        }
    "#,
        7,
    );
}

#[test]
fn arithmetic_nominal_and_compound_calls_execute_natively() {
    parity(
        include_str!("../../../tests/corpus/positive/operator-overloads.jai"),
        29,
    );
}

#[test]
fn authored_floating_record_operators_have_vm_native_parity() {
    parity(
        include_str!("../../../tests/corpus/positive/operator-record-floats.jai"),
        24,
    );
}

#[test]
fn local_generic_operators_preserve_specializations_and_symmetric_order_natively() {
    parity(
        r#"
        Box::struct{value:int;}
        Other::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:T)->T{return .{value=a.value+b.value};}
            a:=Box.{value=3};b:=Box.{value=4};x:=Other.{value=5};y:=Other.{value=6};
            first:=a+b;repeated:=a+b;second:=x+y;
            return first.value+repeated.value+second.value;
        }
    "#,
        25,
    );
    parity(
        r#"
        trace:int;
        Box::struct{value:int;}
        scalar::()->int{trace=trace*10+1;return 3;}
        record::()->Box{trace=trace*10+2;return .{value=4};}
        main::()->int{
            operator *::(a:$T,b:int)->T #symmetric{return .{value=a.value*b};}
            value:=scalar()*record();return trace+value.value;
        }
    "#,
        24,
    );
}

#[test]
fn local_generic_getter_setter_and_outer_environments_have_native_parity() {
    parity(
        r#"
        Box::struct{value:int;}
        trace:int;
        key::()->int{trace=trace*10+1;return 0;}
        operand::()->int{trace=trace*10+3;return 4;}
        main::()->int{
            operator []::(a:$T,index:int)->int{trace=trace*10+2;return a.value+index;}
            operator []=::(a:*$T,index:int,value:int){trace=trace*10+4;a.value=value+index;}
            box:=Box.{value=5};box[key()]+=operand();
            if trace!=1234 return 0;
            return box.value;
        }
    "#,
        9,
    );
    parity(
        r#"
        Box::struct{value:int;}
        work::(a:$T,$bias:int)->int{
            operator +::(x:$U,y:U)->U{return .{value=x.value+y.value+bias};}
            return (a+a).value;
        }
        main::()->int{return work(Box.{value=3},1)+work(Box.{value=3},10);}
    "#,
        23,
    );
}

#[test]
fn baked_operator_source_arity_and_compact_runtime_calls_have_native_parity() {
    parity(
        r#"
        Box::struct{value:u64;}
        trace:int;
        left::()->Box{trace=trace*10+1;return .{value=8};}
        right::()->u8{trace=trace*10+2;return 2;}
        operator <<::(a:Box,$$x:u8)->Box{
            trace=trace*10+3;
            #if is_constant(x) {return .{value=(a.value<<x)+1};}
            else {return .{value=a.value<<x};}
        }
        main::()->int{
            first:=left()<<1;second:=left()<<right();
            if trace!=13123 return 0;
            return cast(int)(first.value+second.value);
        }
    "#,
        49,
    );
    parity(
        "Box::struct{value:int;} operator []::(a:Box,$index:int)->int{return a.value+index;} main::()->int{return Box.{value=5}[3];}",
        8,
    );
}

#[test]
fn indexed_getter_setter_updates_have_vm_native_parity() {
    parity(
        include_str!("../../../tests/corpus/positive/operator-index-updates.jai"),
        11,
    );
    parity(
        r#"
        Box::struct{value:int;}
        storage:Box;
        key::()->int{storage.value=100;return 1;}
        operator []::(box:Box,index:int)->int{return box.value+index;}
        operator []=::(box:*Box,index:int,value:int){box.value=value;}
        main::()->int{storage.value=10;storage[key()]+=3;return storage.value;}
    "#,
        14,
    );
}

#[test]
fn symmetric_source_order_and_captured_update_places_execute_natively() {
    parity(
        r#"
        trace:int;
        Box::struct {value:int;}
        scalar::()->int {trace=trace*10+1;return 3;}
        record::()->Box {trace=trace*10+2;return Box.{value=4};}
        index::()->int {trace+=10;return 0;}
        operator *::(a:Box,b:int)->Box #symmetric {return Box.{value=a.value*b};}
        operator +::(a:Box,b:Box)->Box {return Box.{value=a.value+b.value};}
        main::()->int {
            value:=scalar()*record(); values:[1]Box; values[0]=value;
            values[index()]+=Box.{value=5}; return trace+values[0].value;
        }
    "#,
        39,
    );
}

#[test]
fn original_math_record_operator_source_has_vm_native_parity() {
    let Some(source) =
        optional_original_source("corpus/upstream/withlang-dev--open-jai/modules/Math/module.jai")
    else {
        return;
    };
    let lines = source.lines().collect::<Vec<_>>();
    let definitions = [
        lines[5..22].join("\n"),
        lines[83..89].join("\n"),
        lines[118..136].join("\n"),
    ]
    .join("\n");
    let source = format!(
        "{definitions}\nmain::()->int{{
        value:=2.0*Vector3.{{x=1,y=2,z=3}}+Vector3.{{x=4,y=5,z=6}};
        result:=Quaternion.{{x=1,y=2,z=3,w=4}}*3.0;
        return cast(int)(value.z+result.w);
    }}"
    );
    parity(&source, 24);
}

#[test]
fn original_object_mutation_operator_source_has_vm_native_parity() {
    let Some(source) = optional_original_source(
        "corpus/upstream/withlang-dev--open-jai/examples/24/24.2_overloading_object.jai",
    ) else {
        return;
    };
    let definitions = source
        .lines()
        .skip(2)
        .take(21)
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "{definitions}\nmain::()->int{{
        value:Obj; value.array=.[0,1,2,3,4,5,6,7,8,9]; value[2]=2; value*=3;
        pointer:=*value[2]; <<pointer*=2;
        return value[2]+value[1];
    }}"
    );
    parity(&source, 15);
}

#[test]
fn mutating_operator_calls_capture_targets_and_operands_once_natively() {
    parity(
        r#"
        trace:int;
        Obj::struct {value:int;}
        index::()->int {trace+=1;return 0;}
        slot::()->int {trace+=10;return 0;}
        item::()->int {trace+=100;return 5;}
        scalar::()->int {trace+=1000;return 3;}
        operator []::(obj:Obj,i:int)->int {return obj.value+i;}
        operator []=::(obj:*Obj,i:int,item:int) {obj.value=item+i;}
        operator *[]::(obj:*Obj,i:int)->*int {return *obj.value;}
        operator *=::(obj:*Obj,scalar:int) {obj.value*=scalar;}
        main::()->int {
            objects:[1]Obj;
            objects[index()][slot()]=item();
            objects[index()]*=scalar();
            pointer:=*objects[index()][slot()]; <<pointer+=2;
            if trace!=1123 return 0;
            return objects[0][0];
        }
    "#,
        17,
    );
}

#[test]
fn symmetric_and_mutating_trailing_defaults_have_vm_native_parity() {
    parity(
        r#"
        Obj::struct {value:int;}
        operator *::(obj:Obj,item:int,tag:int=7)->Obj #symmetric {return Obj.{value=obj.value*item+tag};}
        operator +=::(obj:*Obj,item:Obj,tag:int=9) {obj.value+=item.value+tag;}
        main::()->int {value:=2*Obj.{value=3}; value+=Obj.{value=4}; return value.value;}
    "#,
        26,
    );
}

#[test]
fn inequality_equality_fallback_evaluates_each_operand_once_natively() {
    parity(
        r#"
        Obj::struct {value:int;}
        calls:int;
        operand::()->Obj {calls+=1;return Obj.{value=6};}
        operator ==::(a:Obj,b:Obj)->bool{return a.value==b.value;}
        main::()->int {same:=operand()!=operand(); if same return 0; return 10+calls;}
    "#,
        12,
    );
}

#[test]
fn namespace_operator_aliases_keep_original_calls_and_generic_types_natively() {
    parity_files(
        &[
            (
                "/native-operators/main.jai",
                r#"
Library::#import,file "math.jai";
operator-::Library.operator-;
main::()->int{
    a:Library.Box=.{value=10}; b:Library.Box=.{value=3};
    negative:=-a; difference:=a-b;
    return difference.value-negative.value;
}
"#,
            ),
            (
                "/native-operators/math.jai",
                r#"
Box::struct{value:int;}
operator -::(value:Box)->Box{return .{value=-value.value};}
operator -::(a:Box,b:Box)->Box{return .{value=a.value-b.value};}
"#,
            ),
        ],
        17,
    );
    parity_files(
        &[
            (
                "/native-operators/main.jai",
                r#"
Library::#import,file "math.jai";
operator+::Library.operator+;
main::()->int{
    a:Library.Box(int)=.{value=5}; b:Library.Box(int)=.{value=7};
    return (a+b).value;
}
"#,
            ),
            (
                "/native-operators/math.jai",
                r#"
Box::struct($T:Type){value:T;}
operator +::(a:Box($T),b:Box(T))->Box(T){return .{value=a.value+b.value};}
"#,
            ),
        ],
        12,
    );
}
