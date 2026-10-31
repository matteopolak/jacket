//! Doc comments, found from the file's tokens so they work on any file (open or not) without
//! parsing it.
//!
//! The lexer keeps comments as trivia; a comment belongs to the declaration whose first token
//! follows it. A declaration's docs are the run of `//` lines (or one `/* */` block) directly
//! above its line, with no blank line between, plus (for fields, enum members and parameters)
//! a trailing `// ...` on its own line. The text is Markdown: one space is stripped after each
//! `//` and nothing else. Extraction is lazy: callers ask about one declaration at a time, and
//! the last file lexed is remembered per thread so a burst of questions about one file lexes it
//! once.
//!
//! Also here: parameter docs (`- name: text` items in a procedure's docs, or comments on the
//! parameter itself), and `[name]` / `[text](name)` intra-doc links, found with a Markdown
//! parser, with the Markdown rewriting that turns resolved links into real ones.

use crate::lexer::{Comment, CommentKind, Lexed, P, Tok, Token};
use crate::source::{FileId, line_start};
use pulldown_cmark::{
    BrokenLink, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd,
};
use std::cell::RefCell;
use std::rc::Rc;

/// The most lines a doc comment shows.
pub const MAX_LINES: usize = 200;

/// Parameter lists are read this many lines past their `(`, and looked back for this many.
const HEADER_LINES: usize = 80;

/// A rule drawn in comments (`// ------`, `//====`): not documentation.
fn decoration(body: &str) -> bool {
    let body = body.trim();
    body.len() >= 4 && body.chars().all(|c| "/=*~#_-+".contains(c))
}

/// Whether `at` can be sliced at in `text`. Offsets come from a compile of the text as it was
/// (completion data, an index built before an edit), so one can lie past the end or inside a
/// character once the text has changed; every entry point answers nothing for such an offset.
fn valid(text: &str, at: usize) -> bool {
    text.is_char_boundary(at)
}

/// One line of a comment's text, without delimiters, as a slice of the source.
struct Body<'t> {
    /// Where `text` starts in the file.
    src: usize,
    /// Just after the `//` (and any extra `/` or `!`) of a line comment; `src` for a block.
    marker_end: usize,
    text: &'t str,
}

/// A file's tokens and comments, lexed once.
struct Trivia {
    text: String,
    tokens: Vec<Token>,
    comments: Vec<Comment>,
}

thread_local! {
    static LAST: RefCell<Option<Rc<Trivia>>> = const { RefCell::new(None) };
}

/// `text` lexed, reusing the last lexing on this thread when the text is the same.
fn trivia(text: &str) -> Rc<Trivia> {
    LAST.with(|last| {
        let mut last = last.borrow_mut();
        if let Some(found) = last.as_ref()
            && found.text == text
        {
            return Rc::clone(found);
        }
        let lexed = Lexed::new(FileId(0), text);
        let found = Rc::new(Trivia {
            text: text.to_owned(),
            tokens: lexed.tokens,
            comments: lexed.comments,
        });
        *last = Some(Rc::clone(&found));
        found
    })
}

impl Trivia {
    fn start(c: &Comment) -> usize {
        c.span.start as usize
    }

    fn end(c: &Comment) -> usize {
        c.span.end as usize
    }

    /// Whether only whitespace comes before `c` on its line.
    fn owns_line(&self, c: &Comment) -> bool {
        let start = Self::start(c);
        self.text[line_start(&self.text, start)..start]
            .trim()
            .is_empty()
    }

    /// Whether `c` is on the line directly above `next` (an offset at or after its end).
    fn directly_above(&self, c: &Comment, next: usize) -> bool {
        let gap = &self.text[Self::end(c)..next];
        gap.trim().is_empty() && gap.matches('\n').count() == 1 && self.owns_line(c)
    }

    /// The comments that end at or before `offset`.
    fn comments_before(&self, offset: usize) -> &[Comment] {
        let n = self.comments.partition_point(|c| Self::end(c) <= offset);
        &self.comments[..n]
    }

    /// The doc comment attached to the line starting at `line`: one block comment, else the run
    /// of `//` lines, nearest last, ending at a rule.
    fn docs_run(&self, line: usize) -> Vec<&Comment> {
        let mut out: Vec<&Comment> = Vec::new();
        let mut next = line;
        for c in self.comments_before(line).iter().rev() {
            if !self.directly_above(c, next) {
                break;
            }
            if c.kind == CommentKind::Block {
                if out.is_empty() && c.terminated {
                    out.push(c);
                }
                break;
            }
            if decoration(c.body(&self.text)) {
                break;
            }
            out.push(c);
            if out.len() >= MAX_LINES {
                break;
            }
            next = Self::start(c);
        }
        out.reverse();
        out
    }

