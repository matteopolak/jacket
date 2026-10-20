//! Exporting typechecked procedures and the performance report.
use super::constants::message_kind;
use super::procs::export_code_in;
use super::{Exporter, Typechecked};
use crate::records::{Item, Record, RecordTag, Records};
use crate::sema::scope::{EntityKind, Resolved, ScopeKind};
use crate::sema::{Compiler, EntityId, ProcId, procs};

impl Compiler {
    /// A TYPECHECKED message for eligible declarations and ready bodies, or `None`.
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
            let (decl, index, home, file_private) =
                (decl.clone(), *index, entity.home, entity.file_private);
            let scope = self.scope(entity.scope);
            if !matches!(scope.kind, ScopeKind::Module | ScopeKind::File)
                || index >= decl.names.len()
            {
                continue;
            }
            let module = scope.module;
            // Do not force the entire library to resolve for an interception.
            // Already resolved library declarations, including called procedures,
            // have the same canonical records as declarations in user modules.
            let resolved = matches!(entity.state, crate::sema::scope::EntityState::Done(_));
            if self.is_system_module(module) && !resolved {
                continue;
            }
            decls.push((id, decl, index, home, file_private));
        }
        let mut out = Typechecked::default();
        // Bodies lowered since their header was reported.
        let mut exported = self.export_ready_bodies(r, &mut out);
        if decls.is_empty() && exported == 0 {
            return None;
        }
        for (id, decl, index, home, file_private) in decls {
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
                    procedure: None,
                    compound_members: Vec::new(),
                    own: false,
                    out: &mut out,
                };
                let rec = ex.global_decl(&decl, index, &resolved, file_private);
                (rec, ex.sub)
            };
            out.declarations.push((rec, sub));
        }
        // Newly reported calls can schedule ordinary callees or polymorph instances.
        // Drain ready bodies transitively even when the first pass exported none.
        loop {
            exported = self.export_ready_bodies(r, &mut out);
            if exported == 0 {
                break;
            }
        }
        let mut message = Record::new(RecordTag::MessageTypechecked);
        let group = |r: &mut Records, items: &[(i64, Vec<i64>)]| -> Vec<Item> {
            items
                .iter()
                .map(|(expr, sub)| {
                    let mut t = Record::new(RecordTag::Typechecked);
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

    /// The record of a `Message_Performance_Report` for the workspace: timings, instruction
    /// counts and, when asked for, the `#run` and polymorph sections (`perf.rs` has the counts).
    pub fn export_performance_report(
        &mut self,
        r: &mut Records,
        runs: bool,
        polymorphs: bool,
    ) -> i64 {
        let seconds = |d: std::time::Duration| d.as_secs_f64().to_bits() as i64;
        let mut time = Record::new(RecordTag::TimeReport);
        time.int("run_directives", seconds(self.perf.run_time))
            // jaic has no bytecode pass that inlines calls: the native backend inlines later.
            .int("bytecode_inlining", 0.0f64.to_bits() as i64)
            .int("bytecode_generating", seconds(self.perf.lowering));
        let time = r.add(time);

        let mut run_report = Record::new(RecordTag::RunDirectiveReport);
        if runs {
            let stats = self.perf.runs.clone();
            let mut records = Vec::new();
            for run in &stats {
                let callee = run
                    .callee
                    .and_then(|f| self.procs_of_func(f).next())
                    .map_or(0, |p| self.header_record(r, p));
                let mut record = Record::new(RecordTag::Record);
                record
                    .ptr("callee", callee)
                    .int("num_stalls", run.stalls as i64)
                    .int("elapsed_time", seconds(run.elapsed));
                records.push(r.add(record));
            }
            run_report.int("reported", 1).refs("records", records);
        }
        let run_report = r.add(run_report);

        let mut poly_report = Record::new(RecordTag::PolymorphReport);
        poly_report
            .int("num_solves_invoked", self.perf.solves_invoked() as i64)
            .int("num_solves_used", self.perf.solves_used as i64);
        if polymorphs {
            let order = self.perf.poly_order.clone();
            let mut records = Vec::new();
            for source in order {
                let stat = self.perf.polys[&source].clone();
                let mut distinct = Vec::new();
                for (instance, span) in &stat.instances {
                    let polymorphed = self.header_record(r, *instance);
                    let mut location = Record::new(RecordTag::SourceCodeLocation);
                    if let Some(file) = self.sources.try_get(span.file) {
                        let (line, column) = file.line_col(span.start);
                        location
                            .str("fully_pathed_filename", file.path.to_string().as_bytes())
                            .int("line_number", line as i64)
                            .int("character_number", column as i64);
                    }
                    let location = r.add(location);
                    let mut one = Record::new(RecordTag::DistinctPolymorph);
                    one.ptr("polymorphed", polymorphed)
                        .ptr("location", location);
                    distinct.push(r.add(one));
                }
                let header = self.header_record(r, source);
                let mut record = Record::new(RecordTag::Record);
                record
                    .ptr("source", header)
                    .int("num_call_sites", stat.call_sites as i64)
                    .int("num_deduplications_by_constants", stat.by_constants as i64)
                    // Instances are shared by their constants only; no bytecode is compared.
                    .int("num_deduplications_by_bytecode", 0)
                    .refs("distinct_polymorphs", distinct);
                records.push(r.add(record));
            }
            poly_report.int("reported", 1).refs("records", records);
        }
        let poly_report = r.add(poly_report);

        let (calls, instructions) = self.instruction_counts();
        let mut bytecode = Record::new(RecordTag::BytecodeReport);
        bytecode
            .int("num_calls_inlined", 0)
            .int("num_calls_total", calls as i64)
            .int("num_instructions_inlined", 0)
            .int("num_instructions_total", instructions as i64);
        let bytecode = r.add(bytecode);

        let mut message = Record::new(RecordTag::MessagePerformanceReport);
        message
            .int("kind", message_kind::PERFORMANCE_REPORT)
            .int("workspace", self.workspace)
            .ptr("time_report", time)
            .ptr("run_directive_report", run_report)
            .ptr("polymorph_report", poly_report)
            .ptr("bytecode_report", bytecode);
        r.add(message)
    }

    /// The header record of procedure `p` (the one calls resolve to).
    fn header_record(&mut self, r: &mut Records, p: ProcId) -> i64 {
        let scope = self.proc(p).scope;
        let mut out = Typechecked::default();
        let mut exporter = Exporter {
            c: Some(self),
            r,
            scope,
            sub: Vec::new(),
            locals: Vec::new(),
            procedure: None,
            compound_members: Vec::new(),
            own: false,
            out: &mut out,
        };
        exporter.resolved_header(p)
    }

    /// Export the bodies of reported headers that are lowered by now, into `out`. Returns how
    /// many it exported.
    fn export_ready_bodies(&mut self, r: &mut Records, out: &mut Typechecked) -> usize {
        // A concrete procedure may be reached through a member or procedure value,
        // without the syntax exporter's call lookup finding it. Lowering is the
        // authority for these events, not whether its source has a global name.
        let completed: Vec<_> = self
            .procs
            .iter()
            .enumerate()
            .filter(|(index, p)| {
                p.body_state == procs::BodyState::Done
                    && p.lit.body.is_some()
                    && !p.is_poly
                    && !p.is_macro
                    && !self
                        .export
                        .scheduled_bodies
                        .contains(&ProcId(*index as u32))
            })
            .map(|(index, _)| ProcId(index as u32))
            .collect();
        for p in completed {
            let scope = self.proc(p).bindings.unwrap_or(self.proc(p).scope);
            let header = {
                let mut exporter = Exporter {
                    c: Some(self),
                    r,
                    scope,
                    sub: Vec::new(),
                    locals: Vec::new(),
                    procedure: None,
                    compound_members: Vec::new(),
                    own: false,
                    out,
                };
                exporter.resolved_header(p)
            };
            self.export.schedule_body(p, header, scope);
        }
        let pending = std::mem::take(&mut self.export.pending_bodies);
        let (ready, waiting): (Vec<_>, Vec<_>) = pending
            .into_iter()
            .partition(|(p, _, _)| self.proc(*p).body_state == procs::BodyState::Done);
        self.export.pending_bodies = waiting;
        let count = ready.len();
        for (p, _header_id, scope) in ready {
            let lit = self.proc(p).lit.clone();
            let span = self.proc(p).span;
            let sig = self.signature(p, span).ok();
            let notes = self.proc_decl_notes.get(&p).cloned().unwrap_or_default();
            let mut ex = Exporter {
                c: Some(self),
                r,
                scope,
                sub: Vec::new(),
                locals: Vec::new(),
                procedure: None,
                compound_members: Vec::new(),
                own: false,
                out,
            };
            ex.proc(&lit, &notes, Some((p, sig)), span);
        }
        count
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
        let Some(shared) = self.ct.interp.workspaces.clone() else {
            return;
        };
        let (Some(body), Some(&scope), Some((_, text))) = (
            self.codes.get(code).cloned(),
            self.code_scopes.get(code),
            self.ct.interp.codes.get(code).cloned(),
        ) else {
            return;
        };
        // No further requests while exporting: nested compile-time code gets plain nodes.
        let outer = self.ct.interp.run_effects.take();
        let mut records = Records::lend(&mut shared.borrow_mut().records);
        let exported = export_code_in(Some(self), scope, &mut records, &body, &text);
        Records::give_back(&mut shared.borrow_mut().records, records);
        self.ct.interp.run_effects = outer;
        self.ct
            .interp
            .code_exports
            .entry(code)
            .or_default()
            .push(exported);
    }
}
