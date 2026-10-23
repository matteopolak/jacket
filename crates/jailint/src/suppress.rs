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

pub(crate) struct Suppressions {
    file: Vec<String>,
    /// (line start, line end, rules): a covered line.
    lines: Vec<(usize, usize, Vec<String>)>,
    /// (start, end, rules): a covered declaration.
    ranges: Vec<(usize, usize, Vec<String>)>,
}

const MARKER: &str = "jailint:";
const NOTE: &str = "jailint_allow";

impl Suppressions {
    pub fn new(text: &str, tokens: &[Token], comments: &[Comment], ast: &File) -> Self {
        let mut s = Suppressions {
            file: Vec::new(),
            lines: Vec::new(),
            ranges: Vec::new(),
        };
        s.comments(text, tokens, comments);
        s.notes(&ast.stmts);
        s
    }

    /// A comment or note silences `rule` (by its name, its category or `all`) at `offset`.
    pub fn allows_rule(&self, rule: &crate::RuleInfo, offset: usize) -> bool {
        self.allows(rule.name, offset) || self.allows(rule.category.as_str(), offset)
    }

    /// A comment or note silences the rule or category `rule` (or `all`) at `offset`.
    pub fn allows(&self, rule: &str, offset: usize) -> bool {
        let names = |list: &[String]| list.iter().any(|r| r == rule || r == "all");
        names(&self.file)
            || self
                .lines
                .iter()
                .any(|(a, b, r)| *a <= offset && offset <= *b && names(r))
            || self
                .ranges
                .iter()
                .any(|(a, b, r)| *a <= offset && offset < *b && names(r))
    }

    fn comments(&mut self, text: &str, tokens: &[Token], comments: &[Comment]) {
        for comment in comments {
            if comment.kind != CommentKind::Line {
                continue;
            }
            let (start, line_end) = (comment.span.start as usize, comment.span.end as usize);
            let Some(rest) = comment.body(text).trim().strip_prefix(MARKER) else {
                continue;
            };
            let rest = rest.trim();
            let (file_wide, list) = if let Some(l) = rest.strip_prefix("allow-file") {
                (true, l)
            } else if let Some(l) = rest.strip_prefix("allow") {
                (false, l)
            } else {
                continue;
            };
            let Some(rules) = rule_list(list) else {
                continue;
            };
            if file_wide {
                self.file.extend(rules);
                continue;
            }
            let line_start = jaic::source::line_start(text, start);
            if !text[line_start..start].trim().is_empty() {
                self.lines.push((line_start, line_end, rules));
                continue;
            }
            // The next line holding code: the comment lines in between are covered too (a
            // marker inside a doc comment covers the links below it).
            let next = tokens.partition_point(|t| (t.span.start as usize) < line_end);
            if let Some(t) = tokens.get(next).filter(|t| t.tok != Tok::Eof) {
                let end = jaic::source::line_end(text, t.span.start as usize);
                self.lines.push((line_end + 1, end, rules));
            }
        }
    }

    fn notes(&mut self, stmts: &[Stmt]) {
        for s in stmts {
            let mut stack = Vec::new();
            crate::syntax::walk_node(crate::syntax::Node::Stmt(s), &mut stack, &mut |n, _| {
                match n {
                    crate::syntax::Node::Stmt(s) => {
                        self.note_list(&s.notes, s.span);
                        if let StmtKind::Decl(d) = &s.kind {
                            self.note_list(&d.notes, s.span.to(d.span));
                        }
                    }
                    crate::syntax::Node::Expr(e) => {
                        if let jaic::ast::ExprKind::Proc(lit) = &e.kind {
                            self.note_list(&lit.header.notes, e.span);
                            if let Some(body) = &lit.body {
                                self.notes(&body.stmts);
                            }
                        }
                        if let jaic::ast::ExprKind::Struct(st) = &e.kind {
                            self.notes(&st.body);
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
                self.ranges
                    .push((span.start as usize, span.end as usize, rules));
            }
        }
    }
}

/// `(a, b)` as names.
fn rule_list(text: &str) -> Option<Vec<String>> {
    let inner = text.trim().strip_prefix('(')?;
    let inner = &inner[..inner.find(')')?];
    let rules: Vec<String> = inner
        .split(',')
        .map(|r| r.trim().to_string())
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
}
