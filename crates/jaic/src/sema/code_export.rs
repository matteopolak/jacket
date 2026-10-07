//! Export of the workspace's syntax trees, types and files as records for
//! metaprograms (`Message_Typechecked`, `Message_File`, `Message_Import`; see
//! `crate::records`).
//!
//! Each top-level declaration of a user module (anything outside the system
//! module directory and Preload) is resolved and exported once, with the
//! procedure headers, bodies and structs it contains. Global declarations get
//! their resolved types; local declarations and other expressions do not.
//! The record of every exported procedure body remembers its procedure, so
//! `compiler_modify_procedure` can map a modified statement list back.
use super::scope::{EntityKind, Resolved, ScopeKind};
use super::*;
use crate::records::{Field, Item, Record, Records};
use crate::types::{ArrayKind, TypeKind};

/// Values of `Message.kind`.
pub mod message_kind {
    pub const FILE: i64 = 1;
    pub const IMPORT: i64 = 2;
    pub const TYPECHECKED: i64 = 5;
}

/// Values of `Code_Node.Kind`.
mod node {
    pub const BLOCK: i64 = 1;
    pub const LITERAL: i64 = 2;
    pub const IDENT: i64 = 3;
    pub const UNARY_OPERATOR: i64 = 4;
    pub const BINARY_OPERATOR: i64 = 5;
    pub const PROCEDURE_BODY: i64 = 6;
    pub const PROCEDURE_CALL: i64 = 7;
    pub const CONTEXT: i64 = 8;
    pub const WHILE: i64 = 9;
    pub const IF: i64 = 10;
    pub const LOOP_CONTROL: i64 = 11;
    pub const CASE: i64 = 12;
    pub const RETURN: i64 = 14;
    pub const FOR: i64 = 15;
    pub const TYPE_INSTANTIATION: i64 = 17;
    pub const ENUM: i64 = 18;
    pub const PROCEDURE_HEADER: i64 = 19;
    pub const STRUCT: i64 = 20;
    pub const DECLARATION: i64 = 25;
    pub const COMMA_SEPARATED_ARGUMENTS: i64 = 21;
    pub const COMPOUND_DECLARATION: i64 = 48;
    pub const CAST: i64 = 26;
    pub const DIRECTIVE_IMPORT: i64 = 27;
    pub const DIRECTIVE_RUN: i64 = 31;
    pub const DIRECTIVE_EXISTS: i64 = 55;
    pub const DIRECTIVE_CODE: i64 = 32;
    pub const DIRECTIVE_LOCATION: i64 = 44;
    pub const NOTE: i64 = 40;
    pub const DEFER: i64 = 49;
    pub const USING: i64 = 50;
    pub const PUSH_CONTEXT: i64 = 39;
    pub const PLACEHOLDER: i64 = 51;
    pub const DIRECTIVE_INSERT: i64 = 52;
}

/// A statement of a body given to `compiler_modify_procedure`.
#[derive(Debug, Clone)]
pub enum ModifiedStmt {
    /// A statement the compiler exported, unchanged.
    Record(i64),
    /// A new or edited statement, printed by the metaprogram.
    Source(String),
}

/// Exporter state kept by a workspace's compiler between messages.
#[derive(Default)]
pub struct ExportState {
    files: HashMap<FileId, i64>,
    modules: HashMap<ModuleId, i64>,
    types: HashMap<TypeId, i64>,
    files_announced: usize,
    /// Entities before this index were considered for TYPECHECKED messages.
    next_entity: usize,
    /// Directory of the system modules (`None` inside: not known).
    system_dir: Option<Option<PathBuf>>,
    system_modules: HashMap<ModuleId, bool>,
    /// Procedure of each exported procedure-body record.
    pub bodies: HashMap<i64, ProcId>,
    /// Statement behind each exported statement record.
    pub stmts: HashMap<i64, Rc<ast::Stmt>>,
    /// Procedures whose header went out but whose body is not lowered yet:
    /// (procedure, header record, scope for type expressions). The body is reported
    /// in a later TYPECHECKED message, once lowered (like Jai, unused bodies never are).
    pending_bodies: Vec<(ProcId, i64, ScopeId)>,
    /// Records of this compiler's own type descriptors (`__address`), by type.
    own_types: HashMap<TypeId, i64>,
    /// Modified procedures to lower again (`relower_modified`).
    relower: Vec<ProcId>,
    /// The header record of each procedure (shared by `resolved_procedure_expression` and
    /// reported headers), by procedure and whether it is for this compiler's own
    /// compile-time code (`Exporter::own`: those types carry real descriptors).
    resolved_headers: HashMap<(ProcId, bool), i64>,
    /// Declarations made for `resolved_declaration` of names outside the exported tree.
    resolved_decls: HashMap<EntityId, i64>,
}

impl ExportState {
    /// Procedures whose header was reported and whose body is still waiting.
    pub fn pending_procs(&self) -> impl Iterator<Item = ProcId> + '_ {
        self.pending_bodies.iter().map(|&(p, _, _)| p)
    }
}

/// A FILE or IMPORT message: (event kind, message record).
pub type FileEvent = (i64, i64);

impl Compiler {
    /// FILE (and, before a module's first file, IMPORT) messages for files
    /// loaded since the last call.
    pub fn export_file_events(&mut self, r: &mut Records) -> Vec<FileEvent> {
        let mut events = Vec::new();
        while self.export.files_announced < self.files.len() {
            let info = &self.files[self.export.files_announced];
            let (file, module) = (info.id, info.module);
            self.export.files_announced += 1;
            if !self.export.modules.contains_key(&module) {
                let id = self.module_record(r, module);
                events.push((message_kind::IMPORT, id));
            }
            let id = self.file_record_in(r, file, module);
            events.push((message_kind::FILE, id));
        }
        events
    }

    fn file_module(&self, file: FileId) -> ModuleId {
        self.files
            .iter()
            .find(|f| f.id == file)
            .map_or(ModuleId(0), |f| f.module)
    }

    fn module_record(&mut self, r: &mut Records, m: ModuleId) -> i64 {
        if let Some(&id) = self.export.modules.get(&m) {
            return id;
        }
        let module_type = if Some(m) == self.preload {
            1
        } else if Some(m) == self.runtime_support {
            2
        } else if Some(m) == self.main_module {
            3
        } else {
            0
        };
        let module = &self.modules[m.0 as usize];
        let mut rec = Record::new("Message_Import");
        rec.int("kind", message_kind::IMPORT)
            .int("workspace", self.workspace)
            .int("module_type", module_type)
            .str("module_name", module.name.as_bytes())
            .int("__module", m.0 as i64);
        let id = r.add(rec);
        self.export.modules.insert(m, id);
        id
    }

    pub fn file_record(&mut self, r: &mut Records, file: FileId) -> i64 {
        if let Some(&id) = self.export.files.get(&file) {
            return id;
        }
        let module = self.file_module(file);
        self.file_record_in(r, file, module)
    }

    /// `file_record` for a file whose module is known (finding it searches every file).
    fn file_record_in(&mut self, r: &mut Records, file: FileId, module: ModuleId) -> i64 {
        if let Some(&id) = self.export.files.get(&file) {
            return id;
        }
        let import = self.module_record(r, module);
        let path = self.sources.get(file).path.clone();
        let mut rec = Record::new("Message_File");
        rec.int("kind", message_kind::FILE)
            .int("workspace", self.workspace)
            .str("fully_pathed_filename", path.as_bytes())
            .ptr("enclosing_import", import)
            .int("from_a_string", path.starts_with('<') as i64)
            .int("__file", file.0 as i64);
        let id = r.add(rec);
        self.export.files.insert(file, id);
        id
    }

    fn is_system_module(&mut self, m: ModuleId) -> bool {
        if Some(m) == self.preload || Some(m) == self.runtime_support {
            return true;
        }
        if self.export.system_dir.is_none() {
            let dir = self
                .find_module("Runtime_Support", std::path::Path::new(""))
                .and_then(|p| p.parent().map(|d| self.fs.canonical(d)));
            self.export.system_dir = Some(dir);
        }
        if let Some(&system) = self.export.system_modules.get(&m) {
            return system;
        }
        let system = match (&self.export.system_dir, &self.modules[m.0 as usize].path) {
            (Some(Some(dir)), Some(path)) => self.fs.canonical(path).starts_with(dir),
            _ => false,
        };
        self.export.system_modules.insert(m, system);
        system
    }

