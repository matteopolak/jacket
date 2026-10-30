//! Real native/WASM execution of mixed variadic argument normalization.
use test_support::backend;
#[path = "../../jaic/tests/support/variadic_source.rs"]
mod source;

fn execute(name: &str, source: &str) {
    backend::execute(name, "variadic_check", source, &[], None);
}

#[test]
fn mixed_variadics_execute_native_and_wasm_at_o0_and_o2() {
    execute("ordinary", source::SOURCE);
}

#[test]
fn dynamic_packs_survive_continuation_unwind_and_rewind() {
    execute(
        "continuation",
        r#"
    Control :: struct { state: u64; top: *u8; limit: *u8; enabled: u64; }
    control: Control;
    frames: [8192]u8;
    tail: [2]int = .[2,3];
    #program_export "jaic_wasi_green_control"
    get_control :: () -> *Control #no_context { return *control; }
    #program_export "jaic_wasi_frames_exhausted"
    exhausted :: () #no_context { #asm { int3; } }
    #program_export "pthread_create"
    spawn :: () #no_context {}
    #program_export "jaic_wasi_suspend"
    suspend :: () #no_context {
        if control.state == 2 { control.state=0; return; }
        control.state=1;
    }
    sum :: (items: ..int) -> int #no_context {
        suspend();
        result:int; for items result=result*10+it;
        return result;
    }
    thread :: () -> int #no_context { return sum(1,..tail,4); }
    #program_export "jaic_wasi_run_threads"
    schedule :: () -> s32 #no_context {
        spawn();
        for 0..3 {
            control.top=frames.data; control.limit=frames.data+frames.count;
            control.state=0;
            result:=thread();
            if control.state!=1 return 1;
            // Clobber unrelated native-stack storage between the two walks.
            noise:[1024]u8; for *noise it.*=cast(u8)(it_index & 255);
            control.top=frames.data; control.state=2;
            result=thread();
            if control.state!=0 || result!=1234 return 2;
            if control.top!=frames.data return 3;
        }
        return 0;
    }
    #program_export "variadic_check"
    check :: () -> s32 #c_call { return schedule(); }
    "#,
    );
}
