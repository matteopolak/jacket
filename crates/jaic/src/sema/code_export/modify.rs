//! Making procedures live and applying modifications to them.
use super::state::ModifiedStmt;
use crate::records::{Item, Records};
use crate::sema::{Compiler, Rc, Result, SourcePath, Span, ast, err, procs};

impl Compiler {
    /// `compiler_modify_procedure`: give the procedure of body record `body`
    /// the statements `stmts` (statement records exported from this
    /// workspace, or `#code` roots from `compiler_get_nodes`, parsed again
    /// here). A body already lowered is lowered again.
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
                if !self.live_procs.contains(&p) {
                    self.live_procs.push(p);
                }
                true
            }
            None => false,
        }
    }

    pub fn modify_procedure(
        &mut self,
        r: &Records,
        body: i64,
        stmts: &[ModifiedStmt],
    ) -> Result<()> {
        // Ownership is attached to the actual exported workspace procedure, never
        // recovered from a display name, copied serial or caller-created header.
        let Some(&p) = self.meta.export.bodies.get(&body) else {
            return err(
                Span::NONE,
                "compiler_modify_procedure: body record does not belong to this workspace procedure",
            );
        };
        if self.proc(p).body_state == procs::BodyState::Lowering {
            return err(
                self.proc(p).span,
                "compiler_modify_procedure: cannot modify a procedure while it is being lowered",
            );
        }
        let old = self.proc(p).lit.clone();
        let Some(old_body) = &old.body else {
            return err(
                self.proc(p).span,
                "compiler_modify_procedure: procedure has no body",
            );
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
            if let Some(stmt) = self.meta.export.stmts.get(&id) {
                new_stmts.push((**stmt).clone());
                continue;
            }
            let Some(Item::Str(text)) = r.item(id, "__source", None) else {
                return err(
                    old_body.span,
                    "compiler_modify_procedure: a statement is neither from this procedure nor from compiler_get_nodes",
                );
            };
            new_stmts.extend(self.parse_inserted_code(&String::from_utf8_lossy(&text))?);
        }
        let lit = ast::ProcLit {
            header: old.header.clone(),
            body: Some(ast::Block {
                stmts: new_stmts,
                span: old_body.span,
                checks_off: old_body.checks_off,
            }),
        };
        self.procs[p.0 as usize].lit = Rc::new(lit);
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

    /// Statements of the text of a `#code` value, as source of this compiler.
    fn parse_inserted_code(&mut self, text: &str) -> Result<Vec<ast::Stmt>> {
        let source = format!("__jaic_inserted :: () {{\n{text};\n}}\n");
        let label =
            SourcePath::virtual_source(format!("compiler_get_nodes {}", self.src.sources.len()));
        let file = self.src.sources.add(label, source.as_str().into());
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
}
