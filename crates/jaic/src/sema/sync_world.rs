//! Compile-time proof that the values a frozen `&Compiler` hands to checking workers are `Send +
//! Sync`. Parallel body checking (docs/compiler/parallel-sema.md) reads these from several threads;
//! a `Cell`, `RefCell` or `Rc` added to one of them stops compiling here instead of racing later.
use super::{Value, procs::Signature};

#[allow(dead_code)]
fn assert_sync<T: Send + Sync>() {
}

#[allow(dead_code)]
fn world_is_sync() {
    assert_sync::<Value>();
    assert_sync::<super::value::Aggregate>();
    assert_sync::<Signature>();
    assert_sync::<super::procs::ParamInfo>();
    assert_sync::<super::ProcTypeParams>();
    assert_sync::<super::procs::ProcInfo>();
    assert_sync::<super::scope::Scope>();
    assert_sync::<super::scope::Entity>();
    assert_sync::<crate::types::Types>();
    assert_sync::<crate::types::TypeKind>();
}

#[allow(dead_code)]
fn state_is_sync() {
    assert_sync::<super::Namespaces>();
    assert_sync::<super::ProcState>();
    assert_sync::<super::TypeState>();
    assert_sync::<super::Output>();
}