    /// A TYPECHECKED message for user declarations not exported yet, or `None`.
    pub fn export_typechecked(&mut self, r: &mut Records) -> Option<i64> {
        let mut decls = Vec::new();
        while self.export.next_entity < self.entities.len() {
            let id = EntityId(self.export.next_entity as u32);
            self.export.next_entity += 1;
            let entity = self.entity(id);
            let EntityKind::Decl {
                decl,
                index,
            } = &entity.kind
            else {
                continue;
            };
            let (decl, index, home) = (decl.clone(), *index, entity.home);
            let scope = self.scope(entity.scope);
            if !matches!(scope.kind, ScopeKind::Module | ScopeKind::File)
                || index >= decl.names.len()
            {
                continue;
            }
            let module = scope.module;
            if self.is_system_module(module) {
                continue;
            }
            decls.push((id, decl, index, home));
        }
        // Bodies lowered since their header was reported.
        let pending = std::mem::take(&mut self.export.pending_bodies);
        let (ready, waiting): (Vec<_>, Vec<_>) = pending
            .into_iter()
            .partition(|(p, _, _)| self.proc(*p).body_state == procs::BodyState::Done);
        self.export.pending_bodies = waiting;
        if decls.is_empty() && ready.is_empty() {
            return None;
        }
        let mut out = Typechecked::default();
        for (p, header_id, scope) in ready {
            let lit = self.proc(p).lit.clone();
            let Some(body) = &lit.body else {
                continue;
            };
            let mut ex = Exporter {
                c: Some(self),
                r,
                scope,
                sub: Vec::new(),
                locals: Vec::new(),
                compound_members: Vec::new(),
                own: false,
                out: &mut out,
            };
            let body_id = ex.body(body, header_id, Some(p));
            let sub = std::mem::take(&mut ex.sub);
            if let Some(header) = r.get_mut(header_id) {
                header.ptr("body_or_null", body_id);
            }
            out.bodies.push((body_id, sub));
        }
        for (id, decl, index, home) in decls {
            let Ok(resolved) = self.resolve_entity(id) else {
                continue;
            };
            // Jai only reports what typechecked: a procedure whose signature fails is an
            // error there, not a message.
            if let Resolved::Proc(p) = resolved
                && !self.proc(p).is_poly
                && self.signature(p, decl.span).is_err()
            {
                continue;
            }
            let (rec, sub) = {
                let mut ex = Exporter {
                    c: Some(self),
                    r,
                    scope: home,
                    sub: Vec::new(),
                    locals: Vec::new(),
                    compound_members: Vec::new(),
                    own: false,
                    out: &mut out,
                };
                let rec = ex.global_decl(&decl, index, &resolved);
                (rec, ex.sub)
            };
            out.declarations.push((rec, sub));
        }
        let mut message = Record::new("Message_Typechecked");
        let group = |r: &mut Records, items: &[(i64, Vec<i64>)]| -> Vec<Item> {
            items
                .iter()
                .map(|(expr, sub)| {
                    let mut t = Record::new("Typechecked");
                    t.ptr("expression", *expr)
                        .refs("subexpressions", sub.iter().copied());
                    Item::Ref(r.add(t))
                })
                .collect()
        };
        let all: Vec<(i64, Vec<i64>)> = out
            .declarations
            .iter()
            .chain(&out.headers)
            .chain(&out.bodies)
            .chain(&out.structs)
            .cloned()
            .collect();
        message
            .int("kind", message_kind::TYPECHECKED)
            .int("workspace", self.workspace)
            .list("declarations", group(r, &out.declarations))
            .list("procedure_headers", group(r, &out.headers))
            .list("procedure_bodies", group(r, &out.bodies))
            .list("structs", group(r, &out.structs))
            .list("others", Vec::new())
            .list("all", group(r, &all));
        Some(r.add(message))
    }

    /// Export `Code` value `code` with resolved names and types for `compiler_get_nodes`
    /// (asked for by the interpreter, see `call_thunk`). The registry's records are taken
    /// out meanwhile, as resolving declarations may run compile-time code.
    pub fn export_code_typed(&mut self, code: usize) {
        self.adopt_made_codes();
        // `code_of(f)`: f's locals have types once its body is lowered.
        if let Some(&p) = self.code_procs.get(&code)
            && !self.proc(p).is_poly
            && !self.proc(p).is_macro
            && self.proc_func(p, self.proc(p).span).is_ok()
            && self.proc(p).body_state == procs::BodyState::Queued
        {
            let _ = self.lower_with_callees(p);
        }
        let Some(shared) = self.interp.workspaces.clone() else {
            return;
        };
        let (Some(body), Some(&scope), Some((_, text))) = (
            self.codes.get(code).cloned(),
            self.code_scopes.get(code),
            self.interp.codes.get(code).cloned(),
        ) else {
            return;
        };
        // No further requests while exporting: nested compile-time code gets plain nodes.
        let outer = self.interp.run_effects.take();
        let mut records = Records::lend(&mut shared.borrow_mut().records);
        let exported = export_code_in(Some(self), scope, &mut records, &body, &text);
        Records::give_back(&mut shared.borrow_mut().records, records);
        self.interp.run_effects = outer;
        self.interp
            .code_exports
            .entry(code)
            .or_default()
            .push(exported);
    }

    /// `compiler_modify_procedure`: give the procedure of body record `body`
    /// the statements `stmts` (statement records exported from this
    /// workspace, or `#code` roots from `compiler_get_nodes`, parsed again
    /// here). A body already lowered is lowered again.
    pub fn modify_procedure(
        &mut self,
        r: &Records,
        body: i64,
        stmts: &[ModifiedStmt],
    ) -> Result<()> {
        // Bodies of procedure literals without a procedure of their own (nested
        // in other bodies) cannot be modified; they keep their code.
        let Some(&p) = self.export.bodies.get(&body) else {
            return Ok(());
        };
        if self.proc(p).body_state == procs::BodyState::Lowering {
            return Ok(());
        }
        let old = self.proc(p).lit.clone();
        let Some(old_body) = &old.body else {
            return Ok(());
        };
        let mut new_stmts = Vec::new();
        for stmt in stmts {
            let id = match stmt {
                ModifiedStmt::Record(id) => *id,
                ModifiedStmt::Source(text) => {
                    new_stmts.extend(self.parse_inserted_code(text)?);
                    continue;
                }
            };
            if let Some(stmt) = self.export.stmts.get(&id) {
                new_stmts.push((**stmt).clone());
                continue;
            }
            let Some(Item::Str(text)) = r.item(id, "__source", None) else {
                return err(
                    old_body.span,
                    "compiler_modify_procedure: a statement is neither from this procedure nor from compiler_get_nodes",
                );
            };
            new_stmts.extend(self.parse_inserted_code(&String::from_utf8_lossy(text))?);
        }
        let lit = ast::ProcLit {
            header: old.header.clone(),
            body: Some(ast::Block {
                stmts: new_stmts,
                span: old_body.span,
                no_abc: old_body.no_abc,
                no_aoc: old_body.no_aoc,
            }),
        };
        self.procs[p.0 as usize].lit = Rc::new(lit);
        if self.proc(p).body_state == procs::BodyState::Done {
            self.export.relower.push(p);
        }
        Ok(())
    }

    /// Lower again the bodies `modify_procedure` replaced after they were lowered. Separate,
    /// so the caller can release the registry first: the new bodies may run compile-time
    /// code that uses it.
    pub fn relower_modified(&mut self) -> Result<()> {
        for p in std::mem::take(&mut self.export.relower) {
            self.lower_body(p)?;
        }
        Ok(())
    }

    /// Statements of the text of a `#code` value, as source of this compiler.
    fn parse_inserted_code(&mut self, text: &str) -> Result<Vec<ast::Stmt>> {
        let source = format!("__jaic_inserted :: () {{\n{text};\n}}\n");
        let label = format!("<compiler_get_nodes {}>", self.sources.len());
        let file = self.sources.add(label, source.as_str().into());
        let parsed = crate::parser::parse_file(file, &source).map_err(Box::new)?;
        let body = parsed.stmts.into_iter().find_map(|s| match s.kind {
            ast::StmtKind::Decl(d) => match d.value.as_ref().map(|v| &v.kind) {
                Some(ast::ExprKind::Proc(lit)) => lit.body.clone(),
                _ => None,
            },
            _ => None,
        });
        match body {
            Some(b) => Ok(b.stmts),
            None => err(
                Span::NONE,
                "compiler_modify_procedure: could not parse inserted code",
            ),
        }
    }

    /// `specified_parameters`, `constant_storage` and `polymorph_source_struct`
    /// of an instance of a polymorphic struct. Pointers in the constant storage
    /// (`Type` parameters) are listed in `__constant_pointers` as (offset,
    /// record) pairs for the metaprogram to patch.
    fn poly_parameters(&mut self, r: &mut Records, s: crate::types::StructId, rec: &mut Record) {
        let Some(span) = self.struct_asts.get(&s).map(|src| src.lit.span) else {
            return;
        };
        let mut storage = value::Aggregate {
            bytes: Vec::new(),
            relocs: Vec::new(),
        };
        let mut params = Vec::new();
        let mut pointers = Vec::new();
        for (name, value, mut ty) in self.poly_struct_bindings(s) {
            if self.size_of(ty, span).is_err() {
                ty = self.type_of_value(&value); // Untyped literal arguments.
            }
            let (Ok(size), Ok(align)) = (self.size_of(ty, span), self.align_of(ty, span)) else {
                continue;
            };
            let offset = storage.bytes.len().next_multiple_of(align.max(1) as usize);
            storage.bytes.resize(offset + size as usize, 0);
            if let Value::Type(t) = value {
                let t = self.type_record(r, t);
                pointers.extend([Item::Int(offset as i64), Item::Int(t)]);
            } else if self
                .write_value(&mut storage, offset as u64, &value, ty, span)
                .is_err()
            {
                continue;
            }
            let ty = self.type_record(r, ty);
            let mut m = Record::new("Type_Info_Struct_Member");
            m.str("name", name.as_str().as_bytes())
                .ptr("type", ty)
                .int("flags", 0x1)
                .int("offset_into_constant_storage", offset as i64);
            params.push(r.add(m));
        }
        let mut generic = Record::new("Type_Info_Struct");
        generic
            .int("type", 7)
            .str("name", self.types.struct_info(s).name.as_str().as_bytes())
            .int("nontextual_flags", 0x100);
        rec.refs("specified_parameters", params)
            .list(
                "constant_storage",
                storage.bytes.iter().map(|&b| Item::Int(b as i64)).collect(),
            )
            .list("__constant_pointers", pointers)
            .ptr("polymorph_source_struct", r.add(generic));
    }

