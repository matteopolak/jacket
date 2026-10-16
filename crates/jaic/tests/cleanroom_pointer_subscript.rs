//! Independent pointer-subscript dispatch and lvalue controls.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

const STORAGE: &str = r#"
    Storage :: struct(Elem: Type, chunk: int, mode := false) { data: *Elem; }
    operator *[] :: (using storage: Storage, index: int) -> *storage.Elem {
        return data + index;
    }
"#;

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", bootstrap::BOOTSTRAP.as_bytes());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn generic_by_value_receiver_supports_read_address_and_assignment() {
    compile(&format!(
        r#"{STORAGE}
        verify :: () -> bool {{
            values: [3]s64;
            values[0] = 7;
            storage: Storage(s64,1024);
            storage.data = *values[0];
            before := storage[0];
            address := *storage[0];
            storage[1] = 13;
            storage[1] += 4;
            return before == 7 && values[1] == 17 && address == *values[0];
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn pointer_held_in_local_is_loaded_instead_of_addressed_again() {
    compile(&format!(
        r#"{STORAGE}
        verify :: () -> bool {{
            values: [3]s64;
            values[0] = 7;
            storage: Storage(s64,1024);
            storage.data = *values[0];
            pointer := *storage;
            before := pointer[0];
            address := *pointer[0];
            pointer[1] = 13;
            pointer[1] += 4;
            return before == 7 && values[1] == 17 && address == *values[0];
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn by_value_receiver_accepts_temporary_structs_and_evaluates_once() {
    compile(&format!(
        r#"{STORAGE}
        make :: (data:*s64, count:*s64) -> Storage(s64,1024) {{
            count.* += 1;
            return .{{data}};
        }}
        verify :: () -> bool {{
            value:s64=7;
            count:s64;
            before:=make(*value,*count)[0];
            address:=*make(*value,*count)[0];
            make(*value,*count)[0]=13;
            return before==7 && value==13 && address==*value && count==3;
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn using_promoted_storage_member_preserves_element_identity() {
    compile(&format!(
        r#"{STORAGE}
        Inner :: struct {{ storage:Storage(s64,1024); }}
        Outer :: struct {{ using inner:Inner; }}
        verify :: () -> bool {{
            value:s64=7;
            outer:Outer;
            outer.storage.data=*value;
            pointer:=*outer;
            address:=*pointer.storage[0];
            pointer.storage[0]=13;
            return address==*value && value==13;
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn ordinary_getter_wins_while_explicit_address_uses_pointer_getter() {
    compile(&format!(
        r#"{STORAGE}
        operator [] :: (storage:Storage,index:int)->s64 {{ return 91; }}
        verify :: ()->bool {{
            value:s64=7;
            storage:Storage(s64,1024);
            storage.data=*value;
            return storage[0]==91 && *storage[0]==*value;
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn value_getter_does_not_make_result_assignable() {
    let error = compile(
        r#"
        Storage :: struct { value:s64; }
        operator [] :: (storage:Storage,index:int)->s64 {return storage.value;}
        main :: () {storage:Storage; storage[0]=17;}
    "#,
    )
    .expect_err("a value-only getter must not create storage");
    assert!(
        error.contains("assign") || error.contains("lvalue"),
        "{error}"
    );
}

#[test]
fn nonpointer_pointer_getter_result_remains_invalid() {
    let error = compile(
        r#"
        Storage :: struct { value:s64; }
        operator *[] :: (storage:Storage,index:int)->s64 {return storage.value;}
        main :: () {storage:Storage; value:=storage[0];}
    "#,
    )
    .expect_err("pointer subscript cannot manufacture a place from an integer");
    assert!(
        error.contains("index") || error.contains("pointer"),
        "{error}"
    );
}

#[test]
fn builtin_address_fallback_evaluates_operands_once() {
    compile(&format!(
        r#"{STORAGE}
        make :: (data:*s64,count:*s64)->*s64 {{
            count.*+=1;
            return data;
        }}
        verify :: ()->bool {{
            value:s64=7; count:s64;
            address:=*make(*value,*count)[0];
            return address==*value && count==1;
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn pointer_receiver_and_result_type_queries_follow_normal_calls() {
    compile(
        r#"
        Storage :: struct { data:*s64; calls:s64; }
        operator *[] :: (storage:*Storage,index:int)->*s64 {
            storage.calls+=1;
            return storage.data+index;
        }
        verify :: ()->bool {
            value:s64=7; storage:Storage;
            storage.data=*value;
            pointer:=*storage;
            #assert type_of(pointer[0])==s64;
            #assert type_of(*pointer[0])==*s64;
            pointer[0]+=6;
            return value==13 && storage.calls==1;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn explicit_address_rejects_nonpointer_pointer_getter_result() {
    let error = compile(
        r#"
        Storage :: struct { value:s64; }
        operator *[] :: (storage:Storage,index:int)->s64 {return storage.value;}
        main :: () {storage:Storage; address:=*storage[0];}
    "#,
    )
    .expect_err("the address form must also validate the pointer result");
    assert!(error.contains("pointer"), "{error}");
}

#[test]
fn index_argument_keeps_ordinary_binding_diagnostics() {
    let error = compile(&format!(
        r#"{STORAGE}
        main :: () {{ storage:Storage(s64,1024); value:=storage["wrong"]; }}
    "#
    ))
    .expect_err("an invalid index cannot bypass parameter checking");
    assert!(error.contains("int") || error.contains("s64"), "{error}");
}

#[test]
fn filtered_module_reexports_preserve_operator_visibility() {
    for expose in [true, false] {
        let mut options = Options::host();
        options.runtime_support = false;
        options.import_paths = vec!["/lib".into()];
        options.preload = Some("/lib/prelude/module.jai".into());
        let mut fs = VirtualFs::default();
        fs.insert("/lib/prelude/module.jai", bootstrap::BOOTSTRAP.as_bytes());
        fs.insert("/lib/Implementation/module.jai", STORAGE.as_bytes());
        let facade = if expose {
            r#"using,only(.["Storage","*[]"]) Impl :: #import "Implementation";"#
        } else {
            r#"using,only(Storage) Impl :: #import "Implementation";"#
        };
        fs.insert("/lib/Facade/module.jai", facade.as_bytes());
        fs.insert(
            "/app/main.jai",
            br#"
            #import "Facade";
            verify :: () -> bool {
                value:s64=7;
                storage:Storage(s64,1024);
                storage.data=*value;
                storage[0]=13;
                return value==13 && *storage[0]==*value;
            }
            #assert verify();
        "#
            .as_slice(),
        );
        let mut compiler = Compiler::new(options, Rc::new(fs));
        let result = compiler.compile_program(std::path::Path::new("/app/main.jai"));
        if expose {
            result
                .map_err(|error| error.render(&compiler.sources))
                .unwrap();
        } else {
            let error =
                result.expect_err("a type-only reexport must not leak its defining operators");
            assert!(
                error.message.contains("index"),
                "{}",
                error.render(&compiler.sources)
            );
        }
    }
}

#[test]
fn unrelated_pointer_getter_does_not_hide_builtin_pointer_indexing() {
    compile(&format!(
        r#"{STORAGE}
        Other :: struct {{ value:s64; }}
        verify :: ()->bool {{
            values:[2]Other;
            values[1].value=17;
            pointer:=*values[0];
            pointer[0].value=7;
            address:=*pointer[1];
            return pointer[0].value==7 && address==*values[1] && address.value==17;
        }}
        #assert verify();
    "#
    ))
    .unwrap();
}

#[test]
fn receiver_filter_recognizes_implicit_generic_setters() {
    compile(
        r#"
        Slots :: struct(Elem:Type) {data:*Elem; writes:s64;}
        operator []= :: (slots:*Slots,index:int,value:slots.Elem) {
            slots.writes+=1;
            slots.data[index]=value+1;
        }
        verify :: ()->bool {
            value:s64; slots:Slots(s64); slots.data=*value;
            slots[0]=7;
            return value==8 && slots.writes==1;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn generic_elsewhere_in_header_does_not_widen_fixed_receiver() {
    compile(
        r#"
        Other :: struct {data:*s64;}
        operator *[] :: (other:Other,index:$Index)->*s64 {return other.data+index;}
        Record :: struct {value:s64;}
        verify :: ()->bool {
            values:[2]Record;
            pointer:=*values[0];
            pointer[1].value=13;
            return pointer[1].value==13 && *pointer[1]==*values[1];
        }
        #assert verify();
    "#,
    )
    .unwrap();
}