    /// The comments directly above the line starting at `line`, for link sites: lines and
    /// blocks, back to the nearest block.
    fn sites_run(&self, line: usize) -> Vec<&Comment> {
        let mut out: Vec<&Comment> = Vec::new();
        let mut next = line;
        for c in self.comments_before(line).iter().rev() {
            if !self.directly_above(c, next) {
                break;
            }
            out.push(c);
            if c.kind == CommentKind::Block {
                break;
            }
            next = Self::start(c);
        }
        out.reverse();
        out
    }

    /// The `//` lines directly above the line starting at `line`.
    fn line_run(&self, line: usize) -> Vec<&Comment> {
        let mut out: Vec<&Comment> = Vec::new();
        let mut next = line;
        for c in self.comments_before(line).iter().rev() {
            if c.kind != CommentKind::Line || !self.directly_above(c, next) {
                break;
            }
            out.push(c);
            next = Self::start(c);
        }
        out.reverse();
        out
    }

    /// The first comment that starts at or after `from` on its line.
    fn comment_after(&self, from: usize) -> Option<&Comment> {
        let end = self.text[from..]
            .find('\n')
            .map_or(self.text.len(), |n| from + n);
        let at = self.comments.partition_point(|c| Self::start(c) < from);
        self.comments.get(at).filter(|c| Self::start(c) < end)
    }

    /// The lines of `c`'s text.
    fn bodies(&self, c: &Comment) -> Vec<Body<'_>> {
        let (start, end) = (Self::start(c), Self::end(c));
        if c.kind == CommentKind::Line {
            let bytes = self.text.as_bytes();
            let mut at = start + 2;
            let marker_end = at
                + bytes[at..end]
                    .iter()
                    .take_while(|&&b| b == b'/' || b == b'!')
                    .count();
            if at < end && matches!(bytes[at], b'/' | b'!') {
                at += 1;
            }
            if at < end && bytes[at] == b' ' {
                at += 1;
            }
            let text = self.text[at..end].trim_end();
            return vec![Body {
                src: at,
                marker_end,
                text,
            }];
        }
        self.block_bodies(start, end, c.terminated)
    }

    /// The lines of a block comment: the delimiters are dropped, and a ` * ` margin or the
    /// common indent is removed.
    fn block_bodies(&self, start: usize, end: usize, terminated: bool) -> Vec<Body<'_>> {
        let inner_start = start + 2;
        let inner_end = if terminated {
            end - 2
        } else {
            end
        };
        let text = self.text.as_str();
        let mut lines: Vec<(usize, usize)> = Vec::new();
        let mut from = inner_start;
        for (i, _) in text[inner_start..inner_end].match_indices('\n') {
            lines.push((from, inner_start + i));
            from = inner_start + i + 1;
        }
        lines.push((from, inner_end));
        let margin = lines.iter().skip(1).all(|&(a, b)| {
            let t = text[a..b].trim_start();
            t.is_empty() || t.starts_with('*')
        });
        // Without a margin, the indent the continuation lines share is removed.
        let indent = if margin {
            0
        } else {
            lines
                .iter()
                .skip(1)
                .filter(|&&(a, b)| !text[a..b].trim().is_empty())
                .map(|&(a, b)| text[a..b].len() - text[a..b].trim_start().len())
                .min()
                .unwrap_or(0)
        };
        let mut out = Vec::with_capacity(lines.len());
        for (i, &(a, b)) in lines.iter().enumerate() {
            let b = a + text[a..b].trim_end().len();
            let skip_space = |a: usize| a + (text[a..b].len() - text[a..b].trim_start().len());
            let a = if i == 0 {
                skip_space(a)
            } else if margin {
                let mut a = skip_space(a);
                if text[a..b].starts_with('*') {
                    a += 1;
                }
                if text[a..b].starts_with(' ') {
                    a += 1;
                }
                a
            } else if a + indent <= b && text.is_char_boundary(a + indent) {
                a + indent
            } else {
                skip_space(a)
            };
            out.push(Body {
                src: a,
                marker_end: a,
                text: &text[a..b],
            });
        }
        out
    }

    /// The index of the first token starting at or after `at`.
    fn token_from(&self, at: usize) -> usize {
        self.tokens
            .partition_point(|t| (t.span.start as usize) < at)
    }

    /// Whether the token before `paren` makes it a procedure header's `(`.
    fn header_before(&self, paren: usize) -> bool {
        let Some(before) = paren.checked_sub(1) else {
            return false;
        };
        match &self.tokens[before].tok {
            Tok::Punct(P::ColonColon) => true,
            Tok::Directive(s) => s.as_str() == "type",
            Tok::Ident(s) if s.as_str() == "inline" => true,
            // `#type,distinct (`
            Tok::Ident(s) if s.as_str() == "distinct" => {
                before >= 2
                    && self.tokens[before - 1].tok == Tok::Punct(P::Comma)
                    && matches!(&self.tokens[before - 2].tok, Tok::Directive(s) if s.as_str() == "type")
            }
            _ => false,
        }
    }
}

