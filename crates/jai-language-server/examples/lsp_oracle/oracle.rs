//! The compiler's own answers for one program, recorded by a plain compile with editor facts:
//! what each identifier of the file resolved to, and the struct each literal builds.
use std::path::{Path, PathBuf};
use std::rc::Rc;

use jaic::build::{BuildEnv, Workspaces};
use jaic::ide::{IdeFacts, IdeRoot, IdeWhat};
use jaic::interp::{SandboxHost, SharedHost};
use jaic::lexer::{P, Tok};
use jaic::sema::{Compiler, Options};
use jaic::source::FileId;
use jaic::types::{TypeId, TypeKind};

/// What a use resolved to, as far as the checks need it.
#[derive(Clone, Debug)]
pub(crate) enum Resolved {
    /// A declared name and where (file, first and last line), when it is in a file. A use that
    /// the compile reached more than once (a macro or polymorphic body, once per instance) lists
    /// each declaration it resolved to; any of them is a right answer.
    Entity(Vec<(PathBuf, u32, u32)>),
    /// A parameter that `#bake_arguments` fixed, as the baked copy's body sees it: a constant
    /// whose span is the argument that bakes it. The declaration is the original's parameter,
    /// which the copy has no record of.
    Baked,
    /// A member; `true` when the receiver is a struct, enum or distinct type (whose members
    /// are declared somewhere), `false` for builtin members such as `count` or `data`.
    Member(bool),
    /// A type; `true` when it has a declaration (not `s64` or `string`).
    Type(bool),
    Procs,
    Module,
}

#[derive(Clone, Debug)]
pub(crate) struct Ref {
    pub start: usize,
    pub end: usize,
    pub resolved: Resolved,
}

/// A place in a struct literal where a field name goes, and the fields completion must offer
/// there: inside an empty `.{}`, all of them; at a named field, that one.
#[derive(Clone, Debug)]
pub(crate) struct Literal {
    pub at: usize,
    pub type_name: String,
    pub fields: Vec<String>,
}

/// A note of the file that compile-time code read, and where each reader is (file, 0-based
/// line).
#[derive(Clone, Debug)]
pub(crate) struct ReadNote {
    pub at: usize,
    pub readers: Vec<(PathBuf, u32)>,
}

pub(crate) struct Oracle {
    pub refs: Vec<Ref>,
    pub literals: Vec<Literal>,
    pub notes: Vec<ReadNote>,
}

fn file_of(compiler: &Compiler, path: &Path) -> Option<FileId> {
    let want = path.canonicalize().ok()?;

    (0..compiler.src.sources.len() as u32)
        .map(FileId)
        .find(|&f| {
            compiler
                .src
                .sources
                .get(f)
                .path
                .as_file()
                .and_then(|p| p.canonicalize().ok())
                .is_some_and(|p| p == want)
        })
}

fn line_of(text: &str, byte: usize) -> u32 {
    text[..byte.min(text.len())].matches('\n').count() as u32
}

/// Whether members of `t` are declared in source: a struct or enum, through pointers and
/// distinct wrappers. A distinct string or array only has the builtin members of its base.
fn declared(compiler: &Compiler, t: TypeId) -> bool {
    let types = &compiler.ty.types;
    let mut t = types.repr_struct(t);

    while let TypeKind::Pointer(inner) = *types.kind(t) {
        t = types.repr_struct(inner);
    }

    matches!(types.kind(t), TypeKind::Struct(_) | TypeKind::Enum(_))
}

