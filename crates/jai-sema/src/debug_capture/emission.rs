//! Resolve emission positions and preserve source origins while constructing IR.
use super::*;

impl Capture {
    pub(crate) fn new(source: Option<SourceId>) -> Self {
        Self {
            source,
            ..Self::default()
        }
    }
    /// A code capture's origin is independent of the scope used for name lookup.
    pub(crate) fn source(&self) -> Option<SourceId> {
        self.source
    }
    /// Actual invocation sites, ordered from the outermost to the current expansion.
    pub(crate) fn caller_origins(&self) -> &[SourceSpan] {
        &self.caller_origins
    }
    /// The immediate caller origin is independent of AST provenance.
    pub(crate) fn caller_origin(&self) -> Option<SourceSpan> {
        self.caller_origins().last().copied()
    }
    pub(crate) fn replace_caller_origins(&mut self, origins: Vec<SourceSpan>) -> Vec<SourceSpan> {
        std::mem::replace(&mut self.caller_origins, origins)
    }
    pub(crate) fn push_caller_origin(&mut self, origin: SourceSpan) -> usize {
        let previous_depth = self.caller_origins.len();
        self.caller_origins.push(origin);
        previous_depth
    }
    pub(crate) fn restore_caller_origins(&mut self, previous_depth: usize) {
        self.caller_origins.truncate(previous_depth);
    }
    pub(crate) fn policy(&self) -> DebugPolicy {
        self.policy
    }
    pub(crate) fn enter_policy(&mut self, declared: DebugPolicy) -> DebugPolicy {
        let previous = self.policy;
        self.policy = previous.nested(declared);
        previous
    }
    pub(crate) fn restore_policy(&mut self, previous: DebugPolicy) {
        self.policy = previous;
    }
    pub(crate) fn replace_source(&mut self, source: Option<SourceId>) -> Option<SourceId> {
        std::mem::replace(&mut self.source, source)
    }
    pub(crate) fn begin_block(&mut self, location: Option<DebugSourceLocation>) {
        let scope = ScopeToken(self.next_scope);
        self.next_scope += 1;
        self.blocks.push(BlockCapture {
            scope,
            location,
            statements: vec![],
        });
    }
    pub(crate) fn finish_block(&mut self) {
        self.completed_block = self.blocks.pop();
    }
    pub(crate) fn begin_statement(&mut self, location: Option<DebugSourceLocation>) {
        let token = StatementToken(self.next_statement);
        self.next_statement += 1;
        self.statements.push(StatementCapture {
            token,
            location,
            blocks: vec![],
            replacement: None,
            prefix: vec![],
            generated: vec![],
        });
    }
    pub(crate) fn finish_statement(&mut self, emitted: Option<&jai_ir::Statement>) {
        let mut statement = self.statements.pop().expect("active source statement");
        if let Some(replacement) = statement.replacement.take() {
            // An expression insertion returns the quoted IR node unchanged.
            statement.token = replacement.token;
            statement.location = replacement.location.clone();
            statement.blocks.extend(replacement.blocks);
        }
        if statement.location.is_some()
            && let Some(emitted) = emitted
        {
            statement.generated = generated_paths(emitted, &statement.blocks);
        }
        self.completed_statement = Some(statement);
    }
    pub(crate) fn emit_statement(&mut self, index: usize) {
        if let Some(statement) = self.completed_statement.take() {
            self.blocks
                .last_mut()
                .expect("emitted statement has a block")
                .statements
                .push((index, statement));
        }
    }
    /// Attach a resolved block at its exact position in its final parent IR node.
    pub(crate) fn attach_block(&mut self, relative: &[DebugPathStep]) {
        if let Some(block) = self.completed_block.take() {
            self.statements
                .last_mut()
                .expect("nested block has a source statement")
                .blocks
                .push((relative.into(), block));
        }
    }
    /// A generated IR block can contain an original declaration without introducing
    /// a new source lexical scope. Keep its token at its actual emitted child path.
    pub(crate) fn attach_completed_statement_in_block(
        &mut self,
        relative: &[DebugPathStep],
        index: usize,
    ) {
        let Some(statement) = self.completed_statement.take() else {
            return;
        };
        let scope = ScopeToken(self.next_scope);
        self.next_scope += 1;
        let parent = self
            .statements
            .last_mut()
            .expect("generated block has an owning source statement");
        // The named declaration belongs to the wrapper's surrounding lexical
        // scope. Its emitted child retains its stepping source; the declaration
        // anchor is the actual owning wrapper statement, which contains that child.
        for local in self.locals.values_mut() {
            if let CapturedDeclaration::Statement {
                token,
                relative,
            } = &mut local.declaration
                && *token == statement.token
                && relative.is_empty()
            {
                *token = parent.token;
            }
        }
        parent.blocks.push((
            relative.into(),
            BlockCapture {
                scope,
                location: parent.location.clone(),
                statements: vec![(index, statement)],
            },
        ));
    }
    /// Preserve a quoted expression's source when insertion returns its node directly.
    pub(crate) fn forward_statement(&mut self) {
        let statement = self.completed_statement.take();
        self.statements
            .last_mut()
            .expect("forwarded source statement")
            .replacement = statement.map(Box::new);
    }
    /// The helper has inserted these actual IR nodes before the captured body.
    pub(crate) fn prepend_block(&mut self, count: usize) {
        if let Some(block) = &mut self.completed_block {
            for (index, _) in &mut block.statements {
                *index += count;
            }
            if let Some(statement) = self.statements.last_mut() {
                for (index, prefix, local) in statement.prefix.drain(..) {
                    if self.policy.emits() {
                        self.locals
                            .get_mut(&local)
                            .expect("captured prefix local")
                            .scope = block.scope;
                    } else {
                        // Argument expressions were captured before entering
                        // the macro body; retain their stepping locations.
                        self.locals.remove(&local);
                    }
                    block.statements.push((index, prefix));
                }
            }
            if let Some(location) = &block.location {
                for index in 0..count {
                    if block
                        .statements
                        .iter()
                        .any(|(emitted, _)| *emitted == index)
                    {
                        continue;
                    }
                    let token = StatementToken(self.next_statement);
                    self.next_statement += 1;
                    block.statements.push((
                        index,
                        StatementCapture {
                            token,
                            location: Some(location.clone()),
                            blocks: vec![],
                            replacement: None,
                            prefix: vec![],
                            generated: vec![],
                        },
                    ));
                }
            }
        }
    }
    pub(crate) fn prefix_local(
        &mut self,
        id: LocalId,
        name: String,
        declaration: DebugSourceLocation,
        initializer: DebugSourceLocation,
        index: usize,
    ) {
        let token = StatementToken(self.next_statement);
        self.next_statement += 1;
        self.locals.insert(
            id,
            CapturedLocal {
                name,
                location: declaration,
                scope: ScopeToken(0),
                declaration: CapturedDeclaration::Statement {
                    token,
                    relative: Box::new([]),
                },
            },
        );
        self.statements
            .last_mut()
            .expect("macro has a source statement")
            .prefix
            .push((
                index,
                StatementCapture {
                    token,
                    location: Some(initializer),
                    blocks: vec![],
                    replacement: None,
                    prefix: vec![],
                    generated: vec![],
                },
                id,
            ));
    }
    pub(crate) fn cleanup(&mut self, id: CleanupId) {
        if let Some(block) = self.completed_block.take() {
            let parent = self
                .blocks
                .last()
                .map_or(ScopeToken(0), |block| block.scope);
            self.cleanups.insert(id, (parent, block));
        }
    }
    pub(crate) fn emit_cleanup(&mut self, id: CleanupId, index: usize) {
        let Some(location) = self
            .cleanups
            .get(&id)
            .and_then(|(_, block)| block.location.clone())
        else {
            return;
        };
        self.begin_statement(Some(location));
        self.finish_statement(None);
        self.emit_statement(index);
    }
    pub(crate) fn generated_cleanup(
        &mut self,
        id: CleanupId,
        emitted: &jai_ir::Block,
        location: Option<DebugSourceLocation>,
    ) {
        let Some(parent) = self.completed_block.as_ref().map(|block| block.scope) else {
            return;
        };
        let scope = ScopeToken(self.next_scope);
        self.next_scope += 1;
        let statements = emitted
            .statements
            .iter()
            .enumerate()
            .map(|(index, emitted)| {
                let token = StatementToken(self.next_statement);
                self.next_statement += 1;
                (
                    index,
                    StatementCapture {
                        token,
                        location: location.clone(),
                        blocks: vec![],
                        replacement: None,
                        prefix: vec![],
                        generated: generated_paths(emitted, &[]),
                    },
                )
            })
            .collect();
        self.cleanups.insert(
            id,
            (
                parent,
                BlockCapture {
                    scope,
                    location,
                    statements,
                },
            ),
        );
    }
    pub(crate) fn append_cleanup(&mut self, id: CleanupId, index: usize) {
        let location = self
            .cleanups
            .get(&id)
            .and_then(|(_, block)| block.location.clone());
        let Some(location) = location else {
            return;
        };
        let token = StatementToken(self.next_statement);
        self.next_statement += 1;
        if let Some(block) = &mut self.completed_block {
            block.statements.push((
                index,
                StatementCapture {
                    token,
                    location: Some(location),
                    blocks: vec![],
                    replacement: None,
                    prefix: vec![],
                    generated: vec![],
                },
            ));
        }
    }
    /// Result locals point to their actual initializer in the completed body.
    pub(crate) fn named_result_local(
        &mut self,
        id: LocalId,
        name: String,
        location: DebugSourceLocation,
        index: usize,
    ) {
        let Some(block) = self.completed_block.as_mut() else {
            return;
        };
        let token = if let Some((_, statement)) =
            block.statements.iter_mut().find(|(at, _)| *at == index)
        {
            statement.location = Some(location.clone());
            statement.token
        } else {
            let token = StatementToken(self.next_statement);
            self.next_statement += 1;
            block.statements.push((
                index,
                StatementCapture {
                    token,
                    location: Some(location.clone()),
                    blocks: vec![],
                    replacement: None,
                    prefix: vec![],
                    generated: vec![],
                },
            ));
            token
        };
        self.locals.insert(
            id,
            CapturedLocal {
                name,
                location,
                scope: block.scope,
                declaration: CapturedDeclaration::Statement {
                    token,
                    relative: Box::new([]),
                },
            },
        );
    }