/// The comments of one text, lexed once, for asking about many declarations of it.
pub struct Docs(Rc<Trivia>);

impl Docs {
    pub fn new(text: &str) -> Self {
        Self(trivia(text))
    }

    /// [`above`] for this text.
    pub fn above(&self, at: usize) -> Option<String> {
        if !valid(&self.0.text, at) {
            return None;
        }
        above_in(&self.0, at)
    }
}

/// The docs above the declaration whose line holds `at`.
pub fn above(text: &str, at: usize) -> Option<String> {
    if !valid(text, at) {
        return None;
    }
    above_in(&trivia(text), at)
}

fn above_in(t: &Trivia, at: usize) -> Option<String> {
    let run = t.docs_run(line_start(&t.text, at));
    let mut lines: Vec<&str> = run
        .into_iter()
        .flat_map(|c| t.bodies(c))
        .map(|b| b.text)
        .collect();
    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let doc = lines.join("\n");
    (!doc.trim().is_empty()).then_some(doc)
}

/// The `// ...` after a field, member or parameter declared at `at` (its name), if what
/// comes before it on the line is a whole declaration rather than the start of a body.
pub fn trailing(text: &str, at: usize) -> Option<String> {
    if !valid(text, at) {
        return None;
    }
    trailing_in(&trivia(text), at)
}

fn trailing_in(t: &Trivia, at: usize) -> Option<String> {
    let comment = t
        .comment_after(at)
        .filter(|c| c.kind == CommentKind::Line)?;
    let body = t.bodies(comment).pop()?.text;
    let code = t.text[at..Trivia::start(comment)].trim_end();
    if code.contains('{') {
        return None;
    }
    let ends = code.ends_with(';') || code.ends_with(',');
    if (!ends && code.contains("::")) || body.trim().is_empty() {
        return None;
    }
    Some(body.to_owned())
}

/// The docs of the declaration named at `at`: the comment lines above it, else its trailing
/// comment; for a parameter, its own comments, else its item in the procedure's docs.
pub fn declaration(text: &str, at: usize) -> Option<String> {
    if !valid(text, at) {
        return None;
    }
    let t = trivia(text);
    if let Some(open) = enclosing_header_in(&t, at) {
        let name = word_at(text, at)?;
        return parameter(&t, open, name);
    }
    above_in(&t, at).or_else(|| trailing_in(&t, at))
}

/// The identifier starting at `at`.
fn word_at(text: &str, at: usize) -> Option<&str> {
    let rest = text.get(at..)?;
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

/// The `(` of the procedure header the parameter named at `at` is in, if it is in one: the
/// nearest unclosed `(` before `at`, without a `{`, `}` or `;` in between, that follows `::`
/// (or `#type`, `inline`).
pub fn enclosing_header(text: &str, at: usize) -> Option<usize> {
    if !valid(text, at) {
        return None;
    }
    enclosing_header_in(&trivia(text), at)
}

fn enclosing_header_in(t: &Trivia, at: usize) -> Option<usize> {
    // Only the lines just before `at` are read.
    let mut bound = line_start(&t.text, at);
    for _ in 0..HEADER_LINES {
        if bound == 0 {
            break;
        }
        bound = line_start(&t.text, bound - 1);
    }
    let mut depth = 0i32;
    for i in (0..t.token_from(at)).rev() {
        let token = &t.tokens[i];
        if (token.span.start as usize) < bound {
            return None;
        }
        let Tok::Punct(p) = &token.tok else {
            continue;
        };
        match p {
            P::RParen | P::RBracket => depth += 1,
            P::LBracket | P::DotBracket => depth -= 1,
            P::LParen if depth > 0 => depth -= 1,
            P::LParen => return t.header_before(i).then_some(token.span.start as usize),
            P::LBrace | P::DotBrace | P::RBrace | P::Semi => return None,
            _ => {}
        }
    }
    None
}

/// The docs of parameter `name` of the header opened at `open`: comments on the parameter
/// itself, else its item in the docs above the header.
fn parameter(t: &Trivia, open: usize, name: &str) -> Option<String> {
    let name = name.trim_start_matches('$');
    header_parameters(t, open)
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, doc)| doc)
        .or_else(|| {
            let doc = above_in(t, open)?;
            list_parameters(&doc)
                .into_iter()
                .find(|(n, _)| n == name)
                .map(|(_, d)| d)
        })
}

