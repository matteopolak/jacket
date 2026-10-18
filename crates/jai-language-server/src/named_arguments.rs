//! Named arguments are another completion source, never a replacement for expression items.
use crate::{CompletionItem, CompletionKind, DocumentUri, Session, analysis::Span};
use jaic::parser::ide_facts::{self, CallArgument, CallSite};
use jaic::sema::ide::IdeWhat;
use jaic::source::FileId;
use std::collections::BTreeMap;

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

impl Callable {
    /// Mirror direct-call slot assignment for the actual declaration header.
    /// Missing parameters are intentional while editing; conflicting supplied arguments aren't.
    fn bindings(
        &self,
        args: &[CallArgument],
        active: usize,
        candidate: &str,
    ) -> Option<Vec<usize>> {
        let mut slots = vec![0u8; self.params.len()]; // 0 empty, 1 scalar/spread, 2 variadic list
        let mut next = 0;
        let mut bindings = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let name = if i == active {
                Some(candidate)
            } else {
                arg.name.map(jaic::intern::Sym::as_str)
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

/// Expression facts lack operand flags/values: only prove scalar incompatibility.
fn compatible(
    compiler: &mut jaic::sema::Compiler,
    argument: &CallArgument,
    from: jaic::types::TypeId,
    constant: bool,
    to: jaic::types::TypeId,
) -> bool {
    // Expression facts lack operand flags/values. Only exclude a
    // candidate when these facts prove scalar incompatibility. Code
    // quotation, procedure values, auto-dereference, contextual
    // aggregates and other operand-aware conversions stay unresolved.
    use jaic::types::TypeKind;
    let scalar = |kind: &TypeKind| {
        matches!(
            kind,
            TypeKind::Bool | TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::String
        )
    };
    if !scalar(compiler.types.kind(from)) || !scalar(compiler.types.kind(to)) {
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
        ) && !argument.literal)
            || (from == jaic::types::TypeId::STRING && to == jaic::types::TypeId::U8))
    {
        return true;
    }
    compiler.implicit_cost(from, argument.literal, to).is_some()
}

fn callback(
    compiler: &mut jaic::sema::Compiler,
    ty: jaic::types::TypeId,
    header: &jaic::sema::ProcTypeParams,
    ctx: &CallSite,
    facts: &[Option<(jaic::types::TypeId, bool)>],
) -> Option<Callable> {
    let jaic::types::TypeKind::Proc(signature) = compiler.types.kind(ty) else {
        return None;
    };
    let signature = signature.clone();
    if header.names.len() != signature.params.len()
        || header.defaults.len() != signature.params.len()
    {
        return None;
    }
    let mut callable = Callable {
        params: signature
            .params
            .iter()
            .enumerate()
            .map(|(i, &ty)| {
                let name = header.names[i].map(|n| n.as_str().to_owned());
                let mut detail = format!(
                    "{}: {}",
                    name.as_deref().unwrap_or("_"),
                    compiler.types.name(ty)
                );
                if let Some(default) = &header.defaults[i] {
                    detail.push_str(" = ");
                    detail.push_str(compiler.sources.snippet(default.span));
                }
                Parameter {
                    name,
                    variadic: signature.variadic && i + 1 == signature.params.len(),
                    detail,
                }
            })
            .collect(),
        eligible: None,
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
                    let mut to = signature.params[slot];
                    if callable.params[slot].variadic
                        && !ctx.args[i].spread
                        && let jaic::types::TypeKind::Array {
                            elem, ..
                        } = compiler.types.kind(to)
                    {
                        to = *elem;
                    }
                    compatible(compiler, &ctx.args[i], from, constant, to)
                })
            })
            .collect(),
    );
    Some(callable)
}