    pub(crate) fn local(&mut self, id: LocalId, name: String, location: DebugSourceLocation) {
        let scope = self
            .blocks
            .last()
            .map_or(ScopeToken(0), |block| block.scope);
        let Some(statement) = self.statements.last() else {
            return;
        };
        self.locals.insert(
            id,
            CapturedLocal {
                name,
                location,
                scope,
                declaration: CapturedDeclaration::Statement {
                    token: statement.token,
                    relative: Box::new([]),
                },
            },
        );
    }
    pub(crate) fn parameter(
        &mut self,
        id: LocalId,
        name: String,
        location: DebugSourceLocation,
        ordinal: usize,
    ) {
        self.locals.insert(
            id,
            CapturedLocal {
                name,
                location,
                scope: ScopeToken(0),
                declaration: CapturedDeclaration::Parameter(ordinal),
            },
        );
    }
    /// Loop bindings are visible in their emitted body, whose scope is now known.
    pub(crate) fn local_in_completed_block(&mut self, id: LocalId) {
        if let (Some(local), Some(block)) =
            (self.locals.get_mut(&id), self.completed_block.as_ref())
        {
            local.scope = block.scope;
        }
    }
    pub(crate) fn local_in_completed_block_at(&mut self, id: LocalId, relative: &[DebugPathStep]) {
        self.local_in_completed_block(id);
        if let Some(local) = self.locals.get_mut(&id)
            && let CapturedDeclaration::Statement {
                relative: declaration,
                ..
            } = &mut local.declaration
        {
            *declaration = relative.into();
        }
    }
}

