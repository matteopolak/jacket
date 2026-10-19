//! The `__jaic_*` primitives declared in the stdlib and the `MetaOp` table must agree.
use jaic::build::MetaOp;
use std::collections::BTreeSet;
use std::path::Path;

/// Names of the `name :: (...) ... #compiler;` declarations whose name starts with `__jaic_`.
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
fn every_declared_primitive_is_a_meta_op() {
    let declared = declared_primitives();
    assert!(declared.len() > 40, "found only {declared:?}");
    for name in &declared {
        assert!(
            MetaOp::from_name(name).is_some(),
            "`{name}` is declared #compiler in the stdlib but is not a MetaOp"
        );
    }
}

#[test]
fn every_meta_op_is_declared_in_the_stdlib() {
    let declared = declared_primitives();
    for op in MetaOp::ALL {
        assert!(
            declared.contains(op.name()) || op.name() == "__jaic_current_workspace",
            "{op:?} (`{}`) is not declared in the stdlib",
            op.name()
        );
    }
}
