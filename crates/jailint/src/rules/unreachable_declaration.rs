//! `unreachable_declaration`: a top-level declaration of a program that nothing reachable from
//! its roots uses.
//!
//! ```jai
//! describe :: (n: int) -> string { return "number"; }    // only `report` calls it
//! report   :: (n: int) { log(describe(n)); }             // nothing calls it
//!
//! main :: () { }
//! ```
//!
//! Where `unused_private_declaration` looks at one file's `#scope_file` names, this follows uses
//! through every file the program `#load`s: a declaration is live when the entry point, a
//! `#run`, an `#if`, another `#load`ed file's statements or other code that is live uses it. A
//! helper that only dead code calls is dead too, as are two procedures that only call each
//! other. Delete the group, or call it.
//!
//! The roots are the programs jailint compiles (the entries, tests, benches, build files and
//! examples it was given). A file two programs share is judged by all of them: it is reported
//! only when none uses the declaration. What counts as a use:
//!
//! - a name written in live code, including the condition of an `#if`, a `#run` expression, the
//!   body of an `#expand` macro and a type named in a struct reached through `using`;
//! - a call, which keeps the one overload it selects; naming the overload set (`p := show;`)
//!   keeps every overload;
//! - a field or enum member used through its struct or enum;
//! - a name written in code the compiler did not check (an `#if` branch for another system), in
//!   the argument of a note (`@cases(TABLE)`) or as `Alias.member` of a module alias, which
//!   counts by its spelling.
//!
//! These are roots without a use: the entry point, `#program_export`, `#c_call`, `#foreign`,
//! `#compiler`, `#expand` and operator declarations, declarations with notes, `using` and
//! `` ` `` declarations, declarations that bind several names, `for_expansion`, `get_plugin`
//! and `build`, and names starting with `__`.
//!
//! Only declarations in the program's own files are judged. A module's exported names are its
//! API and are never reported, and the rule stays quiet when the program did not compile
//! without errors, because code that was not checked records no uses.
//!
//! No fix: deleting is the author's call.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::ExprKind as E;
use jaic::ast::StmtKind as S;
use jaic::ast::walk::Node;
use jaic::fxhash::{HashMap, HashSet};
use jaic::lexer::Tok;
use jaic::sema::Compiler;
use jaic::source::{FileId, Span};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unreachable_declaration",
    category: crate::config::Category::Pedantic,
    summary: "a top-level declaration that nothing reachable from the program's roots uses",
    check,
};

/// A declaration the program never reaches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Dead {
    pub file: FileId,
    /// The declared name.
    pub name: Span,
    pub text: String,
}

/// The unreachable declarations of every program file of one compile.
#[derive(Clone, Debug, Default)]
pub(crate) struct Reach {
    pub dead: Vec<Dead>,
}

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let Some(Some(reach)) = cx.run.reach.get() else {
        return;
    };
    for dead in reach.dead.iter().filter(|d| d.file == cx.file) {
        out.push(finding(
            dead.name,
            format!(
                "`{}` is not used by anything reachable from the program's roots",
                dead.text
            ),
            Some("delete it, or use it from code that runs".into()),
        ));
    }
}

/// The statement contains a `#run`, which executes whether or not anything uses the name it
/// declares.
fn runs_at_compile_time(stmt: &jaic::ast::Stmt) -> bool {
    let mut found = false;
    jaic::ast::walk::walk(std::slice::from_ref(stmt), &mut |node, _| {
        found |= matches!(node, Node::Expr(e) if matches!(e.kind, E::Run { .. }));
        !found
    });
    found
}

/// One top-level statement of a program file.
struct Item {
    span: Span,
    /// Declared name of a judged declaration; the rest are roots.
    judged: Option<(Span, String)>,
}

