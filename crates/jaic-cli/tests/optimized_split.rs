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

/// A driver whose callees (each called once) would together outgrow the optimizer's reach
/// (`partition::inline_cuts`): the largest are kept out of it, and the program still computes the
/// same result as an unoptimized build.
fn driver_source() -> String {
    let mut src = String::from("#import \"Basic\";\n\n");
    for k in 0..60 {
        src += &format!("step_{k} :: (x: s64) -> s64 {{\n    for 0..3 {{\n");
        for j in 0..40 {
            src += &format!("        x = x * 31 + {};\n", k * 40 + j);
        }
        src += "    }\n    return x;\n}\n\n";
    }
    src += "drive :: (x: s64) -> s64 {\n";
    for k in 0..60 {
        src += &format!("    x = step_{k}(x) % 1000003;\n");
    }
    src += "    return x;\n}\n\nmain :: () { print(\"%\\n\", drive(7)); }\n";
    src
}

#[test]
fn a_function_that_would_absorb_its_callees_keeps_the_largest_out() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("optimized-inline-cap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.jai"), driver_source()).unwrap();
    let build = |level: &str, exe: &str, ir: Option<&str>| {
        let mut cmd = Command::new(JAIC);
        cmd.args(["build", "main.jai", level, "-o", exe]);
        if let Some(ir) = ir {
            cmd.args(["--emit-ir", ir]);
        }
        let out = cmd.current_dir(&dir).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let run = Command::new(dir.join(exe)).output().unwrap();
        assert!(run.status.success());
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n")
    };
    let unoptimized = build("-O0", "o0", Some("o0.ll"));
    let optimized = build("-O2", "o2", Some("o2.ll"));
    assert_eq!(optimized, unoptimized);
    // `noinline` sits in an attribute group (`attributes #3 = { ... }`) that a definition names.
    let kept_out = |file: &str| {
        let text = std::fs::read_to_string(dir.join(file)).unwrap();
        let groups: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("attributes #") && l.contains("noinline"))
            .filter_map(|l| l.strip_prefix("attributes ")?.split(' ').next())
            .collect();
        text.lines()
            .filter(|l| l.starts_with("define") && l.contains("@step_"))
            .filter(|l| groups.iter().any(|g| l.contains(&format!(" {g} "))))
            .count()
    };
    assert_eq!(kept_out("o0.ll"), 0);
    assert!(kept_out("o2.ll") > 0);
}
