//! Moves a tree parsed on a worker thread into the compile that asked for it.
//!
//! A worker cannot know the file id sema will give its file, the `AstId`s the file's
//! declarations come after, or the main interner's symbol numbers: all three depend on the order
//! the compile loads files in. It parses with placeholders, and `relocate` rewrites them on the
//! main thread, so the tree is the one a serial parse at that point would have built.
//!
//! Every field holding a `Span`, an `Ident`, a `Sym` or an `AstId` must be visited here. A field
//! left out keeps its worker value, which `parse_ahead`'s tests catch by comparing against a
//! serial parse.
use super::{
    Arg, ArraySize, AsmBlock, AsmDecl, AsmItem, AsmOperand, AsmPin, AsmSize, AstId, Block, Case,
    CodeBody, Decl, EnumItem, Expr, ExprKind, File, ForOver, Ident, Note, Notes, Param, ProcHeader,
    Rc, RunBody, Span, Stmt, StmtKind, StructLit, SwitchStmt, Sym, UsingFilter,
};
use crate::fxhash::HashSet;
use crate::intern::Remap;
use crate::source::FileId;

/// The file id a worker parses with. No real file has it, and `Span::NONE`'s differs.
pub const PLACEHOLDER_FILE: FileId = FileId(u32::MAX - 1);

pub struct Relocator<'a> {
    file: FileId,
    ast_base: u32,
    syms: &'a Remap,
    shared: HashSet<*const ()>,
}

/// Rewrites `file`, parsed under `PLACEHOLDER_FILE` with ids from zero and a job interner, for
/// the compile: spans name `real`, ids continue from `ast_base`, symbols go through `syms`.
pub fn relocate(file: &mut File, real: FileId, ast_base: u32, syms: &Remap) {
    let mut r = Relocator {
        file: real,
        ast_base,
        syms,
        shared: HashSet::default(),
    };
    file.file = real;
    for stmt in &mut file.stmts {
        r.stmt(stmt);
    }
}

/// Mutable access to an `Rc`'s contents. The tree being relocated is owned by one thread and
/// nothing else holds its nodes, so no reference to the contents exists while it is edited.
#[allow(unsafe_code, clippy::mut_from_ref)]
fn rc_mut<T>(rc: &Rc<T>) -> &mut T {
    // SAFETY: see above. A node reachable twice (`strong_count > 1`) is visited once, through
    // `Relocator::once`, so the edits never alias.
    unsafe { &mut *Rc::as_ptr(rc).cast_mut() }
}

