//! The path patterns of `jacket.toml` (`exclude`, `[[lint.overrides]] paths`, `portable`,
//! `platforms`): compiled once when the file is read, matched in time linear in the path.
//!
//! - `*` is any run of characters but `/`; `?` is one character but `/`.
//! - `**` is any run, `/` included; `**/` is any run ending in `/`, or nothing, so `**/x` finds `x` at the top.
//! - `[abc]`, `[a-z]` and `[!abc]` (or `[^abc]`) are one character but `/` from a set.
//! - `{a,b}` is one of the comma separated alternatives; they may hold the other forms and nest.
//! - `\` makes the next character literal. Anything else stands for itself.
//! - A trailing `/` is dropped, an empty pattern matches nothing, and a pattern that matches a
//!   directory matches everything inside it.
//!
//! Paths are `/` separated text. [`relative_text`] writes a path that way: a `\` is a separator
//! only where the platform's paths use it, never inside a Unix file name.
//!
//! `stdlib/Extensions/Jai_Format/config.jai` matches the first, second and last forms for
//! `jaifmt`; `tests/glob/cases.txt` holds the cases both run.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::path::{Component, Path};
use std::str::FromStr;

/// Most alternatives one pattern may expand to (`{a,b}{c,d}` is four).
const MAX_ALTERNATIVES: usize = 256;

/// A pattern that could not be compiled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternError {
    pub pattern: String,
    pub message: String,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid path pattern `{}`: {}",
            self.pattern, self.message
        )
    }
}

impl std::error::Error for PatternError {
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Literal(char),
    /// `?`
    Any,
    /// `*`
    Star,
    /// `**`
    Globstar,
    /// The start of `**/`, which is this token, a `Globstar` and a `Literal('/')`: it lets the
    /// state jump over the other two, as `**/` may match nothing.
    OptionalGlobstarSlash,
    Class {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

impl Token {
    /// Whether the token can match nothing.
    fn can_skip(&self) -> bool {
        matches!(self, Token::Star | Token::Globstar)
    }
}

/// A compiled path pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathPattern {
    source: String,
    /// One token list per alternative of the `{a,b}` forms (one when there are none).
    alternatives: Vec<Vec<Token>>,
}

impl PathPattern {
    pub fn new(source: &str) -> Result<PathPattern, PatternError> {
        let fail = |message: String| PatternError {
            pattern: source.to_string(),
            message,
        };
        let trimmed = source.trim_end_matches('/');
        let alternatives = if trimmed.is_empty() {
            Vec::new()
        } else {
            let mut parser = Parser {
                chars: trimmed.chars().collect(),
                at: 0,
            };
            let nodes = parser.sequence(0).map_err(fail)?;
            let mut out = Vec::new();
            expand(&nodes, &mut out).map_err(fail)?;
            out
        };
        Ok(PathPattern {
            source: source.to_string(),
            alternatives,
        })
    }

    /// The pattern as written.
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// How many characters of the pattern are neither `*` nor `?`: more is more specific.
    pub fn specificity(&self) -> usize {
        self.source
            .chars()
            .filter(|c| !matches!(c, '*' | '?'))
            .count()
    }

