//! Exporting procedure bodies and headers, and code passed to `#code`.
use super::constants::node;
use super::{Exporter, Typechecked};
use crate::builtin_keys::SyntaxKey;
use crate::records::{FieldRef, Item, RecordTag, Records};
use crate::sema::*;

impl Exporter<'_> {
    /// A `Code_Procedure_Body` record for `body`, belonging to header record `header_id`.
    fn body(&mut self, body: &ast::Block, header_id: i64, proc: Option<ProcId>) -> i64 {
        // The header's arguments and named returns are in scope in the body.
        let locals = self.locals.len();
        for list in ["arguments", "returns"] {
            let Some(FieldRef::List(items)) = self.r.field(header_id, list) else {
                continue;
            };
            let decls: Vec<i64> = items
                .iter()
                .filter_map(|i| match i {
                    Item::Ref(d) => Some(d),
                    _ => None,
                })
                .collect();
            for d in decls {
                if let Some(FieldRef::Str(name)) = self.r.field(d, "name")
                    && !name.is_empty()
                {
                    let name = Sym::intern(&String::from_utf8_lossy(name));
                    self.locals.push((name, d));
                }
            }
        }
        let previous = std::mem::replace(&mut self.procedure, proc);
        let block = self.block(&body.stmts, 1, body.span);
        self.procedure = previous;
        self.locals.truncate(locals);
        let mut rec = self.node(
            RecordTag::CodeProcedureBody,
            node::PROCEDURE_BODY,
            body.span,
        );
        rec.ptr("block", block).ptr("header", header_id);
        let body_id = self.add(rec);
        if let (Some(p), Some(c)) = (proc, self.c.as_deref_mut()) {
            c.export.bodies.insert(body_id, p);
        }
        body_id
    }

    /// A procedure header (and its body). `decl_notes` are the notes of the
    /// declaration naming it: Jai attaches notes after a body to both.
    pub(super) fn proc(
        &mut self,
        lit: &ast::ProcLit,
        decl_notes: &[ast::Note],
        resolved: Option<(ProcId, Option<Rc<procs::Signature>>)>,
        span: Span,
    ) -> i64 {
        let start = self.sub.len();
        let h = &lit.header;
        let sig = resolved.as_ref().and_then(|(_, s)| s.clone());
        let mut arguments = Vec::new();
        let mut usings = Vec::new();
        let mut runtime_index = 0;
        for param in &h.params {
            let ty = self.parameter_type(
                resolved.as_ref().map(|(p, _)| *p),
                param,
                sig.as_deref(),
                &mut runtime_index,
            );
            let decl = ast::Decl {
                id: h.id,
                names: param.name.into_iter().collect(),
                kind: ast::DeclKind::Var,
                ty: param.ty.clone(),
                value: param.default.clone(),
                using: param.using,
                as_: false,
                backtick: false,
                notes: ast::Notes::new(param.notes.clone()),
                span: param.span,
                rare: None,
            };
            let name = param.name.map_or(Sym::intern(""), |n| n.name);
            let arg = self.decl(&decl, name, ty, None, false, 0);
            if let Some(arg) = self.r.get_mut(arg) {
                let flags = i64::from(param.baked)
                    | (i64::from(param.auto_bake) * 0x4000)
                    | (i64::from(param.discard) * 4);
                arg.int("flags", flags);
            }
            if param.variadic {
                self.mark_varargs(arg, param.span);
            }
            arguments.push(arg);
            // `using p: T` parameters are listed again as `Code_Using` of the parameter.
            if param.using {
                let mut ident = self.node(RecordTag::CodeIdent, node::IDENT, param.span);
                ident
                    .str("name", name.as_str().as_bytes())
                    .ptr("resolved_declaration", arg);
                let ident = self.add(ident);
                let (filter_type, filter_expression) = param
                    .using_filter
                    .as_ref()
                    .map(|filter| self.using_filter(filter))
                    .unwrap_or((0, 0));
                let mut using = self.node(RecordTag::CodeUsing, node::USING, param.span);
                using
                    .ptr("expression", ident)
                    .int("filter_type", filter_type)
                    .ptr("filter_expression", filter_expression);
                if let Some(filter) = &param.using_filter {
                    using
                        .str(SyntaxKey::Filter, Self::filter_syntax(filter).as_bytes())
                        .int(
                            SyntaxKey::StaticFilter,
                            if filter_expression == 0 {
                                filter_type
                            } else {
                                0
                            },
                        )
                        .str(
                            SyntaxKey::FilterArguments,
                            Self::filter_arguments(filter).as_bytes(),
                        );
                }
                usings.push(self.add(using));
            }
        }
        let mut returns = Vec::new();
        for (i, ret) in h.returns.iter().enumerate() {
            let ty = sig.as_ref().and_then(|s| s.returns.get(i)).copied();
            let decl = ast::Decl {
                id: h.id,
                names: ret.name.into_iter().collect(),
                kind: ast::DeclKind::Var,
                ty: ret.ty.clone(),
                value: ret.default.clone(),
                using: false,
                as_: false,
                backtick: false,
                notes: ast::Notes::default(),
                span: ret.span,
                rare: None,
            };
            let name = ret.name.map_or(Sym::intern(""), |n| n.name);
            let declaration = self.decl(&decl, name, ty, None, false, 0);
            self.r
                .get_mut(declaration)
                .unwrap()
                .int(SyntaxKey::ReturnMust, i64::from(ret.must));
            returns.push(declaration);
        }
        let mut notes: Vec<&ast::Note> = h.notes.iter().collect();
        for n in decl_notes {
            if !notes.iter().any(|m| m.span == n.span) {
                notes.push(n);
            }
        }
        let notes = self.notes(&notes);
        let polymorphic = match (resolved.as_ref(), self.c.as_deref()) {
            (Some((p, _)), Some(c)) => c.proc(*p).is_poly,
            _ => h
                .params
                .iter()
                .any(|p| p.baked || p.ty.as_ref().is_some_and(procs::has_poly)),
        };
        let flags = procedure_flags(h, polymorphic);
        // One header record per procedure: calls that resolved to it before it was reported
        // (`resolved_header`) and later ones see the reported header, body included.
        let known = match (&resolved, self.c.as_deref()) {
            (Some((p, _)), Some(c)) => c.export.resolved_headers.get(&(*p, self.own)).copied(),
            _ => None,
        };
        let header_id = known.unwrap_or_else(|| self.r.reserve(RecordTag::CodeProcedureHeader));
        let own = self.own;
        if let (Some((p, _)), Some(c)) = (&resolved, self.c.as_deref_mut()) {
            c.export.resolved_headers.insert((*p, own), header_id);
        }
        let mut body_id = 0;
        // A procedure's body is reported once it is lowered; until then only its header.
        let deferred = match (&resolved, self.c.as_deref()) {
            (Some((p, _)), Some(c)) => {
                let info = c.proc(*p);
                lit.body.is_some()
                    && !info.is_poly
                    && !info.is_macro
                    && info.body_state != procs::BodyState::Done
            }
            _ => false,
        };
        if deferred {
            let (p, _) = resolved.as_ref().unwrap();
            let scope = self.scope;
            let c = self.c.as_deref_mut().unwrap();
            c.export.schedule_body(*p, header_id, scope);
        } else if let Some(body) = &lit.body {
            body_id = match self.r.field(header_id, "body_or_null") {
                Some(FieldRef::Ref(body)) if body != 0 => body,
                _ => self.body(body, header_id, resolved.as_ref().map(|(p, _)| *p)),
            };
        }
        let mut rec = self.node(RecordTag::CodeProcedureHeader, node::PROCEDURE_HEADER, span);
        let foreign_name = match h.foreign.as_ref().map(|f| &f.name) {
            Some(ast::ForeignName::Named(n)) => n.as_bytes().to_vec(),
            _ => Vec::new(),
        };
        let name = resolved
            .as_ref()
            .and_then(|(p, _)| Some(self.c.as_deref()?.proc(*p).name.as_str()))
            .unwrap_or("");
        let ty = sig.as_ref().map_or(0, |s| self.ty(s.ty));
        let library = h
            .foreign
            .as_ref()
            .and_then(|foreign| foreign.library)
            .or(h.flags.elsewhere.flatten())
            .map_or(0, |name| self.name_ident(name.name, name.span));
        let modify = self.modify_syntax(h.modify.as_ref());
        let mut extra_flags = String::new();
        for (enabled, name) in [
            (h.flags.no_abc, "no_abc"),
            (h.flags.no_aoc, "no_aoc"),
            (h.flags.runtime_support, "runtime_support"),
            (h.flags.type_info_none, "type_info_none"),
            (
                h.flags.cpp_return_type_is_non_pod,
                "cpp_return_type_is_non_pod",
            ),
        ] {
            if enabled {
                extra_flags.push_str(&format!(" #{name}"));
            }
        }
        for flag in &h.flags.other {
            extra_flags.push_str(&format!(" #{}", flag.name));
        }
        rec.refs("arguments", arguments)
            .refs("parameter_usings", usings)
            .refs("returns", returns)
            .str("name", name.as_bytes())
            .str("foreign_function_name", &foreign_name)
            .ptr("library_identifier", library)
            .refs("modify_directives", modify)
            .int(SyntaxKey::Foreign, i64::from(h.foreign.is_some()))
            .int(
                SyntaxKey::ForeignNamed,
                i64::from(matches!(
                    h.foreign.as_ref().map(|f| &f.name),
                    Some(ast::ForeignName::Named(_))
                )),
            )
            .str(
                SyntaxKey::BuiltinName,
                h.flags.builtin_name.as_deref().unwrap_or("").as_bytes(),
            )
            .str(SyntaxKey::Flags, extra_flags.as_bytes())
            .int(
                SyntaxKey::ProgramExport,
                i64::from(h.flags.program_export.is_some()),
            )
            .int(
                SyntaxKey::ExportNamed,
                i64::from(matches!(h.flags.program_export, Some(Some(_)))),
            )
            .str(
                SyntaxKey::ExportName,
                h.flags
                    .program_export
                    .as_ref()
                    .and_then(|name| name.as_deref())
                    .unwrap_or(&[]),
            )
            .int(
                SyntaxKey::DeprecationNamed,
                i64::from(matches!(h.flags.deprecated, Some(Some(_)))),
            )
            .str(
                "deprecation_string",
                h.flags
                    .deprecated
                    .as_ref()
                    .and_then(|message| message.as_deref())
                    .unwrap_or(&[]),
            )
            .ptr("body_or_null", body_id)
            .ptr("type", ty)
            .int("procedure_flags", flags)
            .refs("notes", notes)
            .int("serial", header_id);
        *self.r.get_mut(header_id).unwrap() = rec;
        self.sub.push(header_id);
        let sub = self.sub[start..].to_vec();
        let first_header = match (resolved.as_ref(), self.c.as_deref_mut()) {
            (Some((p, _)), Some(c)) if !own => c.export.reported_headers.insert(*p),
            (None, Some(_)) if !own => false,
            _ => true,
        };
        if first_header {
            self.out.headers.push((header_id, sub.clone()));
        }
        let first_body = match (resolved.as_ref(), self.c.as_deref_mut()) {
            (Some((p, _)), Some(c)) if !own && body_id != 0 => {
                c.export.scheduled_bodies.insert(*p);
                c.export.reported_bodies.insert(*p)
            }
            (None, Some(_)) if !own => false,
            _ => true,
        };
        if body_id != 0 && first_body {
            self.out.bodies.push((body_id, sub));
        }
        header_id
    }
}

