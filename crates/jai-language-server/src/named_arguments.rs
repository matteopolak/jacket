//! Named arguments are another completion source, never a replacement for expression items.
use crate::{CompletionItem, CompletionKind, DocumentUri, Session, analysis::Span};
use jaic::lexer::{self, P, Tok, Token};
use jaic::sema::ide::IdeWhat;
use jaic::source::FileId;
use std::collections::BTreeMap;

struct Context {
    callee: usize,
    open: usize,
    end: usize,
    active: usize,
    args: Vec<Argument>,
    start: usize,
    replace_end: usize,
    prefix: String,
    has_equals: bool,
}

#[derive(Clone)]
struct Argument {
    name: Option<String>,
    spread: bool,
    value: Option<(usize, usize)>,
    literal: bool,
}

struct Parameter {
    name: Option<String>,
    variadic: bool,
    detail: String,
}

struct Callable {
    params: Vec<Parameter>,
    eligible: Option<Vec<bool>>,
}

fn procedure(compiler: &jaic::sema::Compiler, id: jaic::sema::ProcId) -> Callable {
    let signature = compiler.ide_signature(id);
    let params = compiler
        .proc(id)
        .lit
        .header
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| Parameter {
            name: p.name.map(|n| n.name.as_str().to_owned()),
            variadic: p.variadic,
            detail: signature.params.get(i).cloned().unwrap_or_default(),
        })
        .collect();
    Callable {
        params,
        eligible: None,
    }
}

fn opener(p: P) -> Option<P> {
    match p {
        P::LParen => Some(P::RParen),
        P::LBrace | P::DotBrace => Some(P::RBrace),
        P::LBracket | P::DotBracket => Some(P::RBracket),
        _ => None,
    }
}

fn context(text: &str, byte: usize) -> Option<Context> {
    // Comments and literals are opaque to the parser, including their commas and brackets.
    // The prefix also detects a literal/comment which has not been closed yet.
    let regions = lexer::lexical_regions(FileId(0), text)
        .or_else(|_| lexer::lexical_regions(FileId(0), &text[..byte]))
        .ok()?;
    if regions
        .iter()
        .any(|s| s.start as usize <= byte && byte < s.end as usize)
    {
        return None;
    }
    let (tokens, errors) = lexer::lex_ide(FileId(0), text);
    if errors
        .iter()
        .any(|e| e.span.start as usize <= byte && byte <= e.span.end as usize)
    {
        return None;
    }
    let mut stack = Vec::new();
    for (i, t) in tokens
        .iter()
        .enumerate()
        .take_while(|(_, t)| (t.span.start as usize) < byte)
    {
        if let Tok::Punct(p) = t.tok {
            if let Some(close) = opener(p) {
                stack.push((i, close));
            } else if matches!(p, P::RParen | P::RBrace | P::RBracket) {
                if stack.last().is_some_and(|&(_, close)| close == p) {
                    stack.pop();
                } else {
                    stack.clear();
                }
            } else if p == P::Semi {
                // A semicolon is a statement boundary even after an unfinished call.
                while stack.last().is_some_and(|&(_, close)| close != P::RBrace) {
                    stack.pop();
                }
            }
        }
    }
    let &(open, close) = stack.last()?;
    if close != P::RParen || open == 0 || !matches!(tokens[open - 1].tok, Tok::Ident(_)) {
        return None;
    }
    let mut groups: Vec<Vec<&Token>> = vec![Vec::new()];
    let mut nested = Vec::new();
    let mut active = 0;
    let mut end = text.len();
    for t in &tokens[open + 1..] {
        if let Tok::Punct(p) = t.tok {
            if nested.is_empty() {
                if p == P::RParen {
                    end = t.span.end as usize;
                    break;
                }
                if matches!(p, P::Semi | P::RBrace | P::RBracket) {
                    end = t.span.start as usize;
                    break;
                }
                if p == P::Comma {
                    if groups.last()?.is_empty() && groups.len() > 1 {
                        return None;
                    } // context override `,,`
                    groups.push(Vec::new());
                    if (t.span.start as usize) < byte {
                        active = groups.len() - 1;
                    }
                    continue;
                }
            }
            if let Some(close) = opener(p) {
                nested.push(close);
            } else if nested.last() == Some(&p) {
                nested.pop();
            }
        }
        if !matches!(t.tok, Tok::Eof) {
            groups.last_mut()?.push(t);
        }
    }
    let current = &groups[active];
    let before: Vec<_> = current
        .iter()
        .filter(|t| (t.span.start as usize) < byte)
        .collect();
    if before.len() > 1
        || before
            .first()
            .is_some_and(|t| !matches!(t.tok, Tok::Ident(_)))
    {
        return None;
    }
    let word = before.first().copied().copied().or_else(|| {
        current
            .first()
            .copied()
            .filter(|t| t.span.start as usize == byte && matches!(t.tok, Tok::Ident(_)))
    });
    let start = word.map_or(byte, |t| t.span.start as usize);
    let mut replace_end = word.map_or(byte, |t| t.span.end as usize);
    let separator = current.get(usize::from(word.is_some()));
    let has_equals = separator.is_some_and(|t| t.tok == Tok::Punct(P::Eq));
    if separator.is_some_and(|t| matches!(t.tok, Tok::Punct(P::Colon | P::ColonEq))) {
        replace_end = separator.unwrap().span.end as usize;
    } else if separator.is_some() && !has_equals {
        return None;
    }
    let args = groups
        .iter()
        .map(|g| {
            let name = match g.as_slice() {
                [
                    Token {
                        tok: Tok::Ident(name),
                        ..
                    },
                    Token {
                        tok: Tok::Punct(P::Eq),
                        ..
                    },
                    ..,
                ] => Some(name.as_str().to_owned()),
                _ => None,
            };
            let spread = g
                .get(if name.is_some() {
                    2
                } else {
                    0
                })
                .is_some_and(|t| t.tok == Tok::Punct(P::DotDot));
            let value_tokens =
                &g[(usize::from(name.is_some()) * 2 + usize::from(spread)).min(g.len())..];
            let value = value_tokens
                .first()
                .zip(value_tokens.last())
                .map(|(first, last)| (first.span.start as usize, last.span.end as usize));
            let literal = matches!(
                value_tokens,
                [Token {
                    tok: Tok::Int(_) | Tok::Float(_),
                    ..
                }] | [
                    Token {
                        tok: Tok::Punct(P::Minus | P::Plus),
                        ..
                    },
                    Token {
                        tok: Tok::Int(_) | Tok::Float(_),
                        ..
                    }
                ]
            );
            Argument {
                name,
                spread,
                value,
                literal,
            }
        })
        .collect();
    Some(Context {
        callee: tokens[open - 1].span.start as usize,
        open: tokens[open].span.start as usize,
        end,
        active,
        args,
        start,
        replace_end,
        prefix: text[start..byte].to_owned(),
        has_equals,
    })
}