    /// The record of a type descriptor (`Type_Info_*`). Primitive types carry
    /// `__builtin`, their name, so the metaprogram can use its own descriptor.
    pub fn type_record(&mut self, r: &mut Records, ty: TypeId) -> i64 {
        if let Some(&id) = self.export.types.get(&ty) {
            return id;
        }
        let kind = self.types.kind(ty).clone();
        let span = Span::NONE;
        let size = match &kind {
            TypeKind::Struct(s) => {
                let _ = self.layout_struct(*s, span);
                self.size_of(ty, span).map_or(-1, |s| s as i64)
            }
            _ => self.size_of(ty, span).map_or(0, |s| s as i64),
        };
        let builtin = matches!(
            kind,
            TypeKind::Void
                | TypeKind::Bool
                | TypeKind::Int { .. }
                | TypeKind::Float { .. }
                | TypeKind::WideFloat(_)
                | TypeKind::String
                | TypeKind::Type
                | TypeKind::Any
                | TypeKind::Code
        );
        let (tag, info_tag) = match &kind {
            TypeKind::Int {
                ..
            } => ("Type_Info_Integer", 0),
            TypeKind::Float {
                ..
            }
            | TypeKind::WideFloat(_) => ("Type_Info_Float", 1),
            TypeKind::Bool => ("Type_Info", 2),
            TypeKind::String => ("Type_Info_String", 3),
            TypeKind::Pointer(_) | TypeKind::Null => ("Type_Info_Pointer", 4),
            TypeKind::Proc(_) => ("Type_Info_Procedure", 5),
            TypeKind::Void => ("Type_Info", 6),
            TypeKind::Struct(_) => ("Type_Info_Struct", 7),
            TypeKind::Array {
                ..
            } => ("Type_Info_Array", 8),
            TypeKind::CompileTimeOnly => ("Type_Info", 9),
            TypeKind::Any => ("Type_Info", 10),
            TypeKind::Enum(_) => ("Type_Info_Enum", 11),
            TypeKind::Type => ("Type_Info", 13),
            TypeKind::Code => ("Type_Info", 14),
            TypeKind::Distinct(_) => ("Type_Info_Variant", 18),
        };
        // Registered before the fields: types may refer to themselves.
        let id = r.reserve(tag);
        self.export.types.insert(ty, id);
        let mut rec = Record::new(tag);
        rec.int("type", info_tag).int("runtime_size", size);
        if builtin {
            rec.str("__builtin", self.types.name(ty).as_bytes());
        }
        match kind {
            TypeKind::Int {
                signed, ..
            } => {
                rec.int("signed", signed as i64);
            }
            TypeKind::Pointer(to) => {
                let to = self.type_record(r, to);
                rec.ptr("pointer_to", to);
            }
            TypeKind::Array {
                elem,
                kind,
            } => {
                let elem = self.type_record(r, elem);
                let (array_type, count) = match kind {
                    ArrayKind::Fixed(n) => (0, n as i64),
                    ArrayKind::View => (1, -1),
                    ArrayKind::Resizable => (2, -1),
                };
                rec.ptr("element_type", elem)
                    .int("array_type", array_type)
                    .int("array_count", count);
            }
            TypeKind::Proc(p) => {
                let args: Vec<i64> = p.params.iter().map(|&t| self.type_record(r, t)).collect();
                let rets: Vec<i64> = p.returns.iter().map(|&t| self.type_record(r, t)).collect();
                let flags = ((p.no_context as i64) * 0x8) | ((p.c_call as i64) * 0x20);
                rec.refs("argument_types", args)
                    .refs("return_types", rets)
                    .int("procedure_flags", flags);
            }
            TypeKind::Enum(e) => {
                let info = self.types.enum_info(e).clone();
                let base = self.type_record(r, info.base);
                rec.str("name", info.name.as_str().as_bytes())
                    .ptr("internal_type", base)
                    .list(
                        "names",
                        info.members
                            .iter()
                            .map(|(n, _)| Item::Str(n.as_str().as_bytes().into()))
                            .collect(),
                    )
                    .list(
                        "values",
                        info.members
                            .iter()
                            .map(|&(_, v)| Item::Int(v as i64))
                            .collect(),
                    )
                    .int("enum_type_flags", info.type_flags());
            }
            TypeKind::Distinct(d) => {
                let info = self.types.distincts[d.0 as usize].clone();
                let base = self.type_record(r, info.base);
                rec.str("name", info.name.as_str().as_bytes())
                    .ptr("variant_of", base)
                    .int(
                        "variant_flags",
                        if info.isa {
                            0x2
                        } else {
                            0x1
                        },
                    );
            }
            TypeKind::Struct(s) => {
                let info = self.types.struct_info(s).clone();
                let mut members = Vec::new();
                for field in &info.fields {
                    let fty = self.type_record(r, field.ty);
                    let mut m = Record::new("Type_Info_Struct_Member");
                    m.str("name", field.name.map_or("", |n| n.as_str()).as_bytes())
                        .ptr("type", fty)
                        .int("offset_in_bytes", field.offset as i64)
                        .int(
                            "flags",
                            ((field.using as i64) * 0x4) | ((field.as_ as i64) * 0x10),
                        )
                        .list(
                            "notes",
                            field
                                .notes
                                .iter()
                                .map(|n| Item::Str(n.as_bytes().into()))
                                .collect(),
                        )
                        .int("offset_into_constant_storage", -1);
                    members.push(r.add(m));
                }
                rec.str("name", info.name.as_str().as_bytes())
                    .refs("members", members)
                    .int("textual_flags", (info.is_union as i64) * 0x2);
                if !info.poly_args.is_empty() {
                    self.poly_parameters(r, s, &mut rec);
                }
            }
            _ => {}
        }
        *r.get_mut(id).unwrap() = rec;
        id
    }
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
        self.c.as_deref_mut()?.eval_type(scope, expr).ok()
    }

    /// A node record with the `Code_Node` fields filled in.
    fn node(&mut self, tag: &'static str, kind: i64, span: Span) -> Record {
        let mut rec = Record::new(tag);
        rec.int("kind", kind);
        let Some(c) = self.c.as_deref_mut() else {
            return rec;
        };
        if (span.file.0 as usize) < c.sources.len() {
            let file = c.file_record(self.r, span.file);
            let source = c.sources.get(span.file);
            let (l0, c0) = source.line_col(span.start);
            let (l1, c1) = source.line_col(span.end);
            rec.ptr("enclosing_load", file)
                .int("l0", l0 as i64)
                .int("c0", c0 as i64)
                .int("l1", l1 as i64)
                .int("c1", c1 as i64);
        }
        rec
    }

    fn add(&mut self, mut rec: Record) -> i64 {
        let id = self.r.reserve(rec.tag);
        rec.int("serial", id);
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
            if let Some(&id) = c.export.own_types.get(&ty) {
                return id;
            }
            let addr = c
                .type_info_global(ty, Span::NONE)
                .ok()
                .and_then(|g| c.interp.global_addr(&c.program, g).ok());
            if let Some(addr) = addr {
                let mut rec = Record::new("Type_Info");
                rec.int("__address", addr as i64);
                let id = self.r.add(rec);
                c.export.own_types.insert(ty, id);
                return id;
            }
        }
        c.type_record(self.r, ty)
    }

    fn notes(&mut self, notes: &[&ast::Note]) -> Vec<i64> {
        notes
            .iter()
            .map(|n| {
                let mut rec = self.node("Code_Note", node::NOTE, n.span);
                rec.str("text", n.text.as_bytes());
                self.add(rec)
            })
            .collect()
    }

    fn ident(&mut self, name: Sym, span: Span) -> i64 {
        let mut rec = self.node("Code_Ident", node::IDENT, span);
        rec.str("name", name.as_str().as_bytes());
        if let Some(decl) = self.resolve_name(name) {
            rec.ptr("resolved_declaration", decl);
            if let Some(Field::Item(Item::Ref(ty))) = self.r.field(decl, "type") {
                rec.ptr("type", *ty);
            }
        }
        self.add(rec)
    }

    /// The declaration record `name` refers to here: a parameter or local of the tree being
    /// exported, else what the scope's lookup finds (a declaration made for it once).
    fn resolve_name(&mut self, name: Sym) -> Option<i64> {
        if let Some(&(_, decl)) = self.locals.iter().rev().find(|(n, _)| *n == name) {
            return Some(decl);
        }
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        let id = *c.lookup(scope, name).ok()?.first()?;
        if let Some(&decl) = c.export.resolved_decls.get(&id) {
            return Some(decl);
        }
        let entity = c.entity(id);
        let span = entity.span;
        let (ty, header, flags) = match entity.kind.clone() {
            EntityKind::Local {
                ty, ..
            } => (Some(ty), None, 0),
            // Builtin procedures (`size_of`, `type_of`...) have no declaration.
            EntityKind::Builtin(_) => return None,
            _ => match c.resolve_entity(id).ok()? {
                Resolved::Proc(p)
                | Resolved::Const {
                    value: Value::Proc(p),
                    ..
                } => {
                    let ty = c.signature(p, span).ok().map(|s| s.ty);
                    (ty, Some(p), 0x1 | 0x10_0000)
                }
                Resolved::ProcSet(ps) => (None, ps.first().copied(), 0x1 | 0x10_0000),
                Resolved::Const {
                    ty, ..
                } => (Some(ty), None, 0x1 | 0x10_0000),
                Resolved::Global {
                    ty, ..
                } => (Some(ty), None, 0x10_0000),
                Resolved::PolyStruct(_) => (Some(TypeId::TYPE), None, 0x1 | 0x10_0000),
                Resolved::Module(_) | Resolved::Library(_) => (None, None, 0x1 | 0x10_0000),
            },
        };
        let expression = header.map_or(0, |p| self.resolved_header(p));
        let decl = self.reference_decl(name, ty, expression, flags, span);
        self.c
            .as_deref_mut()?
            .export
            .resolved_decls
            .insert(id, decl);
        Some(decl)
    }

    /// A `Code_Declaration` that only describes a name (type, value) for resolution: not part
    /// of the tree being exported, so not in `sub`.
    fn reference_decl(
        &mut self,
        name: Sym,
        ty: Option<TypeId>,
        expression: i64,
        flags: i64,
        span: Span,
    ) -> i64 {
        let ty = ty.map_or(0, |t| self.ty(t));
        let mut inst = self.node("Code_Type_Instantiation", node::TYPE_INSTANTIATION, span);
        inst.ptr("result", ty).ptr("type", ty);
        let inst = self.r.add(inst);
        let mut rec = self.node("Code_Declaration", node::DECLARATION, span);
        rec.str("name", name.as_str().as_bytes())
            .ptr("type", ty)
            .ptr(
                "type_inst",
                if ty != 0 {
                    inst
                } else {
                    0
                },
            )
            .ptr("expression", expression)
            .int("flags", flags);
        let id = self.r.reserve(rec.tag);
        rec.int("serial", id);
        *self.r.get_mut(id).unwrap() = rec;
        id
    }

    /// Set `.VARARGS` in the `inst_flags` of a variadic parameter's declaration (`args: ..Any`),
    /// which checkers such as Check's @PrintLike read; adds the instantiation when the
    /// declaration has none (a polymorphic header's unresolved type).
    fn mark_varargs(&mut self, decl: i64, span: Span) {
        let inst = match self.r.get(decl).and_then(|d| d.field("type_inst")) {
            Some(Field::Item(Item::Ref(id))) => *id,
            _ => {
                let rec = self.node("Code_Type_Instantiation", node::TYPE_INSTANTIATION, span);
                let id = self.r.reserve(rec.tag);
                let mut rec = rec;
                rec.int("serial", id);
                *self.r.get_mut(id).unwrap() = rec;
                if let Some(d) = self.r.get_mut(decl) {
                    d.set("type_inst", Field::Item(Item::Ref(id)));
                }
                id
            }
        };
        if let Some(rec) = self.r.get_mut(inst) {
            rec.set("inst_flags", Field::Item(Item::Int(0x1)));
        }
    }

    /// The header `resolved_procedure_expression` points to: name, typed arguments and
    /// returns, flags. Made once per procedure; no body.
    fn resolved_header(&mut self, p: ProcId) -> i64 {
        let own = self.own;
        let Some(c) = self.c.as_deref_mut() else {
            return 0;
        };
        if let Some(&h) = c.export.resolved_headers.get(&(p, own)) {
            return h;
        }
        let id = self.r.reserve("Code_Procedure_Header");
        c.export.resolved_headers.insert((p, own), id);
        let info = c.proc(p);
        let (name, lit, span, is_poly) = (info.name, info.lit.clone(), info.span, info.is_poly);
        let sig = if is_poly {
            None
        } else {
            c.signature(p, span).ok()
        };
        let h = &lit.header;
        let mut arguments = Vec::new();
        for (i, param) in h.params.iter().enumerate() {
            let ty = sig.as_ref().and_then(|s| s.params.get(i)).map(|p| p.ty);
            let name = param.name.map_or(Sym::intern(""), |n| n.name);
            let arg = self.reference_decl(name, ty, 0, 0, param.span);
            if param.variadic {
                self.mark_varargs(arg, param.span);
            }
            arguments.push(arg);
        }
        let mut returns = Vec::new();
        for (i, ret) in h.returns.iter().enumerate() {
            let ty = sig.as_ref().and_then(|s| s.returns.get(i)).copied();
            let name = ret.name.map_or(Sym::intern(""), |n| n.name);
            returns.push(self.reference_decl(name, ty, 0, 0, ret.span));
        }
        let mut flags = 0;
        if h.flags.elsewhere.is_some() {
            flags |= 0x1;
        }
        if is_poly {
            flags |= 0x4;
        }
        if h.flags.c_call {
            flags |= 0x20;
        }
        if h.flags.expand {
            flags |= 0x4000;
        }
        let ty = sig.as_ref().map_or(0, |s| self.ty(s.ty));
        // Checkers such as MetaThreadSafe read the callee's notes (`@thread`).
        let decl_notes = (self.c.as_deref())
            .and_then(|c| c.proc_decl_notes.get(&p).cloned())
            .unwrap_or_default();
        let mut notes: Vec<&ast::Note> = h.notes.iter().collect();
        for n in &decl_notes {
            if !notes.iter().any(|m| m.span == n.span) {
                notes.push(n);
            }
        }
        let notes = self.notes(&notes);
        let mut rec = self.node("Code_Procedure_Header", node::PROCEDURE_HEADER, span);
        rec.refs("arguments", arguments)
            .refs("returns", returns)
            .refs("notes", notes)
            .str("name", name.as_str().as_bytes())
            .ptr("type", ty)
            .int("procedure_flags", flags)
            .int("serial", id);
        *self.r.get_mut(id).unwrap() = rec;
        id
    }

    /// The procedure a call to `callee` with `args` resolves to: the one candidate, or the
    /// first whose parameter count fits the arguments.
    fn resolve_call(&mut self, callee: &ast::Expr, args: usize) -> Option<ProcId> {
        let ast::ExprKind::Ident(name) = &callee.kind else {
            return None;
        };
        let name = *name;
        if self.locals.iter().any(|(n, _)| *n == name) {
            return None;
        }
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        let mut procs = Vec::new();
        for id in c.lookup(scope, name).ok()? {
            if matches!(c.entity(id).kind, EntityKind::Builtin(_)) {
                continue;
            }
            match c.resolve_entity(id) {
                Ok(
                    Resolved::Proc(p)
                    | Resolved::Const {
                        value: Value::Proc(p),
                        ..
                    },
                ) => procs.push(p),
                Ok(Resolved::ProcSet(ps)) => procs.extend(ps),
                _ => {}
            }
        }
        if procs.len() <= 1 {
            return procs.first().copied();
        }
        procs.into_iter().find(|&p| {
            let params = &c.proc(p).lit.header.params;
            let required = params
                .iter()
                .filter(|p| p.default.is_none() && !p.variadic)
                .count();
            let variadic = params.iter().any(|p| p.variadic);
            args >= required && (variadic || args <= params.len())
        })
    }

    /// A `Code_Type_Instantiation` for a type expression, with its resolved type when it has one.
    fn type_inst(&mut self, expr: &ast::Expr) -> i64 {
        let mut rec = self.node(
            "Code_Type_Instantiation",
            node::TYPE_INSTANTIATION,
            expr.span,
        );
        if !procs::has_poly(expr)
            && let Some(t) = self.eval_type(expr)
        {
            let t = self.ty(t);
            rec.ptr("result", t);
            rec.ptr("type", t);
        }
        let value = self.expr(expr);
        rec.ptr("type_valued_expression", value);
        self.add(rec)
    }

    fn opt_expr(&mut self, e: Option<&ast::Expr>) -> i64 {
        e.map_or(0, |e| self.expr(e))
    }

    fn literal(&mut self, span: Span, value_type: i64) -> Record {
        let mut rec = self.node("Code_Literal", node::LITERAL, span);
        rec.int("value_type", value_type);
        rec
    }

    fn expr(&mut self, e: &ast::Expr) -> i64 {
        use ast::ExprKind as E;
        let span = e.span;
        let rec = match &e.kind {
            E::Ident(name) => return self.ident(*name, span),
            E::Int(v) => {
                let mut rec = self.literal(span, 1);
                rec.int("_s64", *v as i64).int("value_flags", 0x1);
                rec
            }
            E::Char(v) => {
                let mut rec = self.literal(span, 1);
                rec.int("_s64", *v as i64).int("value_flags", 0x1);
                rec
            }
            E::Float(v) => {
                let mut rec = self.literal(span, 1);
                rec.int("_float64", v.to_bits() as i64)
                    .int("value_flags", 0x1 | 0x4);
                rec
            }
            E::Str(s) => {
                let mut rec = self.literal(span, 2);
                rec.str("_string", s);
                rec
            }
            E::Bool(b) => {
                let mut rec = self.literal(span, 3);
                rec.int("_s64", *b as i64);
                rec
            }
            E::Null => {
                let mut info = Record::new("Code_Pointer_Literal_Info");
                info.int("pointer_literal_type", 2);
                let info = self.r.add(info);
                let mut rec = self.literal(span, 8);
                rec.ptr("pointer_literal_info", info);
                rec
            }
            E::Uninit => self.literal(span, 0),
            E::Context => self.node("Code_Context", node::CONTEXT, span),
            E::Binary(op, a, b) => {
                let (a, b) = (self.expr(a), self.expr(b));
                let mut rec = self.node("Code_Binary_Operator", node::BINARY_OPERATOR, span);
                rec.int("operator_type", bin_op(*op))
                    .ptr("left", a)
                    .ptr("right", b);
                rec
            }
            // Jai folds a negated number literal into one literal (`#code case -1` is a
            // Code_Literal whose `_s64` metaprograms overwrite).
            E::Unary(ast::UnOp::Neg, a) if matches!(a.kind, E::Int(_) | E::Float(_)) => {
                match &a.kind {
                    E::Int(v) => {
                        let mut rec = self.literal(span, 1);
                        rec.int("_s64", (*v as i64).wrapping_neg())
                            .int("value_flags", 0x1);
                        rec
                    }
                    E::Float(v) => {
                        let mut rec = self.literal(span, 1);
                        rec.int("_float64", (-v).to_bits() as i64)
                            .int("value_flags", 0x1 | 0x4);
                        rec
                    }
                    _ => unreachable!(),
                }
            }
            E::Unary(op, a) => {
                let a = self.expr(a);
                let op = match op {
                    ast::UnOp::Neg => b'-' as i64,
                    ast::UnOp::Plus => b'+' as i64,
                    ast::UnOp::Not => b'!' as i64,
                    ast::UnOp::BitNot => b'~' as i64,
                    ast::UnOp::Star => b'*' as i64,
                    ast::UnOp::Deref => 169,
                };
                let mut rec = self.node("Code_Unary_Operator", node::UNARY_OPERATOR, span);
                rec.int("operator_type", op).ptr("subexpression", a);
                rec
            }
            E::Member(a, name) => {
                let (a, b) = (self.expr(a), self.ident(name.name, name.span));
                let mut rec = self.node("Code_Binary_Operator", node::BINARY_OPERATOR, span);
                rec.int("operator_type", b'.' as i64)
                    .ptr("left", a)
                    .ptr("right", b);
                rec
            }
            E::InferredMember(name) => {
                let b = self.ident(name.name, name.span);
                let mut rec = self.node("Code_Binary_Operator", node::BINARY_OPERATOR, span);
                rec.int("operator_type", b'.' as i64).ptr("right", b);
                rec
            }
            E::Index(a, b) => {
                let (a, b) = (self.expr(a), self.expr(b));
                let mut rec = self.node("Code_Binary_Operator", node::BINARY_OPERATOR, span);
                rec.int("operator_type", 500).ptr("left", a).ptr("right", b);
                rec
            }
            E::Call {
                callee,
                args,
                ..
            } => {
                let resolved = self.resolve_call(callee, args.len());
                let callee = self.expr(callee);
                let (unsorted, sorted) = self.args(args);
                let mut rec = self.node("Code_Procedure_Call", node::PROCEDURE_CALL, span);
                rec.ptr("procedure_expression", callee)
                    .refs("arguments_unsorted", unsorted)
                    .refs("arguments_sorted", sorted);
                if let Some(p) = resolved {
                    let header = self.resolved_header(p);
                    rec.ptr("resolved_procedure_expression", header);
                    // The call's type: its first return.
                    let first = match self.r.item(header, "returns", Some(0)) {
                        Some(Item::Ref(ret)) => Some(*ret),
                        _ => None,
                    };
                    if let Some(Field::Item(Item::Ref(ty))) =
                        first.and_then(|ret| self.r.field(ret, "type"))
                    {
                        rec.ptr("type", *ty);
                    }
                }
                rec
            }
            E::Cast {
                ty,
                value,
                flags,
            } => {
                let target = ty.as_ref().map_or(0, |t| self.type_inst(t));
                let value = self.expr(value);
                let mut cast_flags = 0;
                if ty.is_none() {
                    cast_flags |= 0x4;
                }
                if flags.no_check {
                    cast_flags |= 0x8;
                }
                if flags.truncate {
                    cast_flags |= 0x10;
                }
                if flags.force {
                    cast_flags |= 0x80;
                }
                let mut rec = self.node("Code_Cast", node::CAST, span);
                rec.ptr("target_type", target)
                    .ptr("expression", value)
                    .int("cast_flags", cast_flags);
                rec
            }
            E::Ifx {
                cond,
                then_value,
                else_value,
                is_static,
            } => {
                let cond = self.expr(cond);
                let then_block = then_value.as_deref().map_or(0, |v| self.expr_block(v));
                let else_block = else_value.as_deref().map_or(0, |v| self.expr_block(v));
                let mut rec = self.node("Code_If", node::IF, span);
                rec.ptr("condition", cond)
                    .ptr("then_block", then_block)
                    .ptr("else_block", else_block)
                    .int("if_flags", 0x2 | ((*is_static as i64) * 0x4));
                rec
            }
            E::StructLit {
                ty,
                fields,
            } => {
                let ty = ty.as_ref().map_or(0, |t| self.type_inst(t));
                // `name = value` fields are `=` binary operators, as in Jai.
                let mut values = Vec::new();
                for field in fields {
                    let value = self.expr(&field.value);
                    let target = match (&field.name, &field.target) {
                        (Some(n), _) => self.ident(n.name, n.span),
                        (None, Some(t)) => self.expr(t),
                        (None, None) => {
                            values.push(value);
                            continue;
                        }
                    };
                    let mut rec = self.node(
                        "Code_Binary_Operator",
                        node::BINARY_OPERATOR,
                        field.value.span,
                    );
                    rec.int("operator_type", i64::from(b'='))
                        .ptr("left", target)
                        .ptr("right", value);
                    values.push(self.add(rec));
                }
                let mut info = Record::new("Code_Struct_Literal_Info");
                info.ptr("type_expression", ty).refs("arguments", values);
                let info = self.r.add(info);
                let mut rec = self.literal(span, 7);
                rec.ptr("struct_literal_info", info);
                rec
            }
            E::ArrayLit {
                ty,
                elems,
            } => {
                let elem_ty = ty.as_ref().map_or(0, |t| self.type_inst(t));
                let elems: Vec<i64> = elems.iter().map(|e| self.expr(e)).collect();
                let mut info = Record::new("Code_Array_Literal_Info");
                info.ptr("element_type", elem_ty)
                    .refs("array_members", elems)
                    .int("array_literal_flags", ty.is_none() as i64);
                let info = self.r.add(info);
                let mut rec = self.literal(span, 6);
                rec.ptr("array_literal_info", info);
                rec
            }
            E::ArrayType {
                ..
            }
            | E::ProcType(_)
            | E::TypeDirective {
                ..
            } => {
                let mut rec = self.node("Code_Type_Instantiation", node::TYPE_INSTANTIATION, span);
                if let Some(t) = self.eval_type(e) {
                    let t = self.ty(t);
                    rec.ptr("result", t);
                }
                rec
            }
            E::Proc(lit) => return self.proc(lit, &[], None, span),
            E::Lambda {
                header, ..
            } => {
                let lit = ast::ProcLit {
                    header: header.clone(),
                    body: None,
                };
                return self.proc(&lit, &[], None, span);
            }
            E::Block(block) => return self.block(&block.stmts, 1, block.span),
            E::Struct(lit) => return self.struct_lit(lit, None, Sym::intern("")),
            E::Enum(lit) => return self.enum_lit(lit, None, span),
            E::Run {
                ..
            } => self.node("Code_Directive_Run", node::DIRECTIVE_RUN, span),
            E::Code(body) => {
                let expr = match &**body {
                    ast::CodeBody::Expr(e) => self.expr(e),
                    // `#code if x == {...}`: a brace-less statement is its own node (its
                    // block has the statement's span).
                    ast::CodeBody::Block(b) if b.stmts.len() == 1 && b.span == b.stmts[0].span => {
                        match self.stmt(&b.stmts[0]) {
                            Some(id) => id,
                            None => self.block(&b.stmts, 1, b.span),
                        }
                    }
                    ast::CodeBody::Block(b) => self.block(&b.stmts, 1, b.span),
                };
                let mut rec = self.node("Code_Directive_Code", node::DIRECTIVE_CODE, span);
                rec.ptr("expression", expr);
                rec
            }
            E::Insert {
                value, ..
            } => {
                let value = self.expr(value);
                let mut rec = self.node("Code_Directive_Insert", node::DIRECTIVE_INSERT, span);
                rec.ptr("expression", value);
                rec
            }
            E::Location(e) => {
                let e = self.opt_expr(e.as_deref());
                let mut rec = self.node("Code_Directive_Location", node::DIRECTIVE_LOCATION, span);
                rec.ptr("expression", e);
                rec
            }
            E::CallerLocation => {
                let mut rec = self.node("Code_Directive_Location", node::DIRECTIVE_LOCATION, span);
                rec.int("is_caller_location", 1);
                rec
            }
            E::Backtick(e) => {
                let id = self.expr(e);
                if matches!(e.kind, E::Ident(_))
                    && let Some(rec) = self.r.get_mut(id)
                {
                    rec.set("flags", Field::Item(Item::Int(0x40))); // HAS_SCOPE_MODIFIER
                }
                return id;
            }
            E::Exists(query) => {
                let query = self.expr(query);
                let mut rec = self.node("Code_Directive_Exists", node::DIRECTIVE_EXISTS, span);
                rec.ptr("query_expression", query);
                rec
            }
            _ => self.node("Code_Node", node::PLACEHOLDER, span),
        };
        let mut rec = rec;
        // Typechecked expressions carry their type (Epic_Fail compares operand types).
        if rec.field("type").is_none()
            && matches!(
                e.kind,
                E::Binary(..)
                    | E::Unary(..)
                    | E::Int(_)
                    | E::Float(_)
                    | E::Str(_)
                    | E::Char(_)
                    | E::Bool(_)
                    | E::Index(..)
                    | E::Member(..)
                    | E::Cast { .. }
            )
            && let Some(ty) = self.expr_type(e)
        {
            let ty = self.ty(ty);
            if ty != 0 {
                rec.ptr("type", ty);
            }
        }
        self.add(rec)
    }

    /// The type `e` checks to in the exported code's scope, when it checks there.
    fn expr_type(&mut self, e: &ast::Expr) -> Option<TypeId> {
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        let ty = c.check_expr_no_emit(scope, e).ok()?.ty();
        (ty != TypeId::COMPILE_TIME).then_some(ty)
    }

    /// `Code_Argument` records and the argument values.
    fn args(&mut self, args: &[ast::Arg]) -> (Vec<i64>, Vec<i64>) {
        let mut unsorted = Vec::new();
        let mut values = Vec::new();
        for arg in args {
            let value = self.expr(&arg.value);
            let name = arg.name.map_or(0, |n| self.ident(n.name, n.span));
            let mut rec = Record::new("Code_Argument");
            rec.ptr("expression", value).ptr("name", name);
            unsorted.push(self.r.add(rec));
            values.push(value);
        }
        (unsorted, values)
    }

    /// A block holding one expression (branches of `ifx`).
    fn expr_block(&mut self, e: &ast::Expr) -> i64 {
        let value = self.expr(e);
        let mut rec = self.node("Code_Block", node::BLOCK, e.span);
        rec.int("block_type", 1).refs("statements", [value]);
        self.add(rec)
    }

    fn block(&mut self, stmts: &[ast::Stmt], block_type: i64, span: Span) -> i64 {
        let locals = self.locals.len();
        let id = self.block_inner(stmts, block_type, span);
        self.locals.truncate(locals);
        id
    }

    fn block_inner(&mut self, stmts: &[ast::Stmt], block_type: i64, span: Span) -> i64 {
        let mut statements = Vec::new();
        let mut members = Vec::new();
        for s in stmts {
            let Some(id) = self.stmt(s) else {
                continue;
            };
            if let ast::StmtKind::Decl(d) = &s.kind {
                // A block's members are declarations: each name of `a, b := f()`.
                if d.names.len() > 1 {
                    members.append(&mut self.compound_members);
                } else {
                    members.push(id);
                }
            }
            statements.push(id);
        }
        let mut rec = self.node("Code_Block", node::BLOCK, span);
        rec.int("block_type", block_type)
            .refs("members", members)
            .refs("statements", statements);
        self.add(rec)
    }

    /// A statement as a block (branches and loop bodies).
    fn stmt_block(&mut self, s: &ast::Stmt) -> i64 {
        match &s.kind {
            ast::StmtKind::Block(b) => self.block(&b.stmts, 1, b.span),
            _ => self.block(std::slice::from_ref(s), 1, s.span),
        }
    }

    fn stmt(&mut self, s: &ast::Stmt) -> Option<i64> {
        use ast::StmtKind as S;
        let span = s.span;
        let id = match &s.kind {
            S::Decl(d) => self.local_decl(d),
            S::Expr(e) => self.expr(e),
            S::Assign {
                op,
                lhs,
                rhs,
            } => {
                let l = lhs.first().map_or(0, |e| self.expr(e));
                let r = rhs.first().map_or(0, |e| self.expr(e));
                let mut rec = self.node("Code_Binary_Operator", node::BINARY_OPERATOR, span);
                rec.int("operator_type", assign_op(*op))
                    .ptr("left", l)
                    .ptr("right", r);
                self.add(rec)
            }
            S::Block(b) => self.block(&b.stmts, 1, b.span),
            S::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.expr(cond);
                let then_block = self.stmt_block(then_branch);
                let else_block = else_branch.as_deref().map_or(0, |b| self.stmt_block(b));
                let mut rec = self.node("Code_If", node::IF, span);
                rec.ptr("condition", cond)
                    .ptr("then_block", then_block)
                    .ptr("else_block", else_block);
                self.add(rec)
            }
            S::Switch {
                value,
                cases,
                complete,
            } => self.switch(value, cases, 0x1 | ((*complete as i64) * 0x8), span),
            S::StaticSwitch {
                value,
                cases,
            } => self.switch(value, cases, 0x1 | 0x4, span),
            S::StaticIf {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.expr(cond);
                let then_block = self.block(then_branch, 1, span);
                let else_block = self.block(else_branch, 1, span);
                let mut rec = self.node("Code_If", node::IF, span);
                rec.ptr("condition", cond)
                    .ptr("then_block", then_block)
                    .ptr("else_block", else_block)
                    .int("if_flags", 0x4);
                self.add(rec)
            }
            S::While {
                cond,
                body,
                ..
            } => {
                let cond = self.expr(cond);
                let block = self.stmt_block(body);
                let mut rec = self.node("Code_While", node::WHILE, span);
                rec.ptr("condition", cond).ptr("block", block);
                self.add(rec)
            }
            S::For(f) => {
                let (a, b) = match &f.over {
                    ast::ForOver::Range(a, b) => (self.expr(a), self.expr(b)),
                    ast::ForOver::Collection(c) => (self.expr(c), 0),
                };
                let it = f.it.map_or(0, |i| self.ident(i.name, i.span));
                let index = f.index.map_or(0, |i| self.ident(i.name, i.span));
                let block = self.stmt_block(&f.body);
                let flags = (f.by_pointer as i64) | ((f.reverse as i64) * 0x2);
                let mut rec = self.node("Code_For", node::FOR, span);
                rec.ptr("iteration_expression", a)
                    .ptr("iteration_expression_right", b)
                    .ptr("ident_it", it)
                    .ptr("ident_it_index", index)
                    .ptr("block", block)
                    .int("for_flags", flags);
                self.add(rec)
            }
            S::Break(label) | S::Continue(label) | S::Remove(label) => {
                let control = match s.kind {
                    S::Break(_) => 0,
                    S::Continue(_) => 1,
                    _ => 2,
                };
                let target = label.map_or(0, |l| self.ident(l.name, l.span));
                let mut rec = self.node("Code_Loop_Control", node::LOOP_CONTROL, span);
                rec.int("control_type", control).ptr("target_ident", target);
                self.add(rec)
            }
            S::Return {
                values,
                backtick,
            } => {
                let (unsorted, sorted) = self.args(values);
                // Jai leaves a return's `arguments_sorted` empty (yield-jai checks).
                let _ = sorted;
                let mut rec = self.node("Code_Return", node::RETURN, span);
                rec.refs("arguments_unsorted", unsorted)
                    .int("return_flags", *backtick as i64);
                self.add(rec)
            }
            S::Defer {
                body,
                backtick,
            } => {
                let block = self.stmt_block(body);
                let mut rec = self.node("Code_Defer", node::DEFER, span);
                rec.ptr("block", block)
                    .int("is_backticked", *backtick as i64);
                self.add(rec)
            }
            S::Using {
                value, ..
            } => {
                let value = self.expr(value);
                let mut rec = self.node("Code_Using", node::USING, span);
                rec.ptr("expression", value);
                self.add(rec)
            }
            S::PushContext {
                context,
                body,
            } => {
                let context = self.expr(context);
                let block = self.stmt_block(body);
                let mut rec = self.node("Code_Push_Context", node::PUSH_CONTEXT, span);
                rec.ptr("to_push", context).ptr("block", block);
                self.add(rec)
            }
            S::Insert {
                value, ..
            } => {
                let value = self.expr(value);
                let mut rec = self.node("Code_Directive_Insert", node::DIRECTIVE_INSERT, span);
                rec.ptr("expression", value);
                self.add(rec)
            }
            S::Run(e) => {
                let mut rec = self.node("Code_Directive_Run", node::DIRECTIVE_RUN, span);
                let e = self.expr(e);
                rec.ptr("expression", e);
                self.add(rec)
            }
            S::Case(case) => self.case(case),
            S::Assert {
                cond,
                message,
                ..
            } => {
                let mut rec = self.node("Code_Directive_Run", node::DIRECTIVE_RUN, span);
                let text = self
                    .c
                    .as_deref()
                    .map_or(String::new(), |c| c.sources.snippet(cond.span).to_string());
                let cond = self.expr(cond);
                let message = self.opt_expr(message.as_ref());
                rec.int("flags", 0x1) // ASSERTION
                    .str("assertion_string", text.as_bytes())
                    .ptr("expression", cond)
                    .ptr("message", message);
                self.add(rec)
            }
            S::Import(import) => {
                let mut rec = self.node("Code_Directive_Import", node::DIRECTIVE_IMPORT, span);
                if let ast::ImportSource::Module(name) = &import.source {
                    rec.str("name", name.as_bytes());
                }
                self.add(rec)
            }
            S::Empty | S::Scope(_) | S::Through => return None,
            _ => {
                let rec = self.node("Code_Node", node::PLACEHOLDER, span);
                self.add(rec)
            }
        };
        if let Some(c) = self.c.as_deref_mut() {
            c.export.stmts.insert(id, Rc::new(s.clone()));
        }
        Some(id)
    }

    fn case(&mut self, case: &ast::Case) -> i64 {
        let cond = case.values.first().map_or(0, |v| self.expr(v));
        let then_block = self.block(&case.body, 1, case.span);
        let mut rec = self.node("Code_Case", node::CASE, case.span);
        rec.ptr("condition", cond)
            .ptr("then_block", then_block)
            .int("marked_as_fallthrough", case.through as i64);
        self.add(rec)
    }

    fn switch(&mut self, value: &ast::Expr, cases: &[ast::Case], flags: i64, span: Span) -> i64 {
        let value = self.expr(value);
        let case_ids = cases.iter().map(|case| self.case(case)).collect::<Vec<_>>();
        let mut block = self.node("Code_Block", node::BLOCK, span);
        block.int("block_type", 1).refs("statements", case_ids);
        let block = self.add(block);
        let mut rec = self.node("Code_If", node::IF, span);
        rec.ptr("condition", value)
            .ptr("then_block", block)
            .int("if_flags", flags);
        self.add(rec)
    }

    /// A declaration inside a body or struct, with its type once the body was lowered.
    fn local_decl(&mut self, d: &ast::Decl) -> i64 {
        if d.names.len() > 1 {
            return self.compound_decl(d);
        }
        let name = d.names.first().map_or(Sym::intern(""), |n| n.name);
        let ty = self
            .c
            .as_deref()
            .and_then(|c| c.local_decl_types.get(&(d.id, 0)).copied());
        let rec = self.decl(d, name, ty, None, true);
        // `x := value`: the value has the declared type (its own typing may not see locals).
        if d.ty.is_none()
            && let Some(Field::Item(Item::Ref(ty))) = self.r.field(rec, "type").cloned()
            && let Some(Field::Item(Item::Ref(value))) = self.r.field(rec, "expression").cloned()
            && value != 0
            && let Some(value) = self.r.get_mut(value)
            && value
                .field("type")
                .is_none_or(|t| matches!(t, Field::Item(Item::Ref(0))))
        {
            value.ptr("type", ty);
        }
        for n in &d.names {
            self.locals.push((n.name, rec));
        }
        rec
    }

    /// `a, b := f();` / `a, b: int = 1, 2;` / `a=, b := f();`: the names (declarations, or
    /// identifiers for `=` targets) in `comma_separated_assignment`, the shared type and
    /// values in a nameless `declaration_properties`.
    fn compound_decl(&mut self, d: &ast::Decl) -> i64 {
        let values = if d.extra_values.is_empty() {
            None
        } else {
            let mut ids = Vec::new();
            for v in d.value.iter().chain(&d.extra_values) {
                let node = self.expr(v);
                let mut a = Record::new("Code_Comma_Separated_Argument");
                a.ptr("node", node);
                ids.push(self.r.add(a));
            }
            let mut rec = self.node(
                "Code_Comma_Separated_Arguments",
                node::COMMA_SEPARATED_ARGUMENTS,
                d.span,
            );
            rec.refs("arguments", ids);
            Some(self.add(rec))
        };
        // Before the names are in scope: the values may mention variables they shadow.
        let properties = self.decl(d, Sym::intern(""), None, values, false);
        let mut arguments = Vec::new();
        let mut declared = Vec::new();
        for (i, n) in d.names.iter().enumerate() {
            let assign = d.existing.get(i).copied().unwrap_or(false);
            let target = if let Some(Some(place)) = d.targets.get(i) {
                self.expr(place)
            } else if assign {
                self.ident(n.name, n.span)
            } else {
                let ty = self
                    .c
                    .as_deref()
                    .and_then(|c| c.local_decl_types.get(&(d.id, i)).copied())
                    .map_or(0, |t| self.ty(t));
                let mut rec = self.node("Code_Declaration", node::DECLARATION, n.span);
                rec.str("name", n.name.as_str().as_bytes())
                    .ptr("type", ty)
                    .int("flags", i64::from(d.kind == ast::DeclKind::Const));
                let id = self.add(rec);
                declared.push((n.name, id));
                id
            };
            let mut a = Record::new("Code_Comma_Separated_Argument");
            a.ptr("node", target).int(
                "modifier",
                if assign {
                    2
                } else {
                    1
                },
            );
            arguments.push(self.r.add(a));
        }
        self.compound_members = declared.iter().map(|&(_, id)| id).collect();
        self.locals.extend(declared);
        let mut names = self.node(
            "Code_Comma_Separated_Arguments",
            node::COMMA_SEPARATED_ARGUMENTS,
            d.span,
        );
        names.refs("arguments", arguments);
        let names = self.add(names);
        let mut rec = self.node(
            "Code_Compound_Declaration",
            node::COMPOUND_DECLARATION,
            d.span,
        );
        rec.ptr("comma_separated_assignment", names)
            .ptr("declaration_properties", properties)
            .int("node_flags", 0x4);
        self.add(rec)
    }

    fn decl(
        &mut self,
        d: &ast::Decl,
        name: Sym,
        ty: Option<TypeId>,
        expression: Option<i64>,
        statement: bool,
    ) -> i64 {
        let type_inst = d.ty.as_ref().map_or(0, |t| self.type_inst(t));
        let expression = match expression {
            Some(e) => e,
            None => match &d.value {
                Some(v) => match &v.kind {
                    ast::ExprKind::Proc(lit) => self.proc(lit, &d.notes, None, v.span),
                    _ => self.expr(v),
                },
                None => 0,
            },
        };
        let notes: Vec<&ast::Note> = d.notes.iter().collect();
        let notes = self.notes(&notes);
        let mut flags = 0;
        if d.kind == ast::DeclKind::Const {
            flags |= 0x1;
        }
        if d.as_ {
            flags |= 0x2;
        }
        if matches!(&d.value, Some(v) if matches!(v.kind, ast::ExprKind::Uninit)) {
            flags |= 0x80;
        }
        let backticked = d.backtick
            || d.names
                .iter()
                .zip(&d.backtick_names)
                .any(|(n, &b)| b && n.name == name);
        if backticked {
            flags |= 0x200000; // HAS_SCOPE_MODIFIER
        }
        let ty = ty.map_or(0, |t| self.ty(t));
        let mut rec = self.node("Code_Declaration", node::DECLARATION, d.span);
        rec.str("name", name.as_str().as_bytes())
            .ptr("type", ty)
            .ptr("type_inst", type_inst)
            .ptr("expression", expression)
            .int("flags", flags)
            .refs("notes", notes);
        if statement {
            rec.int("node_flags", 0x4); // ALLOWED_BY_CONTEXT
        }
        self.add(rec)
    }

    /// A top-level declaration with its resolved type.
    fn global_decl(&mut self, d: &ast::Decl, index: usize, resolved: &Resolved) -> i64 {
        let name = d.names[index].name;
        let span = d.span;
        let (ty, expression) = match resolved {
            Resolved::Proc(p)
            | Resolved::Const {
                value: Value::Proc(p),
                ..
            } => {
                let p = *p;
                let c = self.c.as_deref_mut().unwrap();
                let sig = c.signature(p, span).ok();
                let lit = c.proc(p).lit.clone();
                let header = self.proc(&lit, &d.notes, Some((p, sig.clone())), span);
                (sig.map(|s| s.ty), Some(header))
            }
            Resolved::Const {
                value: Value::Type(t),
                ty,
            } => {
                let expression = match d.value.as_ref().map(|v| &v.kind) {
                    Some(ast::ExprKind::Struct(lit)) => Some(self.struct_lit(lit, Some(*t), name)),
                    Some(ast::ExprKind::Enum(lit)) => Some(self.enum_lit(lit, Some(*t), span)),
                    _ => None,
                };
                (Some(*ty), expression)
            }
            Resolved::PolyStruct(_) => {
                let expression = match d.value.as_ref().map(|v| &v.kind) {
                    Some(ast::ExprKind::Struct(lit)) => Some(self.struct_lit(lit, None, name)),
                    _ => None,
                };
                (Some(TypeId::TYPE), expression)
            }
            Resolved::Const {
                ty, ..
            }
            | Resolved::Global {
                ty, ..
            } => (Some(*ty), None),
            _ => (None, None),
        };
        self.decl(d, name, ty, expression, false)
    }

    /// An enum literal; `external_type` is its type when the declaration resolved to one.
    fn enum_lit(&mut self, lit: &ast::EnumLit, defined: Option<TypeId>, span: Span) -> i64 {
        let notes: Vec<&ast::Note> = lit.notes.iter().collect();
        let notes = self.notes(&notes);
        let external = defined
            .filter(|&t| {
                self.c
                    .as_deref()
                    .is_some_and(|c| matches!(c.types.kind(t), TypeKind::Enum(_)))
            })
            .map_or(0, |t| self.ty(t));
        let mut rec = self.node("Code_Enum", node::ENUM, span);
        rec.refs("notes", notes)
            .ptr("external_type", external)
            .int("marked_as_complete", lit.complete as i64)
            .int("marked_as_specified", lit.specified as i64)
            .int("is_flags", lit.flags_enum as i64);
        self.add(rec)
    }

    /// A struct literal. Without a concrete type (polymorphic structs, anonymous
    /// structs in bodies) `defined_type` is a bare descriptor named `name`.
    fn struct_lit(&mut self, lit: &ast::StructLit, defined: Option<TypeId>, name: Sym) -> i64 {
        let start = self.sub.len();
        let block = self.block(&lit.body, 2, lit.span);
        let notes: Vec<&ast::Note> = lit.notes.iter().collect();
        let notes = self.notes(&notes);
        let defined = defined
            .filter(|&t| {
                self.c
                    .as_deref()
                    .is_some_and(|c| matches!(c.types.kind(t), TypeKind::Struct(_)))
            })
            .map(|t| self.ty(t))
            .filter(|&t| t != 0);
        let defined = defined.unwrap_or_else(|| {
            let mut generic = Record::new("Type_Info_Struct");
            generic
                .int("type", 7)
                .str("name", name.as_str().as_bytes())
                .int("nontextual_flags", (!lit.params.is_empty()) as i64 * 0x100);
            self.r.add(generic)
        });
        let mut rec = self.node("Code_Struct", node::STRUCT, lit.span);
        rec.ptr("block", block)
            .refs("notes", notes)
            .ptr("defined_type", defined)
            .int(
                "textual_flags",
                (lit.kind == ast::StructKind::Union) as i64 * 0x2,
            );
        let id = self.add(rec);
        let sub = self.sub[start..].to_vec();
        self.out.structs.push((id, sub));
        id
    }

    /// A `Code_Procedure_Body` record for `body`, belonging to header record `header_id`.
    fn body(&mut self, body: &ast::Block, header_id: i64, proc: Option<ProcId>) -> i64 {
        // The header's arguments and named returns are in scope in the body.
        let locals = self.locals.len();
        for list in ["arguments", "returns"] {
            let Some(Field::List(items)) = self.r.field(header_id, list) else {
                continue;
            };
            let decls: Vec<i64> = items
                .iter()
                .filter_map(|i| match i {
                    Item::Ref(d) => Some(*d),
                    _ => None,
                })
                .collect();
            for d in decls {
                if let Some(Field::Item(Item::Str(name))) = self.r.field(d, "name")
                    && !name.is_empty()
                {
                    let name = Sym::intern(&String::from_utf8_lossy(name));
                    self.locals.push((name, d));
                }
            }
        }
        let block = self.block(&body.stmts, 1, body.span);
        self.locals.truncate(locals);
        let mut rec = self.node("Code_Procedure_Body", node::PROCEDURE_BODY, body.span);
        rec.ptr("block", block).ptr("header", header_id);
        let body_id = self.add(rec);
        if let (Some(p), Some(c)) = (proc, self.c.as_deref_mut()) {
            c.export.bodies.insert(body_id, p);
        }
        body_id
    }

    /// A procedure header (and its body). `decl_notes` are the notes of the
    /// declaration naming it: Jai attaches notes after a body to both.
    fn proc(
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
        for (i, param) in h.params.iter().enumerate() {
            let ty = sig.as_ref().and_then(|s| s.params.get(i)).map(|p| p.ty);
            let decl = ast::Decl {
                id: h.id,
                names: param.name.into_iter().collect(),
                kind: ast::DeclKind::Var,
                ty: param.ty.clone(),
                value: param.default.clone(),
                extra_values: Vec::new(),
                existing: Vec::new(),
                targets: Vec::new(),
                using: param.using,
                using_filter: None,
                as_: false,
                backtick: false,
                backtick_names: Vec::new(),
                align: None,
                flags: Vec::new(),
                foreign: None,
                union_tag: None,
                notes: param.notes.clone(),
                span: param.span,
            };
            let name = param.name.map_or(Sym::intern(""), |n| n.name);
            let arg = self.decl(&decl, name, ty, None, false);
            if param.variadic {
                self.mark_varargs(arg, param.span);
            }
            arguments.push(arg);
            // `using p: T` parameters are listed again as `Code_Using` of the parameter.
            if param.using {
                let mut ident = self.node("Code_Ident", node::IDENT, param.span);
                ident
                    .str("name", name.as_str().as_bytes())
                    .ptr("resolved_declaration", arg);
                let ident = self.add(ident);
                let mut using = self.node("Code_Using", node::USING, param.span);
                using.ptr("expression", ident);
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
                extra_values: Vec::new(),
                existing: Vec::new(),
                targets: Vec::new(),
                using: false,
                using_filter: None,
                as_: false,
                backtick: false,
                backtick_names: Vec::new(),
                align: None,
                flags: Vec::new(),
                foreign: None,
                union_tag: None,
                notes: Vec::new(),
                span: ret.span,
            };
            let name = ret.name.map_or(Sym::intern(""), |n| n.name);
            returns.push(self.decl(&decl, name, ty, None, false));
        }
        let mut notes: Vec<&ast::Note> = h.notes.iter().collect();
        for n in decl_notes {
            if !notes.iter().any(|m| m.span == n.span) {
                notes.push(n);
            }
        }
        let notes = self.notes(&notes);
        let f = &h.flags;
        let mut flags = 0;
        if f.elsewhere.is_some() {
            flags |= 0x1;
        }
        if h.params
            .iter()
            .any(|p| p.baked || p.ty.as_ref().is_some_and(procs::has_poly))
        {
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
        // One header record per procedure: calls that resolved to it before it was reported
        // (`resolved_header`) and later ones see the reported header, body included.
        let known = match (&resolved, self.c.as_deref()) {
            (Some((p, _)), Some(c)) => c.export.resolved_headers.get(&(*p, self.own)).copied(),
            _ => None,
        };
        let header_id = known.unwrap_or_else(|| self.r.reserve("Code_Procedure_Header"));
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
            c.export.pending_bodies.push((*p, header_id, scope));
        } else if let Some(body) = &lit.body {
            body_id = self.body(body, header_id, resolved.as_ref().map(|(p, _)| *p));
        }
        let mut rec = self.node("Code_Procedure_Header", node::PROCEDURE_HEADER, span);
        let foreign_name = match h.foreign.as_ref().map(|f| &f.name) {
            Some(ast::ForeignName::Named(n)) => n.as_bytes().to_vec(),
            _ => Vec::new(),
        };
        let name = resolved
            .as_ref()
            .and_then(|(p, _)| Some(self.c.as_deref()?.proc(*p).name.as_str()))
            .unwrap_or("");
        let ty = sig.as_ref().map_or(0, |s| self.ty(s.ty));
        rec.refs("arguments", arguments)
            .refs("parameter_usings", usings)
            .refs("returns", returns)
            .str("name", name.as_bytes())
            .str("foreign_function_name", &foreign_name)
            .ptr("body_or_null", body_id)
            .ptr("type", ty)
            .int("procedure_flags", flags)
            .refs("notes", notes)
            .int("serial", header_id);
        *self.r.get_mut(header_id).unwrap() = rec;
        self.sub.push(header_id);
        let sub = self.sub[start..].to_vec();
        self.out.headers.push((header_id, sub.clone()));
        if body_id != 0 {
            self.out.bodies.push((body_id, sub));
        }
        header_id
    }
}