/// Every parameter of the procedure whose header opens at `open` that has docs, from its own
/// comments (above it, or after it on its line) and from the list in the docs above the
/// header, as (name, docs). Own comments win.
pub fn parameters(text: &str, open: usize) -> Vec<(String, String)> {
    if !valid(text, open) {
        return Vec::new();
    }
    let t = trivia(text);
    let mut found = header_parameters(&t, open);
    if let Some(doc) = above_in(&t, open) {
        for (name, doc) in list_parameters(&doc) {
            if !found.iter().any(|(n, _)| *n == name) {
                found.push((name, doc));
            }
        }
    }
    found
}

/// Where the parameter list of the procedure declared at `at` (its name, or its literal)
/// opens: the first `(` from there.
pub fn header_open(text: &str, at: usize) -> Option<usize> {
    if !valid(text, at) {
        return None;
    }
    let end = text.get(at..)?.find('\n').map_or(text.len(), |n| at + n);
    text[at..end].find('(').map(|n| at + n)
}

/// Parameters with a comment directly above them or after them on their line.
fn header_parameters(t: &Trivia, open: usize) -> Vec<(String, String)> {
    let first = t.token_from(open);
    let starts_here = t
        .tokens
        .get(first)
        .is_some_and(|k| k.span.start as usize == open && k.tok == Tok::Punct(P::LParen));
    if !starts_here {
        return Vec::new();
    }
    let open_line = line_start(&t.text, open);
    let mut limit = open_line;
    for _ in 0..HEADER_LINES {
        let Some(n) = t.text[limit..].find('\n') else {
            limit = t.text.len();
            break;
        };
        limit += n + 1;
    }
    // The tokens of each top-level parameter, up to the closing parenthesis.
    let mut segments: Vec<(usize, usize)> = Vec::new();
    let (mut depth, mut seg_start) = (0i32, first + 1);
    for (i, token) in t.tokens.iter().enumerate().skip(first) {
        if token.span.start as usize >= limit {
            segments.push((seg_start, i));
            break;
        }
        let Tok::Punct(p) = &token.tok else {
            continue;
        };
        match p {
            P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket => depth += 1,
            P::RParen | P::RBracket | P::RBrace => {
                depth -= 1;
                if depth == 0 {
                    segments.push((seg_start, i));
                    break;
                }
            }
            P::Comma if depth == 1 => {
                segments.push((seg_start, i));
                seg_start = i + 1;
            }
            _ => {}
        }
    }
    // (name, the start of its line)
    let mut named: Vec<(String, usize)> = Vec::new();
    for (a, b) in segments {
        let mut j = a;
        while j < b && matches!(t.tokens[j].tok, Tok::Punct(P::Dollar | P::DollarDollar)) {
            j += 1;
        }
        if j >= b {
            continue;
        }
        let Tok::Ident(name) = &t.tokens[j].tok else {
            continue;
        };
        let declares = j + 1 >= b
            || matches!(
                t.tokens[j + 1].tok,
                Tok::Punct(P::Colon | P::ColonEq | P::ColonColon)
            );
        if declares {
            let line = line_start(&t.text, t.tokens[j].span.start as usize);
            named.push((name.as_str().to_owned(), line));
        }
    }
    let mut out: Vec<(String, String)> = Vec::new();
    let mut at = 0;
    while at < named.len() {
        let line = named[at].1;
        let count = named[at..].iter().take_while(|n| n.1 == line).count();
        let leading: Vec<&str> = if line == open_line {
            Vec::new()
        } else {
            t.line_run(line)
                .into_iter()
                .flat_map(|c| t.bodies(c))
                .map(|b| b.text)
                .collect()
        };
        let from = if line == open_line {
            open
        } else {
            line
        };
        let after = t
            .comment_after(from)
            .filter(|c| c.kind == CommentKind::Line)
            .and_then(|c| t.bodies(c).pop())
            .map(|b| b.text)
            .filter(|b| !b.trim().is_empty());
        for (k, (name, _)) in named[at..at + count].iter().enumerate() {
            let mut parts: Vec<&str> = Vec::new();
            if k == 0 {
                parts.extend(&leading);
            }
            if k == count - 1 {
                parts.extend(after);
            }
            if !parts.is_empty() {
                out.push((name.clone(), parts.join("\n")));
            }
        }
        at += count;
    }
    out
}

/// Items of a Markdown list in `doc` that document a parameter: `- name: text`,
/// ``- `name`: text`` or `- name — text`. Following indented lines continue the item.
pub fn list_parameters(doc: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut fenced = false;
    let mut open = false;
    for line in doc.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            open = false;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some((name, rest)) = list_item(trimmed) {
            out.push((name, rest));
            open = true;
        } else if open && line.starts_with([' ', '\t']) && !trimmed.is_empty() {
            if let Some((_, text)) = out.last_mut() {
                text.push('\n');
                text.push_str(trimmed);
            }
        } else {
            open = false;
        }
    }
    out
}

