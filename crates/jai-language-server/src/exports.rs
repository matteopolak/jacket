//! What a Jai file declares at its top level and what it brings in, from the compiler's
//! editor parse (`jaic::parser::ide_facts`), so a module or project file that does not parse
//! yet still has the names its intact statements declare. This is the raw material of
//! auto-import completion (`auto_import.rs`).
use crate::model::CompletionKind;
use jaic::ast::ImportSource;
use jaic::lexer::{P, Tok, Token};
use jaic::parser::ide_facts::{self, DeclClass, TopLevel, TopLevelKind};
use jaic::source::FileId;

pub use jaic::parser::ide_facts::Visibility;
use jaic::sema::ide::IdeCall;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decl {
    pub name: String,
    /// `name` in lower case, for case-insensitive prefix matching.
    pub lower: String,
    pub kind: CompletionKind,
    pub visibility: Visibility,
    /// The declaration's header on one line: `print :: (format: string, args: ..Any) -> s64`.
    pub signature: String,
    /// The `//` comment lines right above it.
    pub doc: String,
    /// A simple constant alias path, without calls or other computed expressions.
    pub alias: Option<Vec<String>>,
    /// The parameters of a procedure, for inserting a call.
    pub call: Option<IdeCall>,
}

/// A top-level `#import "Name"` (bound to `name` for `Name :: #import`), or `#load`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directive {
    pub path: String,
    pub name: Option<String>,
    /// Byte offset just past the statement (its `;`, else its string).
    pub end: usize,
    /// Inside an `#if` block rather than directly at the top level.
    pub nested: bool,
    pub visibility: Visibility,
}

/// A top-level name and where it is declared, whatever its visibility (workspace symbols).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: CompletionKind,
    /// Byte offset of the name.
    pub offset: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileScan {
    /// Every top-level declaration, `#scope_file` ones included.
    pub symbols: Vec<Symbol>,
    /// Top-level declarations outside `#scope_file`.
    pub decls: Vec<Decl>,
    /// Declarations including file-private ones, for lexical alias resolution.
    pub local_decls: Vec<Decl>,
    pub imports: Vec<Directive>,
    pub loads: Vec<Directive>,
    /// Files a build metaprogram compiles: the string literal of each `add_build_file("x", w)`
    /// call, anywhere in the file. A static scan for a metaprogram that has not run: once it has,
    /// the files it really added (`semantic::BuildFacts`) are used instead.
    pub builds: Vec<String>,
    /// The targets a top-level `#assert OS == .A || OS == .B` allows (`["A", "B"]`).
    pub only_os: Option<Vec<String>>,
    /// The file declares `main` at its top level (it is a program's entry, not a part).
    pub declares_main: bool,
    /// The string literals a statement mentioning `import_path` writes (`modules.rs`).
    pub import_paths: Vec<String>,
    /// Byte offset of the first token (the end marker in a file of comments): where code
    /// starts below the leading comments.
    pub first_code: Option<usize>,
}

/// The longest signature kept, in bytes.
const SIGNATURE_BYTES: usize = 200;

/// The most comment lines kept as a declaration's documentation.
const DOC_LINES: usize = 40;

pub fn scan(text: &str) -> FileScan {
    let parsed = jaic::parser::parse_file_ide(FileId(0), text);
    let tokens = &parsed.tokens;
    let mut out = FileScan {
        builds: build_files(tokens),
        import_paths: import_paths(tokens),
        first_code: tokens.first().map(|t| t.span.start as usize),
        ..FileScan::default()
    };
    for item in ide_facts::top_level(&parsed.file) {
        match item.kind {
            TopLevelKind::Decl(decl) => out.declaration(text, tokens, decl, &item),
            TopLevelKind::Import(import) => {
                if let ImportSource::Module(path) = &import.source {
                    let name = import.name.map(|n| n.name.as_str().to_owned());
                    out.imports.push(directive(tokens, path, name, &item));
                }
            }
            TopLevelKind::Load(path, _) => out.loads.push(directive(tokens, path, None, &item)),
            TopLevelKind::Assert(assert) => {
                if let Some(targets) = ide_facts::asserted_targets(assert) {
                    out.only_os = Some(targets.iter().map(|t| t.as_str().to_owned()).collect());
                }
            }
        }
    }
    out
}