    /// Whether `path` (`/` separated, relative to the config's directory) is matched, or a
    /// directory above it is.
    pub fn is_match(&self, path: &str) -> bool {
        self.alternatives.iter().any(|tokens| run(tokens, path))
    }
}

/// Simulates the pattern on `path` one character at a time: every state it could be in at once
/// is tracked, so the time is the path length times the pattern length however many `*` there are.
fn run(tokens: &[Token], path: &str) -> bool {
    let end = tokens.len();
    let mut current = vec![false; end + 1];
    let mut next = vec![false; end + 1];
    current[0] = true;
    close(tokens, &mut current);
    for c in path.chars() {
        // A directory of the path ends here.
        if c == '/' && current[end] {
            return true;
        }
        next.fill(false);
        for (i, token) in tokens.iter().enumerate() {
            if !current[i] {
                continue;
            }
            match token {
                Token::Literal(l) => next[i + 1] |= *l == c,
                Token::Any => next[i + 1] |= c != '/',
                Token::Class {
                    negated,
                    ranges,
                } => {
                    let inside = ranges.iter().any(|&(lo, hi)| lo <= c && c <= hi);
                    next[i + 1] |= c != '/' && inside != *negated;
                }
                Token::Star => next[i] |= c != '/',
                Token::Globstar => next[i] = true,
                Token::OptionalGlobstarSlash => {}
            }
        }
        close(tokens, &mut next);
        std::mem::swap(&mut current, &mut next);
        if !current.contains(&true) {
            return false;
        }
    }
    current[end]
}

/// Adds the states reached by tokens that match nothing.
fn close(tokens: &[Token], states: &mut [bool]) {
    for (i, token) in tokens.iter().enumerate() {
        if !states[i] {
            continue;
        }
        if token.can_skip() {
            states[i + 1] = true;
        } else if *token == Token::OptionalGlobstarSlash {
            states[i + 1] = true;
            states[i + 3] = true;
        }
    }
}

impl FromStr for PathPattern {
    type Err = PatternError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        PathPattern::new(text)
    }
}

impl fmt::Display for PathPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}

impl PartialOrd for PathPattern {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathPattern {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source.cmp(&other.source)
    }
}

impl Serialize for PathPattern {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.source)
    }
}

impl<'de> Deserialize<'de> for PathPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        PathPattern::new(&text).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

/// What the parser reads before alternatives are expanded.
enum Node {
    Token(Token),
    Alternatives(Vec<Vec<Node>>),
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        self.at += 1;
        c
    }

    /// Up to the end of the pattern, or (inside `{}`) to a `,` or `}`.
    fn sequence(&mut self, depth: usize) -> Result<Vec<Node>, String> {
        let mut out = Vec::new();
        loop {
            let Some(c) = self.peek() else {
                return if depth > 0 {
                    Err("`{` is never closed".into())
                } else {
                    Ok(out)
                };
            };
            if depth > 0 && matches!(c, ',' | '}') {
                return Ok(out);
            }
            self.at += 1;
            let node = match c {
                '{' => {
                    let mut alternatives = Vec::new();
                    loop {
                        alternatives.push(self.sequence(depth + 1)?);
                        match self.bump() {
                            Some(',') => {}
                            Some('}') => break,
                            _ => return Err("`{` is never closed".into()),
                        }
                    }
                    Node::Alternatives(alternatives)
                }
                '[' => Node::Token(self.class()?),
                '\\' => Node::Token(Token::Literal(
                    self.bump().ok_or("`\\` at the end escapes nothing")?,
                )),
                '?' => Node::Token(Token::Any),
                '*' if self.peek() == Some('*') => {
                    self.at += 1;
                    if self.peek() == Some('/') {
                        self.at += 1;
                        out.push(Node::Token(Token::OptionalGlobstarSlash));
                        out.push(Node::Token(Token::Globstar));
                        Node::Token(Token::Literal('/'))
                    } else {
                        Node::Token(Token::Globstar)
                    }
                }
                '*' => Node::Token(Token::Star),
                c => Node::Token(Token::Literal(c)),
            };
            out.push(node);
        }
    }

    /// A `[...]` class, after its `[`.
    fn class(&mut self) -> Result<Token, String> {
        let negated = matches!(self.peek(), Some('!' | '^'));
        if negated {
            self.at += 1;
        }
        let mut ranges = Vec::new();
        let mut first = true;
        loop {
            let lo = match self.bump() {
                None => return Err("`[` is never closed".into()),
                Some(']') if !first => break,
                Some('\\') => self.bump().ok_or("`[` is never closed")?,
                Some(c) => c,
            };
            first = false;
            let mut hi = lo;
            if self.peek() == Some('-') && self.chars.get(self.at + 1).is_some_and(|&c| c != ']') {
                self.at += 1;
                hi = match self.bump() {
                    Some('\\') => self.bump().ok_or("`[` is never closed")?,
                    Some(c) => c,
                    None => return Err("`[` is never closed".into()),
                };
                if hi < lo {
                    return Err(format!("the range `{lo}-{hi}` runs backwards"));
                }
            }
            ranges.push((lo, hi));
        }
        Ok(Token::Class {
            negated,
            ranges,
        })
    }
}

