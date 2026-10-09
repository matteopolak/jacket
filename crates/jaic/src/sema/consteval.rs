//! Compile-time execution: `#run`, constant thunks, global initializers,
//! `#insert`, and reading interpreter memory back into constant values.
use super::lower::{FnCtx, Operand};
use super::value::Aggregate;
use super::*;

use crate::ir::Ty;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// A fresh compile-time function context. Thunks take the context pointer.
    pub fn thunk_ctx(&self, origin: ir::FuncOrigin, file: FileId) -> FnCtx {
        let sig = ir::Sig {
            params: vec![Ty::Ptr],
            returns: vec![],
            conv: ir::Conv::Jai,
            c_varargs: false,
            c_fixed: 0,
            c_abi: None,
        };
        let mut f = FnCtx::new(origin.thunk_name().into(), sig, file);
        f.b.func.origin = origin;
        f.compile_time = true;
        f.context = Some(f.b.param(0));
        f
    }

    /// A procedure-level child of `scope` so compile-time code cannot see runtime locals.
    pub fn thunk_scope(&mut self, scope: ScopeId) -> ScopeId {
        if let Some(&s) = self.thunk_scopes.get(&scope) {
            return s;
        }
        let module = self.scope(scope).module;
        let s = self.new_scope(scope::ScopeKind::Proc, Some(scope), module, None);
        self.thunk_scopes.insert(scope, s);
        s
    }

    /// Finish thunk `f` that computes `op`, run it, and read the result back as a constant.
    pub fn run_thunk(&mut self, f: FnCtx, op: Operand, span: Span) -> Result<Operand> {
        let first = self.run_thunk_all(f, op, span)?.into_iter().next();
        Ok(first.unwrap_or(Operand::Void))
    }

    /// `run_thunk` keeping every value of a multi-value result (`a, b :: #run f();`).
    pub fn run_thunk_all(&mut self, mut f: FnCtx, op: Operand, span: Span) -> Result<Vec<Operand>> {
        let op = self.settle_untyped(op, None);
        let values = match op {
            Operand::Void => Vec::new(),
            Operand::Multi(values) => values,
            other => {
                let ty = other.ty();
                if ty == TypeId::VOID {
                    Vec::new()
                } else {
                    let (_, v) = self.rvalue(&mut f, other, span)?;
                    vec![(ty, v)]
                }
            }
        };
        let mut out = Vec::new();
        for (ty, v) in values {
            let size = self.size_of(ty, span)?;
            let align = self.align_of(ty, span)?;
            let g = self.program.add_global(ir::Global {
                name: "run.result".into(),
                size: size.max(1),
                align,
                init: Vec::new(),
                relocs: Vec::new(),
                read_only: false,
                export: None,
            });
            let addr = f.b.global_addr(g);
            self.store_value(&mut f, ty, addr, v, span)?;
            out.push((ty, g));
        }
        f.b.ret(Vec::new());
        self.call_thunk(f, span)?;
        let mut results = Vec::new();
        for (ty, g) in out {
            let addr = self
                .interp
                .global_addr(&self.program, g)
                .map_err(|t| Box::new(Diagnostic::error(span, t.message)))?;
            results.push(match self.read_value(addr, ty, span)? {
                Value::Type(t) => Operand::Type(t),
                value => Operand::Const {
                    ty,
                    value,
                    untyped: false,
                },
            });
        }
        Ok(results)
    }

    /// Finish the compile-time function `f` and run it; returns its scalar results.
    pub fn call_thunk(&mut self, f: FnCtx, span: Span) -> Result<Vec<u64>> {
        let func = f.b.finish();
        let id = self.program.reserve_func(func.name.clone());
        self.program.funcs[id.0 as usize] = Some(func);
        // Inside another body's lowering, queued bodies may be unrelated code whose own
        // compile-time runs need what is being lowered right now: lower on demand instead.
        let nested = self.lowering_depth > 0;
        let mut deferred = if nested {
            self.lower_reachable_from(id)
        } else {
            self.drain_bodies_lenient()
        };
        let ctx = self.compile_time_context(span)?;
        self.enable_stack_traces(span);
        self.interp.compile_time = true;
        let outer = self.interp.run_effects.replace(self.interp.effects);
        // Typed exports belong to this run (a nested one keeps the outer run's aside).
        let outer_exports = (
            std::mem::take(&mut self.interp.code_exports),
            std::mem::take(&mut self.interp.code_export_cursor),
            std::mem::take(&mut self.interp.run_codes),
        );
        let mut result = self.interp.call(&self.program, id, &[ctx]);
        // Run again from the start while nothing observable happened, when the run
        // - asked for a code with resolved names and types (`compiler_get_nodes`), or
        // - called a procedure whose body is still queued (on-demand lowering).
        for _ in 0..10_000 {
            if result.is_ok() || self.interp.run_effects != Some(self.interp.effects) {
                break;
            }
            if let Some(code) = self.interp.export_request.take() {
                self.export_code_typed(code);
            } else if let Some(missing) = self.interp.missing_func.take()
                && let Some(p) = self.queued_proc_of(missing)
            {
                if let Some(e) = self.lower_with_callees(p) {
                    deferred.get_or_insert(e);
                    break;
                }
            } else {
                break;
            }
            // The earlier attempt may have edited the nodes it got (built once per record):
            // the next one gets them exported anew.
            let counts: Vec<(usize, usize)> = self
                .interp
                .code_exports
                .iter()
                .map(|(&code, exports)| (code, exports.len()))
                .collect();
            self.interp.code_exports.clear();
            for (code, n) in counts {
                for _ in 0..n {
                    self.export_code_typed(code);
                }
            }
            self.interp.run_effects = Some(self.interp.effects);
            self.interp.code_export_cursor.clear();
            self.interp.run_codes.1 = 0;
            result = self.interp.call(&self.program, id, &[ctx]);
        }
        (
            self.interp.code_exports,
            self.interp.code_export_cursor,
            self.interp.run_codes,
        ) = outer_exports;
        self.interp.export_request = None;
        self.interp.missing_func = None;
        self.interp.run_effects = outer;
        self.flush_interp_output();
        if !self.interp.pending_type_flags.is_empty() {
            self.apply_type_info_flags(span)?;
        }
        match result {
            Ok(values) => Ok(values),
            Err(_) if deferred.is_some() => Err(deferred.unwrap()),
            Err(trap) => {
                self.ct_traps += 1;
                let d =
                    self.trap_diagnostic(&trap, "error during compile-time execution", Some(span));
                Err(Box::new(d))
            }
        }
    }

    fn flush_interp_output(&mut self) {
    }

    /// Context pointer used by compile-time code (allocated and initialized once).
    pub fn compile_time_context(&mut self, span: Span) -> Result<u64> {
        if let Some(addr) = self.ct_context {
            return Ok(addr);
        }
        let Ok(ty) = self.context_type(span) else {
            // No runtime support loaded: compile-time code without a context.
            self.ct_context = Some(0);
            return Ok(0);
        };
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        let img = self.default_initializer(ty, span)?;
        let (init, relocs) = img
            .map(|a| (a.bytes.clone(), a.relocs.clone()))
            .unwrap_or_default();
        let g = self.program.add_global(ir::Global {
            name: "compile_time_context".into(),
            size,
            align,
            init,
            relocs,
            read_only: false,
            export: None,
        });
        // Temporary storage backed by a static buffer.
        if let Some(rs) = self.runtime_support
            && let Ok(ts_ty) = self.module_type(rs, "Temporary_Storage", span)
        {
            let ts_size = self.size_of(ts_ty, span)?;
            let buf_size = 1 << 20;
            let buf = self.program.add_global(ir::Global {
                name: "compile_time_temporary".into(),
                size: buf_size,
                align: 16,
                init: Vec::new(),
                relocs: Vec::new(),
                read_only: false,
                export: None,
            });
            let mut agg = Aggregate {
                bytes: vec![0; ts_size as usize],
                relocs: Vec::new(),
            };
            for (field, value) in [
                ("data", None),
                ("original_data", None),
                ("size", Some(buf_size)),
                ("original_size", Some(buf_size)),
            ] {
                if let Some((path, _)) = self.find_member(ts_ty, Sym::intern(field), span)? {
                    let off: u64 = path
                        .iter()
                        .map(|s| {
                            if let structs::PathStep::Offset(o) = s {
                                *o
                            } else {
                                0
                            }
                        })
                        .sum();
                    match value {
                        Some(v) => agg.bytes[off as usize..off as usize + 8]
                            .copy_from_slice(&v.to_le_bytes()),
                        None => agg.relocs.push(ir::Reloc {
                            offset: off,
                            target: ir::RelocTarget::Global(buf),
                            addend: 0,
                        }),
                    }
                }
            }
            let ts = self.program.add_global(ir::Global {
                name: "compile_time_temporary_storage".into(),
                size: ts_size,
                align: 8,
                init: agg.bytes,
                relocs: agg.relocs,
                read_only: false,
                export: None,
            });
            if let Some((path, _)) = self.find_member(ty, Sym::intern("temporary_storage"), span)? {
                let off: u64 = path
                    .iter()
                    .map(|s| {
                        if let structs::PathStep::Offset(o) = s {
                            *o
                        } else {
                            0
                        }
                    })
                    .sum();
                let global = &mut self.program.globals[g.0 as usize];
                if global.init.len() < size as usize {
                    global.init.resize(size as usize, 0);
                }
                global.relocs.push(ir::Reloc {
                    offset: off,
                    target: ir::RelocTarget::Global(ts),
                    addend: 0,
                });
            }
        }
        // A body that fails here is reported by the final drain.
        if self.lowering_depth > 0 {
            self.lower_reachable_from_global(g);
        } else {
            self.drain_bodies_lenient();
        }
        let addr = self
            .interp
            .global_addr(&self.program, g)
            .map_err(|t| Box::new(Diagnostic::error(span, t.message)))?;
        self.ct_context = Some(addr);
        Ok(addr)
    }

    /// A type exported by a module.
    pub fn module_type(&mut self, module: ModuleId, name: &str, span: Span) -> Result<TypeId> {
        let ids = self.module_declarations(module, Sym::intern(name))?;
        let Some(&id) = ids.first() else {
            return err(span, format!("`{name}` not found"));
        };
        match self.resolve_entity(id)? {
            scope::Resolved::Const {
                value: Value::Type(t),
                ..
            } => Ok(t),
            _ => err(span, format!("`{name}` is not a type")),
        }
    }

    /// `#run expr` / `#run { ... }`.
    pub fn check_run(
        &mut self,
        scope: ScopeId,
        body: &ast::RunBody,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        if self.ide.is_none() {
            return self.check_run_inner(scope, body, expected, span);
        }
        let mark = self.ide_output_mark();
        let result = self.check_run_inner(scope, body, expected, span);
        if let Ok(op) = &result {
            self.ide_note_run(span, op, mark);
        }
        result
    }

    fn check_run_inner(
        &mut self,
        scope: ScopeId,
        body: &ast::RunBody,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let tscope = self.thunk_scope(scope);
        let file = self.scope_file(scope);
        match body {
            ast::RunBody::Expr(e) => {
                let mut f = self.thunk_ctx(ir::FuncOrigin::Run, file);
                let op = self.check_expr(&mut f, tscope, e, expected)?;
                match op {
                    Operand::Const {
                        ..
                    }
                    | Operand::Type(_)
                    | Operand::Procs(_) => Ok(op),
                    other => self.run_thunk(f, other, span),
                }
            }
            ast::RunBody::Block(block) => {
                let mut f = self.thunk_ctx(ir::FuncOrigin::Run, file);
                let module = self.scope(tscope).module;
                let inner = self.new_scope(scope::ScopeKind::Block, Some(tscope), module, None);
                self.check_block_stmts(&mut f, inner, &block.stmts)?;
                if !f.b.is_terminated() {
                    self.emit_defers(&mut f, 0, span)?;
                }
                self.run_thunk(f, Operand::Void, span)
            }
        }
    }

    /// Execute top-level `#run` directives (in declaration order).
    /// Execute the `#run`/`#assert` directives not executed yet. Sources a
    /// `#run` adds to this compiler's own workspace are loaded right after it.
    pub fn run_top_level(&mut self) -> Result<()> {
        while self.runs_done < self.top_level_runs.len() {
            let (expr, scope) = self.top_level_runs[self.runs_done].clone();
            let misses = self.placeholder_misses;
            let result = self.eval_const(scope, &expr, None);
            // Needs a `#placeholder` a metaprogram may define later: retry at the next settle.
            if result.is_err() && !self.placeholders_final && self.placeholder_misses != misses {
                return Ok(());
            }
            self.runs_done += 1;
            result?;
            self.pull_workspace_sources()?;
        }
        while self.asserts_done < self.asserts.len() {
            let (cond, message, args, scope) = self.asserts[self.asserts_done].clone();
            let misses = self.placeholder_misses;
            let holds = self.eval_static_condition(scope, &cond);
            if holds.is_err() && !self.placeholders_final && self.placeholder_misses != misses {
                return Ok(());
            }
            self.asserts_done += 1;
            if !holds? {
                let msg = self.assert_message(scope, message.as_ref(), &args)?;
                return Err(Box::new(self.static_assert_failed(&cond, cond.span, msg)));
            }
        }
        Ok(())
    }

    /// Initial bytes of a global variable.
    pub fn global_initializer(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        ty: TypeId,
    ) -> Result<Rc<Aggregate>> {
        let value = self.const_value_of_type(scope, expr, ty)?;
        let size = self.size_of(ty, expr.span)?;
        let mut agg = Aggregate::zeroed(size, expr.span)?;
        self.write_value(&mut agg, 0, &value, ty, expr.span)?;
        Ok(Rc::new(agg))
    }

    // -----------------------------------------------------------------------
    // #insert
    // -----------------------------------------------------------------------

    /// Evaluate the operand of `#insert`. `#insert -> string { ... }` (or `-> Code`) is a
    /// procedure without parameters that runs at compile time; its result is inserted.
    pub fn eval_insert_operand(&mut self, scope: ScopeId, value: &ast::Expr) -> Result<Operand> {
        if let ast::ExprKind::Proc(lit) = &value.kind
            && lit.header.params.is_empty()
            && !lit.header.returns.is_empty()
        {
            let call = ast::Expr {
                kind: ast::ExprKind::Call {
                    callee: Box::new(value.clone()),
                    args: Vec::new(),
                    hint: ast::CallHint::None,
                },
                span: value.span,
            };
            let op = self.check_run_inner(scope, &ast::RunBody::Expr(call), None, value.span)?;
            if self.ide.is_some() {
                self.ide_note_insert(value, &op);
            }
            return Ok(op);
        }
        let op = self.eval_const(scope, value, None)?;
        if self.ide.is_some() {
            self.ide_note_insert(value, &op);
        }
        Ok(op)
    }

    pub fn insert_stmts_from(&mut self, op: Operand, span: Span) -> Result<Vec<ast::Stmt>> {
        match op {
            Operand::Const {
                value: Value::Code(code),
                ..
            } => Ok(match &*self.codes[code.0 as usize] {
                ast::CodeBody::Block(b) => b.stmts.clone(),
                ast::CodeBody::Expr(e) => vec![ast::Stmt {
                    kind: ast::StmtKind::Expr(e.clone()),
                    span: e.span,
                    notes: Vec::new(),
                }],
            }),
            Operand::Const {
                value: Value::String(s),
                ..
            } => {
                let text: Rc<str> = String::from_utf8_lossy(&s).into();
                let file = self.insert_source(span, text.clone())?;
                let ast = crate::parser::parse_file(file, &text).map_err(Box::new)?;
                Ok(ast.stmts)
            }
            Operand::Void => Ok(Vec::new()),
            other => err(
                span,
                format!(
                    "#insert needs a string or Code, found {}",
                    self.describe(&other)
                ),
            ),
        }
    }

    /// Register the text of an `#insert`ed string inserted at `at`. Inserted text may insert
    /// more (`X :: "#insert X;"`), so nesting is bounded.
    pub(super) fn insert_source(&mut self, at: Span, text: Rc<str>) -> Result<FileId> {
        const MAX_INSERT_DEPTH: u32 = 256;
        let depth = self.insert_depth.get(&at.file).map_or(1, |d| d + 1);
        if depth > MAX_INSERT_DEPTH {
            return err(
                at,
                format!("#insert strings are nested more than {MAX_INSERT_DEPTH} deep"),
            );
        }
        // Named after the outermost real file, so nested inserts keep a short path.
        let path = self.sources.get(at.file).path.clone();
        let path = if depth > 1 {
            path
        } else {
            format!("<#insert at {path}>")
        };
        let file = self.sources.add(path, text);
        self.insert_depth.insert(file, depth);
        Ok(file)
    }

    pub fn eval_insert_stmts(
        &mut self,
        scope: ScopeId,
        value: &ast::Expr,
    ) -> Result<Vec<ast::Stmt>> {
        let op = self.eval_insert_operand(scope, value)?;
        self.insert_stmts_from(op, value.span)
    }

    /// The expression an expression-position `#insert` produces, and the scope it is checked
    /// in: a `Code` value's own scope (where its names were written), else `scope`.
    pub fn eval_insert_expr(
        &mut self,
        scope: ScopeId,
        value: &ast::Expr,
    ) -> Result<(ast::Expr, ScopeId)> {
        let op = self.eval_insert_operand(scope, value)?;
        if let Operand::Const {
            value: Value::Code(code),
            ..
        } = &op
        {
            let code_scope = self.code_scope_at(*code, scope);
            return Ok((self.insert_expr_of(op, value)?, code_scope));
        }
        Ok((self.insert_expr_of(op, value)?, scope))
    }

    fn insert_expr_of(&mut self, op: Operand, value: &ast::Expr) -> Result<ast::Expr> {
        match op {
            Operand::Const {
                value: Value::Code(code),
                ..
            } => match &*self.codes[code.0 as usize] {
                ast::CodeBody::Expr(e) => Ok(e.clone()),
                ast::CodeBody::Block(b) => match b.stmts.as_slice() {
                    [
                        ast::Stmt {
                            kind: ast::StmtKind::Expr(e),
                            ..
                        },
                    ] => Ok(e.clone()),
                    _ => err(
                        value.span,
                        "#insert of a statement block in expression position",
                    ),
                },
            },
            Operand::Const {
                value: Value::String(s),
                ..
            } => {
                // A trailing `;` (`#insert "x.y;"` as an expression) ends the statement only.
                let source = String::from_utf8_lossy(&s);
                let source = source.trim_end().trim_end_matches(';');
                let text = format!("__jaic_insert :: ({source});");
                let file = self.insert_source(value.span, text.clone().into())?;
                let ast = crate::parser::parse_file(file, &text).map_err(Box::new)?;
                match ast.stmts.first().map(|s| &s.kind) {
                    Some(ast::StmtKind::Decl(d)) if d.value.is_some() => {
                        Ok(d.value.clone().unwrap())
                    }
                    _ => err(value.span, "could not parse inserted expression"),
                }
            }
            other => err(
                value.span,
                format!(
                    "#insert needs a string or Code, found {}",
                    self.describe(&other)
                ),
            ),
        }
    }

    /// `Source_Code_Location` constant.
    pub fn build_location_value(
        &mut self,
        ty: TypeId,
        path: &str,
        line: i64,
        col: i64,
        span: Span,
    ) -> Result<Value> {
        let size = self.size_of(ty, span)?;
        let mut agg = Aggregate {
            bytes: vec![0; size as usize],
            relocs: Vec::new(),
        };
        for (name, value) in [
            (
                "fully_pathed_filename",
                Value::String(super::expr::forward_slashes(path).as_bytes().into()),
            ),
            ("line_number", Value::Int(line as i128)),
            ("character_number", Value::Int(col as i128)),
        ] {
            if let Some((off, fty)) = self.member_offset(ty, name, span)? {
                self.write_value(&mut agg, off, &value, fty, span)?;
            }
        }
        Ok(Value::Bytes(Rc::new(agg)))
    }

    // -----------------------------------------------------------------------
    // Reading interpreter memory back
    // -----------------------------------------------------------------------

    /// Read a value of type `ty` at interpreter address `addr` as a constant.
    pub fn read_value(&mut self, addr: u64, ty: TypeId, span: Span) -> Result<Value> {
        match self.types.kind(ty).clone() {
            TypeKind::Bool => Ok(Value::Bool(self.program_bytes(addr, 1, span)?[0] != 0)),
            TypeKind::Int {
                bits,
                signed,
            } => {
                let bytes = self.program_bytes(addr, bits as usize / 8, span)?;
                let mut buf = [0u8; 8];
                buf[..bytes.len()].copy_from_slice(&bytes);
                let raw = u64::from_le_bytes(buf);
                Ok(Value::Int(expr::wrap_int(raw as i128, bits, signed)))
            }
            TypeKind::Float {
                bits: 32,
            } => {
                let b = self.program_bytes(addr, 4, span)?;
                Ok(Value::Float(
                    f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64
                ))
            }
            TypeKind::Float {
                ..
            } => Ok(Value::Float(f64::from_bits(self.program_u64(addr, span)?))),
            TypeKind::Enum(_) | TypeKind::Distinct(_) => {
                let r = self.types.repr(ty);
                if r != ty && self.ir_ty(r).is_some() && !self.types.is_pointer(r) {
                    return self.read_value(addr, r, span);
                }
                self.read_aggregate(addr, ty, span)
            }
            TypeKind::Type => {
                let p = self.program_u64(addr, span)?;
                self.type_at(p, span).map(Value::Type)
            }
            TypeKind::Code => {
                self.adopt_made_codes();
                // Any integer can be cast to `Code`; only ids of real code are usable.
                let id = self.program_u64(addr, span)?;
                if id >= self.codes.len() as u64 {
                    return err(span, format!("{id} is not a valid Code value"));
                }
                Ok(Value::Code(value::CodeId(id as u32)))
            }
            TypeKind::String => {
                let count = self.program_u64(addr, span)? as usize;
                let data = self.program_u64(addr + 8, span)?;
                Ok(Value::String(self.program_bytes(data, count, span)?.into()))
            }
            TypeKind::Proc(_) => {
                let p = self.program_u64(addr, span)?;
                if p == 0 {
                    return Ok(Value::Null);
                }
                if let Some(func) = self.interp.func_of(p)
                    && let Some(&proc) = self.func_procs.get(&func)
                {
                    return Ok(Value::Proc(proc));
                }
                self.read_aggregate(addr, ty, span)
            }
            TypeKind::Pointer(_) | TypeKind::Null => {
                if self.program_u64(addr, span)? == 0 {
                    return Ok(Value::Null);
                }
                self.read_aggregate(addr, ty, span)
            }
            TypeKind::Void => Ok(Value::Void),
            _ => self.read_aggregate(addr, ty, span),
        }
    }

    fn type_at(&self, p: u64, span: Span) -> Result<TypeId> {
        if let Some((g, 0)) = self.interp.global_at(p)
            && let Some(t) = self.type_from_info_global(g)
        {
            return Ok(t);
        }
        err(
            span,
            "compile-time value of type Type does not refer to a type descriptor",
        )
    }

    pub(super) fn read_aggregate(&mut self, addr: u64, ty: TypeId, span: Span) -> Result<Value> {
        let size = self.size_of(ty, span)?;
        let mut agg = Aggregate {
            bytes: self.program_bytes(addr, size as usize, span)?,
            relocs: Vec::new(),
        };
        self.frozen.clear();
        let result = self.freeze(&mut agg, 0, addr, ty, span);
        self.frozen.clear();
        result?;
        Ok(Value::Bytes(Rc::new(agg)))
    }

    /// Turn pointers inside a value read from interpreter memory into relocations,
    /// copying pointed-to data that does not live in a global.
    fn freeze(
        &mut self,
        agg: &mut Aggregate,
        offset: u64,
        addr: u64,
        ty: TypeId,
        span: Span,
    ) -> Result<()> {
        match self.types.kind(ty).clone() {
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let info = self.types.struct_info(s).clone();
                if info.is_union {
                    return Ok(());
                }
                for field in &info.fields {
                    self.freeze(
                        agg,
                        offset + field.offset,
                        addr + field.offset,
                        field.ty,
                        span,
                    )?;
                }
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                let size = self.size_of(elem, span)?;
                for i in 0..n {
                    self.freeze(agg, offset + i * size, addr + i * size, elem, span)?;
                }
            }
            TypeKind::Array {
                elem,
                kind,
            } => {
                let count = self.program_u64(addr, span)?;
                let data = self.program_u64(addr + 8, span)?;
                let esize = self.size_of(elem, span)?;
                let len = if kind == ArrayKind::Resizable {
                    self.program_u64(addr + 16, span)?.max(count)
                } else {
                    count
                };
                self.freeze_pointer(agg, offset + 8, data, elem, esize * len, len, span)?;
                if kind == ArrayKind::Resizable {
                    // The frozen copy is not owned by any allocator.
                    agg.bytes[(offset + 16) as usize..(offset + 40) as usize].fill(0);
                    agg.bytes[(offset + 16) as usize..(offset + 24) as usize]
                        .copy_from_slice(&count.to_le_bytes());
                }
            }
            TypeKind::String => {
                let count = self.program_u64(addr, span)?;
                let data = self.program_u64(addr + 8, span)?;
                self.freeze_pointer(agg, offset + 8, data, TypeId::U8, count, count, span)?;
            }
            TypeKind::Pointer(to) => {
                let p = self.program_u64(addr, span)?;
                let size = if to == TypeId::VOID {
                    0
                } else {
                    self.size_of(to, span)?
                };
                self.freeze_pointer(agg, offset, p, to, size, 1, span)?;
            }
            TypeKind::Type => {
                let p = self.program_u64(addr, span)?;
                if p != 0 {
                    let t = self.type_at(p, span)?;
                    let g = self.type_info_global(t, span)?;
                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Global(g),
                        addend: 0,
                    });
                    agg.bytes[offset as usize..offset as usize + 8].fill(0);
                }
            }
            TypeKind::Proc(_) => {
                let p = self.program_u64(addr, span)?;
                agg.bytes[offset as usize..offset as usize + 8].fill(0);
                if let Some(func) = self.interp.func_of(p) {
                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Func(func),
                        addend: 0,
                    });
                } else if is_integer_handle(p) {
                    // Sentinels cast to a procedure type (`SIG_IGN`, `SIG_ERR`) stay integers.
                    agg.bytes[offset as usize..offset as usize + 8]
                        .copy_from_slice(&p.to_le_bytes());
                } else {
                    return err(
                        span,
                        "a compile-time value holds a foreign procedure address",
                    );
                }
            }
            TypeKind::Any => {
                let t = self.program_u64(addr, span)?;
                let v = self.program_u64(addr + 8, span)?;
                agg.bytes[offset as usize..offset as usize + 16].fill(0);
                if t != 0 {
                    let vt = self.type_at(t, span)?;
                    let g = self.type_info_global(vt, span)?;
                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Global(g),
                        addend: 0,
                    });
                    let size = self.size_of(vt, span)?;
                    self.freeze_pointer(agg, offset + 8, v, vt, size, 1, span)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Replace the pointer at `offset` with a relocation to (a frozen copy of) `count` elements at `p`.
    #[allow(clippy::too_many_arguments)]
    fn freeze_pointer(
        &mut self,
        agg: &mut Aggregate,
        offset: u64,
        p: u64,
        elem: TypeId,
        bytes: u64,
        count: u64,
        span: Span,
    ) -> Result<()> {
        agg.bytes[offset as usize..offset as usize + 8].fill(0);
        if p == 0 {
            return Ok(());
        }
        if let Some((g, off)) = self.interp.global_at(p) {
            agg.relocs.push(ir::Reloc {
                offset,
                target: ir::RelocTarget::Global(g),
                addend: off as i64,
            });
            return Ok(());
        }
        if is_integer_handle(p) {
            // Integers cast to pointers (handle-like constants such as `cast(*void) 32512` or
            // `HKEY_CLASSES_ROOT`) have no memory to freeze.
            agg.bytes[offset as usize..offset as usize + 8].copy_from_slice(&p.to_le_bytes());
            return Ok(());
        }
        if bytes == 0 && count > 0 {
            return err(
                span,
                "a compile-time value holds a pointer to memory of unknown size",
            );
        }
        // Nothing to copy where nothing can be read: an integer cast to a pointer that the
        // range above does not catch (`cast(*T) 80000000`). It stays a number, like those.
        let Some(contents) = self.interp.read(p, bytes as usize) else {
            agg.bytes[offset as usize..offset as usize + 8].copy_from_slice(&p.to_le_bytes());
            return Ok(());
        };
        if let Some(&g) = self.frozen.get(&(p, bytes)) {
            agg.relocs.push(ir::Reloc {
                offset,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
            return Ok(());
        }
        let align = if elem == TypeId::VOID {
            8
        } else {
            self.align_of(elem, span)?
        };
        // Registered before its contents are frozen, so a pointer back to it ends the walk.
        let g = self.program.add_global(ir::Global {
            name: "frozen".into(),
            size: bytes + 1,
            align,
            init: Vec::new(),
            relocs: Vec::new(),
            read_only: false,
            export: None,
        });
        self.frozen.insert((p, bytes), g);
        let mut inner = Aggregate {
            bytes: contents,
            relocs: Vec::new(),
        };
        if elem != TypeId::U8 && elem != TypeId::VOID {
            let esize = self.size_of(elem, span)?;
            for i in 0..count {
                self.freeze(&mut inner, i * esize, p + i * esize, elem, span)?;
            }
        }
        let mut init = inner.bytes;
        init.push(0); // NUL after frozen strings
        let global = &mut self.program.globals[g.0 as usize];
        global.init = init;
        global.relocs = inner.relocs;
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }

    /// The text of a failed `#assert`: its message with each `%` replaced by the next argument
    /// (`#assert(cond, "type % is unsupported", T)`), `\%` kept as a literal percent.
    pub(super) fn assert_message(
        &mut self,
        scope: ScopeId,
        message: Option<&ast::Expr>,
        args: &[ast::Expr],
    ) -> Result<String> {
        let Some(message) = message else {
            return Ok(String::new());
        };
        let Value::String(text) = self.eval_const_value(scope, message)? else {
            return Ok(String::new());
        };
        let text = String::from_utf8_lossy(&text).into_owned();
        if args.is_empty() {
            return Ok(text);
        }
        let mut shown = Vec::new();
        for arg in args {
            shown.push(match self.eval_const_value(scope, arg)? {
                Value::Int(i) => i.to_string(),
                Value::Float(x) => x.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::String(s) => String::from_utf8_lossy(&s).into_owned(),
                Value::Type(t) => self.types.name(t),
                Value::Null => "null".to_string(),
                _ => "?".to_string(),
            });
        }
        let mut out = String::new();
        let mut next = shown.into_iter();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' if chars.peek() == Some(&'%') => out.push(chars.next().unwrap()),
                '%' => out.push_str(&next.next().unwrap_or_default()),
                _ => out.push(c),
            }
        }
        Ok(out)
    }
}