pub(super) fn procedure_flags(h: &ast::ProcHeader, polymorphic: bool) -> i64 {
    let f = &h.flags;
    let mut flags = 0;
    if f.elsewhere.is_some() {
        flags |= 0x1;
    }
    if polymorphic {
        flags |= 0x4;
    }
    if f.c_call {
        flags |= 0x20;
    }
    if f.intrinsic {
        flags |= 0x80;
    }
    if f.deprecated.is_some() {
        flags |= 0x100;
    }
    if f.no_context {
        flags |= 0x200;
    }
    if f.lambda != ast::LambdaKind::None {
        flags |= 0x400;
    }
    if f.cpp_method {
        flags |= 0x1000;
    }
    if f.no_call {
        flags |= 0x2000;
    }
    if f.expand {
        flags |= 0x4000;
    }
    if f.no_debug {
        flags |= 0x8000;
    }
    if f.symmetric {
        flags |= 0x4_0000;
    }
    if f.compiler {
        flags |= 0x40_0000;
    }
    match f.inline {
        ast::CallHintFlag::Inline => flags |= 0x100_0000,
        ast::CallHintFlag::NoInline => flags |= 0x200_0000,
        ast::CallHintFlag::None => {}
    }
    flags
}

/// Records for a `#code` value outside its compiler (`compiler_get_nodes`):
/// the root node (`__source` holds the code's text, so the target compiler
/// can parse it again when the node is inserted) and every node below it.
pub fn export_code(r: &mut Records, body: &ast::CodeBody, source: &str) -> (i64, Vec<i64>) {
    export_code_in(None, ScopeId(0), r, body, source)
}

/// `export_code`, with names resolved and types filled in by `c` when given: identifiers
/// get `resolved_declaration`, calls `resolved_procedure_expression`, both `type`.
pub(super) fn export_code_in(
    c: Option<&mut Compiler>,
    scope: ScopeId,
    r: &mut Records,
    body: &ast::CodeBody,
    source: &str,
) -> (i64, Vec<i64>) {
    let mut out = Typechecked::default();
    let own = c.is_some();
    let mut ex = Exporter {
        c,
        r,
        scope,
        sub: Vec::new(),
        locals: Vec::new(),
        procedure: None,
        compound_members: Vec::new(),
        own,
        out: &mut out,
    };
    let root = match body {
        ast::CodeBody::Expr(e) => ex.expr(e),
        ast::CodeBody::Stmt(stmt) => match ex.stmt(stmt) {
            Some(id) => id,
            None => ex.block(std::slice::from_ref(stmt), 1, stmt.span),
        },
        ast::CodeBody::Block(b) => ex.block(&b.stmts, 1, b.span),
    };
    let sub = std::mem::take(&mut ex.sub);
    if let Some(rec) = r.get_mut(root) {
        rec.str("__source", source.as_bytes());
    }
    (root, sub)
}
