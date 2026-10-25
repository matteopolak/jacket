//! `$T/.[A, B]` restrictions: a polymorphic parameter limited to a list of types.

use super::{Compiler, E, Result, ScopeId, TypeId, ast};
use crate::sema::scope::{EntityKind, Found};

impl Compiler {
    /// The types listed by a `$T/.[A, B]` restriction, directly or through a constant
    /// declared as such a list; `None` for any other restriction.
    pub(super) fn restriction_type_list(
        &mut self,
        scope: ScopeId,
        restriction: &ast::Expr,
        depth: u8,
    ) -> Result<Option<Vec<TypeId>>> {
        match &restriction.kind {
            E::ArrayLit {
                elems, ..
            } => elems
                .iter()
                .map(|elem| self.eval_type(scope, elem))
                .collect::<Result<Vec<_>>>()
                .map(Some),
            E::Ident(name) if depth < 4 => {
                let Found::Entities(ids) = self.lookup_full(scope, *name)? else {
                    return Ok(None);
                };
                let [id] = ids[..] else {
                    return Ok(None);
                };
                let entity = self.entity(id);
                let (
                    home,
                    EntityKind::Decl {
                        decl, ..
                    },
                ) = (entity.home, entity.kind.clone())
                else {
                    return Ok(None);
                };
                match &decl.value {
                    Some(value) if decl.kind == ast::DeclKind::Const => {
                        self.restriction_type_list(home, value, depth + 1)
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}
