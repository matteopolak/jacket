//! Complete type definitions and enclosing compile-time guards, independent of source layout.
use super::super::refs::{TypeRef, refs_json, refs_named};
use super::{DeclarationToken, Member, json, path};
use crate::ast::{File, Stmt, StmtKind};
use crate::fxhash::HashMap;
use crate::lexer::{Tok, lex};
use crate::source::FileId;
use std::path::{Path, PathBuf};

/// One enclosing compile-time condition; `taken=false` denotes its else branch.
#[derive(Clone, Debug, Default)]
pub struct CompileGuard {
    pub file: PathBuf,
    pub start: u32,
    pub end: u32,
    pub tokens: Vec<String>,
    pub taken: bool,
}

/// One actual parsed definition. Byte bounds identify private source for the formatter;
/// author whitespace is never copied into this semantic contract.
#[derive(Debug, Default)]
pub struct Definition {
    pub kind: &'static str,
    pub docs: String,
    pub file: PathBuf,
    pub line: usize,
    pub start: u32,
    pub end: u32,
    pub platforms: Vec<String>,
    pub guards: Vec<CompileGuard>,
    pub scope: Vec<String>,
    pub tokens: Vec<DeclarationToken>,
    pub members: Vec<Member>,
    pub refs: Vec<TypeRef>,
}

/// Enclosing guard facts for each top-level name, including conditions unrelated to OS.
pub(super) fn guards(file: &File, text: &str, path: &Path) -> HashMap<u32, Vec<CompileGuard>> {
    fn walk(
        stmts: &[Stmt],
        text: &str,
        path: &Path,
        chain: &[CompileGuard],
        out: &mut HashMap<u32, Vec<CompileGuard>>,
    ) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::StaticIf(branches) => {
                    let extent = branches.cond.extent();
                    let source = text
                        .get(extent.start as usize..extent.end as usize)
                        .unwrap_or("");
                    // An invalid empty guard remains explicit and is rejected by strict ingestion.
                    let tokens = match lex(FileId(0), source) {
                        Ok(tokens) => tokens
                            .into_iter()
                            .filter(|token| !matches!(token.tok, Tok::Eof))
                            .map(|token| {
                                source[token.span.start as usize..token.span.end as usize]
                                    .to_owned()
                            })
                            .collect(),
                        Err(_) => Vec::new(),
                    };
                    for (taken, body) in [
                        (true, &branches.then_branch),
                        (false, &branches.else_branch),
                    ] {
                        let mut nested = chain.to_vec();
                        nested.push(CompileGuard {
                            file: path.to_path_buf(),
                            start: extent.start,
                            end: extent.end,
                            tokens: tokens.clone(),
                            taken,
                        });
                        walk(body, text, path, &nested, out);
                    }
                }
                StmtKind::Decl(decl) => {
                    for name in &decl.names {
                        out.insert(name.span.start, chain.to_vec());
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = HashMap::default();
    walk(&file.stmts, text, path, &[], &mut out);
    out
}

/// Serialize complete definition facts with the same link resolution as other model comments.
pub(super) fn to_json(
    definition: &Definition,
    md: &dyn Fn(&str, &Path, usize) -> String,
) -> String {
    let strings = |values: &[String]| {
        values
            .iter()
            .map(|value| json(value))
            .collect::<Vec<_>>()
            .join(",")
    };
    let guards = definition
        .guards
        .iter()
        .map(|guard| {
            format!(
                "{{\"file\":{},\"start\":{},\"end\":{},\"tokens\":[{}],\"taken\":{}}}",
                path(&guard.file),
                guard.start,
                guard.end,
                strings(&guard.tokens),
                guard.taken
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let tokens=definition.tokens.iter().map(|token|format!("{{\"text\":{},\"start\":{},\"end\":{},\"statement\":{},\"reference\":{},\"names\":[{}],\"docs\":{}}}",json(&token.text),token.start,token.end,token.statement,token.reference,strings(&token.names),json(&md(&token.docs,&definition.file,definition.line)))).collect::<Vec<_>>().join(",");
    let members = definition
        .members
        .iter()
        .map(|member| {
            format!(
                "{{\"name\":{},\"type\":{},\"doc\":{}{}}}",
                json(&member.name),
                json(&member.ty),
                json(&md(&member.doc, &definition.file, definition.line)),
                refs_json(&member.refs)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"kind\":{},\"docs\":{},\"file\":{},\"line\":{},\"start\":{},\"end\":{},\"platforms\":[{}],\"guards\":[{}],\"scope\":[{}],\"tokens\":[{}],\"members\":[{}]{}}}",
        json(definition.kind),
        json(&md(&definition.docs, &definition.file, definition.line)),
        path(&definition.file),
        definition.line,
        definition.start,
        definition.end,
        strings(&definition.platforms),
        guards,
        strings(&definition.scope),
        tokens,
        members,
        refs_named("refs", &definition.refs)
    )
}