impl Callable {
    /// Mirror direct-call slot assignment for the actual declaration header.
    /// Missing parameters are intentional while editing; conflicting supplied arguments aren't.
    fn bindings(&self, args: &[Argument], active: usize, candidate: &str) -> Option<Vec<usize>> {
        let mut slots = vec![0u8; self.params.len()]; // 0 empty, 1 scalar/spread, 2 variadic list
        let mut next = 0;
        let mut bindings = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let name = if i == active {
                Some(candidate)
            } else {
                arg.name.as_deref()
            };
            if let Some(name) = name {
                let p = self
                    .params
                    .iter()
                    .position(|p| p.name.as_deref() == Some(name))?;
                if slots[p] != 0 {
                    return None;
                }
                slots[p] = if self.params[p].variadic && !arg.spread {
                    2
                } else {
                    1
                };
                bindings.push(p);
                next = p + usize::from(!self.params[p].variadic);
            } else {
                while next < slots.len() && slots[next] != 0 && !self.params[next].variadic {
                    next += 1;
                }
                if next == slots.len() {
                    return None;
                }
                if self.params[next].variadic && !arg.spread {
                    if slots[next] == 1 {
                        return None;
                    }
                    slots[next] = 2;
                    bindings.push(next);
                } else {
                    slots[next] = 1;
                    bindings.push(next);
                    next += 1;
                }
            }
        }
        Some(bindings)
    }
}