/// The unreachable declarations of the program `compiler` built, or `None` when it did not
/// compile cleanly (`complete`), because unchecked code records no uses.
pub(crate) fn analyze(compiler: &mut Compiler, complete: bool) -> Option<Reach> {
    if !complete {
        return None;
    }
    let main = compiler.ns.main_module?;
    let program_files: Vec<FileId> = compiler
        .src
        .files
        .iter()
        .filter(|f| f.module == main && f.path.as_file().is_some())
        .map(|f| f.id)
        .collect();
    let mut items: HashMap<FileId, Vec<Item>> = HashMap::default();
    for &file in &program_files {
        let text = compiler.src.sources.get(file).text.clone();
        let Ok(ast) = jaic::parser::parse_file(file, &text) else {
            return None;
        };
        let entry = compiler.options.entry_point.clone();
        let list = ast
            .stmts
            .iter()
            .map(|stmt| {
                let judged = match &stmt.kind {
                    S::Decl(decl) => match decl.names.as_slice() {
                        [name]
                            if !runs_at_compile_time(stmt)
                                && !super::unused_private_declaration::read_from_outside(
                                    decl,
                                    name.name.as_str(),
                                    &entry,
                                ) =>
                        {
                            Some((name.span, name.name.as_str().to_owned()))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                Item {
                    span: stmt.span,
                    judged,
                }
            })
            .collect();
        items.insert(file, list);
    }
    // Each judged declaration is a node; everything else the program says is a root.
    let mut nodes: Vec<(FileId, usize)> = Vec::new();
    let mut node_of: HashMap<(FileId, usize), usize> = HashMap::default();
    for &file in &program_files {
        for (i, item) in items[&file].iter().enumerate() {
            if item.judged.is_some() {
                node_of.insert((file, i), nodes.len());
                nodes.push((file, i));
            }
        }
    }
    let item_at = |compiler: &Compiler, mut span: Span| -> Option<(FileId, usize)> {
        while let Some(site) = compiler.src.insert_sites.get(&span.file) {
            span = *site;
        }
        let list = items.get(&span.file)?;
        let at = list
            .partition_point(|i| i.span.start <= span.start)
            .checked_sub(1)?;
        (span.start < list[at].span.end).then_some((span.file, at))
    };
    // What each use names, and which top-level statement the use is in.
    let mut recorded: HashSet<(FileId, u32)> = HashSet::default();
    let mut uses: Vec<(Span, bool)> = Vec::new();
    let mut members: Vec<(Span, Span)> = Vec::new();
    if let Some(ide) = compiler.ed.facts() {
        uses.extend(ide.refs.iter().filter(|r| !r.decl).map(|r| (r.span, false)));
        recorded.extend(ide.refs.iter().map(|r| (r.span.file, r.span.start)));
        members.extend(ide.member_uses.iter().filter(|(u, d)| u != d).copied());
    }
    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    let mut reached: Vec<bool> = vec![false; nodes.len()];
    let mut seen: HashSet<(Span, Span)> = HashSet::default();
    let mut link = |from: Option<usize>, to: Option<usize>, edges: &mut Vec<Vec<usize>>| {
        let Some(to) = to else {
            return;
        };
        match from {
            Some(from) if from == to => {}
            Some(from) => edges[from].push(to),
            None => reached[to] = true,
        }
    };
    // Code the compiler did not check (an `#if` branch for another system), the arguments of
    // notes and names that stand for a module record no use of the declaration, so a name
    // written there counts by its spelling.
    let mut named: HashMap<&str, Vec<usize>> = HashMap::default();
    for (n, &(file, i)) in nodes.iter().enumerate() {
        if let Some((_, text)) = &items[&file][i].judged {
            named.entry(text.as_str()).or_default().push(n);
        }
    }
    for (span, _) in uses {
        let Some(source) = item_at(compiler, span) else {
            // Written in a library, or outside any top-level statement: not the program's.
            continue;
        };
        let from = node_of.get(&source).copied();
        let mut resolved = false;
        for target in compiler.ide_definition(span.file, span.start) {
            if !seen.insert((span, target)) {
                continue;
            }
            let to = item_at(compiler, target).and_then(|t| node_of.get(&t).copied());
            resolved |= to.is_some();
            link(from, to, &mut edges);
        }
        // `Alias.name` resolves to the module, not to the `Alias :: Module;` that spelled it.
        if !resolved {
            let spelled = compiler.src.sources.snippet(span).to_string();
            for &to in named.get(spelled.as_str()).into_iter().flatten() {
                link(from, Some(to), &mut edges);
            }
        }
    }
    for (span, decl) in members {
        let Some(source) = item_at(compiler, span) else {
            continue;
        };
        let from = node_of.get(&source).copied();
        let to = item_at(compiler, decl).and_then(|t| node_of.get(&t).copied());
        link(from, to, &mut edges);
    }
    for &file in &program_files {
        let text = compiler.src.sources.get(file).text.clone();
        let Ok(tokens) = jaic::lexer::lex(file, &text) else {
            continue;
        };
        for token in &tokens {
            let words: Vec<&str> = match &token.tok {
                Tok::Ident(name) if !recorded.contains(&(file, token.span.start)) => {
                    vec![name.as_str()]
                }
                Tok::Note(note) => note
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .collect(),
                _ => continue,
            };
            let from = item_at(compiler, token.span).and_then(|s| node_of.get(&s).copied());
            for word in words {
                for &to in named.get(word).into_iter().flatten() {
                    link(from, Some(to), &mut edges);
                }
            }
        }
    }
    let mut stack: Vec<usize> = (0..nodes.len()).filter(|&n| reached[n]).collect();
    while let Some(n) = stack.pop() {
        for &next in &edges[n] {
            if !reached[next] {
                reached[next] = true;
                stack.push(next);
            }
        }
    }
    let dead = nodes
        .iter()
        .enumerate()
        .filter(|&(n, _)| !reached[n])
        .filter_map(|(_, &(file, i))| {
            let (name, text) = items[&file][i].judged.clone()?;
            Some(Dead {
                file,
                name,
                text,
            })
        })
        .collect();
    Some(Reach {
        dead,
    })
}