impl Relocator<'_> {
    /// True the first time a shared node is met.
    fn once<T>(&mut self, rc: &Rc<T>) -> bool {
        Rc::strong_count(rc) == 1 || self.shared.insert(Rc::as_ptr(rc).cast::<()>())
    }

    fn span(&self, span: &mut Span) {
        if span.file == PLACEHOLDER_FILE {
            span.file = self.file;
        }
    }

    fn sym(&self, sym: &mut Sym) {
        *sym = self.syms.apply(*sym);
    }

    fn id(&self, id: &mut AstId) {
        id.0 += self.ast_base;
    }

    fn ident(&self, ident: &mut Ident) {
        self.sym(&mut ident.name);
        self.span(&mut ident.span);
    }

    fn idents(&self, idents: &mut [Ident]) {
        for ident in idents {
            self.ident(ident);
        }
    }

    fn opt_ident(&self, ident: &mut Option<Ident>) {
        if let Some(ident) = ident {
            self.ident(ident);
        }
    }

    fn notes(&self, notes: &mut [Note]) {
        for note in notes {
            self.span(&mut note.span);
        }
    }

    fn stmt_notes(&self, notes: &mut Notes) {
        if let Some(notes) = &mut notes.0 {
            self.notes(notes);
        }
    }

    fn exprs(&mut self, exprs: &mut [Expr]) {
        for expr in exprs {
            self.expr(expr);
        }
    }

    fn opt_expr(&mut self, expr: &mut Option<Expr>) {
        if let Some(expr) = expr {
            self.expr(expr);
        }
    }

    fn opt_boxed(&mut self, expr: &mut Option<Box<Expr>>) {
        if let Some(expr) = expr {
            self.expr(expr);
        }
    }

    fn args(&mut self, args: &mut [Arg]) {
        for arg in args {
            self.arg(arg);
        }
    }

    fn arg(&mut self, arg: &mut Arg) {
        self.opt_ident(&mut arg.name);
        self.opt_boxed(&mut arg.target);
        self.expr(&mut arg.value);
    }

    fn expr(&mut self, expr: &mut Expr) {
        self.span(&mut expr.span);
        match &mut expr.kind {
            ExprKind::Ident(name)
            | ExprKind::PolyVar {
                name, ..
            } => self.sym(name),
            ExprKind::PolyRestricted {
                name,
                restriction,
                ..
            } => {
                self.sym(name);
                self.expr(restriction);
            }
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::Uninit
            | ExprKind::Context
            | ExprKind::Char(_)
            | ExprKind::CallerLocation
            | ExprKind::CallerCode
            | ExprKind::File
            | ExprKind::Line
            | ExprKind::Filepath
            | ExprKind::This
            | ExprKind::CompileTime => {}
            ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => {
                self.expr(a);
                self.expr(b);
            }
            ExprKind::Unary(_, a)
            | ExprKind::ProcedureOfCall(a)
            | ExprKind::Exists(a)
            | ExprKind::Bytes(a)
            | ExprKind::Backtick(a) => self.expr(a),
            ExprKind::Call {
                callee,
                args,
                ..
            }
            | ExprKind::Bake {
                callee,
                args,
                ..
            } => {
                self.expr(callee);
                self.args(args);
            }
            ExprKind::Member(base, member) => {
                self.expr(base);
                self.ident(member);
            }
            ExprKind::InferredMember(member) => self.ident(member),
            ExprKind::Cast {
                ty,
                value,
                ..
            } => {
                self.opt_boxed(ty);
                self.expr(value);
            }
            ExprKind::Ifx {
                cond,
                then_value,
                else_value,
                ..
            } => {
                self.expr(cond);
                self.opt_boxed(then_value);
                self.opt_boxed(else_value);
            }
            ExprKind::StructLit {
                ty,
                fields,
            } => {
                self.opt_boxed(ty);
                self.args(fields);
            }
            ExprKind::ArrayLit {
                ty,
                elems,
            } => {
                self.opt_boxed(ty);
                self.exprs(elems);
            }
            ExprKind::ArrayType {
                size,
                elem,
            } => {
                if let ArraySize::Fixed(size) = size {
                    self.expr(size);
                }
                self.expr(elem);
            }
            ExprKind::ProcType(header) => self.header(header),
            ExprKind::Proc(lit) => {
                if self.once(lit) {
                    let lit = rc_mut(lit);
                    self.header(&lit.header);
                    if let Some(body) = &mut lit.body {
                        self.block(body);
                    }
                }
            }
            ExprKind::Lambda {
                header,
                body,
            } => {
                self.header(header);
                self.expr(body);
            }
            ExprKind::Block(block) => self.block(block),
            ExprKind::Struct(lit) => self.struct_lit(lit),
            ExprKind::Enum(lit) => {
                if self.once(lit) {
                    let lit = rc_mut(lit);
                    self.id(&mut lit.id);
                    self.opt_expr(&mut lit.base);
                    self.enum_items(&mut lit.items);
                    self.notes(&mut lit.notes);
                    self.span(&mut lit.span);
                }
            }
            ExprKind::TypeDirective {
                ty, ..
            } => self.expr(ty),
            ExprKind::Run {
                body,
                flags,
            } => {
                if self.once(body) {
                    match rc_mut(body) {
                        RunBody::Expr(expr) => self.expr(expr),
                        RunBody::Block(block) => self.block(block),
                    }
                }
                self.idents(flags);
            }
            ExprKind::Code(body) => {
                if self.once(body) {
                    match rc_mut(body) {
                        CodeBody::Expr(expr) => self.expr(expr),
                        CodeBody::Stmt(stmt) => self.stmt(stmt),
                        CodeBody::Block(block) => self.block(block),
                    }
                }
            }
            ExprKind::Insert(insert) => {
                self.expr(&mut insert.value);
                self.idents(&mut insert.flags);
                self.opt_expr(&mut insert.scope);
                self.args(&mut insert.replacements);
            }
            ExprKind::Location(inner) | ExprKind::ProcedureName(inner) => self.opt_boxed(inner),
            ExprKind::UnknownDirective(directive) => {
                self.ident(&mut directive.name);
                self.idents(&mut directive.flags);
                self.opt_expr(&mut directive.operand);
            }
            ExprKind::Asm(block) => {
                if self.once(block) {
                    self.asm(rc_mut(block));
                }
            }
        }
    }

    fn asm(&mut self, block: &mut AsmBlock) {
        self.idents(&mut block.features);
        for item in &mut block.items {
            match item {
                AsmItem::Inst(inst) => {
                    self.ident(&mut inst.mnemonic);
                    match &mut inst.size {
                        Some(AsmSize::Suffix(ident)) => self.ident(ident),
                        Some(AsmSize::Dynamic(expr)) => self.expr(expr),
                        None => {}
                    }
                    for operand in &mut inst.operands {
                        self.asm_operand(operand);
                    }
                    if let Some((mask, _)) = &mut inst.evex.mask {
                        self.asm_operand(mask);
                    }
                    self.span(&mut inst.span);
                }
                AsmItem::Decl(decl) => self.asm_decl(decl),
            }
        }
    }

    fn asm_operand(&mut self, operand: &mut AsmOperand) {
        match operand {
            AsmOperand::Value(expr) => self.expr(expr),
            AsmOperand::Mem(mem) => {
                for term in &mut mem.terms {
                    self.expr(&mut term.value);
                    self.opt_expr(&mut term.scale);
                }
                self.span(&mut mem.span);
            }
            AsmOperand::Decl(decl) => self.asm_decl(decl),
        }
    }

    fn asm_decl(&mut self, decl: &mut AsmDecl) {
        self.ident(&mut decl.name);
        self.opt_ident(&mut decl.class);
        if let Some(AsmPin::Name(name)) = &mut decl.pin {
            self.ident(name);
        }
    }

    fn params(&mut self, params: &mut [Param]) {
        for param in params {
            self.opt_ident(&mut param.name);
            if let Some(filter) = &mut param.using_filter {
                self.filter(filter);
            }
            self.opt_expr(&mut param.ty);
            self.opt_expr(&mut param.default);
            self.notes(&mut param.notes);
            self.span(&mut param.span);
        }
    }

    fn header(&mut self, header: &Rc<ProcHeader>) {
        if !self.once(header) {
            return;
        }
        let header = rc_mut(header);
        self.id(&mut header.id);
        self.params(&mut header.params);
        for ret in &mut header.returns {
            self.opt_ident(&mut ret.name);
            self.opt_expr(&mut ret.ty);
            self.opt_expr(&mut ret.default);
            self.span(&mut ret.span);
        }
        if let Some(Some(library)) = &mut header.flags.elsewhere {
            self.ident(library);
        }
        self.idents(&mut header.flags.other);
        if let Some(foreign) = &mut header.foreign {
            self.opt_ident(&mut foreign.library);
        }
        if let Some(modify) = &mut header.modify {
            self.block(modify);
        }
        self.notes(&mut header.notes);
        self.span(&mut header.span);
    }

    fn struct_lit(&mut self, lit: &Rc<StructLit>) {
        if !self.once(lit) {
            return;
        }
        let lit = rc_mut(lit);
        self.id(&mut lit.id);
        if let Some(tag) = &mut lit.tag {
            self.ident(&mut tag.name);
            self.opt_boxed(&mut tag.ty);
            self.opt_boxed(&mut tag.value);
        }
        self.params(&mut lit.params);
        self.stmts(&mut lit.body);
        self.opt_expr(&mut lit.flags.align);
        if let Some(modify) = &mut lit.modify {
            self.block(modify);
        }
        self.notes(&mut lit.notes);
        self.span(&mut lit.span);
    }

    fn enum_items(&mut self, items: &mut [EnumItem]) {
        for item in items {
            match item {
                EnumItem::Member(member) => {
                    self.ident(&mut member.name);
                    self.opt_expr(&mut member.value);
                    self.notes(&mut member.notes);
                }
                EnumItem::Insert(expr) => self.expr(expr),
                EnumItem::If {
                    cond,
                    then_items,
                    else_items,
                } => {
                    self.expr(cond);
                    self.enum_items(then_items);
                    self.enum_items(else_items);
                }
            }
        }
    }

    fn block(&mut self, block: &mut Block) {
        self.stmts(&mut block.stmts);
        self.span(&mut block.span);
    }

    fn stmts(&mut self, stmts: &mut [Stmt]) {
        for stmt in stmts {
            self.stmt(stmt);
        }
    }

    fn decl(&mut self, decl: &Rc<Decl>) {
        if !self.once(decl) {
            return;
        }
        let decl = rc_mut(decl);
        self.id(&mut decl.id);
        self.idents(&mut decl.names);
        self.opt_expr(&mut decl.ty);
        self.opt_expr(&mut decl.value);
        self.stmt_notes(&mut decl.notes);
        self.span(&mut decl.span);
        if let Some(rare) = &mut decl.rare {
            self.exprs(&mut rare.extra_values);
            for target in rare.targets.iter_mut().flatten() {
                self.expr(target);
            }
            if let Some(filter) = &mut rare.using_filter {
                self.filter(filter);
            }
            self.opt_expr(&mut rare.align);
            self.idents(&mut rare.flags);
            if let Some(foreign) = &mut rare.foreign {
                self.opt_ident(&mut foreign.library);
            }
            self.opt_expr(&mut rare.union_tag);
        }
    }

    fn filter(&mut self, filter: &mut UsingFilter) {
        match filter {
            UsingFilter::None => {}
            UsingFilter::Only(names) | UsingFilter::Except(names) => self.idents(names),
            UsingFilter::Map(pairs) => {
                for (from, to) in pairs {
                    self.ident(from);
                    self.ident(to);
                }
            }
            UsingFilter::Computed {
                expr, ..
            } => self.expr(expr),
        }
    }

    fn case(&mut self, case: &mut Case) {
        self.exprs(&mut case.values);
        self.stmts(&mut case.body);
        self.span(&mut case.span);
    }

    fn switch(&mut self, switch: &mut SwitchStmt) {
        self.expr(&mut switch.value);
        for case in &mut switch.cases {
            self.case(case);
        }
        self.span(&mut switch.header);
    }

    fn stmt(&mut self, stmt: &mut Stmt) {
        self.span(&mut stmt.span);
        self.stmt_notes(&mut stmt.notes);
        match &mut stmt.kind {
            StmtKind::Decl(decl) | StmtKind::AddContext(decl) => self.decl(decl),
            StmtKind::Expr(expr) => self.expr(expr),
            StmtKind::Assign {
                lhs,
                rhs,
                ..
            } => {
                self.exprs(lhs);
                self.exprs(rhs);
            }
            StmtKind::Block(block) => self.block(block),
            StmtKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.expr(cond);
                self.stmt(then_branch);
                if let Some(branch) = else_branch {
                    self.stmt(branch);
                }
            }
            StmtKind::Switch(switch) | StmtKind::StaticSwitch(switch) => self.switch(switch),
            StmtKind::While(w) => {
                self.opt_ident(&mut w.label);
                self.expr(&mut w.cond);
                self.stmt(&mut w.body);
            }
            StmtKind::For(f) => {
                self.opt_ident(&mut f.it);
                self.opt_ident(&mut f.index);
                self.opt_expr(&mut f.pointer_if);
                self.opt_expr(&mut f.reverse_if);
                self.opt_ident(&mut f.iterator);
                match &mut f.over {
                    ForOver::Range(a, b) => {
                        self.expr(a);
                        self.expr(b);
                    }
                    ForOver::Collection(expr) => self.expr(expr),
                }
                self.stmt(&mut f.body);
                self.idents(&mut f.flags);
            }
            StmtKind::Break(label) | StmtKind::Continue(label) | StmtKind::Remove(label) => {
                self.opt_ident(label);
            }
            StmtKind::Return {
                values, ..
            } => self.args(values),
            StmtKind::Defer {
                body, ..
            } => self.stmt(body),
            StmtKind::Using(using) => {
                self.expr(&mut using.value);
                self.filter(&mut using.filter);
            }
            StmtKind::PushContext(push) => {
                self.expr(&mut push.context);
                self.stmt(&mut push.body);
            }
            StmtKind::StaticIf(s) => {
                self.expr(&mut s.cond);
                self.stmts(&mut s.then_branch);
                self.stmts(&mut s.else_branch);
            }
            StmtKind::Insert(insert) => {
                self.expr(&mut insert.value);
                self.idents(&mut insert.flags);
                self.opt_expr(&mut insert.scope);
                self.args(&mut insert.replacements);
            }
            StmtKind::Assert(assert) => {
                self.expr(&mut assert.cond);
                self.opt_expr(&mut assert.message);
                self.exprs(&mut assert.args);
            }
            StmtKind::Run(expr) | StmtKind::Place(expr) | StmtKind::Overlay(expr) => {
                self.expr(expr);
            }
            StmtKind::Import(import) => {
                if self.once(import) {
                    let import = rc_mut(import);
                    self.args(&mut import.params);
                    self.opt_ident(&mut import.name);
                    self.idents(&mut import.flags);
                    if let Some(filter) = &mut import.using {
                        self.filter(filter);
                    }
                    self.span(&mut import.span);
                }
            }
            StmtKind::Load {
                span, ..
            } => self.span(span),
            StmtKind::Scope(_) | StmtKind::Through | StmtKind::Empty => {}
            StmtKind::ModuleParameters(module) => {
                self.params(&mut module.params);
                self.params(&mut module.runtime_params);
                if let Some(body) = &mut module.body {
                    self.block(body);
                }
            }
            StmtKind::Placeholder(names) => self.idents(names),
            StmtKind::Directive(directive) => {
                self.ident(&mut directive.name);
                self.idents(&mut directive.flags);
                self.exprs(&mut directive.args);
            }
            StmtKind::PushContextDefer(push) => self.expr(&mut push.context),
            StmtKind::Case(case) => {
                if self.once(case) {
                    self.case(rc_mut(case));
                }
            }
        }
    }
}
