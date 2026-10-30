//! A member reached through a `using` parameter is part of that parameter, so it cannot be
//! assigned when the parameter itself cannot.
use test_support::compiler::compile_source;

const TYPES: &str = "Inner :: struct { y: s32; } Outer :: struct { x: s32; inner: Inner; }";

fn outcome(procedure: &str, call: &str) -> Result<(), String> {
    let source = format!("{TYPES} {procedure} main :: () {{ o: Outer; run({call}); }}");
    let (c, result) = compile_source(&source);
    result.map_err(|error| error.render(&c.src.sources))
}

#[test]
fn writing_a_used_member_of_an_argument_is_rejected() {
    for body in [
        "x = 9;",
        "x += 9;",
        "inner.y = 9;",
        "inner = .{};",
        "x, inner.y = 1, 2;",
    ] {
        let message =
            outcome(&format!("run :: (using o: Outer) {{ {body} }}"), "o").expect_err(body);
        assert!(
            message.contains("Can't assign to an immutable argument."),
            "{body}: {message}"
        );
    }
}

#[test]
fn other_writes_stay_allowed() {
    for procedure in [
        // Reads, a copy, and a pointer argument.
        "run :: (using o: Outer) { v := x; v = 3; }",
        "run :: (using o: *Outer) { x = 9; }",
        // A local `using` variable is not an argument.
        "run :: (a: Outer) { o := a; using o; x = 9; }",
        // `#compile_time` procedures run in the compiler on plain locals.
        "run :: (using o: Outer) #compile_time { x = 9; }",
    ] {
        let call = if procedure.contains("*Outer") {
            "*o"
        } else {
            "o"
        };
        if let Err(message) = outcome(procedure, call) {
            panic!("{procedure}: {message}");
        }
    }
}
