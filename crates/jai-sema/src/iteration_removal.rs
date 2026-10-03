//! Array-loop state uses ordinary typed storage and lexical cleanup IDs.
use super::*;

pub(super) struct Iteration {
    pub captured: Place,
    pub original: Option<Place>,
    pub index: IntLocal,
    pub removed: BoolLocal,
    pub latch: CleanupId,
    pub readonly: Vec<Place>,
    pub removable: bool,
    pub removal: Option<Span>,
}

impl Resolver<'_> {
    pub(super) fn iteration_direction(
        &mut self,
        default: Direction,
        control: Option<&syntax::Expression>,
    ) -> Result<Direction, Diagnostic> {
        match control {
            Some(control) if self.iteration_modifier(control)? => Ok(Direction::Reverse),
            Some(_) => Ok(Direction::Forward),
            None => Ok(default),
        }
    }

    pub(super) fn iteration_modifier(
        &mut self,
        control: &syntax::Expression,
    ) -> Result<bool, Diagnostic> {
        self.compile_time_condition(control).map_err(|error| {
            Diagnostic::new(
                control.span,
                format!(
                    "iteration modifier requires a compile-time value: {}",
                    error.message
                ),
            )
        })
    }

    pub(crate) fn reject_iteration_write(
        &self,
        place: Place,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.reject_using_write(place, span)?;
        if self.iteration_readonly_owner(place, span)?.is_some() {
            return Err(Diagnostic::new(
                span,
                "a by-value array iterator is read-only; use pointer iteration to modify its element",
            ));
        }
        Ok(())
    }

    pub(crate) fn iteration_readonly_owner(
        &self,
        mut place: Place,
        span: Span,
    ) -> Result<Option<usize>, Diagnostic> {
        loop {
            if let Some(owner) = self.loops.iter().rposition(|loop_| {
                loop_
                    .iteration
                    .as_ref()
                    .is_some_and(|iteration| iteration.readonly.contains(&place))
            }) {
                return Ok(Some(owner));
            }
            place = match place.kind() {
                PlaceKind::Field(id) => self.places.projection(id).map(|p| p.base),
                PlaceKind::Index(id) => self.places.index_projection(id).map(|p| p.base),
                PlaceKind::SequenceField(id) => self.places.sequence_projection(id).map(|p| p.base),
                _ => return Ok(None),
            }
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
    }

    pub(super) fn resolve_remove(
        &mut self,
        target: syntax::LoopTarget,
        span: Span,
    ) -> Result<Statement, Diagnostic> {
        let name = match target {
            syntax::LoopTarget::Named(name) => name,
            syntax::LoopTarget::Innermost => {
                return Err(Diagnostic::new(
                    span,
                    "remove requires an array iterator name",
                ));
            }
        };
        let (loop_index, active) = self
            .loops
            .iter()
            .enumerate()
            .rev()
            .find(|(_, loop_)| loop_.name == Some(name))
            .ok_or_else(|| {
                Diagnostic::new(span, "remove must name an active enclosing array iterator")
            })?;
        if self
            .cleanup_context
            .is_some_and(|context| loop_index < context.loop_depth)
        {
            return Err(Diagnostic::new(
                span,
                "a deferred body cannot remove an enclosing array element",
            ));
        }
        let iteration = active
            .iteration
            .as_ref()
            .filter(|iteration| iteration.removable)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "remove requires a mutable slice or dynamic-array iteration",
                )
            })?;
        if loop_index + 1 != self.loops.len() {
            return Err(Diagnostic::new(
                span,
                "remove targeting an outer loop from a nested loop is not implemented",
            ));
        }
        if iteration.removal.is_some() {
            return Err(Diagnostic::new(
                span,
                "multiple remove statements targeting one array loop are not implemented",
            ));
        }
        let captured = iteration.captured;
        let original = iteration.original;
        let index = iteration.index;
        let removed = iteration.removed;
        self.loops[loop_index]
            .iteration
            .as_mut()
            .expect("checked array iteration")
            .removal = Some(span);
        let count = self
            .places
            .sequence_field(captured, SequenceField::Count, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let count = IntPlace::try_from_place(count, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let last = iteration_binary(IntOp::Subtract, IntExpr::load(count), iteration_integer(1));
        let destination = self
            .places
            .index_with_check(
                captured,
                IntExpr::load(index.place()),
                self.checks.array_bounds,
                self.types,
            )
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let source = self
            .places
            .index_with_check(captured, last.clone(), self.checks.array_bounds, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let mut statements = vec![
            Statement::Store(destination, ValueExpr::Load(source)),
            Statement::StoreInt(count, last),
        ];
        if let Some(original) = original {
            let original_count = self
                .places
                .sequence_field(original, SequenceField::Count, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            statements.push(Statement::Store(
                original_count,
                ValueExpr::Int(IntExpr::load(count)),
            ));
        }
        statements.push(Statement::StoreBool(
            removed.place(),
            BoolExpr::Constant(true),
        ));
        // A removal is a statement, not a transfer. The latch reuses this index.
        Ok(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        }))
    }
}

pub(super) fn iteration_integer(value: i128) -> IntExpr {
    IntExpr::constant(IntegerValue::wrapping(IntegerType::S64, value))
}

pub(super) fn iteration_binary(operation: IntOp, left: IntExpr, right: IntExpr) -> IntExpr {
    IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(operation, Box::new(left), Box::new(right)),
    )
}