fn resolved(
    compiler: &mut jaic::sema::Compiler,
    file: FileId,
    ctx: &CallSite,
) -> Option<Vec<Callable>> {
    let ide = compiler.ide.as_ref()?;
    let selected = ide.calls.iter().find(|call| {
        call.span.file == file
            && !call.ambiguous
            && call.span.start <= ctx.callee.start
            && call.span.end == ctx.end
    });
    let mut ids = if let Some(call) = selected {
        vec![call.chosen]
    } else if ctx.callee_is_name {
        ide.refs_at(file, ctx.callee.start)
            .iter()
            .flat_map(|r| match &r.what {
                IdeWhat::Procs(ids) => ids.clone(),
                _ => Vec::new(),
            })
            .collect()
    } else {
        Vec::new()
    };
    ids.sort_by_key(|p| p.0);
    ids.dedup();
    let facts: Vec<_> = ctx
        .args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            if i == ctx.active && !ctx.has_equals {
                return None;
            }
            let value = arg.value?;
            ide.exprs
                .get(&jaic::source::Span {
                    file,
                    ..value
                })
                .filter(|fact| !fact.conflicting)
                .map(|fact| (fact.ty, fact.constant))
        })
        .collect();
    if ids.is_empty() {
        let Some(target) = compiler.ide_call_target_at(file, ctx.open) else {
            return Some(Vec::new());
        };
        if let Some((ty, header)) = target.parameters {
            return Some(
                callback(compiler, ty, &header, ctx, &facts)
                    .into_iter()
                    .collect(),
            );
        }
        ids = target.procedures;
    }
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
                            compatible(compiler, &ctx.args[i], from, constant, to)
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
        let Some(ctx) = self.call_site(uri, &doc.text, byte) else {
            return Vec::new();
        };
        let (start, end) = (ctx.word.start as usize, ctx.word.end as usize);
        let prefix = doc.text[start..byte].to_lowercase();
        let callables = self
            .with_semantic(uri, &doc.text, |a, path| {
                let file = a.file(path)?;
                resolved(&mut a.compiler, file, &ctx)
            })
            .unwrap_or_default();
        let Ok(range) = doc.index.range(&doc.text, Span::new(start, end)) else {
            return Vec::new();
        };
        let mut items = BTreeMap::new();
        for callable in callables {
            for (index, p) in callable.params.iter().enumerate() {
                let Some(name) = &p.name else {
                    continue;
                };
                if !name.to_lowercase().starts_with(&prefix)
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

    /// The call around `byte`, from the tokens of the document's analysis.
    fn call_site(&self, uri: &DocumentUri, text: &str, byte: usize) -> Option<CallSite> {
        let analysis = self.analysis_for(uri)?;
        ide_facts::call_at(
            text,
            &analysis.syntax_tokens,
            &analysis.lex_errors,
            byte as u32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaic::sema::{Compiler, Options, VirtualFs, ide::IdeFacts};
    use std::{path::Path, rc::Rc};

    #[test]
    fn incomplete_call_query_cannot_make_a_failed_body_executable() {
        let source = "Context :: struct { state: int; }\ncounter: int;\nf :: (option := false) #no_context {}\ndamaged :: () #no_context {\n    counter += 1;\n    f(op\n}\n#run damaged();\n";
        let byte = source.rfind("op\n").unwrap() as u32 + 2;
        let (tokens, _) = jaic::lexer::lex_ide(FileId(0), source);
        let ctx = ide_facts::call_at(source, &tokens, &[], byte).unwrap();
        let mut fs = VirtualFs::default();
        fs.insert("/named-safety/main.jai", source.as_bytes());
        let mut options = Options::host();
        options.runtime_support = false;
        options.stack_trace = false;
        let mut compiler = Compiler::new(options, Rc::new(fs));
        compiler.ide = Some(Box::new(IdeFacts::new(vec![
            jaic::sema::ide::IdeRoot::under("/named-safety/"),
        ])));
        assert!(
            compiler
                .compile_program(Path::new("/named-safety/main.jai"))
                .is_err()
        );
        compiler.ide_check_all();
        let file = (0..compiler.sources.len() as u32)
            .map(FileId)
            .find(|&f| {
                compiler.sources.get(f).path.as_file() == Some(Path::new("/named-safety/main.jai"))
            })
            .unwrap();
        let errors = compiler.ide.as_ref().unwrap().parse_errors.len();
        assert!(errors > 0);
        let procs = compiler.ide_callee_at(file, ctx.open);
        assert_eq!(procs.len(), 1);
        assert_eq!(compiler.proc(procs[0]).name.as_str(), "f");
        assert_eq!(
            &*compiler.sources.get(file).text,
            source,
            "metadata query must not rewrite program text"
        );
        assert_eq!(compiler.ide.as_ref().unwrap().parse_errors.len(), errors);
        assert!(
            compiler
                .compile_program(Path::new("/named-safety/main.jai"))
                .is_err(),
            "metadata query must retain the failed-body execution guard"
        );
    }
}
