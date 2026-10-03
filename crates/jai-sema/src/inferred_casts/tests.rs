use super::*;
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

fn source(source: &str) -> Result<Program, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-inferred-cast-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let input = fixture.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph =
        ModuleGraph::load(&input, GraphOptions::default()).map_err(|error| error.to_string())?;
    resolve_graph(&graph).map_err(|error| error.to_string())
}

fn answer(text: &str) -> i128 {
    let program = source(text).unwrap();
    let jai_vm::Outcome::Complete(values) =
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome
    else {
        panic!("contextual cast did not complete");
    };
    values[0].integer().unwrap().value()
}

#[test]
fn an_inferred_cast_cannot_create_its_own_destination_type() {
    assert!(
        source("main :: () { value := xx 3; }")
            .unwrap_err()
            .contains("destination type")
    );
    assert!(
        source("main :: () { value := xx,no_check 3; }")
            .unwrap_err()
            .contains("destination type")
    );
}

#[test]
fn truncation_rejects_unestablished_domains() {
    for text in [
        "main::(){value:=cast,trunc(bool)2;}",
        "main::(){value:=cast,trunc(float32)2;}",
        "main::(){value:=cast,trunc(u8)42.5;}",
        "main::(){value:=cast,trunc(u8)true;}",
        "main::(){value:=cast,trunc(*void)int;}",
        "Weight::#type,distinct float32; main::(){value:Weight=xx,trunc 2;}",
        "value:bool=xx,trunc 2; main::(){}",
        "take::(value:float32=xx,trunc 2){} main::(){}",
    ] {
        assert!(source(text).unwrap_err().contains("trunc"), "{text}");
    }
    source("sentinel:*void=xx,trunc -1; main::(){}")
        .expect("native sentinel defaults retain their target-dependent address cast");
}

#[test]
fn checked_cast_failures_remain_runtime_checks() {
    let program = source("main :: ()->int { value:u8 = xx 300; return cast(int)value; }").unwrap();
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::CheckedCast)
    );
}

#[test]
fn defaults_accept_the_modern_inline_enum_cast_form() {
    assert_eq!(
        answer(
            "take :: (kind:enum u8 { FIRST::1; SECOND::2; } = xx 3)->int { return cast(int)kind; } main :: ()->int { return take()+39; }"
        ),
        42
    );
}

#[test]
fn conditional_and_compound_operands_receive_context() {
    assert_eq!(
        answer(
            "Bits :: enum_flags u8 { A::1; B::2; } main :: ()->int { value:u8 = ifx true then xx 39 else xx,no_check 300; flags:Bits = .A; flags |= xx 2; return cast(int)value+cast(int)flags; }"
        ),
        42
    );
    assert_eq!(
        answer(
            "main :: ()->int { values:[2]int = .[0,42]; pointer := *values[0]; offset:u8 = 1; pointer += xx offset; return pointer.*; }"
        ),
        42
    );
}

#[test]
fn strong_binary_peers_supply_context_without_defaulting_weak_values() {
    assert_eq!(
        answer(
            "main :: ()->int { value:u8 = 40; source:int = 2; result := value + xx source; return cast(int)result; }"
        ),
        42
    );
    assert!(
        source("main :: () { value := 1 + xx 2; }")
            .unwrap_err()
            .contains("destination type")
    );
}

#[test]
fn implicit_pointer_erasure_is_directional() {
    assert_eq!(
        answer(
            "main :: ()->int { value := 42; erased:*void = *value; restored:*int = xx erased; return restored.*; }"
        ),
        42
    );
    assert!(
        source("main :: () { value := 42; erased:*void = *value; restored:*int = erased; }")
            .unwrap_err()
            .contains("pointee")
    );
}