impl Compiler {
    /// `len` bytes of compile-time memory at `addr`, which a `#run` result points to.
    fn program_bytes(&self, addr: u64, len: usize, span: Span) -> Result<Vec<u8>> {
        match self.interp.read(addr, len) {
            Some(bytes) => Ok(bytes),
            None => unreadable(addr, span),
        }
    }

    /// The `u64` at `addr` in compile-time memory.
    fn program_u64(&self, addr: u64, span: Span) -> Result<u64> {
        match self.interp.read_u64(addr) {
            Some(v) => Ok(v),
            None => unreadable(addr, span),
        }
    }
}

/// A `#run` result points at memory that cannot be read (an integer made into a string's or an
/// array's data pointer, say): an error, where reading it would crash the compiler.
fn unreadable<T>(addr: u64, span: Span) -> Result<T> {
    err(
        span,
        format!("a compile-time value points to memory that cannot be read ({addr:#x})"),
    )
}

/// A pointer value that cannot address compile-time memory, so it is an integer cast to a
/// pointer: the first 64 KiB (never mapped), or any address with the top bit set, which on the
/// 64-bit hosts jaic runs on is never user-space memory (`cast(HKEY) cast(s32) 0x80000000`).
fn is_integer_handle(p: u64) -> bool {
    p < 0x10000 || (p as i64) < 0
}
