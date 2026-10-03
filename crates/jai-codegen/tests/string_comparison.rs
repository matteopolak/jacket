//! Compare source execution in the VM and newly emitted native code only.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn check(source: &str, expected: i128) {
    check_outcome(source, Some(expected));
}

fn check_outcome(source: &str, expected: Option<i128>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-string-equality-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&scratch.0).unwrap();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let outcome = jai_vm::execute(&program, Default::default()).outcome;
    match expected {
        Some(expected) => {
            let jai_vm::Outcome::Complete(values) = outcome else {
                panic!("{outcome:?}");
            };
            assert_eq!(values[0].integer().unwrap().value(), expected);
        }
        None => assert!(matches!(outcome, jai_vm::Outcome::Failed(_)), "{outcome:?}"),
    }
    let ir = jai_codegen::emit(&program).unwrap();
    let llvm = scratch.0.join("program.ll");
    let executable = scratch.0.join("program");
    fs::write(&llvm, ir).unwrap();
    let output = native_tools::clang_command()
        .arg(&llvm)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let status = Command::new(executable).status().unwrap();
    match expected {
        Some(expected) => assert_eq!(status.code(), Some(expected as i32)),
        None => assert!(!status.success()),
    }
}

#[test]
fn named_string_content_selects_static_branches_and_runs_at_compile_time() {
    check(
        r#"
        TEXT :: "Hello";
        matches :: (text:string)->bool {return text == "Hello";}
        MATCHED :: #run matches(TEXT);
        main :: ()->int {
            #if TEXT == "Hello" {answer:=42;} else {unavailable();}
            #if TEXT != "Wrong" {extra:=0;} else {unavailable();}
            if !MATCHED return 1;
            if TEXT != "Hello" return 2;
            return answer+extra;
        }
    "#,
        42,
    );
}

#[test]
fn comparison_uses_count_and_all_bytes_including_nul_across_distinct_backing() {
    check(
        r#"
        main :: ()->int {
            a:[3]u8=.[65,0,66]; b:[3]u8=.[65,0,66];
            left:string; left.count=3; left.data=*a[0];
            right:string; right.count=3; right.data=*b[0];
            if left != right return 1;
            if left != "A\0B" return 2;
            if left == "A" return 3;
            if left == "A\0" return 4;
            b[2]=67;
            if left == right return 5;
            if right != "A\0C" return 6;
            empty:string;
            if empty != "" return 7;
            empty.data=*a[0];
            if empty != "" return 9;
            if empty == left return 8;
            return 42;
        }
    "#,
        42,
    );
}

#[test]
fn both_descriptors_are_evaluated_before_backing_bytes_are_inspected() {
    check(
        r#"
        mutate :: (p:*u8)->string {
            p.*=66;
            result:string; result.count=1; result.data=p;
            return result;
        }
        main :: ()->int {
            bytes:[1]u8=.[65];
            text:string; text.count=1; text.data=*bytes[0];
            if text != mutate(*bytes[0]) return 1;
            if text != "B" return 2;
            return 42;
        }
    "#,
        42,
    );
}

#[test]
fn signed_counts_compare_first_and_invalid_equal_count_fails_before_bytes() {
    check(
        r#"main :: ()->int { bad:string; bad.count=-1; if bad=="" return 1; return 42; }"#,
        42,
    );
    check_outcome(
        r#"main :: ()->int { bad:string; bad.count=-1; if bad==bad return 1; return 0; }"#,
        None,
    );
}
