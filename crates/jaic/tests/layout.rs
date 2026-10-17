//! Inherited field layout contracts of the public record shapes.
mod common;
use common::compile_source as compile;
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[test]
fn reflected_struct_alignment_matches_completed_layout() {
    let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
    let mut c = Compiler::new(options, Rc::new(jaic::sema::NativeFs));
    let source = r#"
        Record :: struct { value: u64; }
        Packed :: struct #no_padding { first: u8; second: u64; }
        Wide :: struct #align 32 { value: u8; }
        Box :: struct(T: Type) { value: T; }
        #assert((cast(*Type_Info_Struct) type_info(Record)).alignment == 8);
        #assert((cast(*Type_Info_Struct) type_info(Packed)).alignment == align_of(Packed));
        #assert((cast(*Type_Info_Struct) type_info(Wide)).alignment == 32);
        #assert((cast(*Type_Info_Struct) type_info(Box(u64))).alignment == 8);
        main :: () {}
    "#;
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn hidden_procedure_members_keep_their_reflected_marker_and_storage() {
    let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
    let mut c = Compiler::new(options, Rc::new(jaic::sema::NativeFs));
    let source = r#"
        Callback :: #type (value: u32) -> u32 #c_call;
        Hidden :: struct #type_info_procedures_are_void_pointers {
            callback: Callback;
            pointer: *void;
        }
        Plain :: struct { callback: Callback; }
        hidden :: type_info(Hidden);
        plain :: type_info(Plain);
        #assert(hidden.members[0].type == type_info(*void));
        #assert((hidden.members[0].flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) != 0);
        #assert((hidden.members[0].flags & .CONSTANT) == 0);
        #assert(hidden.members[0].offset_in_bytes == offset_of(Hidden, "callback"));
        #assert(hidden.members[1].type == type_info(*void));
        #assert((hidden.members[1].flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) == 0);
        #assert(plain.members[0].type == type_info(Callback));
        #assert((plain.members[0].flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) == 0);
        #assert(type_of(Hidden.callback) == Callback);
        main :: () {}
    "#;
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}