fn resolved(
    compiler: &mut jaic::sema::Compiler,
    file: FileId,
    ctx: &Context,
    typed: bool,
) -> Option<Vec<Callable>> {
    let ide = compiler.ide.as_ref()?;
    let selected = ide.calls.iter().find(|call| {
        call.span.file == file
            && !call.ambiguous
            && call.span.start as usize <= ctx.callee
            && call.span.end as usize == ctx.end
    });
    let mut ids = if let Some(call) = selected.filter(|_| typed) {
        vec![call.chosen]
    } else {
        ide.refs_at(file, ctx.callee as u32)
            .iter()
            .flat_map(|r| match &r.what {
                IdeWhat::Procs(ids) => ids.clone(),
                _ => Vec::new(),
            })
            .collect()
    };
    ids.sort_by_key(|p| p.0);
    ids.dedup();
    if ids.is_empty() {
        return None;
    }
    let facts: Vec<_> = ctx
        .args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            if !typed || (i == ctx.active && !ctx.has_equals) {
                return None;
            }
            let (start, end) = arg.value?;
            ide.exprs
                .get(&jaic::source::Span::new(file, start, end))
                .filter(|fact| !fact.conflicting)
                .map(|fact| (fact.ty, fact.constant))
        })
        .collect();
    let mut result = Vec::new();
    for id in ids {
        // Invalid or unresolved headers aren't a source of invented completion signatures.
        if compiler.refresh_implicit_poly(id).is_err() {
            continue;
        }
        let mut callable = procedure(compiler, id);
        if !compiler.proc(id).is_poly {
            let Ok(signature) = compiler.signature(id, compiler.proc(id).span) else {
                continue;
            };
            callable.eligible = Some(
                callable
                    .params
                    .iter()
                    .map(|param| {
                        let Some(name) = &param.name else {
                            return false;
                        };
                        let Some(bindings) = callable.bindings(&ctx.args, ctx.active, name) else {
                            return false;
                        };
                        bindings.iter().enumerate().all(|(i, &slot)| {
                            let Some((from, constant)) = facts[i] else {
                                return true;
                            };
                            let header = &compiler.proc(id).lit.header;
                            if header.params[slot].baked {
                                return true;
                            }
                            let runtime_slot =
                                header.params[..slot].iter().filter(|p| !p.baked).count();
                            let Some(target) = signature.params.get(runtime_slot) else {
                                return false;
                            };
                            let mut to = target.ty;
                            if target.variadic
                                && !ctx.args[i].spread
                                && let jaic::types::TypeKind::Array {
                                    elem, ..
                                } = compiler.types.kind(to)
                            {
                                to = *elem;
                            }
                            // Expression facts lack operand flags/values. Only exclude a
                            // candidate when these facts prove scalar incompatibility. Code
                            // quotation, procedure values, auto-dereference, contextual
                            // aggregates and other operand-aware conversions stay unresolved.
                            use jaic::types::TypeKind;
                            let scalar = |kind: &TypeKind| {
                                matches!(
                                    kind,
                                    TypeKind::Bool
                                        | TypeKind::Int { .. }
                                        | TypeKind::Float { .. }
                                        | TypeKind::String
                                )
                            };
                            if !scalar(compiler.types.kind(from))
                                || !scalar(compiler.types.kind(to))
                            {
                                return true;
                            }
                            // Constants can carry untyped arithmetic or one-byte string
                            // conversions; the type fact cannot establish their value/flags.
                            if constant
                                && ((matches!(
                                    compiler.types.kind(from),
                                    TypeKind::Int { .. } | TypeKind::Float { .. }
                                ) && matches!(
                                    compiler.types.kind(to),
                                    TypeKind::Int { .. } | TypeKind::Float { .. }
                                ) && !ctx.args[i].literal)
                                    || (from == jaic::types::TypeId::STRING
                                        && to == jaic::types::TypeId::U8))
                            {
                                return true;
                            }
                            compiler
                                .implicit_cost(from, ctx.args[i].literal, to)
                                .is_some()
                        })
                    })
                    .collect(),
            );
        }
        result.push(callable);
    }
    Some(result)
}

impl Session {
    pub(crate) fn named_argument_completions(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Vec<CompletionItem> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let Some(ctx) = context(&doc.text, byte) else {
            return Vec::new();
        };
        let current = self.with_semantic(uri, &doc.text, |a, path| {
            let file = a.file(path)?;
            resolved(&mut a.compiler, file, &ctx, true)
        });
        // Retain the actual callee expression and scope. An unfinished argument list may have
        // been dropped by IDE parser recovery; eliding only that list lets normal expression
        // resolution produce its real overloads/type. This view is never used for diagnostics.
        let mut view = doc.text.as_bytes().to_vec();
        for c in &mut view[ctx.open + 1..ctx.end] {
            if *c != b'\n' && *c != b'\r' {
                *c = b' ';
            }
        }
        if doc.text.as_bytes().get(ctx.end.wrapping_sub(1)) == Some(&b')') {
            view[ctx.end - 1] = b')';
        } else {
            view[ctx.open] = b';';
        }
        let view = String::from_utf8(view).expect("ASCII masking preserves UTF-8");
        let callables = current
            .or_else(|| {
                self.with_semantic(uri, &view, |a, path| {
                    let file = a.file(path)?;
                    // Procedure-value types are structural. Their current global name map
                    // is not declaration-bound and can contain another callable's names.
                    // Require resolved procedure identity; never manufacture those names.
                    resolved(&mut a.compiler, file, &ctx, false)
                })
            })
            .unwrap_or_default();
        let Ok(range) = doc
            .index
            .range(&doc.text, Span::new(ctx.start, ctx.replace_end))
        else {
            return Vec::new();
        };
        let mut items = BTreeMap::new();
        for callable in callables {
            for (index, p) in callable.params.iter().enumerate() {
                let Some(name) = &p.name else {
                    continue;
                };
                if !name.to_lowercase().starts_with(&ctx.prefix.to_lowercase())
                    || callable
                        .eligible
                        .as_ref()
                        .is_some_and(|flags| !flags[index])
                    || callable.bindings(&ctx.args, ctx.active, name).is_none()
                {
                    continue;
                }
                items.entry(name.clone()).or_insert_with(|| CompletionItem {
                    label: name.clone(),
                    kind: CompletionKind::Field,
                    detail: format!("parameter {}", p.detail),
                    insert_text: Some(if ctx.has_equals {
                        name.clone()
                    } else {
                        format!("{name} = ")
                    }),
                    replace: Some(range),
                    ..CompletionItem::default()
                });
            }
        }
        items.into_values().take(self.limits.symbols).collect()
    }
}
