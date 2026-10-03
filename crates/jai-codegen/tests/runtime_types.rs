//! Runtime descriptor relocations are tied to the selected target layout.
use jai_types::{LayoutPolicy, ScalarLayout};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Source(std::path::PathBuf);
impl Source {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-native-type-policy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(
            path.join("main.jai"),
            "main::()->int{value:Type=*s32;if value==null return 1;return 42;}",
        )
        .unwrap();
        Self(path)
    }
    fn program(&self, layout: LayoutPolicy) -> jai_ir::Program {
        let graph = jai_modules::ModuleGraph::load(
            &self.0.join("main.jai"),
            jai_modules::GraphOptions::default(),
        )
        .unwrap();
        jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(layout),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap()
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn reflected_types_require_matching_selected_native_target_data() {
    let source = Source::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let native = target.layout_policy().unwrap();
    let context = jai_codegen::Context::create();
    let program = source.program(native);
    jai_codegen::lower(&context, &program)
        .unwrap()
        .verify()
        .unwrap();
    jai_codegen::lower_for_target(&context, &program, &target)
        .unwrap()
        .verify()
        .unwrap();
    let other = if native.pointer().size == 8 {
        LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap()
    } else {
        LayoutPolicy::lp64()
    };
    let other_program = source.program(other);
    let error = jai_codegen::lower_for_target(&context, &other_program, &target).unwrap_err();
    assert!(matches!(
        error,
        jai_codegen::Error::Type(jai_codegen::types::Error::DescriptorTargetMismatch { .. })
    ));
}