/// `- name: text` and its variants as (name, text).
fn list_item(line: &str) -> Option<(String, String)> {
    let rest = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))?
        .trim_start();
    let (name, rest) = if let Some(inner) = rest.strip_prefix('`') {
        let end = inner.find('`')?;
        (&inner[..end], &inner[end + 1..])
    } else {
        let rest = rest.strip_prefix('$').unwrap_or(rest);
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        (&rest[..end], &rest[end..])
    };
    let name = name.trim_start_matches('$');
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let rest = rest
        .strip_prefix(':')
        .or_else(|| rest.trim_start().strip_prefix('—'))
        .or_else(|| rest.trim_start().strip_prefix('–'))
        .or_else(|| rest.trim_start().strip_prefix("- "))?;
    Some((name.to_owned(), rest.trim().to_owned()))
}


// Intra-doc links

/// A `[name]`, `` [`name`] `` or `[text](name)` in documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The whole link, brackets (and target) included, as byte offsets into the text.
    pub whole: (usize, usize),
    /// The text between the brackets.
    pub label: (usize, usize),
    /// The dotted name the link spells (`Basic.print`), without backticks or `()`: the label,
    /// or the target of `[text](target)`.
    pub path: (usize, usize),
    /// What it names: `print`, `Basic`, `print` for `Basic.print`'s last segment, ...
    pub names: Vec<String>,
    /// Written `[text](name)`.
    pub targeted: bool,
    /// The label was written in backticks or with `()`: code, not prose.
    pub code: bool,
    /// A `[label]: url` definition in the same text gives the label a target, so it is a link
    /// to that URL and not an intra-doc one.
    pub defined: bool,
}

/// A path such as `print` or `Basic.print` from link text (backticks and a trailing `()`
/// removed), with where it starts and ends in `text` and whether it was written as code.
fn link_path(text: &str) -> Option<(Vec<String>, usize, usize, bool)> {
    let mut start = text.len() - text.trim_start().len();
    let mut t = text.trim();
    let mut code = false;
    if let Some(inner) = t.strip_prefix('`').and_then(|t| t.strip_suffix('`')) {
        start += 1 + (inner.len() - inner.trim_start().len());
        t = inner.trim();
        code = true;
    }
    if let Some(inner) = t.strip_suffix("()") {
        t = inner;
        code = true;
    }
    if t.is_empty() {
        return None;
    }
    let mut names = Vec::new();
    for segment in t.split('.') {
        let mut chars = segment.chars();
        let first = chars.next()?;
        if !(first.is_alphabetic() || first == '_')
            || !chars.all(|c| c.is_alphanumeric() || c == '_')
        {
            return None;
        }
        names.push(segment.to_owned());
    }
    Some((names, start, start + t.len(), code))
}


/// The links in `doc`, in order, skipping code, URLs and things that are not links (`a[i]`,
/// images, `[x][y]`, a label that is not a name). A Markdown parser finds them, so escapes,
/// code spans and fences are handled as Markdown handles them; a bracketed label with no
/// definition counts as a link, since intra-doc links have none.
pub fn links(doc: &str) -> Vec<Link> {
    let mut unresolved = |_: BrokenLink<'_>| Some((CowStr::Borrowed(""), CowStr::Borrowed("")));
    let parser =
        Parser::new_with_broken_link_callback(doc, Options::empty(), Some(&mut unresolved));
    let mut out = Vec::new();
    // The open link: its range, kind and where its last inner event ends.
    let mut open: Option<(std::ops::Range<usize>, LinkType, usize)> = None;
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Link {
                link_type, ..
            }) => {
                let inner = range.start + 1;
                open = Some((range, link_type, inner));
            }
            Event::End(TagEnd::Link) => {
                if let Some((range, kind, inner)) = open.take() {
                    out.extend(link_from(doc, range, kind, inner));
                }
            }
            _ => {
                if let Some((_, _, inner)) = open.as_mut() {
                    *inner = (*inner).max(range.end);
                }
            }
        }
    }
    // A definition line directly under a paragraph line is not one to CommonMark, but it is
    // written (and meant) as one.
    let defined: Vec<&str> = doc.lines().filter_map(reference_definition).collect();
    for link in &mut out {
        link.defined |= defined.contains(&doc[link.label.0..link.label.1].trim());
    }
    out
}

/// The label of a Markdown reference definition (`[label]: url`) that `line` is.
fn reference_definition(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix('[')?;
    let end = rest.find("]:")?;
    let after = rest[end + 2..].trim_start();
    (!after.is_empty() && !rest[..end].contains('[')).then(|| rest[..end].trim())
}