/// Compiles `path` as a program with `options`. `Err` is the compiler's first error.
pub(crate) fn compile(options: Options, path: &Path, text: &str) -> Result<Oracle, String> {
    let fs: Rc<dyn jaic::sema::FileSystem> = Rc::new(jaic::sema::NativeFs);
    let mut compiler = Compiler::new(options.clone(), fs.clone());
    // Compile-time code runs as the server runs it: output kept in memory, not on stdout.
    let dir = path
        .parent()
        .unwrap_or(Path::new("."))
        .to_string_lossy()
        .into_owned();
    let host = Rc::new(std::cell::RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &dir,
    )));

    compiler
        .ct
        .interp
        .set_host(Box::new(SharedHost(host.clone())));
    compiler.attach_workspaces(Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs,
        options,
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move |_, _| Box::new(SharedHost(host.clone()))),
        report: Box::new(|_, _| {}),
        observer: None,
    }));
    let mut facts = IdeFacts::new(vec![IdeRoot::under(path)]);

    // Lint mode also records each expression's type, which says what a literal builds.
    facts.lint = true;
    compiler.ed.record(facts);
    compiler
        .compile_program(path)
        .map_err(|e| e.message.clone())?;
    let file = file_of(&compiler, path).ok_or("the program does not contain its own file")?;
    let ide = compiler.ed.facts().ok_or("no editor facts")?;
    let word = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80)
            && !s.as_bytes()[0].is_ascii_digit()
    };
    let limit = ide.sketch_from.as_ref().map_or(ide.refs.len(), |m| m.refs);
    let mut refs: Vec<Ref> = Vec::new();

    for r in &ide.refs[..limit] {
        let (start, end) = (r.span.start as usize, r.span.end as usize);

        if r.span.file != file || r.decl || end > text.len() || !word(&text[start..end]) {
            continue;
        }

        let resolved = match &r.what {
            IdeWhat::Entity(e) if compiler.entity(*e).baked_argument => Resolved::Baked,
            IdeWhat::Entity(e) => {
                let span = compiler.entity(*e).span;
                let place = (span.end > span.start)
                    .then(|| {
                        let source = compiler.src.sources.get(span.file);
                        let path = source.path.as_file()?.to_path_buf();
                        let text = &source.text;

                        Some((
                            path,
                            line_of(text, span.start as usize),
                            line_of(text, span.end as usize),
                        ))
                    })
                    .flatten();

                Resolved::Entity(place.into_iter().collect())
            }
            IdeWhat::Member(_) => Resolved::Member(
                ide.receivers
                    .get(&r.span)
                    .copied()
                    .flatten()
                    .is_some_and(|t| declared(&compiler, t)),
            ),
            IdeWhat::Type(t) => Resolved::Type(declared(&compiler, *t)),
            IdeWhat::Procs(_) => Resolved::Procs,
            IdeWhat::Module(_) => Resolved::Module,
        };

        refs.push(Ref {
            start,
            end,
            resolved,
        });
    }

    refs.sort_by_key(|r| (r.start, r.end));
    refs.dedup_by(|later, kept| {
        if (later.start, later.end) != (kept.start, kept.end) {
            return false;
        }

        match (&mut kept.resolved, &mut later.resolved) {
            (Resolved::Entity(into), Resolved::Entity(from)) => {
                for place in from.drain(..) {
                    if !into.contains(&place) {
                        into.push(place);
                    }
                }
            }
            // The original's own record says what the source means.
            (kept @ Resolved::Baked, other @ Resolved::Entity(_)) => *kept = other.clone(),
            _ => {}
        }

        true
    });
    let literals = literals(&compiler, file, text);
    let notes = read_notes(&compiler, path, text);

    Ok(Oracle {
        refs,
        literals,
        notes,
    })
}

