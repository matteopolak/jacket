use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-reflection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let options = ResolveOptions {
        layout: Some(LayoutPolicy::lp64()),
        ..ResolveOptions::default()
    };
    let program = resolve_graph_with_options(&fixture.graph(), &options, &mut NoEffects)
        .unwrap_or_else(|error| panic!("reflection source must resolve: {error:?}"));
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("reflection source must execute: {:?}", execution.outcome);
    };
    let [Value::Int(result)] = values.as_slice() else {
        panic!("expected one integer result, got {values:?}");
    };
    result.value()
}

#[test]
fn type_of_does_not_execute_its_operand() {
    assert_eq!(
        run(
            "calls:int=0; tick :: () -> int { calls += 1; return 9; } main :: () -> int { T :: type_of(tick()); value:T=42; return value+calls*100; }"
        ),
        42
    );
}

#[test]
fn size_of_uses_target_record_array_and_descriptor_layouts() {
    assert_eq!(
        run(
            "Pair :: struct { small:u8; value:s64; } main :: () -> int { return size_of(Pair)+size_of([3]u16)+size_of(*Pair)+size_of([]Pair); }"
        ),
        46
    );
}

#[test]
fn initializer_of_preserves_declaration_defaults() {
    assert_eq!(
        run(
            "Pair :: struct { first:int=17; second:int=25; } main :: () -> int { pair := initializer_of(Pair); return pair.first+pair.second; }"
        ),
        42
    );
}

#[test]
fn nominal_type_namespace_constants_have_owned_relocation_storage() {
    assert_eq!(
        run(r#"
        Namespace :: struct {
            value: int;
            Array_Type :: enum u16 { FIXED :: 0; VIEW :: 1; RESIZABLE :: 2; }
        }
        main :: () -> int {
            info := type_info(Namespace);
            if info.members.count != 2 return 1;
            member := info.members[1];
            if member.name != "Array_Type" return 2;
            if cast(u32)member.flags != 1 return 3;
            if member.offset_into_constant_storage != 0 return 4;
            if info.constant_storage.count != size_of(Type) return 5;
            stored := cast(*Type)(info.constant_storage.data).*;
            if stored != Namespace.Array_Type return 6;
            if member.type != cast(*Type_Info)type_info(Type) return 7;
            return 42;
        }
    "#),
        42
    );
}

#[test]
fn direct_then_nested_type_info_has_one_descriptor_address() {
    assert_eq!(
        run(
            "Pair :: struct { value:int; } Wrap :: struct { pair:Pair; } main :: () -> int { direct:=type_info(Pair); outer:=type_info(Wrap); if cast(*void)(outer.members[0].type) != cast(*void)direct return 1; pointer:=type_info(*Pair); if cast(*void)(pointer.pointer_to) != cast(*void)direct return 2; return 42; }"
        ),
        42
    );
}

#[test]
fn nested_then_direct_type_info_has_one_descriptor_address() {
    assert_eq!(
        run(
            "Pair :: struct { value:int; } Wrap :: struct { pair:Pair; } main :: () -> int { pointer:=type_info(*Pair); outer:=type_info(Wrap); direct:=type_info(Pair); if cast(*void)(pointer.pointer_to) != cast(*void)direct return 1; if cast(*void)(outer.members[0].type) != cast(*void)direct return 2; return 42; }"
        ),
        42
    );
}

#[test]
fn recursive_type_info_points_back_to_the_root_descriptor() {
    assert_eq!(
        run(
            "Node :: struct { next:*Node; value:int; } main :: () -> int { root:=type_info(Node); next:=cast(*Type_Info_Pointer)(root.members[0].type); if cast(*void)(next.pointer_to) != cast(*void)root return 1; return 42; }"
        ),
        42
    );
}

#[test]
fn descriptor_header_upcasts_preserve_identity_across_calls_and_storage() {
    assert_eq!(
        run(r#"
            Pair :: struct { value:int; }
            Wrap :: struct { pair:Pair; }
            identity :: (info:*Type_Info) -> *Type_Info { return info; }
            main :: () -> int {
                direct := type_info(Pair);
                header:*Type_Info=direct;
                if identity(type_info(Pair)) != header return 1;
                if identity(direct) != header return 2;
                outer := type_info(Wrap);
                if outer.members[0].type != identity(direct) return 3;
                pointer := type_info(*Pair);
                if pointer.pointer_to != identity(direct) return 4;
                return 42;
            }
        "#),
        42
    );
}

#[test]
fn field_notes_and_offsets_preserve_declaration_metadata() {
    assert_eq!(
        run(r#"
            Pair :: struct {
                first:u8; @JsonName("key")
                second:u64; @JsonIgnore
            }
            same_bytes :: (a:string, b:string) -> bool {
                if a.count != b.count return false;
                if a.count == 0 return true;
                for i: 0..a.count-1 {
                    if a[i] != b[i] return false;
                }
                return true;
            }
            main :: () -> int {
                info := type_info(Pair);
                if info.runtime_size != 16 return 1;
                if info.members.count != 2 return 2;
                if info.members[0].offset_in_bytes != 0 return 3;
                if info.members[1].offset_in_bytes != 8 return 4;
                if info.members[0].notes.count != 1 return 5;
                if info.members[1].notes.count != 1 return 6;
                if !same_bytes(info.members[0].notes[0], "JsonName(\"key\")") return 7;
                if !same_bytes(info.members[1].notes[0], "JsonIgnore") return 8;
                return 42;
            }
        "#),
        42
    );
}