/// The intra-doc link a Markdown link is, if it is one.
fn link_from(
    doc: &str,
    whole: std::ops::Range<usize>,
    kind: LinkType,
    inner_end: usize,
) -> Option<Link> {
    let (targeted, defined) = match kind {
        LinkType::Inline => (true, false),
        LinkType::Shortcut | LinkType::Collapsed => (false, true),
        LinkType::ShortcutUnknown | LinkType::CollapsedUnknown => (false, false),
        _ => return None,
    };
    let prev = doc[..whole.start].chars().next_back();
    if prev.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | ')' | ']' | '!')) {
        return None;
    }
    let label_start = whole.start + 1;
    let label_end =
        inner_end.max(label_start) + doc[inner_end.max(label_start)..whole.end].find(']')?;
    let label = (label_start, label_end);
    if targeted {
        // `[text](name)`: only a name is a link; a URL is left alone.
        let target = label_end + 2;
        if doc.as_bytes().get(label_end + 1) != Some(&b'(') || target > whole.end - 1 {
            return None;
        }
        let (names, from, to, code) = link_path(&doc[target..whole.end - 1])?;
        return Some(Link {
            whole: (whole.start, whole.end),
            label,
            path: (target + from, target + to),
            names,
            targeted,
            code,
            defined,
        });
    }
    let (names, from, to, code) = link_path(&doc[label.0..label.1])?;
    Some(Link {
        whole: (whole.start, whole.end),
        label,
        path: (label.0 + from, label.0 + to),
        names,
        targeted,
        code,
        defined,
    })
}

/// A comment's text, one line per source line, with where each line came from.
struct DocText {
    text: String,
    lines: Vec<DocLine>,
}

struct DocLine {
    /// Where the line starts in `text`.
    doc: usize,
    /// Where it starts and ends in the file, and where its comment marker ends.
    src: usize,
    marker_end: usize,
    len: usize,
}

impl DocText {
    fn new(t: &Trivia, run: &[&Comment]) -> Self {
        let mut doc = Self {
            text: String::new(),
            lines: Vec::new(),
        };
        for body in run.iter().flat_map(|c| t.bodies(c)) {
            doc.lines.push(DocLine {
                doc: doc.text.len(),
                src: body.src,
                marker_end: body.marker_end,
                len: body.text.len(),
            });
            doc.text.push_str(body.text);
            doc.text.push('\n');
        }
        doc
    }

    fn line_of(&self, offset: usize) -> usize {
        self.lines
            .partition_point(|l| l.doc <= offset)
            .saturating_sub(1)
    }

    /// The file offset of the byte at `offset` in the text.
    fn source(&self, offset: usize) -> usize {
        let line = &self.lines[self.line_of(offset)];
        line.src + (offset - line.doc).min(line.len)
    }

    /// The file offset just after the byte before `end`, for the end of a range.
    fn source_end(&self, end: usize) -> usize {
        self.source(end.saturating_sub(1)) + usize::from(end > 0)
    }
}

/// The code in the Jai code blocks (fences untagged or tagged `jai`) of `//` comments in
/// `text`: a byte range per line, after the comment marker.
pub fn fenced_code(text: &str) -> Vec<(usize, usize)> {
    let t = trivia(text);
    let mut out = Vec::new();
    // Consecutive `//` lines, each directly under the one before.
    let mut run: Vec<&Comment> = Vec::new();
    let mut flush = |run: &mut Vec<&Comment>| {
        if !run.is_empty() {
            fenced_in(&t, run, &mut out);
            run.clear();
        }
    };
    for c in &t.comments {
        let continues = run
            .last()
            .is_some_and(|last| t.directly_above(last, Trivia::start(c)));
        if !continues {
            flush(&mut run);
        }
        if c.kind == CommentKind::Line && t.owns_line(c) {
            run.push(c);
        } else {
            flush(&mut run);
        }
    }
    flush(&mut run);
    out
}

fn fenced_in(t: &Trivia, run: &[&Comment], out: &mut Vec<(usize, usize)>) {
    let doc = DocText::new(t, run);
    for (event, range) in Parser::new(&doc.text).into_offset_iter() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = event else {
            continue;
        };
        let info = info.trim();
        if !(info.is_empty() || info == "jai") {
            continue;
        }
        let (first, last) = (
            doc.line_of(range.start),
            doc.line_of(range.end.saturating_sub(1)),
        );
        // Everything between the fence lines.
        let closed = doc.text[doc.lines[last].doc..]
            .trim_start()
            .starts_with(['`', '~'])
            && last > first;
        let end = if closed {
            last
        } else {
            last + 1
        };
        for line in &doc.lines[(first + 1).min(end)..end] {
            let body = &doc.text[line.doc..line.doc + line.len];
            if !body.trim().is_empty() {
                out.push((line.marker_end, line.src + line.len));
            }
        }
    }
}

