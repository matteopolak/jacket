//! Lazy insertion replacements preserve definition capture and caller loop ownership.
use super::*;
use crate::{Block, Flow};

#[derive(Clone)]
struct Replacement {
    kind: syntax::JumpKind,
    code: CodeValueId,
    assertion: bool,
}

pub(super) struct ReplacementFrame {
    procedure: jai_ir::ProcedureId,
    loop_depth: usize,
    iterator: Option<Symbol>,
    replacements: Vec<Replacement>,
}

impl Resolver<'_> {
    pub(super) fn reject_nonstatement_replacements(
        &self,
        directive: &syntax::InsertDirective,
    ) -> Result<(), Diagnostic> {
        if !directive.replacements.is_empty() {
            return Err(Diagnostic::new(
                directive.span,
                "loop-control insertion replacements require statement insertion",
            ));
        }
        Ok(())
    }

    pub(super) fn capture_loop_replacements(
        &mut self,
        directive: &syntax::InsertDirective,
    ) -> Result<ReplacementFrame, Diagnostic> {
        let mut replacements = Vec::with_capacity(directive.replacements.len());
        for replacement in &directive.replacements {
            let (body, assertion, span) = match &replacement.body {
                syntax::LoopControlReplacementBody::Code(body) => {
                    (body.clone(), false, replacement.span)
                }
                syntax::LoopControlReplacementBody::Assert {
                    condition,
                    span,
                } => (syntax::CodeBody::Expression(condition.clone()), true, *span),
            };
            let Expr::Code(code) = self.capture_code(&body, span)? else {
                unreachable!("source capture produces Code")
            };
            replacements.push(Replacement {
                kind: replacement.kind,
                code,
                assertion,
            });
        }
        // The alias belongs to the current typed export invocation. A same-named
        // iterator introduced by caller code shadows it through loop identity.
        let iterator = self.symbols.find("it").and_then(|it| {
            self.meta.codes.exports.iter().rev().find_map(|frame| {
                frame
                    .names
                    .contains(&it)
                    .then(|| frame.remap.get(&it).copied().unwrap_or(it))
            })
        });
        Ok(ReplacementFrame {
            procedure: self.procedure,
            loop_depth: self.loops.len(),
            iterator,
            replacements,
        })
    }

    pub(crate) fn resolve_loop_replacement(
        &mut self,
        kind: syntax::JumpKind,
        target: syntax::LoopTarget,
        span: Span,
    ) -> Result<Option<Statement>, Diagnostic> {
        // Macro source and replacement source operate on their own physical
        // loops. Only an inserted caller quotation participates in substitution.
        if !self
            .meta
            .codes
            .return_regions
            .last()
            .is_some_and(|&(owner, quoted)| owner == self.procedure && quoted)
        {
            return Ok(None);
        }
        let replacement = self.meta.codes.replacements.iter().rev().find_map(|frame| {
            if frame.procedure != self.procedure
                || (frame.loop_depth == 0 && frame.iterator.is_none())
            {
                return None;
            }
            let applies = match target {
                syntax::LoopTarget::Innermost => self.loops.len() == frame.loop_depth,
                syntax::LoopTarget::Named(name) => {
                    frame.iterator == Some(name)
                        && !self.loops.iter().enumerate().rev().any(|(index, active)| {
                            index >= frame.loop_depth && active.name == Some(name)
                        })
                }
            };
            if !applies {
                return None;
            }
            frame
                .replacements
                .iter()
                .find(|replacement| replacement.kind == kind)
                .cloned()
                .map(|replacement| (frame.loop_depth, replacement))
        });
        let Some((loop_depth, replacement)) = replacement else {
            return Ok(None);
        };
        if self
            .cleanup_context
            .is_some_and(|context| loop_depth <= context.loop_depth)
        {
            return Err(Diagnostic::new(
                span,
                "a deferred body cannot replace an enclosing loop-control statement",
            ));
        }
        self.resolve_replacement_body(&replacement, loop_depth, span)
            .map(Some)
    }

    fn resolve_replacement_body(
        &mut self,
        replacement: &Replacement,
        loop_depth: usize,
        trigger: Span,
    ) -> Result<Statement, Diagnostic> {
        let code = Arc::clone(self.meta.codes.get(replacement.code).ok_or_else(|| {
            Diagnostic::new(
                trigger,
                "replacement code belongs to another semantic context",
            )
        })?);
        let capture = code
            .scope
            .as_ref()
            .expect("replacement source has a capture");
        self.with_definition_scope(capture.file, capture.substitution.as_ref(), |resolver| {
            resolver.resolve_captured_replacement_body(&code, replacement, loop_depth, trigger)
        })
    }

    fn resolve_captured_replacement_body(
        &mut self,
        code: &CapturedCode,
        replacement: &Replacement,
        loop_depth: usize,
        trigger: Span,
    ) -> Result<Statement, Diagnostic> {
        let capture = code
            .scope
            .as_ref()
            .expect("replacement source has a capture");
        self.meta.codes.enter(replacement.code, trigger)?;
        let original_source = self.debug.replace_source(Some(capture.location.source));
        self.meta.codes.source_files.push(capture.source_file);
        let original_callers = self.debug.replace_caller_origins(Vec::new());
        let original_debug_policy = self.debug.enter_policy(capture.debug);
        let original_span = std::mem::replace(&mut self.span, capture.location.span);
        let original_checks = std::mem::replace(&mut self.checks, capture.checks);
        let original_scopes = std::mem::replace(&mut self.scopes, capture.frames.clone());
        let original_file = self.graph_scope;
        let mut original_locals = self.install_captured_local_scopes(code);
        self.meta.codes.return_regions.push((self.procedure, false));
        // Physical targets come from the insertion directive's loop prefix.
        // Caller loops still contribute their cleanup scopes, but a caller loop
        // named `row` must not capture source `break row` in a replacement.
        let caller_loops = self.loops.split_off(loop_depth);
        let result = (|| {
            if replacement.assertion {
                let syntax::CodeBody::Expression(condition) = &code.body else {
                    unreachable!("assertion capture retains its condition")
                };
                if !self.compile_time_condition(condition)? {
                    return Err(Diagnostic::new(
                        capture.location.span,
                        "loop-control insertion replacement assertion failed",
                    ));
                }
                return Ok(Statement::Block(Block {
                    statements: Vec::new(),
                    flow: Flow::FallsThrough,
                }));
            }
            let result = match &code.body {
                syntax::CodeBody::Block(statements) => self.block(statements, true),
                syntax::CodeBody::Statement(statement) => {
                    self.block(std::slice::from_ref(statement), true)
                }
                syntax::CodeBody::Expression(expression) => self.block(
                    &[syntax::Statement::new(
                        expression.span,
                        syntax::StatementKind::Expression((**expression).clone()),
                    )],
                    true,
                ),
                syntax::CodeBody::Null => unreachable!("replacement always retains source syntax"),
            }?;
            self.debug
                .attach_block(&[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::Block)]);
            Ok(Statement::Block(result))
        })()
        .map_err(|error: Diagnostic| error.with_fallback_source(capture.location.source));
        self.loops.truncate(loop_depth);
        self.loops.extend(caller_loops);
        self.meta.codes.return_regions.pop();
        self.scopes = original_scopes;
        original_locals.resume_after_expansion(&self.local_scopes);
        self.local_scopes = original_locals;
        self.graph_scope = original_file;
        self.checks = original_checks;
        self.debug.replace_source(original_source);
        self.meta.codes.source_files.pop();
        self.debug.replace_caller_origins(original_callers);
        self.debug.restore_policy(original_debug_policy);
        self.span = original_span;
        self.meta.codes.leave(replacement.code);
        result
    }
}