impl FileScan {
    fn declaration(
        &mut self,
        text: &str,
        tokens: &[Token],
        decl: &jaic::ast::Decl,
        item: &TopLevel,
    ) {
        let start = item.span.start as usize;
        let kind = match ide_facts::decl_class(decl) {
            DeclClass::Variable => CompletionKind::Variable,
            DeclClass::Procedure => CompletionKind::Function,
            DeclClass::Struct => CompletionKind::Struct,
            DeclClass::Enum => CompletionKind::Enum,
            DeclClass::TypeAlias => CompletionKind::TypeAlias,
            DeclClass::Constant => CompletionKind::Constant,
            // A library is no name to complete.
            DeclClass::Library => return,
        };
        let alias = ide_facts::alias_path(decl).map(|path| {
            path.iter()
                .map(|n| n.as_str().to_owned())
                .collect::<Vec<_>>()
        });
        let call = match decl.value.as_ref().map(|v| &v.kind) {
            Some(jaic::ast::ExprKind::Proc(lit)) => Some(IdeCall::of_headers([&*lit.header])),
            _ => None,
        };
        for name in &decl.names {
            let name_text = name.name.as_str();
            self.declares_main |= name_text == "main";
            self.symbols.push(Symbol {
                name: name_text.into(),
                kind,
                offset: name.span.start as usize,
            });
            let decl = Decl {
                name: name_text.into(),
                lower: name_text.to_lowercase(),
                kind,
                visibility: item.visibility,
                signature: signature(text, tokens, start),
                doc: doc_above(text, start),
                alias: alias.clone(),
                call: call.clone(),
            };
            if item.visibility != Visibility::File {
                self.decls.push(decl.clone());
            }
            self.local_decls.push(decl);
        }
    }
}

/// An `#import` or `#load` statement; it ends past its `;`.
fn directive(tokens: &[Token], path: &str, name: Option<String>, item: &TopLevel) -> Directive {
    let end = item.span.end;
    let next = tokens.partition_point(|t| t.span.start < end);
    let end = match tokens.get(next) {
        Some(t) if t.tok == Tok::Punct(P::Semi) => t.span.end,
        _ => end,
    };
    Directive {
        path: path.to_owned(),
        name,
        end: end as usize,
        nested: item.nested,
        visibility: item.visibility,
    }
}

