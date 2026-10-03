//! Independently authored source exercises actual byte-storage conversions.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Error, Limits, NoEffects, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "jai-storage-bitcast-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default())
            .map_err(|error| error.to_string())?;
        resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout: Some(LayoutPolicy::lp64()),
                ..ResolveOptions::default()
            },
            &mut NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn outcome(source: &str) -> Outcome {
    let fixture = Fixture::new(source);
    jai_vm::execute(&fixture.program().unwrap(), Limits::default()).outcome
}
fn complete(source: &str) {
    let result = outcome(source);
    assert!(
        matches!(&result, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{result:?}"
    );
}
#[test]
fn signed_and_unsigned_two_word_records_copy_storage_and_evaluate_producer_once() {
    complete(include_str!("fixtures/int128-storage-bitcasts.jai.pending"));
}
#[test]
fn fixed_array_element_bits_and_smaller_record_prefix_keep_storage_strength() {
    complete(include_str!("fixtures/force-array-prefix.jai.pending"));
}
#[test]
fn scalar_float_bit_patterns_are_reinterpreted_instead_of_numerically_cast() {
    complete(
        "main::()->int{ bits:u32=0x42280000; value:=cast,force(float32)bits; recovered:=cast,force(u32)value; if value!=42 || recovered!=bits return 1; return 42; }",
    );
}
#[test]
fn prefix_reads_written_leading_storage_without_initializing_the_tail() {
    complete(
        "Wide::struct{first:u64;tail:u64;} Prefix::struct{first:u64;} main::()->int{wide:Wide=---;wide.first=42;prefix:=cast,FORCE(Prefix)wide;return cast(int)prefix.first;}",
    );
}
#[test]
fn aggregate_storage_carrier_keeps_interior_padding_for_another_cast() {
    complete(
        "Padded::struct{tag:u8;word:u32;} main::()->int{bits:u64=0x11223344aabbcc2a;padded:=cast,force(Padded)bits;recovered:=cast,force(u64)padded;if recovered!=bits return 1;return cast(int)padded.tag;}",
    );
}
#[test]
fn explicit_storage_cast_recovers_only_an_unchanged_pointer_receipt() {
    complete(
        "main::()->int{value:int=42;pointer:=*value;bits:=cast,force(u64)pointer;recovered:=cast,force(*int)bits;if recovered!=pointer return 1;return recovered.*;}",
    );
}
#[test]
fn invalid_boolean_and_unsealed_nonnull_pointer_views_have_precise_vm_boundaries() {
    for (source, reason) in [
        (
            "main::()->int{byte:u8=2;value:=cast,force(bool)byte;if value return 1;return 42;}",
            "storage cast contains an invalid boolean representation",
        ),
        (
            "main::()->int{bits:u64=42;pointer:=cast,force(*int)bits;if pointer==null return 1;return 42;}",
            "storage cast cannot forge pointer provenance",
        ),
    ] {
        let result = outcome(source);
        assert!(
            matches!(result, Outcome::Failed(Error::UnsupportedPointerOperation(actual)) if actual==reason),
            "{result:?}"
        );
    }
}
#[test]
fn written_target_extent_is_required_and_lowercase_never_reads_a_smaller_prefix() {
    let result = outcome(
        "Wide::struct{first:u64;tail:u64;} main::()->int{wide:Wide=---;wide.first=42;copy:=cast,force([2]u64)wide;return cast(int)copy[0];}",
    );
    assert!(
        matches!(result, Outcome::Failed(Error::Uninitialized)),
        "{result:?}"
    );
    for (spelling, source, target) in [
        ("force", "u64", "u32"),
        ("force", "u32", "u64"),
        ("FORCE", "u32", "u64"),
    ] {
        let fixture = Fixture::new(&format!(
            "main::()->int{{value:{source}=42;copy:=cast,{spelling}({target})value;return cast(int)copy;}}"
        ));
        let error = fixture.program().unwrap_err();
        assert!(error.contains("storage cast requires"), "{error}");
    }
}

#[test]
fn declaration_default_storage_cast_does_not_fall_back_to_record_or_numeric_coercion() {
    let fixture = Fixture::new(
        "Word::struct{bits:u32;} take::(word:Word=cast,force(Word)cast(u32)42)->int{return cast(int)word.bits;} main::()->int{return take();}",
    );
    let error = fixture.program().unwrap_err();
    assert!(
        error.contains(
            "storage cast declaration defaults require target-layout VM constant materialization"
        ),
        "{error}"
    );
}
