//! Compile-time proof that the syntax tree is `Send + Sync`.
use super::{
    AsmBlock, Case, CodeBody, Decl, EnumLit, Expr, File, Import, Note, ProcHeader, ProcLit,
    RunBody, Stmt, StructLit,
};

/// The syntax tree is shared by reference between threads (parse-ahead workers hand finished trees
/// to the checker, and checking workers read the declarations of a frozen compiler). A node with a
/// `Cell`, `RefCell` or `Rc` in it stops compiling here.
#[allow(dead_code)]
fn assert_sync<T: Send + Sync>() {
}

#[allow(dead_code)]
fn tree_is_sync() {
    assert_sync::<File>();
    assert_sync::<Stmt>();
    assert_sync::<Expr>();
    assert_sync::<Decl>();
    assert_sync::<ProcLit>();
    assert_sync::<ProcHeader>();
    assert_sync::<StructLit>();
    assert_sync::<EnumLit>();
    assert_sync::<CodeBody>();
    assert_sync::<RunBody>();
    assert_sync::<AsmBlock>();
    assert_sync::<Import>();
    assert_sync::<Case>();
    assert_sync::<Note>();
}