/// `doc` as Markdown for display: untagged code fences become `jai`, and links `resolve`
/// gives a URL for become Markdown links to it (the rest stay as written).
pub fn markdown(doc: &str, resolve: &dyn Fn(&[String]) -> Option<String>) -> String {
    rewrite(doc, resolve, true)
}

/// [`markdown`] with the code fences left as written, for a page that has to tell a fence that
/// says `jai` from one that says nothing.
pub fn linked(doc: &str, resolve: &dyn Fn(&[String]) -> Option<String>) -> String {
    rewrite(doc, resolve, false)
}

fn rewrite(doc: &str, resolve: &dyn Fn(&[String]) -> Option<String>, fences: bool) -> String {
    // (start, end, replacement) in source order.
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for (event, range) in Parser::new(doc).into_offset_iter() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = event else {
            continue;
        };
        if !fences {
            continue;
        }
        if !info.trim().is_empty() {
            continue;
        }
        let line_end = doc[range.start..]
            .find('\n')
            .map_or(doc.len(), |n| range.start + n);
        let line = &doc[range.start..line_end];
        if let Some(at) = line.find('`') {
            let run = line[at..].chars().take_while(|&c| c == '`').count();
            let end = range.start + at + run;
            edits.push((end, end, "jai".to_owned()));
        }
    }
    for link in links(doc) {
        if let Some(url) = resolve(&link.names) {
            let label = &doc[link.label.0..link.label.1];
            edits.push((link.whole.0, link.whole.1, format!("[{label}]({url})")));
        }
    }
    edits.sort_by_key(|e| e.0);
    let mut out = String::with_capacity(doc.len());
    let mut at = 0;
    for (start, end, text) in edits {
        out.push_str(&doc[at..start]);
        out.push_str(&text);
        at = end;
    }
    out.push_str(&doc[at..]);
    out.truncate(out.trim_end_matches(['\n', '\r']).len());
    out
}

/// Every link name path in `doc` (outside code), without repeats.
pub fn link_names(doc: &str) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    if !doc.contains('[') {
        return out;
    }
    for link in links(doc) {
        if !out.contains(&link.names) {
            out.push(link.names);
        }
    }
    out
}

/// A link in a source file: where its label is and what it names, resolved from `scope_at`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    pub label: (usize, usize),
    pub names: Vec<String>,
    /// An offset in the declaration the comment documents (its scope resolves the names).
    pub scope_at: usize,
    /// The whole link, brackets (and target) included.
    pub whole: (usize, usize),
    /// The dotted name it spells, without backticks or `()`.
    pub path: (usize, usize),
    /// The comment documents a declaration (the line it is above, or on, is one), rather
    /// than a statement.
    pub on_declaration: bool,
    /// A lowercase word in plain brackets (`[sic]`): as likely prose as a link, so it is
    /// worth reporting only as a misspelling of something in scope.
    pub plain: bool,
    /// It is written like an intra-doc link and not like ordinary bracketed prose (see
    /// [`meant_as_link`]).
    pub meant: bool,
}

/// The links in the doc comments of `text`: comment runs and blocks directly above a
/// declaration line, and trailing comments.
pub fn sites(text: &str) -> Vec<Site> {
    let t = trivia(text);
    let mut out = Vec::new();
    for (i, token) in t.tokens.iter().enumerate() {
        let start = token.span.start as usize;
        let line = line_start(text, start);
        // Comments belong to the first token of the line below them.
        if i > 0 && t.tokens[i - 1].span.end as usize > line {
            continue;
        }
        let line_end = text[start..].find('\n').map_or(text.len(), |n| start + n);
        let declares = declaration_line(text[start..line_end].trim());
        let run = t.sites_run(line);
        if !run.is_empty() {
            collect_sites(&t, &run, start, declares, &mut out);
        }
        if let Some(c) = t.comment_after(start)
            && c.kind == CommentKind::Line
        {
            collect_sites(&t, &[c], start, declares, &mut out);
        }
    }
    out
}

/// The site whose label holds `byte`.
pub fn site_at(text: &str, byte: usize) -> Option<Site> {
    if !valid(text, byte) {
        return None;
    }
    // Only a comment can hold a link: skip the scan for other lines.
    let start = line_start(text, byte);
    let end = text[byte..].find('\n').map_or(text.len(), |n| byte + n);
    let line = &text[start..end];
    if !line.contains("//")
        && !line.contains("*/")
        && !line.contains("/*")
        && !line.trim_start().starts_with('*')
    {
        return None;
    }
    sites(text)
        .into_iter()
        .find(|s| s.label.0 <= byte && byte <= s.label.1)
}

