//! A checked optional formal retains its original source until the selected value is ready.
use crate::ParameterDefault;
use jai_ir::ProcedureId;
use jai_modules::FileInstanceId;
use jai_source::{DeclarationId, Diagnostic, SourceSpan};
use jai_types::TypeId;
use std::cell::{Cell, OnceCell};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceParameterDefaultKey {
    pub(crate) declaration: DeclarationId,
    pub(crate) file: FileInstanceId,
    pub(crate) parameter: usize,
    pub(crate) expected: TypeId,
    pub(crate) procedure: ProcedureId,
    pub(crate) location: SourceSpan,
}

impl std::hash::Hash for SourceParameterDefaultKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        self.declaration.hash(state);
        self.file.hash(state);
        self.parameter.hash(state);
        self.expected.hash(state);
        self.procedure.hash(state);
        self.location.source.hash(state);
        self.location.span.start.hash(state);
        self.location.span.end.hash(state);
    }
}

pub(crate) struct SourceParameterDefault {
    pub(crate) key: SourceParameterDefaultKey,
    pub(crate) expression: jai_syntax::Expression,
    pub(crate) source_bytes: Arc<str>,
    ready: OnceCell<ParameterDefault>,
    requests: Cell<usize>,
}
impl SourceParameterDefault {
    pub(crate) fn new(
        key: SourceParameterDefaultKey,
        expression: jai_syntax::Expression,
        source_bytes: Arc<str>,
    ) -> Rc<Self> {
        Rc::new(Self {
            key,
            expression,
            source_bytes,
            ready: OnceCell::new(),
            requests: Cell::new(0),
        })
    }
    pub(crate) fn ready(&self) -> Option<&ParameterDefault> {
        self.ready.get()
    }
    pub(crate) fn requests(&self) -> usize {
        self.requests.get()
    }
    pub(crate) fn require(&self) -> Result<&ParameterDefault, Diagnostic> {
        if let Some(value) = self.ready() {
            return Ok(value);
        }
        let count = self.requests.get().checked_add(1).ok_or_else(|| {
            Diagnostic::at_source(self.key.location, "source default request limit exceeded")
        })?;
        self.requests.set(count);
        Err(Diagnostic::at_source(
            self.key.location,
            "selected parameter default is pending its original checked source job",
        ))
    }
    pub(crate) fn publish(&self, value: ParameterDefault) -> Result<(), Diagnostic> {
        let exact = match &value {
            ParameterDefault::Constant(value) => value.ty == self.key.expected,
            ParameterDefault::RuntimeRead(read) => read.ty() == self.key.expected,
            ParameterDefault::CodeNull {
                ty,
            } => *ty == self.key.expected,
            ParameterDefault::CallerLocation | ParameterDefault::Discarded => true,
            ParameterDefault::Source(_) => false,
        };
        if !exact {
            return Err(Diagnostic::at_source(
                self.key.location,
                "checked source default changed its actual formal type",
            ));
        }
        if let Some(old) = self.ready() {
            if crate::procedure_values::contracts::same_parameter_default(Some(old), Some(&value)) {
                return Ok(());
            }
            return Err(Diagnostic::at_source(
                self.key.location,
                "checked source default changed its published value policy",
            ));
        }
        self.ready.set(value).map_err(|_| {
            Diagnostic::at_source(
                self.key.location,
                "source default publication is already occupied",
            )
        })
    }
}

impl ParameterDefault {
    /// Read published metadata without demanding an unrelated source initializer.
    pub(crate) fn prepared(&self) -> Option<&Self> {
        match self {
            Self::Source(source) => source.ready(),
            value => Some(value),
        }
    }
}
