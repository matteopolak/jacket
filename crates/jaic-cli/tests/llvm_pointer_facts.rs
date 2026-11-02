//! `nonnull` facts must follow ABI parameter expansion and leave user pointers unrestricted.
use std::process::Command;
use test_support::TempDir;

const SOURCE: &str = r#"
#import "Basic";
Big :: struct { a, b, c: s64; }
consume_copy :: (value: Big, raw: *s64) -> s64 #c_call { return value.a; }
trusted_inputs :: (value: Big, raw: *s64) -> Big { return value; }
mutate_c :: (value: Big, raw: *s64) -> s64 #c_call { raw.* = 99; return value.a; }
mutate_jai :: (value: Big, raw: *s64) -> s64 { raw.* = 77; return value.a; }
main :: () {
    b: Big = .{1, 2, 3};
    result := trusted_inputs(b, null);
    n := consume_copy(result, null);
    assert(n == 1);
    assert(mutate_c(b, *b.a) == 1);
    assert(b.a == 99);
    assert(mutate_jai(b, *b.a) == 99);
    assert(b.a == 77);
}

"#;

#[test]
fn pointer_facts_respect_abi_mapping_and_nullable_user_pointers() {
    let dir = TempDir::new("llvm-pointer-facts");
    std::fs::write(dir.join("main.jai"), SOURCE).unwrap();
    // AArch64 passes this struct indirectly; SysV x86-64 uses byval instead.
    // Cross builds may lack a linker, but emit_ir is written before linking.
    for target in ["aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .args([
                "main.jai", "-target", target, "-emit_ir", "main.ll", "-exe", "main",
            ])
            .current_dir(&dir)
            .output()
            .unwrap();
        let ir = std::fs::read_to_string(dir.join("main.ll")).unwrap_or_else(|e| {
            panic!("{target}: {e}: {}", String::from_utf8_lossy(&output.stderr))
        });
        let definition = |name: &str| {
            ir.lines()
                .find(|line| line.starts_with("define ") && line.contains(name))
                .unwrap()
        };
        let jai = definition("@trusted_inputs.");
        assert!(
            jai.contains("(ptr nonnull %0, ptr nonnull %1, ptr %2, ptr nonnull %3)"),
            "{target}: {jai}"
        );
        assert!(
            !jai.contains("noalias"),
            "Jai's incoming aggregate can alias: {jai}"
        );
        assert!(!jai.contains("noundef"), "{jai}");
    }
}
