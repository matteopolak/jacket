//! Custom iteration invokes a source macro with typed aliases and quoted body.
use super::*;
use crate::{Local, Storage, TypeId, ValueExpr};
use jai_types::TypeKind;

impl Resolver<'_> {
    pub(crate) fn resolve_custom_loop(
        &mut self,
        loop_: &syntax::ArrayLoop,
        source: Expr,
        source_ty: TypeId,
        direction: syntax::Direction,
        by_pointer: bool,
    ) -> Result<Statement, Diagnostic> {
        let span = loop_.sequence.span;
        if loop_.iterator_export || loop_.index_export {
            return Err(Diagnostic::new(
                span,
                "exporting iterator bindings of nested custom for expansions is not implemented",
            ));
        }
        self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "custom iteration requires a defining file scope")
        })?;
        let path = loop_
            .expansion
            .clone()
            .or_else(|| {
                self.symbols
                    .find("for_expansion")
                    .map(|root| syntax::NamePath {
                        root,
                        members: vec![],
                    })
            })
            .ok_or_else(|| {
                Diagnostic::new(span, "this type has no source for_expansion procedure")
            })?;
        let iteration_protocol::IterationProtocol {
            target,
            annotations,
            source_parameter_ty,
            substitution,
        } = self.select_iteration_protocol(&path, source_ty, span)?;
        let procedure = &target.procedure;
        let body_ty =
            self.expanded_annotation(&target, &annotations[1], procedure.parameters[1].span)?;
        if body_ty != self.types.code_type() {
            return Err(Diagnostic::new(
                span,
                "the second for expansion parameter must be Code",
            ));
        }
        let flags_ty =
            self.expanded_annotation(&target, &annotations[2], procedure.parameters[2].span)?;
        if !self.expanded_enum_flags(&target, flags_ty) {
            return Err(Diagnostic::new(
                span,
                "the third for expansion parameter must be a flags enum defining POINTER and REVERSE",
            ));
        }
        let flag = |name: &str| {
            self.symbols
                .find(name)
                .and_then(|symbol| self.expanded_enum_member(&target, flags_ty, symbol))
                .ok_or_else(|| {
                    Diagnostic::new(span, format!("for expansion flags are missing {name}"))
                })
        };
        let pointer = flag("POINTER")?;
        let reverse = flag("REVERSE")?;
        let representation = self
            .types
            .enum_definition(flags_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .representation;
        let bits = if by_pointer { pointer.bits() } else { 0 }
            | if direction == syntax::Direction::Reverse {
                reverse.bits()
            } else {
                0
            };
        let flags = jai_ir::ConstantValue {
            ty: flags_ty,
            kind: jai_ir::ConstantKind::Enum(jai_types::Integer::wrapping(
                representation,
                i128::from(bits),
            )),
        };
        let mut initializers = Vec::new();
        let source_binding = self.custom_iteration_source(
            source,
            source_ty,
            source_parameter_ty,
            span,
            &mut initializers,
        )?;
        let Expr::Code(body) =
            self.capture_code(&syntax::CodeBody::Block(loop_.body.clone()), span)?
        else {
            unreachable!("code capture");
        };
        let bindings = vec![
            (procedure.parameters[0].name, source_binding),
            (procedure.parameters[1].name, Binding::Code(body)),
            (
                procedure.parameters[2].name,
                Binding::TypedConstant(self.meta.intern_constant(flags)),
            ),
        ];
        let it = self
            .symbols
            .find("it")
            .ok_or_else(|| Diagnostic::new(span, "a for expansion must export it and it_index"))?;
        let it_index = self
            .symbols
            .find("it_index")
            .ok_or_else(|| Diagnostic::new(span, "a for expansion must export it and it_index"))?;
        self.meta.codes.enter_macro(target.id, span)?;
        self.meta.codes.push_export_remap(vec![
            (it, loop_.iterator),
            (it_index, loop_.index.unwrap_or(it_index)),
        ]);
        let result = self
            .expand_bound_substituted_target_body(
                &target,
                bindings,
                initializers,
                Some(&substitution),
                span,
            )
            .and_then(|body| {
                if !self.meta.codes.exported_named(it) || !self.meta.codes.exported_named(it_index)
                {
                    return Err(Diagnostic::new(
                        span,
                        "a for expansion must export it and it_index",
                    ));
                }
                Ok(body)
            });
        self.meta.codes.pop_export_remap();
        self.meta.codes.leave_macro(target.id);
        result.map(Statement::Block)
    }

    fn custom_iteration_source(
        &mut self,
        source: Expr,
        source_ty: TypeId,
        parameter_ty: TypeId,
        span: Span,
        initializers: &mut Vec<Statement>,
    ) -> Result<Binding, Diagnostic> {
        let value = self.coerce_value(source, source_ty, span)?;
        let parameter_pointee = match self
            .types
            .kind(parameter_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Pointer(pointee) => Some(*pointee),
            _ => None,
        };
        let source_pointee = match self
            .types
            .kind(source_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Pointer(pointee) => Some(*pointee),
            _ => None,
        };
        if parameter_ty == source_ty && parameter_pointee.is_some() {
            let local = self.capture_iteration_value(value, parameter_ty, initializers)?;
            return Ok(Binding::Storage(Storage::local(local, self.types)));
        }
        if source_pointee == Some(parameter_ty) {
            let pointer = self.capture_iteration_value(value, source_ty, initializers)?;
            let place = self
                .places
                .dereference(ValueExpr::Load(pointer.place()), self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return Ok(Binding::Storage(
                Storage::from_place(place, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
            ));
        }
        if source_ty != parameter_ty && parameter_pointee != Some(source_ty) {
            return Err(Diagnostic::new(
                span,
                "source type does not match the for expansion's first parameter",
            ));
        }
        let place = match value {
            ValueExpr::Load(place) => {
                if parameter_pointee.is_some() {
                    self.reject_iteration_write(place, span)?;
                }
                place
            }
            value => self
                .capture_iteration_value(value, source_ty, initializers)?
                .place(),
        };
        let readonly_owner = self.iteration_readonly_owner(place, span)?;
        let pointer_ty = self
            .types
            .pointer(source_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let pointer = self.capture_iteration_value(
            ValueExpr::AddressOf {
                place,
                ty: pointer_ty,
            },
            pointer_ty,
            initializers,
        )?;
        let storage = if parameter_pointee.is_some() {
            Storage::local(pointer, self.types)
        } else {
            let place = self
                .places
                .dereference(ValueExpr::Load(pointer.place()), self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            if let Some(owner) = readonly_owner {
                self.loops[owner]
                    .iteration
                    .as_mut()
                    .expect("readonly iteration owner")
                    .readonly
                    .push(place);
            }
            Storage::from_place(place, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
        };
        Ok(Binding::Storage(storage))
    }

    fn capture_iteration_value(
        &mut self,
        value: ValueExpr,
        ty: TypeId,
        initializers: &mut Vec<Statement>,
    ) -> Result<Local, Diagnostic> {
        let local = self.allocate_typed(ty)?;
        initializers.push(Statement::Store(local.place(), value));
        Ok(local)
    }
}
