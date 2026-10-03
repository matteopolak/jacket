//! Original lookup failures are source demands only inside an admitted job attempt.
use super::*;
use jai_modules::LookupError;
use jai_source::DiagnosticMarker;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Demand {
    pub(super) consumer: DeclarationId,
    pub(super) file: FileInstanceId,
    pub(super) location: SourceSpan,
    pub(super) root: Symbol,
    pub(super) error: LookupError,
}
impl Demand {
    pub(super) fn diagnostic(self, graph: &ModuleGraph) -> LocatedDiagnostic {
        let message = match self.error {
            LookupError::UnknownName(name) => {
                format!("unknown name '{}'", graph.symbols().name(name))
            }
            LookupError::UnknownMember { name, .. } => {
                format!("unknown module member '{}'", graph.symbols().name(name))
            }
            _ => "lookup does not describe an unavailable source binding".into(),
        };
        LocatedDiagnostic {
            location: self.location,
            message,
        }
    }
}

#[derive(Clone)]
struct Origin {
    declaration: DeclarationId,
    file: FileInstanceId,
    location: SourceSpan,
    source_bytes: Arc<str>,
}
struct Proof {
    marker: DiagnosticMarker,
    demand: Demand,
    path: NamePath,
    origin: Origin,
}
struct Frame {
    origin: Origin,
    failures: Vec<Proof>,
}
#[derive(Default)]
pub(super) struct Requests {
    active: RefCell<Option<Frame>>,
    retained: RefCell<Vec<Proof>>,
}
pub(super) struct Attempt<'a> {
    requests: &'a Requests,
    previous: Option<Frame>,
    finished: bool,
}
impl Requests {
    /// The controller supplies the real selected initializer or body owner.
    pub(super) fn begin<'a>(
        &'a self,
        graph: &ModuleGraph,
        consumer: DeclarationId,
        file: FileInstanceId,
        location: SourceSpan,
    ) -> Result<Attempt<'a>, LocatedDiagnostic> {
        let source = graph
            .declaration(consumer)
            .ok_or_else(|| LocatedDiagnostic {
                location,
                message: "source lookup owner is absent from the retained graph".into(),
            })?;
        if source.file() != file || source.location().source != location.source {
            return Err(LocatedDiagnostic {
                location,
                message: "source lookup owner changed its defining identity".into(),
            });
        }
        let source_bytes = graph
            .sources()
            .get(location.source)
            .ok_or_else(|| LocatedDiagnostic {
                location,
                message: "source lookup immutable source is absent".into(),
            })?
            .shared_text();
        let origin = Origin {
            declaration: consumer,
            file,
            location,
            source_bytes,
        };
        let previous = self.active.replace(Some(Frame {
            origin,
            failures: vec![],
        }));
        Ok(Attempt {
            requests: self,
            previous,
            finished: false,
        })
    }

    pub(super) fn record(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
        error: LookupError,
    ) -> Option<DiagnosticMarker> {
        if !matches!(
            error,
            LookupError::UnknownName(_) | LookupError::UnknownMember { .. }
        ) {
            return None;
        }
        let mut active = self.active.borrow_mut();
        let frame = active.as_mut()?;
        let location = graph.locate(file, span)?;
        if location.source != frame.origin.location.source
            || span.start < frame.origin.location.span.start
            || span.end > frame.origin.location.span.end
        {
            return None;
        }
        let marker = DiagnosticMarker::new();
        frame.failures.push(Proof {
            marker: marker.clone(),
            demand: Demand {
                consumer: frame.origin.declaration,
                file,
                location,
                root: path.root,
                error,
            },
            path: path.clone(),
            origin: frame.origin.clone(),
        });
        Some(marker)
    }
}
impl Attempt<'_> {
    /// Successful alternatives do not request source publication. An error must
    /// retain the exact lookup occurrence before this typed demand is admitted.
    pub(super) fn finish<T>(
        mut self,
        graph: &ModuleGraph,
        result: &Result<T, Diagnostic>,
    ) -> Option<Demand> {
        let frame = self.requests.active.replace(self.previous.take());
        self.finished = true;
        let frame = frame?;
        // A retry replaces this owner's current demand; it never adds a poll
        // history. Successful or hard-failed binding retires the old proof too.
        self.requests.retained.borrow_mut().retain(|proof| {
            proof.origin.declaration != frame.origin.declaration
                || proof.origin.file != frame.origin.file
                || proof.origin.location != frame.origin.location
        });
        let Err(error) = result else {
            return None;
        };
        let proof = frame.failures.into_iter().find(|proof| {
            error.marker() == Some(&proof.marker)
                && proof.demand.location.span == error.span
                && proof.demand.location.source
                    == error.source.unwrap_or(frame.origin.location.source)
        })?;
        // Preserve the actual graph lookup, source allocation, and declaration
        // proof. No generated name or future declaration identity is invented.
        let source = graph.sources().get(proof.origin.location.source)?;
        if !Arc::ptr_eq(&source.shared_text(), &proof.origin.source_bytes)
            || graph.declaration(proof.origin.declaration)?.file() != proof.origin.file
            || graph.lookup(proof.demand.file, &proof.path) != Err(proof.demand.error)
        {
            return None;
        }
        let demand = proof.demand;
        self.requests.retained.borrow_mut().push(proof);
        Some(demand)
    }
}
impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.requests.active.replace(self.previous.take());
        }
    }
}