#[test]
fn overlay_and_place_resolve_inline_promoted_fields() {
    let (c, result) = compile(
        "Base :: struct { padding: u64; storage: [16] u8; } Middle :: struct { using base: Base; } Outer :: struct { lead: u64; #as using base: Middle; #overlay(storage) value: u64; } Placed :: struct { using base: Base; #place storage; value: u64; } #assert(offset_of(Outer, \"value\") == 16); #assert(offset_of(Outer, \"value\") == offset_of(Outer, \"storage\")); #assert(offset_of(Placed, \"value\") == 8); #assert(size_of(Outer) == 32); object: Outer; #assert(cast(*u8) *object.value == *object.base.base.storage[0]);",
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn embedded_using_filter_applies_to_member_lookup() {
    let (c, result) = compile(
        "Base :: struct { x: int; y: int; } Outer :: struct { using,only(x) base: Base; } object: Outer; #assert(*object.x == *object.base.x); #assert(*object.y == *object.base.y);",
    );
    let error = result.expect_err("filtered member must be inaccessible");
    assert!(error.message.contains("y"), "{}", error.render(&c.sources));
}

#[test]
fn overlay_cannot_target_storage_behind_using_pointer() {
    let (c, result) = compile(
        "Base :: struct { storage: [16] u8; } Outer :: struct { using base: *Base; #overlay(storage) value: u64; } #assert(size_of(Outer) > 0);",
    );
    let error = result.expect_err("dynamic pointer target has no inline layout offset");
    assert!(
        error.message.contains("storage"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn forward_overlay_offsets_types_and_reflection_order_are_independent() {
    let (c, result) = compile(include_str!(
        "../../../tests/compiler/semantics/forward-overlay.jai"
    ));
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
    let packet = c
        .types
        .structs
        .iter()
        .find(|s| s.name.as_str() == "Packet")
        .unwrap();
    let names: Vec<_> = packet
        .fields
        .iter()
        .map(|f| f.name.unwrap().as_str().to_string())
        .collect();
    assert_eq!(names, ["first", "second", "tag", "payload", "tail"]);
    assert_eq!(
        packet.fields.iter().map(|f| f.overlay).collect::<Vec<_>>(),
        [true, true, false, false, false]
    );
}

#[test]
fn overlay_dependency_cycles_and_dynamic_targets_reject() {
    for source in [
        "R :: struct { #overlay(b) a:u64; #overlay(a) b:u64; } #assert size_of(R)>0;",
        "R :: struct { #overlay(a) a:u64; } #assert size_of(R)>0;",
        "R :: struct { #place b; a:u64; b:u64; } #assert size_of(R)>0;",
        "I :: struct { data:u64; } R :: struct { #overlay(data) alias:u64; using inner:*I; } #assert size_of(R)>0;",
    ] {
        let (c, result) = compile(source);
        let error = result.expect_err("invalid layout dependency must reject");
        assert!(
            error.message.contains("cycle") || error.message.contains("inline storage offset"),
            "{}",
            error.render(&c.sources)
        );
    }
    let (c, result) = compile(
        "R :: struct { lead:u64; #overlay(anchor) alias:[281474976710656]u8; anchor:u64; } #assert size_of(R)>0;",
    );
    let error = result.expect_err("forward overlay extent must stay within MAX_SIZE");
    assert!(
        error.message.contains("too large"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn forward_layouts_keep_filters_alignment_conditionals_and_cursor_semantics() {
    let (c, result) = compile(
        r##"
        Inner :: struct { data:u64; other:u64; }
        R :: struct { #overlay(data) alias:u64; using,only(data) inner:Inner; tail:u8; }
        PromotedType :: struct { #overlay(data) alias:type_of(data); using inner:Inner; }
        AnonymousType :: struct { #overlay(data) alias:type_of(data); struct { data:u64; } }
        mapping_calls:int;
        rename :: (names:[]string) { mapping_calls+=1; for *names if it.*=="data" it.*="mapped"; }
        Mapped :: struct { #overlay(mapped) alias:type_of(mapped); using,map(rename) inner:Inner; }
        Conditional :: struct { #overlay(anchor) alias:type_of(anchor); #if true { anchor:u64; } tail:u8; }
        Inserted :: struct { #overlay(anchor) alias:type_of(anchor); #insert "anchor:u64;"; tail:u8; }
        Qualified :: struct { #overlay(anchor) alias:Qualified.anchor; anchor:u64; tail:u8; }
        Zero :: struct { anchor:[0]u8; #overlay(anchor) alias:u64; tail:u8; }
        Packed :: struct #no_padding { #overlay(anchor) alias:u32; lead:u8; anchor:u32; tail:u8; }
        Placed :: struct { base:u64; #place alias; value:u32; #overlay(base) alias:u64; }
        #assert offset_of(R,"alias")==0 && offset_of(R,"tail")==16;
        #assert size_of(PromotedType)==16;
        #assert size_of(AnonymousType)==8;
        #assert size_of(Mapped)==16 && offset_of(Mapped,"alias")==0;
        #assert mapping_calls==1;
        #assert offset_of(Conditional,"tail")==8 && offset_of(Inserted,"tail")==8;
        #assert offset_of(Qualified,"tail")==8;
        #assert offset_of(Zero,"tail")==0 && size_of(Zero)==8;
        #assert offset_of(Packed,"alias")==1 && offset_of(Packed,"tail")==5 && size_of(Packed)==6;
        #assert offset_of(Placed,"value")==0 && size_of(Placed)==8;
    "##,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
    for source in [
        "R :: struct { #overlay(anchor) alias:type_of(anchor); anchor:type_of(alias); } #assert size_of(R)>0;",
        "I :: struct { data:u64; } R :: struct { #overlay(data) alias:u64; using,except(data) inner:I; } #assert size_of(R)>0;",
    ] {
        let (c, result) = compile(source);
        let e = result.expect_err("cycle/filter must remain strict");
        assert!(
            e.message.contains("cycle") || e.message.contains("no field"),
            "{}",
            e.render(&c.sources)
        );
    }
}

#[test]
fn long_forward_overlay_chains_do_not_recurse_in_the_layout_solver() {
    let mut source = String::from("R :: struct {\n");
    for i in 0..5000 {
        source.push_str(&format!("#overlay(f{}) f{i}:u64;\n", i + 1));
    }
    source.push_str("f5000:u64; tail:u8; } #assert offset_of(R,\"f0\")==0 && offset_of(R,\"tail\")==8 && size_of(R)==16;");
    let (c, result) = compile(&source);
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn filtered_member_paths_preserve_pointer_storage_and_filters() {
    let (c, result) = compile(
        r#"
        Leaf :: struct {kept:s64;other:s64;}
        Wrap :: struct {link:*Leaf;using,only(kept) link;}
        check :: () -> bool {
            leaf:=Leaf.{kept=4};wrap:=Wrap.{link=*leaf};
            if wrap.kept!=4 return false;
            wrap.kept=9;
            return leaf.kept==9 && leaf.other==0;
        }
        #assert check();
    "#,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
    let (c, result) = compile(
        "Leaf :: struct {kept:s64;other:s64;} Wrap :: struct {link:*Leaf;using,only(kept) link;} main :: () {wrap:Wrap;wrap.other=1;}",
    );
    let error = result.expect_err("unselected pointer members must remain hidden");
    assert!(
        error.message.contains("no field") || error.message.contains("no member"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn ide_forward_anchor_navigation_and_padding_use_storage_order() {
    let source = "R :: struct { #overlay(anchor) alias:type_of(anchor); tag:u32; anchor:u64; } main :: () { r:R; r.alias=1; r.anchor=2; }";
    let mut options = Options::host();
    options.runtime_support = false;
    let mut fs = VirtualFs::default();
    fs.insert("/app/main.jai", source.as_bytes());
    let mut c = Compiler::new(options, Rc::new(fs));
    c.ide = Some(Box::new(jaic::sema::ide::IdeFacts::new(vec![
        "/app/".into(),
    ])));
    if let Err(e) = c.compile_program(std::path::Path::new("/app/main.jai")) {
        panic!("{}", e.render(&c.sources));
    }
    assert!(c.ide_check_all().is_empty());
    let file = jaic::source::FileId(0);
    let declaration = source.find("anchor:u64").unwrap() as u32;
    for at in [
        source.find("overlay(anchor").unwrap() + 8,
        source.find("type_of(anchor").unwrap() + 8,
    ] {
        let spans = c.ide_definition(file, at as u32);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start, declaration);
    }
    let hover = c
        .ide_hover(file, (source.find("r.anchor").unwrap() + 2) as u32)
        .unwrap();
    let layout = hover.layout.unwrap();
    assert_eq!(
        (layout.offset, layout.align, layout.padding),
        (Some(8), 8, 4)
    );
}
