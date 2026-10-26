//! The filter language of `jacket test -E/--filter-expr`: predicates combined with `and`, `or`
//! and `not`, in the style of nextest's filtersets. See docs/tools/jacket-test.md.
//!
//! ```text
//! expr  := or
//! or    := and (("or" | "|") and)*
//! and   := unary (("and" | "&") unary)*
//! unary := ("not" | "!") unary | "(" expr ")" | predicate
//! ```
//!
//! `not` binds tightest, then `and`, then `or`. A predicate is a name and its argument in
//! parentheses; the argument runs to the closing `)`, except `/regex/`, which may hold any.
use jacket_config::PathPattern;
use regex_lite::Regex;
use std::fmt;

/// The kinds `kind(...)` knows.
pub const KINDS: [&str; 8] = [
    "unit",
    "property",
    "snapshot",
    "compile_time",
    "golden",
    "compile_fail",
    "doc",
    "bench",
];

/// What a filter looks at in a test.
pub struct Subject<'a> {
    /// The name in output (`Root::name` when several roots run) and the name in its program.
    pub names: [&'a str; 2],
    /// The test's root: the module it tests as a program imports it, and its folder's name.
    pub modules: [&'a str; 2],
    /// The test file, written relative to the root, the project and as it is on disk.
    pub files: &'a [String],
    /// Its entries of [`KINDS`].
    pub kinds: &'a [&'static str],
}

/// A parsed filter expression.
#[derive(Debug)]
pub struct FilterExpr {
    node: Node,
}

#[derive(Debug)]
enum Node {
    All,
    None,
    Test(Text),
    Module(Text),
    File(PathPattern),
    Kind(&'static str),
    Not(Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
}

/// The argument of `test` and `module`.
#[derive(Debug)]
enum Text {
    /// Plain text: part of the name.
    Contains(String),
    /// `=text`: the whole name.
    Exact(String),
    /// A glob (`*`, `?`, `[..]`, `{a,b}`) for the whole name.
    Glob(PathPattern),
    /// `/regex/`: found anywhere in the name.
    Regex(Regex),
}

impl Text {
    fn is_match(&self, name: &str) -> bool {
        match self {
            Text::Contains(part) => name.contains(part.as_str()),
            Text::Exact(whole) => name == whole,
            Text::Glob(pattern) => pattern.is_match(name),
            Text::Regex(regex) => regex.is_match(name),
        }
    }
}

/// A filter expression that does not parse: what is wrong and where.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    /// The offset in characters of the source.
    pub at: usize,
    source: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bad filter expression: {}\n  {}\n  {}^",
            self.message,
            self.source,
            " ".repeat(self.at)
        )
    }
}

impl std::error::Error for ParseError {
}

impl FilterExpr {
    pub fn parse(source: &str) -> Result<FilterExpr, ParseError> {
        let mut parser = Parser {
            chars: source.chars().collect(),
            at: 0,
            source,
        };
        let node = parser.or()?;
        parser.skip_space();
        if parser.at < parser.chars.len() {
            return Err(parser.error("expected `and`, `or` or the end of the expression"));
        }
        Ok(FilterExpr {
            node,
        })
    }

    /// Whether the expression looks at `kind(snapshot)`, which costs reading the test's source.
    pub fn uses_snapshot_kind(&self) -> bool {
        fn walk(node: &Node) -> bool {
            match node {
                Node::Kind(kind) => *kind == "snapshot",
                Node::Not(inner) => walk(inner),
                Node::And(a, b) | Node::Or(a, b) => walk(a) || walk(b),
                _ => false,
            }
        }
        walk(&self.node)
    }

    pub fn matches(&self, subject: &Subject) -> bool {
        eval(&self.node, subject)
    }
}