/// Records for a `#code` value outside its compiler (`compiler_get_nodes`):
/// the root node (`__source` holds the code's text, so the target compiler
/// can parse it again when the node is inserted) and every node below it.
pub fn export_code(r: &mut Records, body: &ast::CodeBody, source: &str) -> (i64, Vec<i64>) {
    export_code_in(None, ScopeId(0), r, body, source)
}

/// `export_code`, with names resolved and types filled in by `c` when given: identifiers
/// get `resolved_declaration`, calls `resolved_procedure_expression`, both `type`.
fn export_code_in(
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
        compound_members: Vec::new(),
        own,
        out: &mut out,
    };
    let root = match body {
        ast::CodeBody::Expr(e) => ex.expr(e),
        // `#code a := 1;` without braces is the bare statement, not a block.
        ast::CodeBody::Block(b) if b.stmts.len() == 1 && !source.trim_start().starts_with('{') => {
            match ex.stmt(&b.stmts[0]) {
                Some(id) => id,
                None => ex.block(&b.stmts, 1, b.span),
            }
        }
        ast::CodeBody::Block(b) => ex.block(&b.stmts, 1, b.span),
    };
    let sub = std::mem::take(&mut ex.sub);
    if let Some(rec) = r.get_mut(root) {
        rec.str("__source", source.as_bytes());
    }
    (root, sub)
}
