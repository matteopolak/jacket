//! Opaque code addresses retain capability and publication boundaries in the VM.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Error, Limits, NoEffects, Outcome};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "jai-procedure-address-safety-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
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
fn outcome(body: &str) -> Outcome {
    let source = format!(
        r#"
        Increment::#type (value:s32)->s32;
        increment::(value:s32)->s32{{return value+1;}}
        main::()->int {{
            original:Any=increment;
            address:=(cast(**void)original.value_pointer).*;
            {body}
        }}
    "#
    );
    let fixture = Fixture::new(&source);
    jai_vm::execute(&fixture.program().unwrap(), Limits::default()).outcome
}

#[test]
fn opaque_code_cell_cannot_be_recovered_with_another_signature() {
    let result = outcome(
        r#"
        Other::#type (value:s64)->s64;
        copy:Any=address;
        wrong:=(cast(*Other)copy.value_pointer).*;
        return cast(int)wrong(41);
    "#,
    );
    assert!(
        matches!(result, Outcome::Failed(Error::TypeMismatch { .. })),
        "{result:?}"
    );
}

#[test]
fn opaque_code_address_cannot_be_read_as_data() {
    let result = outcome("return cast(int)(cast(*u8)address).*;");
    assert!(
        matches!(
            result,
            Outcome::Failed(Error::UnsupportedPointerOperation(_))
        ),
        "{result:?}"
    );
}

#[test]
fn opaque_code_address_cannot_be_offset_into_invented_storage() {
    let result = outcome("next:=address+1; if next==address return 1; return 42;");
    assert!(
        matches!(
            result,
            Outcome::Failed(Error::UnsupportedPointerOperation(_))
        ),
        "{result:?}"
    );
}

#[test]
fn transformed_code_bits_do_not_create_another_code_receipt() {
    let result = outcome(
        "bits:=cast(u64)address; changed:=cast(*void)(bits+1); if changed==null return 1; return 42;",
    );
    assert!(
        matches!(
            result,
            Outcome::Failed(Error::UnsupportedPointerOperation(_))
        ),
        "{result:?}"
    );
}

#[test]
fn code_address_bits_cannot_be_published_as_a_portable_run_result() {
    let fixture = Fixture::new(
        r#"
        increment::(value:s32)->s32{return value+1;}
        expose::()->u64 {
            boxed:Any=increment;
            address:=(cast(**void)boxed.value_pointer).*;
            return cast(u64)address;
        }
        answer::#run expose();
        main::()->int{return 42;}
    "#,
    );
    let error = fixture.program().unwrap_err();
    assert!(
        error.contains("address-derived integer cannot be published as a native constant"),
        "{error}"
    );
}
