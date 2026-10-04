//! Sequence iteration captures its source and advances through a cleanup latch.
use super::iteration_removal::{Iteration, iteration_binary, iteration_integer};
use super::*;
use jai_types::TypeKind;

impl Resolver<'_> {
    pub(crate) fn resolve_array_loop(
        &mut self,
        loop_: &syntax::ArrayLoop,
    ) -> Result<Statement, Diagnostic> {
        match loop_.policy {
            syntax::IterationPolicy::Current | syntax::IterationPolicy::Version2 => {}
        }
        let span = loop_.sequence.span;
        let direction =
            self.iteration_direction(loop_.direction, loop_.reverse_control.as_ref())?;
        let by_pointer = match &loop_.pointer_control {
            Some(control) => self.iteration_modifier(control)?,
            None => loop_.by_pointer,
        };
        // Resolve and capture the source before introducing iterator bindings.
        let source = self.expr(&loop_.sequence)?;
        let source_ty = self.expression_type(&source, span)?;
        let kind = self
            .types
            .kind(source_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        if loop_.expansion.is_some()
            || !matches!(
                kind,
                TypeKind::String
                    | TypeKind::FixedArray { .. }
                    | TypeKind::Slice(_)
                    | TypeKind::DynamicArray(_)
            )
        {
            return self.resolve_custom_loop(loop_, source, source_ty, direction, by_pointer);
        }
        let element = match kind {
            TypeKind::String => self.types.scalar(ScalarType::Int(IntegerType::U8)),
            TypeKind::FixedArray {
                element, ..
            }
            | TypeKind::Slice(element)
            | TypeKind::DynamicArray(element) => element,
            _ => return Err(Diagnostic::new(span, "array iteration requires a sequence")),
        };
        let mut value = self.coerce_value(source, source_ty, span)?;
        if by_pointer && matches!(&value, ValueExpr::StringBytes { bytes, .. } if !bytes.is_empty())
        {
            return Err(Diagnostic::new(
                span,
                "constant string literal backing storage is read-only",
            ));
        }
        let readonly_descriptor = match &value {
            ValueExpr::Load(place) => self.iteration_readonly_owner(*place, span)?.is_some(),
            _ => false,
        };
        if by_pointer && readonly_descriptor && matches!(kind, TypeKind::FixedArray { .. }) {
            return Err(Diagnostic::new(
                span,
                "a by-value array iterator is read-only; use pointer iteration to modify its element",
            ));
        }
        if by_pointer && matches!(value, ValueExpr::Array { .. }) && jai_ir::is_static_value(&value)
        {
            return Err(Diagnostic::new(
                span,
                "constant array literal backing storage is read-only",
            ));
        }
        let mut statements = Vec::new();
        let mut original = None;
        // Capture a descriptor's address once as well as its value. A member or
        // indexed source may itself have side effects in its storage projection.
        if matches!(kind, TypeKind::Slice(_) | TypeKind::DynamicArray(_))
            && let ValueExpr::Load(place) = value
        {
            let pointer_ty = self
                .types
                .pointer(source_ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let pointer = self.allocate_typed(pointer_ty)?;
            statements.push(Statement::Store(
                pointer.place(),
                ValueExpr::AddressOf {
                    place,
                    ty: pointer_ty,
                },
            ));
            let place = self
                .places
                .dereference(ValueExpr::Load(pointer.place()), self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            original = Some(place);
            value = ValueExpr::Load(place);
        }
        let mut captured_ty = source_ty;
        if matches!(kind, TypeKind::FixedArray { .. }) {
            captured_ty = self
                .types
                .slice(element)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            value = match value {
                ValueExpr::Load(array) => ValueExpr::ArrayToSlice {
                    array,
                    ty: captured_ty,
                },
                array => ValueExpr::ArrayView {
                    array: Box::new(array),
                    ty: captured_ty,
                },
            };
        }
        let captured = self.allocate_typed(captured_ty)?;
        statements.push(Statement::Store(captured.place(), value));
        self.scopes.push(HashMap::new());
        let index = match loop_.index.or_else(|| self.symbols.find("it_index")) {
            Some(name) => self.declare_int(name, IntegerType::S64)?,
            None => self
                .allocate(ScalarType::Int(IntegerType::S64))
                .integer(self.types)
                .expect("integer local"),
        };
        let removed = self
            .allocate(ScalarType::Bool)
            .boolean(self.types)
            .expect("bool local");
        let count_place = self
            .places
            .sequence_field(captured.place(), SequenceField::Count, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let count = IntPlace::try_from_place(count_place, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let initial = match direction {
            Direction::Forward => iteration_integer(0),
            Direction::Reverse => {
                iteration_binary(IntOp::Subtract, IntExpr::load(count), iteration_integer(1))
            }
        };
        statements.push(Statement::StoreInt(index.place(), initial));
        let step = Statement::StoreInt(
            index.place(),
            iteration_binary(
                match direction {
                    Direction::Forward => IntOp::Add,
                    Direction::Reverse => IntOp::Subtract,
                },
                IntExpr::load(index.place()),
                iteration_integer(1),
            ),
        );
        let latch_body = match direction {
            Direction::Reverse => vec![step],
            Direction::Forward => vec![Statement::If(
                BoolExpr::Not(Box::new(BoolExpr::Load(removed.place()))),
                Block {
                    statements: vec![step],
                    flow: Flow::FallsThrough,
                },
                Block {
                    statements: vec![],
                    flow: Flow::FallsThrough,
                },
            )],
        };
        let latch = CleanupId::new(self.cleanups.len());
        self.cleanups.push(Cleanup {
            body: Block {
                statements: latch_body,
                flow: Flow::FallsThrough,
            },
            context: self
                .active_push
                .map(CleanupContext::Push)
                .unwrap_or(CleanupContext::Procedure),
        });
        let mut prefix = vec![Statement::StoreBool(
            removed.place(),
            BoolExpr::Constant(false),
        )];
        let mut iterator_local = None;
        let mut iterator_declaration = None;
        let readonly_place = {
            let place = self
                .places
                .index_with_check(
                    captured.place(),
                    IntExpr::load(index.place()),
                    self.checks.array_bounds,
                    self.types,
                )
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            if by_pointer {
                let pointer_ty = self
                    .types
                    .pointer(element)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                let iterator = self.declare_typed(loop_.iterator, pointer_ty)?;
                iterator_local = Some(iterator.id());
                iterator_declaration = Some(prefix.len());
                prefix.push(Statement::Store(
                    iterator.place(),
                    ValueExpr::AddressOf {
                        place,
                        ty: pointer_ty,
                    },
                ));
                None
            } else {
                self.bind_name(
                    loop_.iterator,
                    Binding::Storage(
                        Storage::from_place(place, self.types)
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                    ),
                )?;
                Some(place)
            }
        };
        let id = self.enter_loop(Some(loop_.iterator));
        if loop_.iterator_export {
            self.export_bound_name(loop_.iterator, span)?;
        }
        if loop_.index_export {
            self.export_bound_name(loop_.index.expect("exported named index"), span)?;
        }
        self.loops.last_mut().expect("entered loop").iteration = Some(Iteration {
            captured: captured.place(),
            original,
            index,
            removed,
            latch,
            readonly: readonly_place.into_iter().collect(),
            removable: !readonly_descriptor
                && matches!(kind, TypeKind::Slice(_) | TypeKind::DynamicArray(_)),
            removal: None,
        });
        let mut body = self.block(&loop_.body, true)?;
        let latch_location = self.debug_location(self.span)?;
        self.debug
            .generated_cleanup(latch, &self.cleanups[latch.index()].body, latch_location);
        let while_statement = [
            DebugPathStep::Child(DebugBranch::Block),
            DebugPathStep::Statement(statements.len()),
        ];
        self.debug
            .local_in_completed_block_at(index.local().id(), &while_statement);
        if let Some(local) = iterator_local {
            let declaration = [
                while_statement[0],
                while_statement[1],
                DebugPathStep::Child(DebugBranch::While),
                DebugPathStep::Statement(iterator_declaration.expect("real iterator initializer")),
            ];
            self.debug.local_in_completed_block_at(local, &declaration);
        }
        if body.flow == Flow::FallsThrough {
            self.debug.append_cleanup(latch, body.statements.len());
            body.statements.push(Statement::Cleanup(latch));
        }
        self.debug.prepend_block(prefix.len());
        prefix.extend(body.statements);
        body.statements = prefix;
        self.loops.pop();
        self.scopes.pop();
        let condition = BoolExpr::And(
            Box::new(BoolExpr::CompareInts(
                Relation::GreaterEqual,
                Box::new(IntExpr::load(index.place())),
                Box::new(iteration_integer(0)),
            )),
            Box::new(BoolExpr::CompareInts(
                Relation::Less,
                Box::new(IntExpr::load(index.place())),
                Box::new(IntExpr::load(count)),
            )),
        );
        self.debug.attach_block(&[
            DebugPathStep::Child(DebugBranch::Block),
            DebugPathStep::Statement(statements.len()),
            DebugPathStep::Child(DebugBranch::While),
        ]);
        statements.push(Statement::While {
            id,
            condition: LoopCondition::Value(condition),
            body,
        });
        Ok(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        }))
    }
}
