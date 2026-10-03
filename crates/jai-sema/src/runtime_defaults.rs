//! Checked definition-site storage reads, evaluated only for omitted arguments.
use crate::*;
use jai_source::{DeclarationId, SourceSpan};
use jai_types::FieldId;
mod ready;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DefaultReadRoot {
    Global {
        declaration: DeclarationId,
        ty: TypeId,
    },
    Context {
        ty: TypeId,
    },
}
impl DefaultReadRoot {
    pub(crate) fn ty(self) -> TypeId {
        match self {
            Self::Global {
                ty, ..
            }
            | Self::Context {
                ty,
            } => ty,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DefaultReadStep {
    Dereference(TypeId),
    Field(FieldId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDefaultRead {
    root: DefaultReadRoot,
    steps: Box<[DefaultReadStep]>,
    ty: TypeId,
    source: SourceSpan,
}

pub(crate) type RuntimeDefaultBindings = HashMap<ParameterId, RuntimeDefaultRead>;
pub(crate) type BoundDefaultArguments = (Vec<(ParameterId, ValueExpr)>, RuntimeDefaultBindings);
pub(crate) type DirectCallWithDefaults = (Signature, Call, RuntimeDefaultBindings);
pub(crate) type IndirectCallWithDefaults = (
    ValueExpr,
    Vec<(ParameterId, ValueExpr)>,
    Vec<ResultSignature>,
    RuntimeDefaultBindings,
);

impl RuntimeDefaultRead {
    pub(crate) fn checked(
        root: DefaultReadRoot,
        steps: Vec<DefaultReadStep>,
        ty: TypeId,
        source: SourceSpan,
        types: &TypeRegistry,
    ) -> Result<Self, Diagnostic> {
        types
            .kind(root.ty())
            .map_err(|error| Diagnostic::at_source(source, error.to_string()))?;
        let mut current = root.ty();
        for step in &steps {
            current = match *step {
                DefaultReadStep::Field(field) => types
                    .validate_field(current, field)
                    .map_err(|error| Diagnostic::at_source(source, error.to_string()))?,
                DefaultReadStep::Dereference(expected) => {
                    if !matches!(types.kind(current), Ok(jai_types::TypeKind::Pointer(pointee)) if *pointee == expected)
                    {
                        return Err(Diagnostic::at_source(
                            source,
                            "runtime default has an invalid pointer projection",
                        ));
                    }
                    expected
                }
            };
        }
        if current != ty {
            return Err(Diagnostic::at_source(
                source,
                "runtime default differs from its declared parameter type",
            ));
        }
        Ok(Self {
            root,
            steps: steps.into_boxed_slice(),
            ty,
            source,
        })
    }
    pub fn ty(&self) -> TypeId {
        self.ty
    }
    pub(crate) fn root(&self) -> DefaultReadRoot {
        self.root
    }
    pub(crate) fn steps(&self) -> &[DefaultReadStep] {
        &self.steps
    }
    pub fn source(&self) -> SourceSpan {
        self.source
    }
    pub(crate) fn is_context(&self) -> bool {
        matches!(self.root, DefaultReadRoot::Context { .. })
    }
}

impl Resolver<'_> {
    /// Called in a local declaration's retained definition environment after
    /// type-only header preparation. Eligible paths emit loads, never execute them.
    pub(crate) fn runtime_parameter_default(
        &mut self,
        expression: &syntax::Expression,
        expected: TypeId,
    ) -> Result<Option<RuntimeDefaultRead>, Diagnostic> {
        let mut path = expression;
        let mut depth = 0;
        loop {
            if depth >= 256 {
                return Err(Diagnostic::new(
                    expression.span,
                    "runtime default exceeds storage path depth limit",
                ));
            }
            depth += 1;
            match &path.kind {
                syntax::ExpressionKind::Member {
                    base, ..
                } => path = base,
                syntax::ExpressionKind::Name(_)
                | syntax::ExpressionKind::QualifiedName(_)
                | syntax::ExpressionKind::Context => break,
                _ => return Ok(None),
            }
        }
        let value = self.expr(expression)?;
        if matches!(
            value,
            Expr::Code(_) | Expr::Type(_) | Expr::Null | Expr::Literal(_) | Expr::WeakFloat(_)
        ) {
            return Ok(None);
        }
        let ty = self.expression_type(&value, expression.span)?;
        let value = self.coerce_value(value, ty, expression.span)?;
        let Some(mut place) = self.boxed_value_place(&value, expression.span)? else {
            return Ok(None);
        };
        let mut reversed = Vec::new();
        let root = loop {
            if reversed.len() >= 256 {
                return Err(Diagnostic::new(
                    expression.span,
                    "runtime default exceeds storage path depth limit",
                ));
            }
            match place.kind() {
                jai_ir::PlaceKind::Global(_) => {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(
                            expression.span,
                            "runtime default requires its source graph",
                        )
                    })?;
                    let declaration = scope.runtime_default_owner(place, expression.span)?;
                    break DefaultReadRoot::Global {
                        declaration,
                        ty: place.ty(),
                    };
                }
                jai_ir::PlaceKind::Context(ty) => {
                    break DefaultReadRoot::Context {
                        ty,
                    };
                }
                jai_ir::PlaceKind::Field(id) => {
                    let projection = self
                        .places
                        .projection(id)
                        .map_err(|error| Diagnostic::new(expression.span, error.to_string()))?;
                    reversed.push(DefaultReadStep::Field(projection.field));
                    place = projection.base;
                }
                jai_ir::PlaceKind::Dereference(id) => {
                    let pointer = self
                        .places
                        .dereference_projection(id)
                        .map_err(|error| Diagnostic::new(expression.span, error.to_string()))?
                        .pointer
                        .clone();
                    reversed.push(DefaultReadStep::Dereference(place.ty()));
                    place = self.boxed_value_place(&pointer, expression.span)?.ok_or_else(|| Diagnostic::new(expression.span, "runtime default pointer requires a checked global or context storage read"))?;
                }
                jai_ir::PlaceKind::Local(_) => {
                    return Err(Diagnostic::new(
                        expression.span,
                        "runtime parameter default cannot capture local storage",
                    ));
                }
                _ => {
                    return Err(Diagnostic::new(
                        expression.span,
                        "runtime default requires record or pointer storage projections",
                    ));
                }
            }
        };
        if ty != expected {
            return Err(Diagnostic::new(
                expression.span,
                "runtime default differs from its declared parameter type",
            ));
        }
        reversed.reverse();
        RuntimeDefaultRead::checked(
            root,
            reversed,
            ty,
            self.ast_source_location(expression.span)?,
            self.types,
        )
        .map(Some)
    }

    pub(crate) fn materialize_runtime_default(
        &mut self,
        read: &RuntimeDefaultRead,
        expected: TypeId,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        if read.ty != expected {
            return Err(Diagnostic::at_source(
                read.source,
                "runtime default differs from its declared parameter type",
            ));
        }
        let mut place = match read.root {
            DefaultReadRoot::Global {
                declaration,
                ty,
            } => {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "runtime default requires its retained source graph")
                })?;
                let storage = scope.runtime_default_storage(declaration, read.source)?;
                if storage.ty() != ty {
                    return Err(Diagnostic::at_source(
                        read.source,
                        "runtime default global has a different canonical type",
                    ));
                }
                storage
            }
            DefaultReadRoot::Context {
                ty,
            } => {
                let schema = self.require_context(span)?;
                if schema.definition.record_type != ty {
                    return Err(Diagnostic::at_source(
                        read.source,
                        "runtime default context has a different canonical type",
                    ));
                }
                Place::context(ty, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
            }
        };
        for step in read.steps() {
            place = match *step {
                DefaultReadStep::Field(field) => self.places.field(place, field, self.types),
                DefaultReadStep::Dereference(_) => {
                    self.places.dereference(ValueExpr::Load(place), self.types)
                }
            }
            .map_err(|error| Diagnostic::at_source(read.source, error.to_string()))?;
        }
        if place.ty() != expected {
            return Err(Diagnostic::at_source(
                read.source,
                "runtime default storage has a different canonical type",
            ));
        }
        Ok(ValueExpr::Load(place))
    }
}
