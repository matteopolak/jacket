//! Why an assignment target cannot be assigned to, and what to write instead.
use crate::ast::ExprKind as E;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, Found};
use crate::sema::{Compiler, Diagnostic, EntityId, ScopeId, Span, Sym, Value, ast};
use crate::source::{Applicability, Suggestion};

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// The error for assigning to `target` when it evaluates to `place`, which has no storage:
    /// a constant (declared where, and how to make it a variable) or an expression that is a
    /// value (what kind).
    pub(in crate::sema) fn not_assignable(
        &mut self,
        scope: ScopeId,
        target: &ast::Expr,
        place: &Operand,
    ) -> Box<Diagnostic> {
        let written = self.src.sources.snippet(target.span).trim().to_string();
        let is_constant = matches!(
            place,
            Operand::Const { value, .. } if !matches!(value, Value::Type(_) | Value::Null)
        );
        if is_constant && matches!(target.kind, E::Ident(_) | E::Member(..)) {
            let d = Diagnostic::error(
                target.span,
                format!("cannot assign to constant `{written}`"),
            )
            .with_label("a constant has no storage to assign to");
            return Box::new(match target.kind {
                E::Ident(name) => self.constant_declaration_hints(d, scope, name),
                _ => d,
            });
        }
        let message = match &target.kind {
            E::Int(..) | E::Float(..) | E::Str(_) | E::Bool(_) | E::Null => {
                "cannot assign to a literal".to_string()
            }
            E::Call {
                callee, ..
            } => format!(
                "cannot assign to the result of a call to `{}`",
                self.src.sources.snippet(callee.span).trim()
            ),
            E::Binary(..) | E::Unary(..) => "cannot assign to the result of an operator".into(),
            E::Cast {
                ..
            } => "cannot assign to the result of a cast".into(),
            _ => format!("cannot assign to {}", self.describe(place)),
        };
        Box::new(Diagnostic::error(target.span, message).with_help(
            "only a variable, a member, an element or a dereference (`p.*`) can be assigned to",
        ))
    }

    /// `d`, an error about assigning into the constant `name`, with where it is declared and,
    /// for a plain `NAME :: value;`, the edit that makes it a variable.
    pub(in crate::sema) fn constant_declaration_hints(
        &mut self,
        d: Diagnostic,
        scope: ScopeId,
        name: Sym,
    ) -> Diagnostic {
        let Some(id) = self.sole_entity(scope, name) else {
            return d;
        };
        let entity = self.entity(id);
        if !matches!(entity.kind, EntityKind::Decl { .. }) || entity.span == Span::NONE {
            return d;
        }
        let at = entity.span;
        let d = d.with_note(at, format!("`{name}` is declared as a constant here"));
        let help = "declare it with `:=` to make it a variable";
        match self.constant_marker(at) {
            Some(marker) => d.with_suggestion(
                Suggestion::replace(help, marker, ":=", Applicability::MaybeIncorrect)
                    .with_action("Declare it with `:=`"),
            ),
            None => d.with_help(help),
        }
    }

    /// The `::` that follows the name declared at `name` (`X :: 5;`), if the declaration is
    /// written that way. A typed constant (`X : int : 5;`) needs more than one edit.
    fn constant_marker(&self, name: Span) -> Option<Span> {
        let file = self.src.sources.try_get(name.file)?;
        let rest = file.text.get(name.end as usize..)?;
        let skipped = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        let after = &rest[skipped..];
        if !after.starts_with("::") || after.starts_with("::=") {
            return None;
        }
        let start = name.end + u32::try_from(skipped).ok()?;
        Some(Span {
            start,
            end: start + 2,
            ..name
        })
    }

    /// The one entity `name` finds from `scope`, if it finds exactly one.
    pub(in crate::sema) fn sole_entity(&mut self, scope: ScopeId, name: Sym) -> Option<EntityId> {
        match self.lookup_full(scope, name).ok()? {
            Found::Entities(ids) if ids.len() == 1 => Some(ids[0]),
            _ => None,
        }
    }

    /// The error for writing to the argument `entity` (named `name` where the target starts),
    /// which holds an aggregate the procedure may only read: copy it into a local first.
    pub(in crate::sema) fn immutable_argument(
        &self,
        f: &FnCtx,
        entity: EntityId,
        name: Sym,
        span: Span,
    ) -> Box<Diagnostic> {
        let mut d = Diagnostic::error(span, "Can't assign to an immutable argument.");
        let declared = self.entity(entity);
        if declared.span != Span::NONE {
            d = d.with_note(
                declared.span,
                format!(
                    "`{}` is an argument, which the procedure may only read",
                    declared.name
                ),
            );
        }
        let help = format!("copy it into a local first: `{name} := {name};`");
        // The copy goes at the top of the body. A name that reaches the argument through
        // `using` keeps reaching it, so a copy of the argument would not help there.
        let body = f.proc.and_then(|p| {
            let lit = &self.proc(p).lit;
            let inside = |outer: Span| {
                outer.file == declared.span.file
                    && outer.start <= declared.span.start
                    && declared.span.end <= outer.end
            };
            let body = lit.body.as_ref()?;
            (declared.name == name && inside(lit.header.span)).then_some(body.span)
        });
        Box::new(match body {
            Some(body) => {
                let open = Span {
                    start: body.start + 1,
                    end: body.start + 1,
                    ..body
                };
                d.with_suggestion(
                    Suggestion::insert(
                        help,
                        open,
                        format!(" {name} := {name};"),
                        Applicability::MaybeIncorrect,
                    )
                    .with_action(format!("Copy `{name}` into a local")),
                )
            }
            None => d.with_help(help),
        })
    }
}
