//! The `#compiler` primitives the stdlib declares itself: the `__jaic_syntax_*` reads behind
//! `Code_Node` extension helpers.
use super::meta_op::MetaOp;
use super::state::SharedWorkspaces;
use super::{SyntaxKey, Workspaces};
use crate::interp::{Interp, Rets, Trap};
use crate::records::FieldRef;

/// Execute a primitive. `args` follow the IR calling convention: the context pointer (when
/// `has_context`), then the parameters, then result addresses for memory-typed results. Every
/// primitive takes (node, key) and the string read also an output address.
pub fn call(
    shared: &SharedWorkspaces,
    op: MetaOp,
    has_context: bool,
    args: &[u64],
    interp: &mut Interp,
) -> Result<Rets, Trap> {
    // Hooks reach the compiler, which only the thread holding the baton may touch.
    #[cfg(not(target_arch = "wasm32"))]
    debug_assert!(interp.holds_baton(), "{op:?} called without the baton");
    let args = &args[usize::from(has_context)..];
    let arg = |i: usize| {
        *args
            .get(i)
            .expect("invariant: a primitive reads only the arguments its declaration passes")
    };
    let key =
        SyntaxKey::from_raw(arg(1)).ok_or_else(|| Trap::new("internal: unknown syntax key"))?;
    let mut reg = shared.borrow_mut();
    // Association lookup only: never dereference arbitrary program pointers.
    let found = reg
        .builders
        .iter()
        .enumerate()
        .find_map(|(index, builder)| {
            let id = builder.record_of(arg(0));
            (id != 0).then_some((index, id))
        });
    let (int, string, child) = match found.and_then(|(_, id)| reg.records.syntax_field(id, key)) {
        Some(FieldRef::Int(value)) => (value, None, 0),
        Some(FieldRef::Str(value)) => (0, Some(value.clone()), 0),
        Some(FieldRef::Ref(child)) => (0, None, child),
        _ => (0, None, 0),
    };
    match op {
        MetaOp::SyntaxInt => Ok(Rets::one(int as u64)),
        MetaOp::SyntaxString => {
            let value = string.unwrap_or_default();
            store(interp, arg(2), &(value.len() as u64).to_le_bytes())?;
            store(interp, arg(2) + 8, &(value.as_ptr() as u64).to_le_bytes())?;
            reg.kept.push(value);
            Ok(Rets::default())
        }
        MetaOp::SyntaxChild => {
            let Some((index, _)) = found else {
                return Ok(Rets::one(0));
            };
            let Workspaces {
                records,
                kept,
                builders,
                ..
            } = &mut *reg;
            Ok(Rets::one(builders[index].build(
                records,
                kept,
                interp,
                child,
                arg(2),
            )?))
        }
    }
}

fn store(interp: &mut Interp, p: u64, bytes: &[u8]) -> Result<(), Trap> {
    if interp.write(p, bytes) {
        Ok(())
    } else {
        Err(Trap::unwritable(p, bytes.len()))
    }
}
