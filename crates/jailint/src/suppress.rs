//! In-source suppression.
//!
//! - `// jailint: allow(rule, ...)` on a line of its own covers the next line with code; after
//!   code it covers its own line.
//! - `// jailint: allow-file(rule, ...)` anywhere covers the whole file.
//! - `@jailint_allow(rule, ...)` noted after a declaration (for a procedure, after its body)
//!   covers the declaration, all of the procedure when it is one.
//!
//! `all` stands for every rule.
use jaic::ast::{File, Note, Stmt, StmtKind};
use jaic::lexer::{Comment, CommentKind, Tok, Token};
use std::cell::Cell;

pub(crate) struct Suppressions {
    entries: Vec<Entry>,
}

/// One comment or note: where it applies, and the names it lists.
struct Entry {
    scope: Scope,
    names: Vec<Name>,
    /// Inside an `#if`, or in a file with one: another branch may be what needs it, so it is
    /// never reported as unused.
    conditional: bool,
}

enum Scope {
    File,
    /// (line start, line end): a covered line.
    Lines(usize, usize),
    /// (start, end): a covered declaration.
    Range(usize, usize),
}

struct Name {
    text: String,
    /// Where to point when it is unused.
    at: (usize, usize),
    /// Some finding was silenced by it.
    used: Cell<bool>,
}

/// A name in a suppression that silenced nothing.
pub(crate) struct Unused {
    pub start: usize,
    pub end: usize,
    pub name: String,
}

const MARKER: &str = "jailint:";
const NOTE: &str = "jailint_allow";

impl Suppressions {
    pub fn new(text: &str, tokens: &[Token], comments: &[Comment], ast: &File) -> Self {
        let mut s = Suppressions {
            entries: Vec::new(),
        };
        let mut conditional = Vec::new();
        s.notes(&ast.stmts, &mut conditional);
        s.comments(text, tokens, comments);
        // Code in one branch of an `#if` can be what a suppression outside it is for (an
        // import only the Windows branch uses), so a file with any `#if` is not judged.
        let file_has_if = !conditional.is_empty();
        for entry in &mut s.entries {
            let at = entry.names.first().map_or(0, |n| n.at.0);
            entry.conditional |= file_has_if || conditional.iter().any(|&(a, b)| a <= at && at < b);
        }
        s
    }

    /// A comment or note silences `rule` (by its name, its category or `all`) at `offset`.
    pub fn allows_rule(&self, rule: &crate::RuleInfo, offset: usize) -> bool {
        let by_name = self.allows(rule.name, offset);
        let by_category = self.allows(rule.category.as_str(), offset);
        by_name || by_category
    }

    /// A comment or note silences the rule or category `rule` (or `all`) at `offset`. Every
    /// name that does is marked used.
    pub fn allows(&self, rule: &str, offset: usize) -> bool {
        let mut found = false;
        for entry in &self.entries {
            let covers = match entry.scope {
                Scope::File => true,
                Scope::Lines(a, b) => a <= offset && offset <= b,
                Scope::Range(a, b) => a <= offset && offset < b,
            };
            if !covers {
                continue;
            }
            for name in &entry.names {
                if name.text == rule || name.text == "all" {
                    name.used.set(true);
                    found = true;
                }
            }
        }
        found
    }

    /// The names that silenced nothing. `judge` says whether a name could have: it names rules
    /// that all ran (a rule that was off, or that stays quiet on an incomplete compile, proves
    /// nothing), and it is not misspelt (`unknown_allow` reports that).
    pub fn unused(&self, judge: &dyn Fn(&str) -> bool) -> Vec<Unused> {
        let mut out = Vec::new();
        for entry in self.entries.iter().filter(|e| !e.conditional) {
            for name in &entry.names {
                if !name.used.get() && judge(&name.text) {
                    out.push(Unused {
                        start: name.at.0,
                        end: name.at.1,
                        name: name.text.clone(),
                    });
                }
            }
        }
        out
    }

    fn comments(&mut self, text: &str, tokens: &[Token], comments: &[Comment]) {
        for comment in comments {
            if comment.kind != CommentKind::Line {
                continue;
            }
            let (start, line_end) = (comment.span.start as usize, comment.span.end as usize);
            let Some((file_wide, rules)) = directive(comment.body(text)) else {
                continue;
            };
            let names = rules
                .into_iter()
                .map(|r| {
                    let at = offset_in(text, r).unwrap_or(start);
                    Name {
                        text: r.to_string(),
                        at: (at, at + r.len()),
                        used: Cell::new(false),
                    }
                })
                .collect();
            let scope = if file_wide {
                Scope::File
            } else {
                let line_start = jaic::source::line_start(text, start);
                if text[line_start..start].trim().is_empty() {
                    // The next line holding code: the comment lines in between are covered
                    // too (a marker inside a doc comment covers the links below it).
                    let next = tokens.partition_point(|t| (t.span.start as usize) < line_end);
                    let Some(t) = tokens.get(next).filter(|t| t.tok != Tok::Eof) else {
                        continue;
                    };
                    let end = jaic::source::line_end(text, t.span.start as usize);
                    Scope::Lines(line_end + 1, end)
                } else {
                    Scope::Lines(line_start, line_end)
                }
            };
            self.entries.push(Entry {
                scope,
                names,
                conditional: false,
            });
        }
    }