fn eval(node: &Node, subject: &Subject) -> bool {
    match node {
        Node::All => true,
        Node::None => false,
        Node::Test(text) => subject.names.iter().any(|name| text.is_match(name)),
        Node::Module(text) => subject.modules.iter().any(|module| text.is_match(module)),
        Node::File(pattern) => subject.files.iter().any(|file| pattern.is_match(file)),
        Node::Kind(kind) => subject.kinds.contains(kind),
        Node::Not(inner) => !eval(inner, subject),
        Node::And(a, b) => eval(a, subject) && eval(b, subject),
        Node::Or(a, b) => eval(a, subject) || eval(b, subject),
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    at: usize,
    source: &'a str,
}

impl Parser<'_> {
    fn error(&self, message: impl Into<String>) -> ParseError {
        self.error_at(self.at, message)
    }

    fn error_at(&self, at: usize, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            at,
            source: self.source.to_string(),
        }
    }

    fn skip_space(&mut self) {
        while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
            self.at += 1;
        }
    }

    /// Consumes `symbol` (`|`, `&`, `!`) when it is next.
    fn symbol(&mut self, symbol: char) -> bool {
        self.skip_space();
        let found = self.chars.get(self.at) == Some(&symbol);
        self.at += usize::from(found);
        found
    }

    /// Consumes the word `word` (`and`, `or`, `not`) when it is next, whole.
    fn word(&mut self, word: &str) -> bool {
        self.skip_space();
        let end = self.at + word.len();
        let matches = self
            .chars
            .get(self.at..end)
            .is_some_and(|found| found.iter().copied().eq(word.chars()))
            && !self.chars.get(end).is_some_and(|c| is_word(*c));
        if matches {
            self.at = end;
        }
        matches
    }

    fn or(&mut self) -> Result<Node, ParseError> {
        let mut node = self.and()?;
        while self.word("or") || self.symbol('|') {
            node = Node::Or(Box::new(node), Box::new(self.and()?));
        }
        Ok(node)
    }

    fn and(&mut self) -> Result<Node, ParseError> {
        let mut node = self.unary()?;
        while self.word("and") || self.symbol('&') {
            node = Node::And(Box::new(node), Box::new(self.unary()?));
        }
        Ok(node)
    }

    fn unary(&mut self) -> Result<Node, ParseError> {
        if self.word("not") || self.symbol('!') {
            return Ok(Node::Not(Box::new(self.unary()?)));
        }
        if self.symbol('(') {
            let node = self.or()?;
            if !self.symbol(')') {
                return Err(self.error("expected `)`"));
            }
            return Ok(node);
        }
        self.predicate()
    }

    fn predicate(&mut self) -> Result<Node, ParseError> {
        self.skip_space();
        let start = self.at;
        while self.chars.get(self.at).is_some_and(|c| is_word(*c)) {
            self.at += 1;
        }
        if self.at == start {
            return Err(self.error(match self.chars.get(self.at) {
                Some(c) => format!("expected a predicate or `(`, found `{c}`"),
                None => "expected a predicate, such as `test(name)`".to_string(),
            }));
        }
        let name: String = self.chars[start..self.at].iter().collect();
        if !self.symbol('(') {
            return Err(self.error(format!("expected `(` after `{name}`")));
        }
        let (argument, argument_at) = self.argument()?;
        let bad = |message: String| self.error_at(argument_at, message);
        let node = match name.as_str() {
            "all" | "none" => {
                if !argument.is_empty() {
                    return Err(bad(format!("`{name}()` takes no argument")));
                }
                if name == "all" {
                    Node::All
                } else {
                    Node::None
                }
            }
            "test" => Node::Test(text(&argument).map_err(bad)?),
            "module" => Node::Module(text(&argument).map_err(bad)?),
            "file" => Node::File(PathPattern::new(&argument).map_err(|e| bad(e.message))?),
            "kind" => match KINDS.iter().find(|k| **k == argument) {
                Some(kind) => Node::Kind(kind),
                None => {
                    return Err(bad(format!(
                        "unknown kind `{argument}`; use {}",
                        KINDS.join(", ")
                    )));
                }
            },
            _ => {
                return Err(self.error_at(
                    start,
                    format!(
                        "unknown predicate `{name}`; use test(..), module(..), file(..), kind(..), all() or none()"
                    ),
                ));
            }
        };
        Ok(node)
    }

    /// The text up to the `)` that closes a predicate, trimmed, and where it starts. A
    /// `/regex/` may hold `)`.
    fn argument(&mut self) -> Result<(String, usize), ParseError> {
        self.skip_space();
        let start = self.at;
        let mut regex = self.chars.get(self.at) == Some(&'/');
        let mut escaped = false;
        let mut end = None;
        for (offset, &c) in self.chars[start..].iter().enumerate() {
            let index = start + offset;
            if regex && index > start {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '/' {
                    regex = false;
                }
                continue;
            }
            if c == ')' {
                end = Some(index);
                break;
            }
        }
        let Some(end) = end else {
            return Err(self.error_at(
                start,
                if regex {
                    "the regular expression has no closing `/`"
                } else {
                    "expected `)` to end the predicate"
                },
            ));
        };
        let argument: String = self.chars[start..end].iter().collect();
        self.at = end + 1;
        Ok((argument.trim_end().to_string(), start))
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The argument of `test` or `module`: `/regex/`, `=exact`, a glob when it holds a wildcard,
/// else a part of the name.
fn text(argument: &str) -> Result<Text, String> {
    if argument.is_empty() {
        return Err("expected a name, glob or /regex/".into());
    }
    if let Some(body) = argument.strip_prefix('/') {
        let Some(pattern) = body.strip_suffix('/') else {
            return Err("a regular expression ends with `/`".into());
        };
        return Regex::new(pattern)
            .map(Text::Regex)
            .map_err(|e| format!("invalid regular expression: {e}"));
    }
    if let Some(whole) = argument.strip_prefix('=') {
        return Ok(Text::Exact(whole.trim_start().to_string()));
    }
    if argument.contains(['*', '?', '[', '{']) {
        return PathPattern::new(argument)
            .map(Text::Glob)
            .map_err(|e| e.message);
    }
    Ok(Text::Contains(argument.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(expr: &str, name: &str, module: &str, file: &str, kinds: &[&'static str]) -> bool {
        let files = [file.to_string()];
        FilterExpr::parse(expr).unwrap().matches(&Subject {
            names: [name, name.rsplit("::").next().unwrap_or(name)],
            modules: [module, module],
            files: &files,
            kinds,
        })
    }

    fn table(expr: &str) -> bool {
        run(
            expr,
            "Sort::sorts_table",
            "Sort",
            "tests/sort.jai",
            &["unit"],
        )
    }

    fn error(expr: &str) -> ParseError {
        FilterExpr::parse(expr).unwrap_err()
    }

    #[test]
    fn test_predicates_match_parts_exact_globs_and_regexes() {
        assert!(table("test(sorts)"));
        assert!(!table("test(unsorted)"));
        assert!(table("test(=sorts_table)"));
        assert!(!table("test(=sorts)"));
        assert!(table("test(Sort::*)"));
        assert!(table("test(sorts_{table,list})"));
        assert!(!table("test(sorts_?)"));
        assert!(table("test(/^sorts_t.b/)"));
        assert!(table("test(/(sorts|merges)_table$/)"));
        assert!(!table("test(/^table/)"));
    }

    #[test]
    fn module_file_and_kind_predicates() {
        assert!(table("module(Sort)"));
        assert!(table("module(S*)"));
        assert!(!table("module(Hash)"));
        assert!(table("file(tests/*.jai)"));
        assert!(table("file(**/sort.jai)"));
        assert!(!table("file(tests/hash.jai)"));
        assert!(table("kind(unit)"));
        assert!(!table("kind(property)"));
        assert!(run(
            "kind(property)",
            "p",
            "M",
            "t.jai",
            &["unit", "property"]
        ));
    }

    #[test]
    fn all_and_none() {
        assert!(table("all()"));
        assert!(!table("none()"));
        assert!(table("not none()"));
        assert!(table("all( )"));
    }

    #[test]
    fn operators_bind_not_then_and_then_or() {
        assert!(table("module(Hash) or module(Sort) and test(sorts)"));
        assert!(!table("module(Hash) and module(Sort) or test(nothing)"));
        assert!(table("module(Hash) | module(Sort) & test(sorts)"));
        assert!(!table("not module(Sort) or test(nothing)"));
        assert!(table("!test(nothing) & !module(Hash)"));
        assert!(!table("not (module(Sort) and kind(unit))"));
        assert!(table("(module(Hash) or module(Sort)) and test(sorts)"));
        assert!(table("not not module(Sort)"));
    }

    #[test]
    fn words_are_not_taken_from_inside_names() {
        assert!(table("test(order) or test(sorts)"));
        assert!(!table("test(android) and test(sorts)"));
        assert!(error("test(a) order(b)").message.contains("expected `and`"));
        assert!(
            error("notes(x)")
                .message
                .contains("unknown predicate `notes`")
        );
    }

    #[test]
    fn a_regex_may_hold_parens_and_escaped_slashes() {
        assert!(run("test(/a\\/b(c)/)", "a/bc", "M", "t.jai", &[]));
        assert!(!run("test(/a\\/b(c)/)", "a/b", "M", "t.jai", &[]));
    }

    #[test]
    fn errors_point_at_the_problem() {
        let cases = [
            ("test(a) and", 11, "expected a predicate"),
            ("test(a) and )", 12, "found `)`"),
            ("(test(a)", 8, "expected `)`"),
            ("test(a) test(b)", 8, "expected `and`, `or`"),
            ("name(a)", 0, "unknown predicate `name`"),
            ("test a", 5, "expected `(` after `test`"),
            ("kind(fast)", 5, "unknown kind `fast`"),
            ("all(x)", 4, "takes no argument"),
            ("test()", 5, "expected a name"),
            ("test(a", 5, "expected `)`"),
            ("test(/a", 5, "no closing `/`"),
            ("test(/a(/)", 5, "invalid regular expression"),
            ("tag(slow)", 0, "unknown predicate `tag`"),
        ];
        for (source, at, message) in cases {
            let found = error(source);
            assert_eq!(found.at, at, "{source}: {found}");
            assert!(found.message.contains(message), "{source}: {found}");
        }
    }

    #[test]
    fn the_error_text_has_a_caret_under_the_position() {
        let text = error("test(a) and (").to_string();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[1], "  test(a) and (");
        assert_eq!(lines[2], format!("  {}^", " ".repeat(13)));
    }

    #[test]
    fn only_a_snapshot_kind_needs_the_source() {
        let uses = |expr| FilterExpr::parse(expr).unwrap().uses_snapshot_kind();
        assert!(uses("module(A) and not kind(snapshot)"));
        assert!(!uses("kind(unit) or test(snapshot)"));
    }
}
