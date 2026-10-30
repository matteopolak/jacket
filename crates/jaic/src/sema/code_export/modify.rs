//! Making procedures live and applying modifications to them.
use super::state::Modification;
use crate::diagnostics::Code;
use crate::sema::{Compiler, Rc, Result, SourcePath, Span, ast, err, procs};

impl Compiler {
    /// `compiler_make_procedure_live`: keep the procedure behind header record `header` in the
    /// program though nothing calls it. False when the record is not a procedure header this
    /// compiler exported.
    pub fn make_procedure_live(&mut self, header: i64) -> bool {
        let found = self
            .meta
            .export
            .resolved_headers
            .iter()
            .find(|&(_, &record)| record == header)
            .map(|(&(p, _), _)| p)
            .or_else(|| {
                self.meta
                    .export
                    .pending_bodies
                    .iter()
                    .find(|&&(_, record, _)| record == header)
                    .map(|&(p, _, _)| p)
            });
        match found {
            Some(p) => {
                if !self.pr.live_procs.contains(&p) {
                    self.pr.live_procs.push(p);
                }
                true
            }
            None => false,
        }
    }

    /// `compiler_modify_procedure`: give the procedure of body record `m.body` the statements
    /// `m.stmts`. A body already lowered is lowered again.
    pub fn modify_procedure(&mut self, m: Modification) -> Result<()> {
        let body = m.body;
        // Ownership is attached to the actual exported workspace procedure, never
        // recovered from a display name, copied serial or caller-created header.
        let Some(&p) = self.meta.export.bodies.get(&body) else {
            return err(
                Code::CodeExportModifyProcedureInvalid,
                Span::NONE,
                "compiler_modify_procedure: body record does not belong to this workspace procedure",
            );
        };
        if self.proc(p).body_state == procs::BodyState::Lowering {
            return err(
                Code::CodeExportModifyProcedureInvalid,
                self.proc(p).span,
                "compiler_modify_procedure: cannot modify a procedure while it is being lowered",
            );
        }
        let old = self.proc(p).lit.clone();
        let Some(old_body) = &old.body else {
            return err(
                Code::CodeExportModifyProcedureInvalid,
                self.proc(p).span,
                "compiler_modify_procedure: procedure has no body",
            );
        };
        let label = SourcePath::virtual_source(format!(
            "compiler_modify_procedure {}",
            self.src.sources.len()
        ));
        let file = self.src.sources.add(label, m.source);
        let mut new_stmts = m.stmts;
        ast::relocate::relocate_stmts(&mut new_stmts, file);
        let lit = ast::ProcLit {
            header: old.header.clone(),
            body: Some(ast::Block {
                stmts: new_stmts,
                span: old_body.span,
                checks_off: old_body.checks_off,
            }),
        };
        self.pr.procs[p.0 as usize].lit = Rc::new(lit);
        if self.proc(p).body_state == procs::BodyState::Done {
            self.meta.export.relower.push(p);
        }
        Ok(())
    }

    /// Lower again the bodies `modify_procedure` replaced after they were lowered. Separate,
    /// so the caller can release the registry first: the new bodies may run compile-time
    /// code that uses it.
    pub fn relower_modified(&mut self) -> Result<()> {
        for p in std::mem::take(&mut self.meta.export.relower) {
            self.lower_body(p)?;
        }
        Ok(())
    }
}
