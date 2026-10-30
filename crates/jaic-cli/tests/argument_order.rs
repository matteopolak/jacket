//! Arguments are evaluated left to right: a variable read as an argument has the value it had
//! before the calls among the arguments to its right ran, at run time and at compile time.
use std::process::Command;
use test_support::TempDir;

const PROBE: &str = r#"#import "Basic";

Pt :: struct { a: int; b: int; }

f :: (a: int, b: int) { print("f a=% b=%\n", a, b); }
g :: (a: int, b: int, c: int) { print("g a=% b=% c=%\n", a, b, c); }
h :: (p: Pt, b: int) { print("h p=(%,%) b=%\n", p.a, p.b, b); }
s :: (text: string, b: int) { print("s text=% b=%\n", text, b); }

set2 :: (p: *int) -> int { p.* = 2; return 0; }
setpt :: (p: *Pt) -> int { p.a = 9; p.b = 9; return 0; }
bump :: (p: *int) -> int { p.* += 1; return p.*; }
retext :: (p: *string) -> int { p.* = "after"; return 0; }

probe :: () {
    x := 1;
    f(x, set2(*x));

    pt := Pt.{1, 2};
    f(pt.a, setpt(*pt));

    p := New(int);
    p.* = 1;
    f(p.*, set2(p));

    y := 1;
    g(bump(*y), y, bump(*y));

    z := 1;
    f(bump(*z), bump(*z));

    q := Pt.{1, 2};
    h(q, setpt(*q));

    text := "before";
    s(text, retext(*text));

    w := 1;
    print("% % %\n", w, set2(*w), w);

    k := 1;
    f(k + 0, set2(*k));
}

main :: () { probe(); }
"#;

const EXPECTED: &str = "f a=1 b=0\nf a=1 b=0\nf a=1 b=0\ng a=2 b=2 c=3\nf a=2 b=3\nh p=(1,2) b=0\ns text=before b=0\n1 0 2\nf a=1 b=0\n";

fn source(name: &str, extra: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new("argument-order");
    let file = dir.join(format!("{name}.jai"));
    std::fs::write(&file, format!("{PROBE}{extra}")).unwrap();
    (dir, file)
}

#[test]
fn interpreted_calls_evaluate_arguments_left_to_right() {
    let (_dir, file) = source("interp", "");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-interp")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), EXPECTED);
}

#[test]
fn compile_time_calls_evaluate_arguments_left_to_right() {
    // The `#run` prints while compiling; the program itself runs afterwards.
    let (_dir, file) = source("run", "#run probe();\n");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-interp")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{EXPECTED}{EXPECTED}")
    );
}

#[cfg(feature = "llvm")]
#[test]
fn native_calls_evaluate_arguments_left_to_right() {
    let (_dir, file) = source("native", "");
    let exe = file.with_file_name(format!("native_exe{}", std::env::consts::EXE_SUFFIX));
    for level in ["-O0", "-O2"] {
        let build = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&file)
            .args([level, "-exe"])
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let run = Command::new(&exe).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&run.stdout), EXPECTED, "{level}");
    }
}
