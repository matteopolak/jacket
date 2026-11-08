//! Default initializers and constant values of types.
use super::literals::{image_of, write_agg};
use crate::ast::ExprKind as E;
use crate::diagnostics::Code;
use crate::ir::Ty;
use crate::sema::convert::Site;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::value::Aggregate;
use crate::sema::{Arc, Compiler, FileId, Result, ScopeId, Span, TypeId, Value, ast, err, ir};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// Constant image of a default-initialized value (`None` = all zero).
    pub fn default_initializer(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<Option<Arc<Aggregate>>> {
        if let Some(img) = self.ty.default_images.get(&ty) {
            return Ok(img.clone());
        }
        let img = match self.ty.types.kind(ty).clone() {
            TypeKind::Type => {
                // A Type value is a descriptor address. Its default is the
                // canonical void descriptor, not a null pointer image.
                let mut image = Aggregate::zeroed(self.size_of(ty, span)?, span)?;
                self.write_value(&mut image, 0, &Value::Type(TypeId::VOID), ty, span)?;
                Some(Arc::new(image))
            }
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let size = self.ty.types.struct_info(s).size;
                let fields = self.ty.types.struct_info(s).fields.clone();
                let inits = self.ty.struct_asts[&s].inits.clone();
                // Built on the first nonzero default: a struct whose defaults are all zero
                // (however large) never gets an image.
                let mut image: Option<Aggregate> = None;
                for (field, (init, scope)) in fields.iter().zip(inits) {
                    match init {
                        Some(e) if matches!(e.kind, E::Uninit) => {}
                        Some(e) => {
                            let value = self.const_value_of_type(scope, &e, field.ty)?;
                            let agg = image_of(&mut image, size, span)?;
                            self.write_value(agg, field.offset, &value, field.ty, e.span)?;
                        }
                        None => {
                            if let Some(inner) = self.default_initializer(field.ty, span)? {
                                write_agg(image_of(&mut image, size, span)?, field.offset, &inner);
                            }
                        }
                    }
                }
                // Field collection already selected #if branches and expanded #insert once.
                for (stmt, scope) in self.ty.struct_asts[&s].overrides.clone() {
                    let ast::StmtKind::Assign {
                        op,
                        lhs,
                        rhs,
                    } = &stmt.kind
                    else {
                        unreachable!("only assignments are retained as defaults");
                    };
                    if *op != ast::AssignOp::Assign {
                        return err(
                            Code::AssignTargetInvalid,
                            stmt.span,
                            "a struct default requires a plain assignment",
                        );
                    }
                    if rhs.len() != 1 && rhs.len() != lhs.len() {
                        return err(
                            Code::NamesButValues,
                            stmt.span,
                            format!("{} names but {} values", lhs.len(), rhs.len()),
                        );
                    }
                    let targets = lhs
                        .iter()
                        .map(|target| self.override_target(ty, scope, target))
                        .collect::<Result<Vec<_>>>()?;
                    let mut values = Vec::new();
                    for (index, value) in rhs.iter().enumerate() {
                        let expected = targets[if rhs.len() == 1 {
                            0
                        } else {
                            index
                        }]
                        .1;
                        values.extend(
                            self.eval_const_all_expected(scope, value, Some(expected))?
                                .into_iter()
                                .map(|op| (op, value.span)),
                        );
                    }
                    if values.len() != 1 && values.len() != targets.len() {
                        return err(
                            Code::NamesButValues,
                            stmt.span,
                            format!("{} names but {} values", targets.len(), values.len()),
                        );
                    }
                    let mut writes = Vec::new();
                    for (index, (offset, fty)) in targets.into_iter().enumerate() {
                        let (value, at) = &values[if values.len() == 1 {
                            0
                        } else {
                            index
                        }];
                        let value = self.const_value_of_operand(scope, value.clone(), fty, *at)?;
                        writes.push((offset, fty, value, *at));
                    }
                    for (offset, fty, value, at) in writes {
                        self.write_value(
                            image_of(&mut image, size, span)?,
                            offset,
                            &value,
                            fty,
                            at,
                        )?;
                    }
                }
                image.map(Arc::new)
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => match self.default_initializer(elem, span)? {
                Some(inner) => {
                    let size = self.size_of(elem, span)?;
                    let mut agg = Aggregate::zeroed(size.saturating_mul(n), span)?;
                    for i in 0..n {
                        write_agg(&mut agg, i * size, &inner);
                    }
                    Some(Arc::new(agg))
                }
                None => None,
            },
            TypeKind::Distinct(d) => {
                let base = self.ty.types.distincts[d.0 as usize].base;
                self.default_initializer(base, span)?
            }
            _ => None,
        };
        self.ty.default_images.insert(ty, img.clone());
        Ok(img)
    }

    /// Evaluate a constant expression converted to `ty`.
    pub fn const_value_of_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        ty: TypeId,
    ) -> Result<Value> {
        let op = self.eval_const(scope, expr, Some(ty))?;
        self.const_value_of_operand(scope, op, ty, expr.span)
    }

    /// Convert an evaluated constant operand to type `ty` and read its value.
    pub fn const_value_of_operand(
        &mut self,
        scope: ScopeId,
        op: Operand,
        ty: TypeId,
        span: Span,
    ) -> Result<Value> {
        let file = self.scope_file(scope);
        let mut scratch = FnCtx::new(
            "const".into(),
            ir::Sig {
                params: vec![Ty::Ptr],
                returns: vec![],
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            file,
        );
        scratch.compile_time = true;
        scratch.set_context(Some(scratch.b.param(0)));
        // Lexical callable headers annotate operands, not the constant value
        // stored in a generic binding. Preserve the ordinary conversion first.
        let converted = self
            .convert(&mut scratch, op, ty, Site::opaque(span))?
            .without_callable();
        match converted {
            Operand::Const {
                value, ..
            } => Ok(value),
            Operand::Type(t) => Ok(Value::Type(t)),
            Operand::Procs(p) if p.len() == 1 => Ok(Value::Proc(p[0])),
            other => {
                // Conversions that need code (e.g. boxing into Any) run as a thunk.
                let op = self.run_thunk(scratch, other, span)?.without_callable();
                match op {
                    Operand::Const {
                        value, ..
                    } => Ok(value),
                    other => err(
                        Code::ExpectedConstant,
                        span,
                        format!("expected a constant, found {}", self.describe(&other)),
                    ),
                }
            }
        }
    }

    /// Initialize memory at `addr` with the default value of `ty`.
    pub fn init_default(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        addr: ir::Val,
        span: Span,
    ) -> Result<()> {
        if !f.type_only
            && let Some(record) = &mut f.lifetimes
        {
            record.push(
                f.b.current,
                crate::ide::lifetimes::LifetimeEvent::Initialize {
                    place: addr,
                    ty,
                    span,
                },
            );
        }
        let size = self.size_of(ty, span)?;
        match self.default_initializer(ty, span)? {
            None => f.b.zero(addr, size),
            Some(img) => {
                let g = if let Some(&g) = self.ty.default_globals.get(&ty) {
                    g
                } else {
                    let align = self.align_of(ty, span)?;
                    let g = self.out.program.add_global(ir::Global {
                        name: format!("default.{}", self.ty.types.name(ty)),
                        size,
                        align,
                        init: img.bytes.clone(),
                        relocs: img.relocs.clone(),
                        read_only: true,
                        export: None,
                    });
                    self.ty.default_globals.insert(ty, g);
                    g
                };
                let src = f.b.global_addr(g);
                f.b.copy(addr, src, size);
            }
        }
        Ok(())
    }

    /// `initializer_of(T)`: a `(*void) #no_context` procedure that default-initializes.
    pub fn initializer_proc(&mut self, ty: TypeId, span: Span) -> Result<(ir::FuncId, TypeId)> {
        let proc_ty = self
            .ty
            .types
            .intern(TypeKind::Proc(Arc::new(crate::types::ProcType {
                params: vec![TypeId::VOID_PTR],
                returns: Vec::new(),
                variadic: false,
                c_varargs: false,
                c_call: false,
                no_context: true,
                non_pod_return: false,
            })));
        if let Some(&func) = self.ty.initializers.get(&ty) {
            return Ok((func, proc_ty));
        }
        let name = format!("__init_{}", self.ty.types.name(ty));
        let func = self.out.program.reserve_func(name.clone());
        self.ty.initializers.insert(ty, func);
        let sig = ir::Sig {
            params: vec![Ty::Ptr],
            returns: vec![],
            conv: ir::Conv::Jai,
            c_varargs: false,
            c_fixed: 0,
            c_abi: None,
        };
        let mut f = FnCtx::new(name, sig, FileId(0));
        let addr = f.b.param(0);
        self.init_default(&mut f, ty, addr, span)?;
        f.b.ret(Vec::new());
        self.out.program.funcs[func.0 as usize] = Some(f.b.finish());
        Ok((func, proc_ty))
    }
}
