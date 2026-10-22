//! Only the retained syntax extensions expose private compiler hooks to Jai.
use jaic::{build::MetaOp, interp::Hook};
use std::collections::BTreeSet;
use std::path::Path;

fn declared_primitives() -> BTreeSet<String> {
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let files = jaic::paths::jai_files_under(&stdlib, &mut |_| false);
    let mut names = BTreeSet::new();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        for line in text.lines() {
            let Some((name, rest)) = line.split_once(" :: ") else {
                continue;
            };
            if name.starts_with("__jaic_") && rest.trim_end().ends_with("#compiler;") {
                names.insert(name.to_string());
            }
        }
    }
    names
}

#[test]
fn only_syntax_metadata_extensions_are_declared_and_bindable() {
    let declared = declared_primitives();
    assert_eq!(
        declared,
        [
            "__jaic_syntax_child",
            "__jaic_syntax_int",
            "__jaic_syntax_string"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    for op in MetaOp::ALL {
        assert_eq!(
            Hook::for_compiler_proc(op.name(), false).is_some(),
            declared.contains(op.name()),
            "{}",
            op.name()
        );
    }
}
