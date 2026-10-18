//! Procedures: signatures, IR function/foreign targets, polymorphic
//! instantiation and body lowering.
mod abi;
mod drain;
mod func;
mod lowering;
mod model;
mod signature;

pub(super) use self::func::llvm_intrinsic;
pub use self::model::{BodyState, ParamInfo, ProcInfo, ProcTarget, Signature, has_poly};
pub(super) use self::model::{for_poly_binders, validate_c_variadic_header};
