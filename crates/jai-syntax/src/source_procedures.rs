//! Shared source header and body payload; identity belongs to the enclosing declaration.
use super::*;
use jai_types::{CallingConvention, ContextMode, DebugPolicy, InlineHint, ProcedureExecution};

#[derive(Clone, Debug)]
pub struct CallableHeaderSyntax {
    pub deprecation: Option<Deprecation>,
    pub notes: Vec<NoteSyntax>,
    pub parameters: Vec<Parameter>,
    pub results: Vec<ProcedureResult>,
    pub convention: CallingConvention,
    pub context: ContextMode,
}

#[derive(Clone, Debug)]
pub struct SourceProcedureHeader {
    pub callable: CallableHeaderSyntax,
    pub checks: SafetyChecks,
    pub inline_hint: InlineHint,
    pub execution: ProcedureExecution,
    pub debug: DebugPolicy,
    pub compiler: Option<CompilerProcedure>,
    pub expands: bool,
    pub modify: Option<ModifyDirective>,
}

#[derive(Clone, Debug)]
pub struct SourceProcedureSyntax {
    pub header: SourceProcedureHeader,
    pub body: Vec<Statement>,
    pub span: Span,
}

impl std::ops::Deref for SourceProcedureHeader {
    type Target = CallableHeaderSyntax;
    fn deref(&self) -> &Self::Target {
        &self.callable
    }
}
impl std::ops::DerefMut for SourceProcedureHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.callable
    }
}
impl std::ops::Deref for SourceProcedureSyntax {
    type Target = SourceProcedureHeader;
    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
impl std::ops::DerefMut for SourceProcedureSyntax {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.header
    }
}
