//! `@Interpolates("mode")`: inline holes in a string literal become positional arguments.
//!
//! `check_call` resolves the callee first, so the note is known before any argument is typed.
//! For each call whose literal argument sits in an interpolating parameter the holes are parsed
//! as expressions in the caller's scope, with spans pointing into the string, and the call is
//! rewritten: `f(db, "a %x b")` is checked as `f(db, "a % b", x)`. Everything after that is the
//! ordinary path, which is why diagnostics, hover, go to definition and rename work on a hole.
use super::{Compiler, Diagnostic, ProcId, Result, Span, ast};
use crate::ast::ExprKind as E;
use crate::interpolate::{self, Mode};
use crate::lexer;
use crate::parser;
use std::rc::Rc;

impl Compiler {
    /// The mode of `p`'s `@Interpolates` note, and the index of the parameter it marks.
    fn interpolation_of(&mut self, p: ProcId, span: Span) -> Option<(usize, Mode)> {
        let mode = {
            let info = self.proc(p);
            let decl = self
                .pr
                .proc_decl_notes
                .get(&p)
                .map_or(&[][..], Vec::as_slice);
            info.lit
                .header
                .notes
                .iter()
                .chain(decl)
                .find_map(|n| interpolate::note_mode(&n.text))?
        };
        // A constant parameter (`$sql: string`) has no runtime signature entry to read the type
        // from; its spelling stands in.
        let at = self.format_string_param(p, span).or_else(|| {
            let is_string = |t: &Option<ast::Expr>| {
                t.as_ref()
                    .is_some_and(|t| matches!(&t.kind, E::Ident(n) if n.as_str() == "string"))
            };
            let params = &self.proc(p).lit.header.params;
            params
                .windows(2)
                .position(|w| is_string(&w[0].ty) && w[1].variadic)
        })?;
        Some((at, mode))
    }

    /// The call's arguments with the holes of an interpolated literal made positional, or
    /// `None` when the call has nothing to rewrite.
    pub(super) fn expand_interpolation(
        &mut self,
        procs: &[ProcId],
        args: &[ast::Arg],
        span: Span,
    ) -> Result<Option<Vec<ast::Arg>>> {
        // A literal argument is the only thing worth the work of looking at the callee.
        if !args.iter().any(|a| matches!(a.value.kind, E::Str(_))) {
            return Ok(None);
        }
        // Overloads: the candidates that carry the note must agree on where the string goes
        // and how it is read; the others take the rewritten call or reject it like any other.
        let mut found: Option<(usize, Mode)> = None;
        let mut noted = procs[0];
        for &p in procs {
            if let Some(this) = self.interpolation_of(p, span) {
                match found {
                    None => {
                        found = Some(this);
                        noted = p;
                    }
                    Some(first) if first != this => return Ok(None),
                    Some(_) => {}
                }
            }
        }
        let Some((param, mode)) = found else {
            return Ok(None);
        };
        let name = self.proc(noted).lit.header.params[param].name;
        // The argument for that parameter: named, or the one at its position.
        let mut positional = 0;
        let mut at = None;
        for (k, a) in args.iter().enumerate() {
            if a.spread || a.context || a.target.is_some() {
                break;
            }
            match a.name {
                Some(n) if name.is_some_and(|p| p.name == n.name) => {
                    at = Some(k);
                    break;
                }
                Some(_) => {}
                None if positional == param => {
                    at = Some(k);
                    break;
                }
                None => positional += 1,
            }
        }
        let Some(at) = at else {
            return Ok(None);
        };
        let literal = &args[at].value;
        let E::Str(value) = &literal.kind else {
            return Ok(None);
        };
        if self.ed.is_live() {
            self.ide_note_interpolation(literal.span, mode);
        }
        let holes = interpolate::holes(value, mode);
        if holes.is_empty() {
            return Ok(None);
        }
        let file = literal.span.file;
        if (file.0 as usize) >= self.src.sources.len() {
            return Ok(None);
        }
        let spelling = self.src.sources.snippet(literal.span);
        let Some(map) = lexer::string_value_map(spelling, value.len()) else {
            return Ok(None);
        };
        let base = literal.span.start as usize;
        let explicit = args[at + 1..]
            .iter()
            .any(|a| a.name.is_none() && !a.context || a.spread);
        if explicit {
            return Err(Box::new(
                Diagnostic::error(
                    literal.span,
                    "a mix of inline holes and positional arguments",
                )
                .with_help(
                    "write every value as a `%name` hole, or pass them all as arguments after the string",
                ),
            ));
        }
        let mut values = Vec::with_capacity(holes.len());
        for hole in &holes {
            values.push(self.parse_hole(value, hole, &map, base, file)?);
        }
        let mut out = Vec::with_capacity(args.len() + values.len());
        out.extend_from_slice(&args[..at]);
        let mut rewritten = args[at].clone();
        rewritten.value.kind = E::Str(Rc::from(interpolate::rewrite(value, &holes)));
        self.pr.interpolated.insert(rewritten.value.span);
        out.push(rewritten);
        out.extend(values.into_iter().map(|value| ast::Arg {
            name: None,
            target: None,
            context: false,
            spread: false,
            value,
        }));
        out.extend_from_slice(&args[at + 1..]);
        Ok(Some(out))
    }

    /// The expression of `hole` in the string `value`, with spans in the source text.
    fn parse_hole(
        &self,
        value: &[u8],
        hole: &interpolate::Hole,
        map: &[u32],
        base: usize,
        file: super::FileId,
    ) -> Result<ast::Expr> {
        let range = hole.expr.clone();
        let whole = Span::new(
            file,
            base + map[hole.whole.start] as usize,
            base + map[hole.whole.end] as usize,
        );
        let Ok(text) = std::str::from_utf8(&value[range.clone()]) else {
            return Err(Box::new(Diagnostic::error(
                whole,
                "an inline hole must be valid UTF-8 text",
            )));
        };
        if text.trim().is_empty() {
            return Err(Box::new(Diagnostic::error(
                whole,
                "an inline hole is empty",
            )));
        }
        let moved = |span: Span| Span {
            file,
            start: (base + map[(range.start + span.start as usize).min(range.end)] as usize) as u32,
            end: (base + map[(range.start + span.end as usize).min(range.end)] as usize) as u32,
        };
        let remap = |mut d: Diagnostic| {
            d.span = moved(d.span);
            for note in &mut d.notes {
                note.0 = moved(note.0);
            }
            d.suggestions = None;
            Box::new(d)
        };
        let mut tokens = lexer::lex(file, text).map_err(remap)?;
        // Tokens carry spans of `text`; the parser's own errors and the tree it builds get the
        // spans of the file.
        let mut kept = Vec::with_capacity(tokens.len());
        for mut token in tokens.drain(..) {
            token.span = moved(token.span);
            kept.push(token);
        }
        parser::parse_expression(text, kept).map_err(|mut d| {
            d.suggestions = None;
            Box::new(d)
        })
    }
}