/// The notes of `path` that this compile's compile-time code read. A metaprogram runs in this
/// compiler, so its reads of the notes of a workspace it builds are recorded here too.
fn read_notes(compiler: &Compiler, path: &Path, text: &str) -> Vec<ReadNote> {
    let Some(ide) = compiler.ed.facts() else {
        return Vec::new();
    };
    let same = |a: &Path, b: &Path| {
        a.canonicalize()
            .ok()
            .zip(b.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
    };
    let mut notes: Vec<ReadNote> = Vec::new();

    for r in &ide.note_readers {
        if !same(Path::new(&*r.note.path), path) {
            continue;
        }

        let (line, column) = r.note.start;
        let Some(line_start) = text
            .split_inclusive('\n')
            .take(line.saturating_sub(1) as usize)
            .map(str::len)
            .sum::<usize>()
            .checked_add(column.saturating_sub(1) as usize)
            .filter(|&at| at < text.len())
        else {
            continue;
        };
        let reader = (PathBuf::from(&r.path), line_of(&r.text, r.offset as usize));

        match notes.iter_mut().find(|n| n.at == line_start) {
            Some(n) => n.readers.push(reader),
            None => notes.push(ReadNote {
                at: line_start,
                readers: vec![reader],
            }),
        }
    }

    notes.sort_by_key(|n| n.at);
    notes
}

/// Where the literal opened by the `.{` at `open` ends: the end of its closing brace.
fn closing_brace(tokens: &[jaic::lexer::Token], open: usize) -> Option<u32> {
    let mut depth = 0usize;

    for t in &tokens[open + 1..] {
        match t.tok {
            Tok::Punct(P::LBrace | P::DotBrace | P::LParen | P::LBracket | P::DotBracket) => {
                depth += 1;
            }
            Tok::Punct(P::RBrace | P::RParen | P::RBracket) => {
                if depth == 0 {
                    return matches!(t.tok, Tok::Punct(P::RBrace)).then_some(t.span.end);
                }

                depth -= 1;
            }
            _ => {}
        }
    }

    None
}

/// Struct literals of the file whose type the compile recorded.
fn literals(compiler: &Compiler, file: FileId, text: &str) -> Vec<Literal> {
    let Some(ide) = compiler.ed.facts() else {
        return Vec::new();
    };
    let Ok(tokens) = jaic::lexer::lex(file, text) else {
        return Vec::new();
    };
    let mut out = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if !matches!(token.tok, Tok::Punct(P::DotBrace)) {
            continue;
        }

        // `Name.{` starts at `Name`; `.{` alone at itself.
        let start = match i.checked_sub(1).map(|j| &tokens[j]) {
            Some(prev)
                if matches!(prev.tok, Tok::Ident(_)) && prev.span.end == token.span.start =>
            {
                prev.span.start
            }
            _ => token.span.start,
        };
        // The fact for the literal itself: what ends at its closing brace. A longer expression
        // can start at the same place (`(Name.{}).field` starts at `Name`) and has another type.
        let Some(end) = closing_brace(&tokens, i) else {
            continue;
        };
        let built = ide
            .exprs
            .iter()
            .filter(|(span, fact)| {
                span.file == file && span.start == start && span.end == end && !fact.conflicting
            })
            .map(|(_, fact)| fact.ty)
            .next();
        let Some(ty) = built else {
            continue;
        };
        let TypeKind::Struct(s) = *compiler.ty.types.kind(ty) else {
            continue;
        };
        let info = compiler.ty.types.struct_info(s);
        let fields: Vec<String> = info
            .fields
            .iter()
            .filter_map(|f| f.name.map(|n| n.as_str().to_owned()))
            .collect();
        let type_name = info.name.as_str().to_owned();

        // An empty literal can name any field; a named one names its own.
        if matches!(
            tokens.get(i + 1).map(|t| &t.tok),
            Some(Tok::Punct(P::RBrace))
        ) {
            out.push(Literal {
                at: token.span.end as usize,
                type_name,
                fields,
            });
            continue;
        }

        let mut depth = 0usize;

        for (j, t) in tokens.iter().enumerate().skip(i + 1) {
            match t.tok {
                Tok::Punct(P::LBrace | P::DotBrace | P::LParen | P::LBracket | P::DotBracket) => {
                    depth += 1;
                }
                Tok::Punct(P::RBrace | P::RParen | P::RBracket) => {
                    if depth == 0 {
                        break;
                    }

                    depth -= 1;
                }
                Tok::Ident(ref name)
                    if depth == 0
                        && matches!(tokens.get(j + 1).map(|t| &t.tok), Some(Tok::Punct(P::Eq)))
                        && matches!(tokens[j - 1].tok, Tok::Punct(P::Comma | P::DotBrace))
                        && fields.iter().any(|f| f == name.as_str()) =>
                {
                    out.push(Literal {
                        at: t.span.start as usize,
                        type_name: type_name.clone(),
                        fields: vec![name.as_str().to_owned()],
                    });
                }
                _ => {}
            }
        }
    }

    out.sort_by_key(|l| l.at);
    out
}
