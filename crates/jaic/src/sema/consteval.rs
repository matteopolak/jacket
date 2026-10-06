//! Compile-time execution: `#run`, constant thunks, global initializers,
//! `#insert`, and reading interpreter memory back into constant values.
use super::lower::{FnCtx, Operand};
use super::value::Aggregate;
use super::*;
use crate::interp::FUNC_TAG;
use crate::ir::Ty;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// A fresh compile-time function context. Thunks take the context pointer.
    pub fn thunk_ctx(&self, name: &str, file: FileId) -> FnCtx {
        let sig = ir::Sig {
            params: vec![Ty::Ptr],
            returns: vec![],
            conv: ir::Conv::Jai,
            c_varargs: false,
            c_fixed: 0,
            c_abi: None,
        };
        let mut f = FnCtx::new(name.into(), sig, file);
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
            result = self.interp.call(&self.program, id, &[ctx]);
        }
        (self.interp.code_exports, self.interp.code_export_cursor) = outer_exports;
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
                let mut d = Diagnostic::error(
                    span,
                    format!("error during compile-time execution: {}", trap.message),
                );
                if let Some((file, line, _)) = trap.loc {
                    d = d.with_note(
                        span,
                        format!(
                            "while executing {}:{line}",
                            self.sources.get(FileId(file)).path
                        ),
                    );
                }
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
            return err(span, format!("'{name}' not found"));
        };
        match self.resolve_entity(id)? {
            scope::Resolved::Const {
                value: Value::Type(t),
                ..
            } => Ok(t),
            _ => err(span, format!("'{name}' is not a type")),
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
                let mut f = self.thunk_ctx("run", file);
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
                let mut f = self.thunk_ctx("run", file);
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
            let (cond, message, scope) = self.asserts[self.asserts_done].clone();
            let misses = self.placeholder_misses;
            let holds = self.eval_static_condition(scope, &cond);
            if holds.is_err() && !self.placeholders_final && self.placeholder_misses != misses {
                return Ok(());
            }
            self.asserts_done += 1;
            if !holds? {
                let msg = match message {
                    Some(m) => match self.eval_const_value(scope, &m)? {
                        Value::String(s) => String::from_utf8_lossy(&s).into_owned(),
                        _ => String::new(),
                    },
                    None => String::new(),
                };
                return err(
                    cond.span,
                    format!(
                        "#assert failed{}{msg}",
                        if msg.is_empty() {
                            ""
                        } else {
                            ": "
                        }
                    ),
                );
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
        let mut agg = Aggregate {
            bytes: vec![0; size as usize],
            relocs: Vec::new(),
        };
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
                let file = self.sources.add(
                    format!("<#insert at {}>", self.sources.get(span.file).path),
                    text.clone(),
                );
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
                let file = self.sources.add(
                    format!("<#insert at {}>", self.sources.get(value.span.file).path),
                    text.clone().into(),
                );
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
        for (name, value, vty) in [
            (
                "fully_pathed_filename",
                Value::String(path.as_bytes().into()),
                TypeId::STRING,
            ),
            ("line_number", Value::Int(line as i128), TypeId::S64),
            ("character_number", Value::Int(col as i128), TypeId::S64),
        ] {
            if let Some((p, fty)) = self.find_member(ty, Sym::intern(name), span)? {
                let off: u64 = p
                    .iter()
                    .map(|s| {
                        if let structs::PathStep::Offset(o) = s {
                            *o
                        } else {
                            0
                        }
                    })
                    .sum();
                let _ = vty;
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
            TypeKind::Bool => Ok(Value::Bool(self.interp.read(addr, 1)[0] != 0)),
            TypeKind::Int {
                bits,
                signed,
            } => {
                let bytes = self.interp.read(addr, bits as usize / 8);
                let mut buf = [0u8; 8];
                buf[..bytes.len()].copy_from_slice(&bytes);
                let raw = u64::from_le_bytes(buf);
                Ok(Value::Int(expr::wrap_int(raw as i128, bits, signed)))
            }
            TypeKind::Float {
                bits: 32,
            } => {
                let b = self.interp.read(addr, 4);
                Ok(Value::Float(
                    f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64
                ))
            }
            TypeKind::Float {
                ..
            } => Ok(Value::Float(f64::from_bits(self.interp.read_u64(addr)))),
            TypeKind::Enum(_) | TypeKind::Distinct(_) => {
                let r = self.types.repr(ty);
                if r != ty && self.ir_ty(r).is_some() && !self.types.is_pointer(r) {
                    return self.read_value(addr, r, span);
                }
                self.read_aggregate(addr, ty, span)
            }
            TypeKind::Type => {
                let p = self.interp.read_u64(addr);
                self.type_at(p, span).map(Value::Type)
            }
            TypeKind::Code => {
                self.adopt_made_codes();
                Ok(Value::Code(
                    value::CodeId(self.interp.read_u64(addr) as u32),
                ))
            }
            TypeKind::String => {
                let count = self.interp.read_u64(addr) as usize;
                let data = self.interp.read_u64(addr + 8);
                Ok(Value::String(self.interp.read(data, count).into()))
            }
            TypeKind::Proc(_) => {
                let p = self.interp.read_u64(addr);
                if p == 0 {
                    return Ok(Value::Null);
                }
                if p & 0xFFFF_0000_0000_0000 == FUNC_TAG {
                    let func = ir::FuncId((p & 0xFFFF_FFFF) as u32);
                    if let Some(&proc) = self.func_procs.get(&func) {
                        return Ok(Value::Proc(proc));
                    }
                }
                self.read_aggregate(addr, ty, span)
            }
            TypeKind::Pointer(_) | TypeKind::Null => {
                if self.interp.read_u64(addr) == 0 {
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
            bytes: self.interp.read(addr, size as usize),
            relocs: Vec::new(),
        };
        self.freeze(&mut agg, 0, addr, ty, span)?;
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
                let count = self.interp.read_u64(addr);
                let data = self.interp.read_u64(addr + 8);
                let esize = self.size_of(elem, span)?;
                let len = if kind == ArrayKind::Resizable {
                    self.interp.read_u64(addr + 16).max(count)
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
                let count = self.interp.read_u64(addr);
                let data = self.interp.read_u64(addr + 8);
                self.freeze_pointer(agg, offset + 8, data, TypeId::U8, count, count, span)?;
            }
            TypeKind::Pointer(to) => {
                let p = self.interp.read_u64(addr);
                let size = if to == TypeId::VOID {
                    0
                } else {
                    self.size_of(to, span)?
                };
                self.freeze_pointer(agg, offset, p, to, size, 1, span)?;
            }
            TypeKind::Type => {
                let p = self.interp.read_u64(addr);
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
                let p = self.interp.read_u64(addr);
                agg.bytes[offset as usize..offset as usize + 8].fill(0);
                if p & 0xFFFF_0000_0000_0000 == FUNC_TAG {
                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Func(ir::FuncId((p & 0xFFFF_FFFF) as u32)),
                        addend: 0,
                    });
                } else if p < 0x10000 || p.wrapping_neg() < 0x10000 {
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
                let t = self.interp.read_u64(addr);
                let v = self.interp.read_u64(addr + 8);
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
        if p < 0x10000 || p.wrapping_neg() < 0x10000 {
            // Integers cast to pointers (handle-like constants such as `cast(*void) 32512`) have no memory to freeze.
            agg.bytes[offset as usize..offset as usize + 8].copy_from_slice(&p.to_le_bytes());
            return Ok(());
        }
        if bytes == 0 && count > 0 {
            return err(
                span,
                "a compile-time value holds a pointer to memory of unknown size",
            );
        }
        let mut inner = Aggregate {
            bytes: self.interp.read(p, bytes as usize),
            relocs: Vec::new(),
        };
        if elem != TypeId::U8 && elem != TypeId::VOID {
            let esize = self.size_of(elem, span)?;
            for i in 0..count {
                self.freeze(&mut inner, i * esize, p + i * esize, elem, span)?;
            }
        }
        let align = if elem == TypeId::VOID {
            8
        } else {
            self.align_of(elem, span)?
        };
        let mut init = inner.bytes;
        init.push(0); // NUL after frozen strings
        let g = self.program.add_global(ir::Global {
            name: "frozen".into(),
            size: init.len() as u64,
            align,
            init,
            relocs: inner.relocs,
            read_only: false,
            export: None,
        });
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }
}