    fn notes(&mut self, stmts: &[Stmt], conditional: &mut Vec<(usize, usize)>) {
        for s in stmts {
            let mut stack = Vec::new();
            crate::syntax::walk_node(crate::syntax::Node::Stmt(s), &mut stack, &mut |n, _| {
                match n {
                    crate::syntax::Node::Stmt(s) => {
                        if matches!(s.kind, StmtKind::StaticIf(_) | StmtKind::StaticSwitch(_)) {
                            conditional.push((s.span.start as usize, s.span.end as usize));
                        }
                        self.note_list(&s.notes, s.span);
                        if let StmtKind::Decl(d) = &s.kind {
                            self.note_list(&d.notes, s.span.to(d.span));
                        }
                    }
                    crate::syntax::Node::Expr(e) => {
                        if let jaic::ast::ExprKind::Proc(lit) = &e.kind {
                            self.note_list(&lit.header.notes, e.span);
                            if let Some(body) = &lit.body {
                                self.notes(&body.stmts, conditional);
                            }
                        }
                        if let jaic::ast::ExprKind::Struct(st) = &e.kind {
                            self.notes(&st.body, conditional);
                        }
                    }
                }
                true
            });
        }
    }

    fn note_list(&mut self, notes: &[Note], span: jaic::source::Span) {
        for note in notes {
            let Some(list) = note.text.strip_prefix(NOTE) else {
                continue;
            };
            if let Some(rules) = rule_list(list) {
                let at = (note.span.start as usize, note.span.end as usize);
                self.entries.push(Entry {
                    scope: Scope::Range(span.start as usize, span.end as usize),
                    names: rules
                        .into_iter()
                        .map(|r| Name {
                            text: r.to_string(),
                            at,
                            used: Cell::new(false),
                        })
                        .collect(),
                    conditional: false,
                });
            }
        }
    }
}

/// Whether an unused `name` is evidence of anything: it is a rule, a category or `all`, and
/// every rule it covers ran (`unused_suppression` itself apart).
pub(crate) fn could_judge(name: &str, ran: &[&str]) -> bool {
    let covered: Vec<&str> = if name == "all" {
        crate::RULES.iter().map(|r| r.name).collect()
    } else if let Some(category) = crate::config::Category::parse(name) {
        crate::RULES
            .iter()
            .filter(|r| r.category == category)
            .map(|r| r.name)
            .collect()
    } else if crate::RULES.iter().any(|r| r.name == name) {
        vec![name]
    } else {
        return false;
    };
    name != "unused_suppression"
        && covered
            .iter()
            .all(|r| *r == "unused_suppression" || ran.contains(r))
}

/// Where `part`, a slice of `text`, starts.
pub(crate) fn offset_in(text: &str, part: &str) -> Option<usize> {
    let start = (part.as_ptr() as usize).checked_sub(text.as_ptr() as usize)?;
    (start + part.len() <= text.len()).then_some(start)
}

/// The body of a `// jailint: allow(...)` or `allow-file(...)` comment: whether it covers the
/// whole file, and the names it lists.
pub(crate) fn directive(body: &str) -> Option<(bool, Vec<&str>)> {
    let rest = body.trim().strip_prefix(MARKER)?.trim();
    let (file_wide, list) = if let Some(l) = rest.strip_prefix("allow-file") {
        (true, l)
    } else {
        (false, rest.strip_prefix("allow")?)
    };
    Some((file_wide, rule_list(list)?))
}