impl FileScope<'_> {
    /// Demand the actual checked direct callee before beginning an initializer
    /// VM transaction. Its source body can then be prepared independently.
    pub(crate) fn initializer_call_dependency(
        &self,
        expression: &crate::Expr,
        context: &crate::compile_time::Context<'_>,
        meta: &crate::reflection::MetaContext,
    ) -> Option<ProcedureId> {
        if self.declarations.source_lookup.active.borrow().is_none() {
            return None;
        }
        let procedure = direct_call(expression)?;
        if context.procedures.contains_key(&procedure)
            || meta.local_declarations.ready_procedure(procedure).is_some()
            || self
                .declarations
                .generics
                .borrow()
                .callback_body_ready(procedure)
                == Some(true)
        {
            return None;
        }
        let declaration = self
            .declarations
            .signatures
            .iter()
            .find_map(|(&id, signature)| (signature.id == procedure).then_some(id))
            .or_else(|| {
                self.declarations
                    .generics
                    .borrow()
                    .callback_source_origin(procedure)
                    .map(|(declaration, _)| declaration)
            })?;
        matches!(
            self.declarations
                .graph
                .declaration(declaration)?
                .syntax()
                .kind,
            FileDeclarationKind::Procedure(_)
        )
        .then_some(procedure)
    }
}
fn direct_call(expression: &crate::Expr) -> Option<ProcedureId> {
    match expression {
        crate::Expr::Void(call) => Some(call.procedure),
        crate::Expr::Int(value) => int_call(value),
        crate::Expr::Bool(jai_ir::BoolExpr::Call(call)) => Some(call.procedure),
        crate::Expr::Float(value) => match value.kind() {
            jai_ir::FloatExprKind::Call(call) => Some(call.procedure),
            _ => None,
        },
        crate::Expr::Typed { value, .. }
        | crate::Expr::Pointer { value, .. }
        | crate::Expr::Enum { value, .. } => value_call(value),
        _ => None,
    }
}
fn int_call(expression: &jai_ir::IntExpr) -> Option<ProcedureId> {
    match expression.kind() {
        jai_ir::IntExprKind::Call(call) => Some(call.procedure),
        jai_ir::IntExprKind::Value(value) | jai_ir::IntExprKind::EnumValue(value) => {
            value_call(value)
        }
        jai_ir::IntExprKind::Cast(_, value) => int_call(value),
        _ => None,
    }
}
fn value_call(expression: &jai_ir::ValueExpr) -> Option<ProcedureId> {
    match expression {
        jai_ir::ValueExpr::Call { call, .. } => Some(call.procedure),
        jai_ir::ValueExpr::Int(value) => int_call(value),
        jai_ir::ValueExpr::Bool(jai_ir::BoolExpr::Call(call)) => Some(call.procedure),
        jai_ir::ValueExpr::Distinct { value, .. }
        | jai_ir::ValueExpr::UnwrapDistinct { value, .. }
        | jai_ir::ValueExpr::PointerCast { value, .. }
        | jai_ir::ValueExpr::Bind { body: value, .. } => value_call(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};
    use std::path::Path;

    #[test]
    fn only_the_escaping_lookup_receipt_admits_its_actual_source_occurrence() {
        let source = "value:int=#run generated(); main::()->int{return value;}";
        let path = Path::new("/source-lookup-receipt/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay.insert(path, source.as_bytes().to_vec()).unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let owner = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == "value")
            .unwrap();
        let path = NamePath {
            root: graph.symbols().find("generated").unwrap(),
            members: vec![],
        };
        let start = source.find("generated").unwrap();
        let span = Span::new(start, start + "generated".len());
        let lookup = graph.lookup(owner.file(), &path).unwrap_err();
        let requests = Requests::default();
        let attempt = requests
            .begin(&graph, owner.id(), owner.file(), owner.location())
            .unwrap();
        let marker = requests
            .record(&graph, owner.file(), &path, span, lookup)
            .unwrap();
        let error = Diagnostic::new(span, "actual lookup error")
            .with_marker(marker)
            .with_fallback_source(owner.location().source);
        let result: Result<(), Diagnostic> = Err(error.clone());
        let demand = attempt.finish(&graph, &result).unwrap();
        assert_eq!(demand.consumer, owner.id());
        assert_eq!(demand.error, lookup);
        for _ in 0..3 {
            let attempt = requests
                .begin(&graph, owner.id(), owner.file(), owner.location())
                .unwrap();
            let marker = requests
                .record(&graph, owner.file(), &path, span, lookup)
                .unwrap();
            let retry: Result<(), Diagnostic> =
                Err(Diagnostic::new(span, "actual lookup retry").with_marker(marker));
            assert!(attempt.finish(&graph, &retry).is_some());
            assert_eq!(requests.retained.borrow().len(), 1);
        }

        let attempt = requests
            .begin(&graph, owner.id(), owner.file(), owner.location())
            .unwrap();
        let _probe = requests
            .record(&graph, owner.file(), &path, span, lookup)
            .unwrap();
        // The probe may have resolved through a successful builtin alternative.
        // A later new error at identical coordinates carries no lookup receipt.
        let mismatch: Result<(), Diagnostic> =
            Err(Diagnostic::new(span, "type mismatch")
                .with_fallback_source(owner.location().source));
        assert!(attempt.finish(&graph, &mismatch).is_none());
        assert!(requests.retained.borrow().is_empty());

        let attempt = requests
            .begin(&graph, owner.id(), owner.file(), owner.location())
            .unwrap();
        let _new_probe = requests
            .record(&graph, owner.file(), &path, span, lookup)
            .unwrap();
        // A receipt from an earlier attempt cannot authorize this occurrence.
        assert!(attempt.finish(&graph, &result).is_none());
    }
}
