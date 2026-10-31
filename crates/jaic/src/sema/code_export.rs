//! Export of the workspace's syntax trees, types and files as records for
//! metaprograms (`Message_Typechecked`, `Message_File`, `Message_Import`; see
//! `crate::records`).
//!
//! Each top-level declaration of a user module (anything outside the system
//! module directory and Preload) is resolved and exported once. Already resolved
//! library declarations and called concrete bodies also participate, sharing
//! canonical headers and body scheduling across modules. Lowered expression results
//! retain their checked procedure-instance types; untyped syntax remains untyped.
//! The record of every exported procedure body remembers its procedure, so
//! `compiler_modify_procedure` can map a modified statement list back.
mod asm_syntax;
mod constants;
mod decls;
mod expr;
mod files;
mod modify;
mod names;
mod procs;
mod state;
mod stmt;
mod syntax;
mod type_records;
mod typechecked;

pub use self::constants::message_kind;
use self::constants::node;
pub use self::procs::export_code;
pub use self::state::{ExportState, FileEvent, Modification};
pub use self::typechecked::Contents;
use super::{Compiler, ProcId, ScopeId, Span, Sym, TypeId, ast};
use crate::builtin_keys::SyntaxKey;
use crate::lang_items;
use crate::records::{FieldRef, Name, Record, RecordTag, Records};

/// The names of the fields every node record has, interned once per thread. A node is made
/// for each piece of syntax exported, so they are not looked up by text each time.
#[derive(Clone, Copy)]
struct NodeNames {
    kind: Name,
    enclosing_load: Name,
    l0: Name,
    c0: Name,
    l1: Name,
    c1: Name,
    serial: Name,
}

thread_local! {
    static NODE_NAMES: NodeNames = NodeNames {
        kind: Name::intern("kind"),
        enclosing_load: Name::intern("enclosing_load"),
        l0: Name::intern("l0"),
        c0: Name::intern("c0"),
        l1: Name::intern("l1"),
        c1: Name::intern("c1"),
        serial: Name::intern("serial"),
    };
}

#[derive(Default)]
struct Typechecked {
    declarations: Vec<(i64, Vec<i64>)>,
    headers: Vec<(i64, Vec<i64>)>,
    bodies: Vec<(i64, Vec<i64>)>,
    structs: Vec<(i64, Vec<i64>)>,
}

pub(super) struct Exporter<'a> {
    /// `None` for `#code` exported from the interpreter (no types or locations).
    c: Option<&'a mut Compiler>,
    r: &'a mut Records,
    /// Scope type expressions are evaluated in.
    scope: ScopeId,
    /// Every node record made for the current top-level item.
    sub: Vec<i64>,
    /// Declaration records in scope inside the tree being exported (parameters and
    /// earlier local declarations, innermost last): what an identifier resolves to first.
    locals: Vec<(Sym, i64)>,
    /// Body whose checked results are being exported (templates have no snapshot).
    procedure: Option<ProcId>,
    /// The records are for this compiler's own compile-time code (`compiler_get_nodes`):
    /// types refer to its real descriptors.
    own: bool,
    /// The name declarations of the compound declaration just exported, for the block's
    /// `members`.
    compound_members: Vec<i64>,
    out: &'a mut Typechecked,
}

fn bin_op(op: ast::BinOp) -> i64 {
    use ast::BinOp as B;
    match op {
        B::Add => b'+' as i64,
        B::Sub => b'-' as i64,
        B::Mul => b'*' as i64,
        B::Div => b'/' as i64,
        B::Rem => b'%' as i64,
        B::BitAnd => b'&' as i64,
        B::BitOr => b'|' as i64,
        B::BitXor => b'^' as i64,
        B::Lt => b'<' as i64,
        B::Gt => b'>' as i64,
        B::Eq => 131,
        B::Ne => 132,
        B::And => 133,
        B::Or => 134,
        B::Le => 135,
        B::Ge => 136,
        B::Shl => 137,
        B::Shr => 138,
        B::Rotl => 139,
        B::Rotr => 140,
    }
}

fn assign_op(op: ast::AssignOp) -> i64 {
    use ast::BinOp as B;
    match op {
        ast::AssignOp::Assign => b'=' as i64,
        ast::AssignOp::Op(op) => match op {
            B::Add => 145,
            B::Sub => 146,
            B::Mul => 147,
            B::Div => 148,
            B::Rem => 149,
            B::Shl => 150,
            B::Shr => 151,
            B::Rotl => 152,
            B::Rotr => 153,
            B::BitAnd => 154,
            B::BitOr => 155,
            B::BitXor => 156,
            B::And => 157,
            B::Or => 158,
            other => bin_op(other),
        },
    }
}