/// The string literals written next to an `import_path`.
fn import_paths(tokens: &[Token]) -> Vec<String> {
    let mut found = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(&token.tok, Tok::Ident(name) if name.as_str() == "import_path") {
            continue;
        }
        // The rest of the statement: up to the `;` that is not inside brackets.
        let mut depth = 0i32;
        for next in &tokens[i + 1..] {
            match &next.tok {
                Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                    depth += 1;
                }
                Tok::Punct(P::RParen | P::RBracket | P::RBrace) => {
                    depth -= 1;
                    if depth < 0 {
                        break;
                    }
                }
                Tok::Punct(P::Semi) if depth <= 0 => break,
                Tok::Str(s) => {
                    if let Ok(s) = std::str::from_utf8(s)
                        && !s.is_empty()
                        && !s.contains(['%', '\n'])
                    {
                        found.push(s.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// The string literal of each `add_build_file("x", ...)` call.
fn build_files(tokens: &[Token]) -> Vec<String> {
    tokens
        .windows(3)
        .filter_map(|w| match (&w[0].tok, &w[1].tok, &w[2].tok) {
            (Tok::Ident(name), Tok::Punct(P::LParen), Tok::Str(bytes))
                if name.as_str() == "add_build_file" =>
            {
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
            _ => None,
        })
        .collect()
}

/// The declaration starting at byte `start` up to its body or `;`, on one line.
fn signature(text: &str, tokens: &[Token], start: usize) -> String {
    let i = tokens.partition_point(|t| (t.span.start as usize) < start);
    let Some(first) = tokens.get(i) else {
        return String::new();
    };
    let mut end = first.span.end as usize;
    let mut depth = 0usize;
    for t in tokens[i..].iter().take(400) {
        match t.tok {
            Tok::Punct(P::LParen | P::LBracket | P::DotBrace | P::DotBracket) => depth += 1,
            Tok::Punct(P::RParen | P::RBracket | P::RBrace) if depth > 0 => depth -= 1,
            Tok::Punct(P::Semi | P::LBrace) if depth == 0 => break,
            _ => {}
        }
        end = t.span.end as usize;
    }
    let mut out = String::new();
    for word in text[start..end].split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
        if out.len() > SIGNATURE_BYTES {
            let cut = out.floor_char_boundary(SIGNATURE_BYTES);
            out.truncate(cut);
            out.push_str(" ...");
            break;
        }
    }
    out
}

/// The doc comment of the declaration at `at`, as shown next to a completion item.
fn doc_above(text: &str, at: usize) -> String {
    crate::docs::above(text, at)
        .map(|doc| doc.lines().take(DOC_LINES).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(scan: &FileScan) -> Vec<(&str, CompletionKind, Visibility)> {
        scan.decls
            .iter()
            .map(|d| (d.name.as_str(), d.kind, d.visibility))
            .collect()
    }

    #[test]
    fn top_level_declarations_by_kind_and_scope() {
        let text = "\
#import \"Basic\";
M :: #import \"Math\";
// Prints things.
// Twice.
twice :: (s: string) -> int { inner :: 1; return 0; }
Point :: struct { x: int; }
Color :: enum { RED; }
Callback :: #type (x: int);
SIX :: (1 + 5);
counter: int;
total := 0;
lib :: #library \"foo\";
#if OS == .WINDOWS {
    win_only :: () {}
} else {
    elsewhere :: () {}
}
#scope_module
internal :: () {}
#scope_file
hidden :: () {}
";
        let scan = scan(text);
        use CompletionKind::*;
        use Visibility::{Export, Module as ModuleScope};
        assert_eq!(
            names(&scan),
            [
                ("twice", Function, Export),
                ("Point", Struct, Export),
                ("Color", Enum, Export),
                ("Callback", TypeAlias, Export),
                ("SIX", Constant, Export),
                ("counter", Variable, Export),
                ("total", Variable, Export),
                ("win_only", Function, Export),
                ("elsewhere", Function, Export),
                ("internal", Function, ModuleScope),
            ]
        );
        assert_eq!(scan.decls[0].signature, "twice :: (s: string) -> int");
        assert_eq!(scan.decls[0].doc, "Prints things.\nTwice.");
        let imports: Vec<(&str, Option<&str>)> = scan
            .imports
            .iter()
            .map(|d| (d.path.as_str(), d.name.as_deref()))
            .collect();
        assert_eq!(imports, [("Basic", None), ("Math", Some("M"))]);
        assert_eq!(
            &text[..scan.imports[1].end],
            "#import \"Basic\";\nM :: #import \"Math\";"
        );
        assert!(!scan.declares_main);
    }

    #[test]
    fn build_files_a_metaprogram_adds() {
        let scan = scan(
            "build :: () {\n    w := compiler_create_workspace();\n    add_build_file(\"src/main.jai\", w);\n}\n",
        );
        assert_eq!(scan.builds, ["src/main.jai"]);
    }

    #[test]
    fn loads_main_and_target_asserts() {
        let scan = scan(
            "#assert OS == .MACOS || OS == .IOS \"Apple only\";\n#load \"a/b.jai\";\nmain :: () {}\n",
        );
        assert_eq!(scan.only_os, Some(vec!["MACOS".into(), "IOS".into()]));
        assert_eq!(scan.loads[0].path, "a/b.jai");
        assert!(scan.declares_main);
        assert_eq!(super::scan("#assert(OS != .WINDOWS);").only_os, None);
    }
}
