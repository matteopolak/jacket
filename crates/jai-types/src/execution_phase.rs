//! Source body execution policy, independent of its calling convention and ABI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProcedureExecution {
    #[default]
    RuntimeAndCompileTime,
    CompileTimeOnly,
}