/// Walk the IR that this statement actually produced. Explicit child captures
/// supply their own source locations and are not attributed to their parent.
fn generated_paths<'a>(
    emitted: &'a jai_ir::Statement,
    captured: &[(Box<[DebugPathStep]>, BlockCapture)],
) -> Vec<Box<[DebugPathStep]>> {
    use jai_ir::{DebugBranch as Branch, Statement};
    let mut paths = Vec::new();
    let mut pending = vec![(Vec::new(), emitted)];
    while let Some((path, statement)) = pending.pop() {
        if !path.is_empty() {
            paths.push(path.clone().into_boxed_slice());
        }
        let mut child = |branch, block: &'a jai_ir::Block| {
            let mut block_path = path.clone();
            block_path.push(DebugPathStep::Child(branch));
            if captured
                .iter()
                .any(|(captured, _)| captured.as_ref() == block_path.as_slice())
            {
                return;
            }
            for (index, statement) in block.statements.iter().enumerate() {
                let mut statement_path = block_path.clone();
                statement_path.push(DebugPathStep::Statement(index));
                pending.push((statement_path, statement));
            }
        };
        match statement {
            Statement::Block(block) => child(Branch::Block, block),
            Statement::If(_, yes, no) => {
                child(Branch::IfThen, yes);
                child(Branch::IfElse, no);
            }
            Statement::While {
                body, ..
            } => child(Branch::While, body),
            Statement::Range(range) => child(Branch::Range, &range.body),
            Statement::PushContext {
                body, ..
            } => child(Branch::PushContext, body),
            Statement::Cases(cases) => {
                for (index, arm) in cases.arms.iter().enumerate() {
                    child(Branch::CaseArm(index), &arm.body);
                }
                if let Some(default) = &cases.default {
                    child(Branch::CaseDefault, default);
                }
                let mut subject = path;
                subject.push(DebugPathStep::Child(Branch::CaseSubject));
                pending.push((subject, &cases.subject));
            }
            Statement::Simd(_)
            | Statement::IndirectCallResults {
                ..
            }
            | Statement::Store(..)
            | Statement::DiscardValue(..)
            | Statement::CallResults {
                ..
            }
            | Statement::StoreInt(..)
            | Statement::StoreBool(..)
            | Statement::Exit(..)
            | Statement::Cleanup(..)
            | Statement::DiscardInt(..)
            | Statement::DiscardBool(..)
            | Statement::CallVoid(..) => {}
        }
    }
    paths
}
