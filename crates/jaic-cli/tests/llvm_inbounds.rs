//! Only addressing backed by a live allocation may promise LLVM inbounds.
use std::process::Command;
use test_support::TempDir;

const SOURCE: &str = r#"
#import "Basic";
Pair :: struct { a, b: s64; }
Choice :: union { pair: Pair; words: [2]s64; }
field_address :: () -> s64 { p: Pair; return cast(s64) *p.b - cast(s64) *p; }
union_address :: () -> s64 { p: Choice; return cast(s64) *p.pair.b - cast(s64) *p; }
array_address :: (i: s64) -> s64 { a: [4]Pair; return cast(s64) *a[i].b - cast(s64) *a; }
raw_add :: (p: *s64, i: s64) -> *s64 { return p + i; }
raw_sub :: (p: *s64, i: s64) -> *s64 { return p - i; }
raw_index :: (p: *s64, i: s64) -> *s64 { return *p[i]; }
nullable_field :: (p: *Pair) -> *s64 { return *p.b; }
integer_field :: (p: u64) -> *s64 { return *(cast(*Pair) p).b; }
null_offset :: () -> u64 { return cast(u64) *(cast(*Pair) null).b; }
integer_offset :: () -> u64 { return cast(u64) *(cast(*Pair) 16).b; }
view_address :: (a: []s64, i: s64) -> *s64 { return *a[i]; }
string_address :: (a: string, i: s64) -> *u8 { return *a[i]; }
unchecked_array :: (i: s64) -> s64 #no_abc { a: [4]Pair; return cast(s64) *a[i] - cast(s64) *a; }
#run {
    assert(field_address() == 8);
    assert(union_address() == 8);
    assert(array_address(2) == 40);
    assert(null_offset() == 8);
    assert(integer_offset() == 24);
    assert(cast(u64) nullable_field(null) == 8);
};
main :: () {
    assert(field_address() == 8);
    assert(union_address() == 8);
    assert(array_address(2) == 40);
    assert(null_offset() == 8);
    assert(integer_offset() == 24);
    assert(cast(u64) nullable_field(null) == 8);
    assert(cast(u64) integer_field(0) == 8);
    assert(cast(u64) raw_add(null, 3) == 24);
    assert(cast(u64) raw_sub(cast(*s64) 32, 3) == 8);
    assert(cast(u64) raw_index(null, 3) == 24);
    assert(unchecked_array(2) == 32);
    a: [2]s64;
    view: []s64 = a;
    assert(view_address(view, 1) == *a[1]);
    s := "ab";
    assert(string_address(s, 1).* == #char "b");
}
"#;

#[test]
fn allocation_provenance_controls_llvm_inbounds() {
    let dir = TempDir::new("llvm-inbounds-ir");
    std::fs::write(dir.join("main.jai"), SOURCE).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .args(["main.jai", "-O0", "-emit_ir", "main.ll", "-exe", "main"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = std::fs::read_to_string(dir.join("main.ll")).unwrap();
    let body = |name: &str| {
        let start = ir
            .lines()
            .find(|line| line.starts_with("define ") && line.contains(&format!("@{name}.")))
            .unwrap();
        ir[ir.find(start).unwrap()..].split("\n}").next().unwrap()
    };
    for name in ["field_address", "union_address", "array_address"] {
        assert!(
            body(name).contains("getelementptr inbounds i8"),
            "{name}: {}",
            body(name)
        );
    }
    for name in [
        "raw_add",
        "raw_sub",
        "raw_index",
        "nullable_field",
        "integer_field",
        "null_offset",
        "integer_offset",
        "unchecked_array",
    ] {
        assert!(
            !body(name).contains("getelementptr inbounds"),
            "{name}: {}",
            body(name)
        );
    }
    // Container metadata is a local copy, but the loaded data pointer has no allocation proof.
    for name in ["view_address", "string_address"] {
        assert!(
            body(name)
                .lines()
                .any(|line| line.contains("getelementptr i8") && !line.contains("inbounds")),
            "{name}: {}",
            body(name)
        );
    }
}

#[test]
fn address_only_null_and_integer_offsets_execute_at_o0_and_o2() {
    let dir = TempDir::new("llvm-inbounds-run");
    std::fs::write(dir.join("main.jai"), SOURCE).unwrap();
    for opt in ["-O0", "-O2"] {
        let exe = dir.join(format!("main{}", std::env::consts::EXE_SUFFIX));
        let build = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .args(["main.jai", opt, "-exe"])
            .arg(&exe)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{opt}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let run = Command::new(&exe).output().unwrap();
        assert!(
            run.status.success(),
            "{opt}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}
