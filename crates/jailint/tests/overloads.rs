//! Concrete generic instances created during overload probing are not lint roots.
use std::{path::PathBuf, process::Command};

fn audit(name: &str, source: &str) -> std::process::Output {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("overloads")
        .join(name);
    std::fs::create_dir_all(dir.join("stdlib/prelude")).unwrap();
    std::fs::create_dir_all(dir.join("stdlib/Runtime_Support")).unwrap();
    std::fs::write(dir.join("stdlib/prelude/module.jai"), "").unwrap();
    std::fs::write(dir.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);").unwrap();
    std::fs::write(dir.join("main.jai"), source).unwrap();
    Command::new(env!("CARGO_BIN_EXE_jailint"))
        .current_dir(&dir)
        .env("JAIC_STDLIB", dir.join("stdlib"))
        .args(["-D", "warnings", "main.jai"])
        .output()
        .unwrap()
}

const OVERLOADS: &str = r#"
A :: struct { n:int; }
B :: struct { n:int; }
route :: (b:*B, value:$T) { #assert T == int "selected instance must fail"; b.n=value; }
route :: (a:*A, $handler:$H) { handler(); a.n=7; }
handler :: () {}
"#;

#[test]
fn rejected_instances_do_not_create_body_obligations() {
    let out = audit(
        "rejected",
        &format!("{OVERLOADS} main :: () {{ a:A; route(*a,handler); }}"),
    );
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn later_selected_cached_instance_still_checks_its_body() {
    let out = audit(
        "selected",
        &format!("{OVERLOADS} main :: () {{ a:A; route(*a,handler); b:B; route(*b,handler); }}"),
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("selected instance must fail"));
}

#[test]
fn unreferenced_concrete_declarations_remain_audit_roots() {
    let out = audit(
        "unreferenced",
        "unused :: () { missing_in_unused_body(); } main :: () {}",
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("missing_in_unused_body"));
}

#[test]
fn viable_but_less_preferred_instance_is_not_an_audit_root() {
    let out = audit(
        "less-preferred",
        "pick :: (x:$T) -> int { #assert false \"unselected viable generic\"; return 0; } pick :: (x:int) -> int { return x; } main :: () { #assert pick(7)==7; }",
    );
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
