//! An optimized program divided into several modules before the optimizer (`JAIC_CODEGEN_UNITS`)
//! behaves like the same program built whole: calls and function pointers across modules, small
//! callees copied for inlining, globals read and written from different modules, and read-only
//! data and zero-filled defaults used from several places.
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

const SOURCE: &str = r#"#import "Basic";

Big :: struct { a: [512] s64; tag: s64; }

Op :: #type (s64) -> s64;

counter: s64;
history: [16] s64;
primes :: s64.[2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
names :: string.["zero", "one", "two", "three"];
ops: [4] Op;

bump :: (n: s64) -> s64 { counter += n; return counter; }
twice :: (n: s64) -> s64 { return bump(n) + bump(n); }
square :: (n: s64) -> s64 { return n * n; }
negate :: (n: s64) -> s64 { return -n; }
identity :: (n: s64) -> s64 { return n; }

record :: (slot: s64, value: s64) { history[slot % history.count] = value; }

fib :: (n: s64) -> s64 { if n < 2 return n; return fib(n - 1) + fib(n - 2); }

make_big :: (tag: s64) -> Big { b: Big; b.tag = tag; return b; }

sum_big :: (b: Big) -> s64 {
    s := b.tag;
    for b.a s += it;
    return s;
}

apply_all :: (n: s64) -> s64 {
    total := 0;
    for op: ops total += op(n);
    return total;
}

prime_sum :: () -> s64 { s := 0; for primes s += it; return s; }

describe :: (i: s64) -> string { return names[i % names.count]; }

main :: () {
    ops[0] = square; ops[1] = negate; ops[2] = identity; ops[3] = twice;
    print("apply %\n", apply_all(7));
    print("counter %\n", counter);
    for 0..40 record(it, fib(it % 15));
    sum := 0;
    for history sum = sum * 3 + it;
    print("history %\n", sum);
    print("primes %\n", prime_sum());
    print("big %\n", sum_big(make_big(5)));
    print("name %\n", describe(6));
}
"#;

const EXPECTED: &str = "apply 70\ncounter 14\nhistory 34379605\nprimes 129\nbig 5\nname two\n";

fn build_and_run(units: &str, level: &str) -> String {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("optimized-split-{units}{level}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.jai"), SOURCE).unwrap();
    let exe = dir.join(if cfg!(windows) {
        "out.exe"
    } else {
        "out"
    });
    let build = Command::new(JAIC)
        .args(["build", "main.jai", level, "-o"])
        .arg(&exe)
        .env("JAIC_CODEGEN_UNITS", units)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = Command::new(&exe).output().unwrap();
    assert!(run.status.success());
    String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n")
}

#[test]
fn modules_divided_before_the_optimizer_match_a_whole_build() {
    let whole = build_and_run("1", "-O2");
    assert_eq!(whole, EXPECTED);
    assert_eq!(build_and_run("1", "-O0"), EXPECTED);
    for units in ["2", "3", "5"] {
        assert_eq!(build_and_run(units, "-O2"), whole, "{units} modules, -O2");
    }
    assert_eq!(build_and_run("3", "-O1"), whole, "3 modules, -O1");
}
