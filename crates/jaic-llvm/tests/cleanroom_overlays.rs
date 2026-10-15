//! Execute forward aliases through the actual native and WASM64 backends.
#[path = "support/backend_execution.rs"]
mod backend;
#[path = "../../jaic/tests/support/reflection_bootstrap.rs"]
mod bootstrap;

#[test]
fn forward_overlays_execute_native_and_wasm_at_o0_and_o2() {
    backend::execute(
        "forward-overlay",
        "overlay_check",
        include_str!("../../../tests/compiler-semantics/forward-overlay.jai"),
        &[],
        None,
    );
}

#[test]
fn forward_overlay_reflection_executes_native_and_wasm_at_o0_and_o2() {
    let source = r#"
        R :: struct { #overlay(anchor) alias:u32; tag:u32; anchor:u64; }
        #program_export "overlay_reflection_check"
        check :: () -> s32 #c_call {
            info:=type_info(R);
            if info.members.count!=3 || info.alignment!=8 || info.runtime_size!=16 return 1;
            if info.members[0].offset_in_bytes!=8 || (info.members[0].flags & 0x20)==0 return 2;
            if info.members[1].offset_in_bytes!=0 || info.members[2].offset_in_bytes!=8 return 3;
            if info.members[0].name.count!=5 || info.members[0].name[0]!=97 return 4;
            if info.members[1].name.count!=3 || info.members[1].name[0]!=116 return 5;
            if info.members[2].name.count!=6 || info.members[2].name[0]!=97 return 6;
            return 0;
        }
        main :: () {}
    "#;
    backend::execute(
        "forward-overlay-reflection",
        "overlay_reflection_check",
        source,
        &[("/lib/prelude/module.jai", bootstrap::BOOTSTRAP)],
        None,
    );
}
