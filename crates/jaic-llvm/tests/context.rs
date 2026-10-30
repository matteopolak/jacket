//! Execute original context/cleanup control-flow cases through both backends.
use test_support::backend;

const SOURCE: &str = r#"
#add_context marker: s64 = 7;
observed: int;
conditional :: (enabled: bool) #expand {
    if enabled { { copied := context; `push_context,defer_pop copied; `defer observed += context.marker; } }
}
run :: (enabled: bool) -> bool {
    context.marker = 55;
    { conditional(enabled); context.marker = 91; }
    return context.marker == (ifx enabled then 55 else 91);
}
early :: () -> int {
    context.marker = 55;
    defer observed = context.marker;
    push_context { context.marker = 123; return context.marker; }
}
#program_export "context_check"
check :: () -> s32 #c_call {
    push_context {
        if !run(false) || observed != 0 return 1;
        if !run(true) || observed != 91 return 2;
        if early() != 123 || observed != 55 return 3;
        for i: 0..3 {
            context.marker = 55;
            conditional(true);
            context.marker = 91;
            if i < 2 continue;
            break;
        }
        if context.marker != 55 return 4;
    }
    return 0;
}
"#;

#[test]
fn conditional_context_and_exit_cleanup_native_and_wasm() {
    backend::execute("context-backend", "context_check", SOURCE, &[], None);
}
