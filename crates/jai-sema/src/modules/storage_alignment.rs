//! Bind global allocation annotations in their defining source scope.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct Job {
    pub declaration: DeclarationId,
    pub file: FileInstanceId,
    pub global: jai_ir::GlobalId,
    pub syntax: syntax::Declaration,
}

impl Job {
    pub(super) fn annotation_span(&self) -> Span {
        self.syntax
            .attributes()
            .first()
            .map(|attribute| match attribute {
                syntax::DeclarationAttribute::Alignment(expression) => expression.span,
            })
            .unwrap_or_default()
    }

    pub(super) fn location(&self, graph: &ModuleGraph) -> jai_source::SourceSpan {
        let source = graph
            .declaration(self.declaration)
            .expect("alignment jobs retain their defining declaration")
            .location()
            .source;
        jai_source::SourceSpan {
            source,
            span: self.annotation_span(),
        }
    }
}

pub(super) fn declaration(
    job: &Job,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    options: &crate::ResolveOptions,
    context: &crate::compile_time::Context<'_>,
    places: &mut jai_ir::PlaceRegistry,
) -> Result<Option<u32>, Diagnostic> {
    if job.syntax.attributes().is_empty() {
        return Ok(None);
    }
    let signatures = HashMap::new();
    let globals = HashMap::new();
    let mut resolver = Resolver {
        expression_owner: Some(context.owner),
        debug: crate::debug_capture::Capture::default(),
        checks: crate::safety_checks::ActiveChecks::default(),
        local_scopes: crate::local_declarations::LocalScopes::default(),
        context: declarations.context.as_ref(),
        context_available: true,
        meta,
        procedure: context.owner,
        types,
        target_layout: options.effective_layout(),
        places,
        signatures: &signatures,
        globals: &globals,
        graph_scope: Some(FileScope {
            declarations,
            file: job.file,
            substitution: None,
        }),
        compile_time: Some(context),
        symbols: declarations.graph.symbols(),
        scopes: vec![HashMap::new()],
        locals: vec![],
        span: job.annotation_span(),
        results: &[],
        loops: vec![],
        next_loop: 0,
        cleanups: vec![],
        active_push: None,
        next_push: 0,
        deferred_scopes: vec![],
        cleanup_context: None,
    };
    resolver.declaration_alignment(&job.syntax)
}