impl Exporter<'_> {
    fn eval_type(&mut self, expr: &ast::Expr) -> Option<TypeId> {
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        let ast::ExprKind::Ident(name) = &expr.kind else {
            return c.eval_type(scope, expr).ok();
        };
        c.meta.export.refresh_name_caches(c.ns.entities.len());
        if let Some(&cached) = c.meta.export.type_names.get(&(scope, *name)) {
            return cached;
        }
        let found = c.eval_type(scope, expr).ok();
        // Evaluating may have added entities, which drops the caches; the result is still good.
        c.meta.export.refresh_name_caches(c.ns.entities.len());
        c.meta.export.type_names.insert((scope, *name), found);
        found
    }

    /// A node record with the `Code_Node` fields filled in.
    fn node(&mut self, tag: RecordTag, kind: i64, span: Span) -> Record {
        let names = NODE_NAMES.with(|names| *names);
        // The common fields (kind, location, serial, type) and a node's own few: growing
        // from empty reallocated most records two or three times.
        let mut rec = Record::with_capacity(tag, 12);
        rec.int(names.kind, kind).int(SyntaxKey::OriginalKind, kind);
        let Some(c) = self.c.as_deref_mut() else {
            return rec;
        };
        if (span.file.0 as usize) < c.src.sources.len() {
            // Nodes come in runs from one file: remember the last file's module.
            let module = match c.meta.export.last_file_module {
                Some((file, module)) if file == span.file => module,
                _ => match c.src.files.iter().find(|f| f.id == span.file) {
                    Some(f) => {
                        c.meta.export.last_file_module = Some((span.file, f.module));
                        f.module
                    }
                    None => c.scope(self.scope).module,
                },
            };
            let file = c.file_record_in(self.r, span.file, module);
            let source = c.src.sources.get(span.file);
            let (l0, c0) = source.line_col(span.start);
            let (l1, c1) = source.line_col(span.end);
            rec.ptr(names.enclosing_load, file)
                .int(names.l0, l0 as i64)
                .int(names.c0, c0 as i64)
                .int(names.l1, l1 as i64)
                .int(names.c1, c1 as i64);
        }
        rec
    }

    fn add(&mut self, mut rec: Record) -> i64 {
        let id = self.r.reserve(rec.tag);
        rec.int(NODE_NAMES.with(|names| names.serial), id);
        *self.r.get_mut(id).unwrap() = rec;
        self.sub.push(id);
        id
    }

    fn ty(&mut self, ty: TypeId) -> i64 {
        let Some(c) = self.c.as_deref_mut() else {
            return 0;
        };
        if self.own {
            // Read by this compiler's own compile-time code: its real type descriptor, so
            // `get_type` gives back a `Type` the compiler knows.
            if let Some(&id) = c.meta.export.own_types.get(&ty) {
                return id;
            }
            let addr = c
                .type_info_global(ty, Span::NONE)
                .ok()
                .and_then(|g| c.ct.interp.global_addr(&c.out.program, g).ok());
            if let Some(addr) = addr {
                let mut rec = Record::new(RecordTag::TypeInfo);
                rec.int(lang_items::record_fields::ADDRESS, addr as i64);
                let id = self.r.add(rec);
                c.meta.export.own_types.insert(ty, id);
                return id;
            }
        }
        c.type_record(self.r, ty)
    }

    fn notes(&mut self, notes: &[&ast::Note]) -> Vec<i64> {
        notes
            .iter()
            .map(|n| {
                let mut rec = self.node(RecordTag::CodeNote, node::NOTE, n.span);
                rec.str("text", n.text.as_bytes());
                self.add(rec)
            })
            .collect()
    }

    fn ident(&mut self, name: Sym, span: Span) -> i64 {
        let mut rec = self.node(RecordTag::CodeIdent, node::IDENT, span);
        rec.str("name", name.as_str().as_bytes());
        if let Some(decl) = self.resolve_name(name) {
            rec.ptr("resolved_declaration", decl);
            if let Some(FieldRef::Ref(ty)) = self.r.field(decl, "type") {
                rec.ptr("type", ty);
            }
        }
        self.add(rec)
    }

    /// A `Code_Ident` that is not a name of the scope: a member (`a.b`, `.B`), a struct
    /// literal's field, a named argument or a loop label. Looking it up in the scope would
    /// miss after the slowest search, or find an unrelated global of the same name.
    fn name_ident(&mut self, name: Sym, span: Span) -> i64 {
        let mut rec = self.node(RecordTag::CodeIdent, node::IDENT, span);
        rec.str("name", name.as_str().as_bytes());
        self.add(rec)
    }
}