fn collect_sites(
    t: &Trivia,
    run: &[&Comment],
    scope_at: usize,
    declares: bool,
    out: &mut Vec<Site>,
) {
    let doc = DocText::new(t, run);
    if !doc.text.contains('[') {
        return;
    }
    for link in links(&doc.text) {
        let text = &doc.text;
        let before = &text[text[..link.whole.0].rfind('\n').map_or(0, |n| n + 1)..link.whole.0];
        let after = &text[link.whole.1
            ..text[link.whole.1..]
                .find('\n')
                .map_or(text.len(), |n| link.whole.1 + n)];
        // A label with a definition elsewhere in the comment is an ordinary Markdown link.
        let meant = !link.defined && meant_as_link(&link, before, after);
        let word = link.names[0].as_str();
        let plain = !link.targeted
            && !link.code
            && link.names.len() == 1
            && word.chars().all(|c| c.is_lowercase() || c.is_ascii_digit());
        let at = |(a, b): (usize, usize)| (doc.source(a), doc.source_end(b));
        out.push(Site {
            label: at(link.label),
            names: link.names.clone(),
            scope_at,
            whole: at(link.whole),
            path: at(link.path),
            on_declaration: declares,
            plain,
            meant,
        });
    }
}

/// A link that does not resolve is worth reporting only when it was written like an
/// intra-doc link and not like ordinary bracketed prose (`before` and `after` are the line's
/// text around it).
/// Written as code (`` [`name`] ``, `[name()]`), as `[text](name)` or as a dotted path
/// (`[Type.field]`) it is meant. A bare word is meant unless it is one letter (`[x]`, a
/// checkbox or a variable), all capitals (`[TODO]`, `[NOTE]`), followed by `:` (a
/// reference definition or footnote) or glued to quotes, slashes or similar (`'..[name]'`). Labels that are not a name (`[0]`, `[ ]`, `[^1]`,
/// `[some words]`) are never links at all.
pub fn meant_as_link(link: &Link, before: &str, after: &str) -> bool {
    if after.starts_with(':') {
        return false;
    }
    if link.targeted || link.code || link.names.len() > 1 {
        return true;
    }
    // Inside a quoted or path-like text (`'...[description]'`, `a/[name]`).
    let glued = before.ends_with([
        '.', '\'', '"', '/', '\\', '=', '<', '$', '@', '#', '%', '&', '*', '-',
    ]) || after.starts_with(['\'', '"', '/', '\\', '=', '>']);
    if glued {
        return false;
    }
    let word = link.names[0].as_str();
    let tag = word.chars().all(|c| c.is_uppercase() || c.is_ascii_digit());
    word.chars().count() > 1 && !tag
}

/// Whether a line (already trimmed) looks like it declares something: `name :: ...`,
/// `name := ...`, `name: Type`, `a, b: Type`, or an enum member alone on its line (`NAME,`).
pub fn declaration_line(line: &str) -> bool {
    let line = line.strip_prefix("using ").unwrap_or(line);
    let line = line.strip_prefix("#as ").unwrap_or(line);
    let mut rest = line.strip_prefix('$').unwrap_or(line);
    let mut names = 0;
    loop {
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let starts = rest
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_');
        if end == 0 || !starts {
            return false;
        }
        names += 1;
        rest = rest[end..].trim_start();
        if rest.starts_with(':') {
            return true;
        }
        let Some(after) = rest.strip_prefix([',', ';']) else {
            return false;
        };
        let after = after.trim_start();
        if after.is_empty() || after.starts_with("//") {
            // An enum member (`RED,`), not a statement.
            return names == 1 || rest.starts_with(',');
        }
        if rest.starts_with(';') {
            return false;
        }
        rest = after;
    }
}

/// A declaration's first line as a signature to show: from `at` to the `{` or the end of the
/// line, bounded.
pub fn signature_line(text: &str, at: usize) -> String {
    if !valid(text, at) {
        return String::new();
    }
    let start = line_start(text, at);
    let end = text[at..].find('\n').map_or(text.len(), |n| at + n);
    let line = text[start..end].trim();
    let line = line.split(" //").next().unwrap_or(line);
    let line = line.trim_end_matches('{').trim_end();
    if line.len() > 160 {
        let cut = line.floor_char_boundary(160);
        format!("{} ...", &line[..cut])
    } else {
        line.to_owned()
    }
}


mod forward;
pub use forward::{Resolver, alias_target, docs_site};
mod located;
pub use located::{Located, located_above, opening};
mod items;
pub use items::without_items;

#[cfg(test)]
mod tests;
