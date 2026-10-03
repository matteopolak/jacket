//! Readonly pointer-literal checks execute only in the bounded VM.
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

fn source(text: &str) -> Result<jai_ir::Program, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-c-string-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let input = fixture.0.join("main.jai");
    fs::write(&input, text).unwrap();
    let graph = ModuleGraph::load(&input, GraphOptions::default()).unwrap();
    jai_sema::resolve_graph(&graph).map_err(|error| error.to_string())
}

#[test]
fn literal_pointer_storage_remains_readonly_in_the_vm() {
    for text in [
        "main::(){bytes:*u8=\"A\";bytes[0]=42;}",
        "main::(){bytes:*u8=\"A\";erased:*void=bytes;restored:*u8=xx erased;restored[0]=42;}",
    ] {
        let program = source(text).unwrap();
        assert_eq!(
            jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
            jai_vm::Outcome::Failed(jai_vm::Error::ReadOnlyStorage)
        );
    }
}

#[test]
fn ordinary_string_values_do_not_implicitly_become_c_pointers() {
    assert!(
        source("main::(){value:string=\"A\";bytes:*u8=value;}")
            .unwrap_err()
            .contains("pointee")
    );
    assert!(
        source("main::(){bytes:*int=\"A\";}")
            .unwrap_err()
            .contains("pointee")
    );
}