/// Every token list `nodes` stands for, one per way of choosing among the alternatives.
fn expand(nodes: &[Node], out: &mut Vec<Vec<Token>>) -> Result<(), String> {
    let mut lists: Vec<Vec<Token>> = vec![Vec::new()];
    for node in nodes {
        match node {
            Node::Token(token) => {
                for list in &mut lists {
                    list.push(token.clone());
                }
            }
            Node::Alternatives(alternatives) => {
                let mut tails = Vec::new();
                for alternative in alternatives {
                    expand(alternative, &mut tails)?;
                }
                if lists.len().saturating_mul(tails.len()) > MAX_ALTERNATIVES {
                    return Err(format!(
                        "the `{{}}` alternatives expand to more than {MAX_ALTERNATIVES} patterns"
                    ));
                }
                lists = lists
                    .iter()
                    .flat_map(|head| {
                        tails.iter().map(move |tail| {
                            let mut list = head.clone();
                            list.extend(tail.iter().cloned());
                            list
                        })
                    })
                    .collect();
            }
        }
    }
    out.extend(lists);
    Ok(())
}

/// `path` relative to `root` (the whole path when it is not under it), written with `/`
/// separators for [`PathPattern::is_match`].
pub fn relative_text(path: &Path, root: &Path) -> String {
    let path = path.strip_prefix(root).unwrap_or(path);
    let mut out = String::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push_str(&prefix.as_os_str().to_string_lossy()),
            Component::RootDir => out.push('/'),
            Component::CurDir => {}
            Component::ParentDir | Component::Normal(_) => {
                if !out.is_empty() && !out.ends_with('/') {
                    out.push('/');
                }
                out.push_str(&component.as_os_str().to_string_lossy());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(pattern: &str, path: &str) -> bool {
        PathPattern::new(pattern).unwrap().is_match(path)
    }

    fn invalid(pattern: &str) -> String {
        PathPattern::new(pattern).unwrap_err().message
    }

    /// The cases in `tests/glob/cases.txt` are shared with jaifmt's tests.
    #[test]
    fn golden_cases() {
        let cases = include_str!("../../../tests/glob/cases.txt");
        let mut count = 0;
        for line in cases
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let [pattern, path, expected] = line.split('\t').collect::<Vec<_>>()[..] else {
                panic!("bad glob case: {line:?}");
            };
            assert_eq!(
                matches(pattern, path),
                expected == "match",
                "{pattern:?} against {path:?}"
            );
            count += 1;
        }
        assert!(count > 30);
    }

    #[test]
    fn wildcards() {
        assert!(matches("tests/**", "tests/corpus/a.jai"));
        assert!(matches("*.jai", "a.jai"));
        assert!(!matches("*.jai", "dir/a.jai"));
        assert!(matches("**/gen_*.jai", "x/y/gen_a.jai"));
        assert!(matches("**/gen_*.jai", "gen_a.jai"));
        assert!(!matches("**/gen_*.jai", "x/my_gen_a.jai"));
        assert!(matches("a/**/b", "a/b"));
        assert!(!matches("a/**/b", "a/xb"));
        assert!(!matches("a?c", "a/c"));
        assert!(!matches("", "a/b.jai"));
        assert!(!matches("/", "a/b.jai"));
    }

    #[test]
    fn classes() {
        assert!(matches("file[0-9].jai", "file7.jai"));
        assert!(!matches("file[0-9].jai", "filex.jai"));
        assert!(matches("file[!0-9].jai", "filex.jai"));
        assert!(matches("file[^0-9].jai", "filex.jai"));
        assert!(!matches("file[!0-9].jai", "file7.jai"));
        assert!(matches("[abc]", "b"));
        assert!(!matches("[abc]", "d"));
        assert!(matches("[]x]", "]"));
        assert!(matches("[a-]", "-"));
        assert!(matches(r"[\]]", "]"));
        // A class never matches a separator, even a negated one.
        assert!(!matches("a[!x]b", "a/b"));
        assert!(matches("[a-c][a-c]", "ab"));
    }

    #[test]
    fn alternatives() {
        assert!(matches("src/*.{jai,c}", "src/a.jai"));
        assert!(matches("src/*.{jai,c}", "src/a.c"));
        assert!(!matches("src/*.{jai,c}", "src/a.h"));
        assert!(matches("{a,b}/{c,d}/x", "b/c/x"));
        assert!(!matches("{a,b}/{c,d}/x", "c/b/x"));
        assert!(matches("{a,b{c,d}}x", "bdx"));
        assert!(matches("a{,b}", "a"));
        assert!(matches("a{,b}", "ab"));
        assert!(matches("{tests,docs}/**", "docs/a/b.md"));
        // Outside braces a comma is an ordinary character.
        assert!(matches("a,b", "a,b"));
        // A directory matched by one alternative covers what is inside.
        assert!(matches("{lib,app}", "app/main.jai"));
    }

    #[test]
    fn escapes() {
        assert!(matches(r"a\*b", "a*b"));
        assert!(!matches(r"a\*b", "axb"));
        assert!(matches(r"a\[b", "a[b"));
        assert!(matches(r"a\{b", "a{b"));
        assert!(matches(r"\*\*", "**"));
    }

    #[test]
    fn directories_cover_what_is_inside() {
        assert!(matches("tests/corpus", "tests/corpus/a/b.jai"));
        assert!(!matches("tests/corpus", "tests/corpus2/a.jai"));
        assert!(matches("tests/corpus/", "tests/corpus/a.jai"));
        assert!(matches("*", "dir/a.jai"));
    }

    #[test]
    fn invalid_patterns_say_why() {
        assert!(invalid("a[bc").contains("`[`"));
        assert!(invalid("a[]").contains("`[`"));
        assert!(invalid("a{b,c").contains("`{`"));
        assert!(invalid("a{b").contains("`{`"));
        assert!(invalid("a\\").contains("`\\`"));
        assert!(invalid("[z-a]").contains("backwards"));
        assert!(invalid("{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}").contains("more than"));
        let error = PathPattern::new("a[").unwrap_err().to_string();
        assert!(error.contains("`a[`"), "{error}");
    }

    #[test]
    fn pathological_patterns_stay_fast() {
        let pattern = "**/".repeat(500) + "x";
        let path = "a/".repeat(2000) + "y";
        let started = std::time::Instant::now();
        assert!(!matches(&pattern, &path));
        assert!(matches(&pattern, &(path + "/x")));
        let stars = "*".repeat(500) + "x";
        assert!(!matches(&stars, &"a".repeat(5000)));
        let mixed = "a*".repeat(300);
        assert!(!matches(&mixed, &"a".repeat(2999).replace('a', "b")));
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn serde_round_trip() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Holder {
            paths: Vec<PathPattern>,
        }
        let holder: Holder = toml::from_str("paths = [\"a/**\", \"{b,c}\"]").unwrap();
        assert!(holder.paths[1].is_match("c"));
        assert_eq!(holder.paths[0].as_str(), "a/**");
        let text = toml::to_string(&holder).unwrap();
        assert_eq!(toml::from_str::<Holder>(&text).unwrap(), holder);
        let error = toml::from_str::<Holder>("paths = [\"ok\", \"b[\"]").unwrap_err();
        assert!(error.to_string().contains("line 1"), "{error}");
        assert!(error.to_string().contains("`b[`"), "{error}");
    }

    #[test]
    fn relative_text_uses_slashes() {
        let root = Path::new("/r");
        assert_eq!(relative_text(Path::new("/r/a/b.jai"), root), "a/b.jai");
        assert_eq!(relative_text(Path::new("/x/b.jai"), root), "/x/b.jai");
        assert_eq!(relative_text(Path::new("a/b.jai"), root), "a/b.jai");
        // On Unix a backslash belongs to the name.
        #[cfg(unix)]
        assert_eq!(relative_text(Path::new("/r/a\\b.jai"), root), "a\\b.jai");
        #[cfg(windows)]
        assert_eq!(
            relative_text(Path::new("C:\\r\\a\\b.jai"), Path::new("C:\\r")),
            "a/b.jai"
        );
    }
}