/// `(a, b)` as names.
fn rule_list(text: &str) -> Option<Vec<&str>> {
    let inner = text.trim().strip_prefix('(')?;
    let inner = &inner[..inner.find(')')?];
    let rules: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .collect();
    (!rules.is_empty()).then_some(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaic::source::FileId;

    fn suppressions(text: &str) -> Suppressions {
        let (tokens, comments, _) = jaic::lexer::lex_with_trivia(FileId(0), text);
        let ast = jaic::parser::parse_file(FileId(0), text).unwrap_or(File {
            file: FileId(0),
            stmts: Vec::new(),
        });
        Suppressions::new(text, &tokens, &comments, &ast)
    }

    #[test]
    fn comments_cover_the_next_code_line_or_their_own() {
        let text = "f :: () {\n    // jailint: allow(a)\n    // more words\n    x := 1;\n    y := 2; // jailint: allow(b, c)\n    z := \"// jailint: allow(d)\";\n}\n";
        let s = suppressions(text);
        let at = |needle: &str| text.find(needle).unwrap();
        assert!(s.allows("a", at("x :=")));
        assert!(!s.allows("a", at("y :=")));
        assert!(s.allows("c", at("y :=")));
        assert!(!s.allows("d", at("z :=")));
    }

    #[test]
    fn notes_cover_the_procedure() {
        let text =
            "f :: (a: int) {\n    x := 1;\n} @jailint_allow(unused_parameter)\ng :: () { }\n";
        let s = suppressions(text);
        assert!(s.allows("unused_parameter", text.find("a:").unwrap()));
        assert!(s.allows("unused_parameter", text.find("x :=").unwrap()));
        assert!(!s.allows("unused_parameter", text.find("g ::").unwrap()));
    }

    #[test]
    fn a_category_covers_its_rules() {
        let text = "// jailint: allow-file(pedantic)\nx := 1;\n";
        let s = suppressions(text);
        let rule = |name: &str| crate::RULES.iter().find(|r| r.name == name).unwrap();
        assert!(s.allows_rule(rule("magic_number"), text.find("x :=").unwrap()));
        assert!(!s.allows_rule(rule("double_free"), text.find("x :=").unwrap()));
    }

    #[test]
    fn file_wide() {
        let s = suppressions("// jailint: allow-file(all)\nx := 1;\n");
        assert!(s.allows("anything", 30));
    }

    #[test]
    fn a_slash_pair_inside_a_string_is_not_a_comment() {
        let text = "x := \"http://a\"; // jailint: allow(a)\ny := \"http:// jailint: allow(b)\";\n";
        let s = suppressions(text);
        assert!(s.allows("a", text.find("x :=").unwrap()));
        assert!(!s.allows("b", text.find("y :=").unwrap()));
    }

    #[test]
    fn a_marker_inside_a_block_comment_does_nothing() {
        let text =
            "/* // jailint: allow(a) */\nx := 1;\n/* /* // jailint: allow(b) */ */\ny := 2;\n";
        let s = suppressions(text);
        assert!(!s.allows("a", text.find("x :=").unwrap()));
        assert!(!s.allows("b", text.find("y :=").unwrap()));
    }

    #[test]
    fn a_marker_after_a_nested_block_comment_still_counts() {
        let text = "/* a /* b */ c */\n// jailint: allow(a)\nx := 1;\n";
        let s = suppressions(text);
        assert!(s.allows("a", text.find("x :=").unwrap()));
    }

    #[test]
    fn an_unterminated_block_comment_swallows_the_rest() {
        let text = "// jailint: allow(a)\nx := 1;\n/* open\n// jailint: allow(b)\ny := 2;\n";
        let s = suppressions(text);
        assert!(s.allows("a", text.find("x :=").unwrap()));
        assert!(!s.allows("b", text.find("y :=").unwrap()));
    }

    #[test]
    fn names_that_silenced_nothing_are_unused() {
        let text = "// jailint: allow(a, b)\nx := 1;\ny := 2; // jailint: allow(c)\n";
        let s = suppressions(text);
        assert!(s.allows("a", text.find("x :=").unwrap()));
        let unused: Vec<String> = s.unused(&|_| true).into_iter().map(|u| u.name).collect();
        assert_eq!(unused, ["b", "c"]);
        let none = s.unused(&|name| name == "c");
        assert_eq!(none.len(), 1);
        assert_eq!(&text[none[0].start..none[0].end], "c");
    }

    #[test]
    fn a_file_with_an_if_is_not_judged() {
        // The import is used only in the Windows branch: on other targets the allow is needed.
        let text = "#import \"Basic\"; // jailint: allow(a)\n#if OS == .WINDOWS {\n// jailint: allow(b)\nz := 3;\n}\n";
        let s = suppressions(text);
        assert!(s.unused(&|_| true).is_empty());
    }

    #[test]
    fn a_name_is_judged_only_when_its_rules_ran() {
        let ran = ["unused_variable", "float_equality"];
        assert!(could_judge("unused_variable", &ran));
        assert!(!could_judge("magic_number", &ran));
        assert!(!could_judge("no_such_rule", &ran));
        assert!(!could_judge("all", &ran));
        assert!(!could_judge("pedantic", &ran));
        assert!(!could_judge("unused_suppression", &ran));
    }
}
