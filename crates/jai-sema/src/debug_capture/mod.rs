//! Pair source provenance with IR at the point each statement and block is emitted.
use crate::{Diagnostic, Resolver, Span, Symbol};
use jai_ir::{
    BlockPath, CleanupId, DebugPathStep, DebugPolicy, DebugSourceLocation, DebugSources,
    LocalDeclaration, LocalId, LocalSource, ProcedureId, ProcedureSource, StatementPath,
};
use jai_source::{SourceId, SourceSpan};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ScopeToken(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StatementToken(usize);

#[derive(Default)]
pub(crate) struct Capture {
    source: Option<SourceId>,
    caller_origins: Vec<SourceSpan>,
    policy: DebugPolicy,
    next_scope: usize,
    next_statement: usize,
    blocks: Vec<BlockCapture>,
    statements: Vec<StatementCapture>,
    completed_block: Option<BlockCapture>,
    completed_statement: Option<StatementCapture>,
    cleanups: HashMap<CleanupId, (ScopeToken, BlockCapture)>,
    locals: HashMap<LocalId, CapturedLocal>,
}
struct BlockCapture {
    scope: ScopeToken,
    location: Option<DebugSourceLocation>,
    statements: Vec<(usize, StatementCapture)>,
}
struct StatementCapture {
    token: StatementToken,
    location: Option<DebugSourceLocation>,
    blocks: Vec<(Box<[DebugPathStep]>, BlockCapture)>,
    replacement: Option<Box<StatementCapture>>,
    prefix: Vec<(usize, StatementCapture, LocalId)>,
    generated: Vec<Box<[DebugPathStep]>>,
}
struct CapturedLocal {
    name: String,
    location: DebugSourceLocation,
    scope: ScopeToken,
    declaration: CapturedDeclaration,
}
enum CapturedDeclaration {
    Parameter(usize),
    Statement {
        token: StatementToken,
        relative: Box<[DebugPathStep]>,
    },
}

mod emission;
mod publication;
mod resolver;
