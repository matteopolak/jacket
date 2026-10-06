//! A language-server session driven by a random script of edits and requests, as an editor would
//! send them while someone types: incremental and whole-document changes (also with positions past
//! the end, inside surrogate pairs, or reversed), every position-based request at random places,
//! a second document, closing and reopening. Errors are fine; panics are not.
//!
//! The input is a document, then optionally a NUL byte and the script's bytes. Without a NUL (a
//! plain Jai seed) the script comes from a hash of the document, so every seed still drives edits.
use arbitrary::{Result, Unstructured};
use jai_language_server::{DocumentUri, Limits, Position, Range, Session, TextChange};

/// Requests and edits per input. Each request that needs types compiles the document (cached per
/// version), so this stays small to keep libFuzzer's executions per second up.
const MAX_STEPS: usize = 24;

/// Text an edit inserts: tokens that open or close constructs, declarations, and characters whose
/// UTF-8 and UTF-16 lengths differ.
const SNIPPETS: &[&str] = &[
    "x",
    "x.",
    ".",
    "(",
    ")",
    "{",
    "}",
    "[",
    "]",
    ";",
    "\n",
    "\r\n",
    " ",
    "\t",
    ":",
    "::",
    ":=",
    "=",
    "\"",
    "\\",
    "/*",
    "*/",
    "//",
    "#",
    "#run ",
    "#import \"Basic\";\n",
    "#load \"b.jai\";\n",
    "#string END\n",
    "END\n",
    "main :: () {\n}\n",
    "S :: struct { a: int; }\n",
    "f :: (a: int) -> int { return a; }\n",
    "E :: enum { A; B; }\n",
    "using ",
    "$T",
    "..",
    "->",
    "if x == {\ncase 1;\n}\n",
    "for 0..3 ",
    "print(\"%\", 1);",
    "s.a",
    "f(",
    "f(1, ",
    "#insert \"x := 1;\";",
    "é",
    "\u{a0}",
    "𝄞",
    "\u{feff}",
    "\0",
];

/// New names for rename requests, valid and not.
const NAMES: &[&str] = &["y", "renamed", "_", "1x", "", "a b", "if", "é", "x\n"];

fn snippet(u: &mut Unstructured) -> Result<String> {
    if u.ratio(1, 8)? {
        // Raw text from the script, lossily decoded.
        let len = u.int_in_range(0..=16)?;
        let bytes = u.bytes(len.min(u.len()))?;
        return Ok(String::from_utf8_lossy(bytes).into_owned());
    }
    let mut out = String::new();
    for _ in 0..u.int_in_range(1..=3)? {
        out.push_str(u.choose(SNIPPETS)?);
    }
    Ok(out)
}

/// A position: usually on an existing line (any character offset, including past the end and
/// inside a surrogate pair), sometimes past the last line or at `u32::MAX`.
fn position(u: &mut Unstructured, text: &str) -> Result<Position> {
    let lines: Vec<&str> = text.split('\n').collect();
    Ok(match u.int_in_range(0..=15)? {
        0 => Position {
            line: lines.len() as u32 + u.int_in_range(0..=2)?,
            character: u.int_in_range(0..=4)?,
        },
        1 => Position {
            line: u.int_in_range(0..=lines.len() as u32)?,
            character: u32::MAX,
        },
        _ => {
            let line = u.int_in_range(0..=lines.len().saturating_sub(1))?;
            let units = lines[line].encode_utf16().count() as u32;
            Position {
                line: line as u32,
                character: u.int_in_range(0..=units + 2)?,
            }
        }
    })
}

fn range(u: &mut Unstructured, text: &str) -> Result<Range> {
    let (a, b) = (position(u, text)?, position(u, text)?);
    // Mostly ordered; a reversed range is a client bug the server must survive.
    Ok(if a <= b || u.ratio(1, 6)? {
        Range {
            start: a,
            end: b,
        }
    } else {
        Range {
            start: b,
            end: a,
        }
    })
}

struct Script<'a> {
    session: Session,
    uris: [DocumentUri; 2],
    versions: [i32; 2],
    u: Unstructured<'a>,
}

impl Script<'_> {
    fn text(&self, doc: usize) -> String {
        self.session
            .document_text(&self.uris[doc])
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn step(&mut self) -> Result<()> {
        let doc = usize::from(self.u.ratio(1, 5)?);
        let uri = self.uris[doc].clone();
        let text = self.text(doc);
        let u = &mut self.u;
        let s = &mut self.session;
        match u.int_in_range(0..=21)? {
            0..=4 => {
                // One to three incremental changes in one notification; positions refer to the text
                // as changed by the earlier ones, which the server applies in order.
                let mut changes = Vec::new();
                for _ in 0..u.int_in_range(1..=3)? {
                    changes.push(TextChange {
                        range: Some(range(u, &text)?),
                        range_length: None,
                        text: if u.ratio(1, 3)? {
                            String::new()
                        } else {
                            snippet(u)?
                        },
                    });
                }
                // Now and then a version that does not increase.
                let version = if u.ratio(1, 10)? {
                    self.versions[doc]
                } else {
                    self.versions[doc] + 1
                };
                if s.change(&uri, version, &changes).is_ok() {
                    self.versions[doc] = version;
                }
            }
            5 => {
                // Whole-document replacement: the current text with a cut and an insertion.
                let mut next = text.clone();
                let at = u.int_in_range(0..=next.len())?;
                let at = (0..=at)
                    .rev()
                    .find(|&i| next.is_char_boundary(i))
                    .unwrap_or(0);
                next.truncate(at);
                next.push_str(&snippet(u)?);
                next.push_str(&text[at..]);
                let change = TextChange {
                    range: None,
                    range_length: None,
                    text: next,
                };
                if s.change(&uri, self.versions[doc] + 1, &[change]).is_ok() {
                    self.versions[doc] += 1;
                }
            }
            6 => {
                // Close; the next requests on it fail until it is opened again.
                let _ = s.close(&uri);
                if u.ratio(1, 2)? {
                    self.versions[doc] += 1;
                    let _ = s.open(uri, self.versions[doc], text);
                }
            }
            7 => {
                let p = position(u, &text)?;
                let _ = s.hover(&uri, p);
                let _ = s.definition(&uri, p);
            }
            8 => {
                let _ = s.completion(&uri, position(u, &text)?);
            }
            9 => {
                let p = position(u, &text)?;
                let _ = s.references(&uri, p, u.arbitrary()?);
                let _ = s.document_highlights(&uri, p);
            }
            10 => {
                let p = position(u, &text)?;
                let _ = s.prepare_rename(&uri, p);
                let _ = s.rename(&uri, p, u.choose(NAMES)?);
            }
            11 => {
                let p = position(u, &text)?;
                let _ = s.signature_help(&uri, p);
                let _ = s.type_definition(&uri, p);
            }
            12 => {
                let p = position(u, &text)?;
                let _ = s.expansion(&uri, p);
                let _ = s.polymorphs(&uri, p);
            }
            13 => {
                let r = range(u, &text)?;
                let _ = s.code_actions(&uri, r);
                let _ = s.inlay_hints(&uri, r);
            }
            14 => {
                let _ = s.diagnostics(&uri);
                let _ = s.document_symbols(&uri);
            }
            15 => {
                let _ = s.semantic_tokens(&uri);
                let _ = s.folding_ranges(&uri);
            }
            16 => {
                let _ = s.code_lenses(&uri);
                let _ = s.document_links(&uri);
            }
            17 => {
                let len = u.int_in_range(0..=3)?;
                let query: String = text
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .take(len)
                    .collect();
                let _ = s.workspace_symbols(&query);
            }
            18 => {
                let _ = s.publications();
            }
            _ => {
                let p = position(u, &text)?;
                let _ = s.hover(&uri, p);
            }
        }
        Ok(())
    }
}

/// The script interpreter; see the module docs for the input format.
pub fn lsp_edits(data: &[u8]) {
    let (doc, ops) = match data.iter().position(|&b| b == 0) {
        Some(at) => (&data[..at], data[at + 1..].to_vec()),
        None => {
            // A plain document: derive the script from it (splitmix64 over a hash).
            let mut state = data.iter().fold(0x9e37_79b9_7f4a_7c15u64, |h, &b| {
                (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
            });
            let bytes = (0..256)
                .flat_map(|_| {
                    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
                    let mut z = state;
                    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                    (z ^ (z >> 31)).to_le_bytes()
                })
                .collect();
            (data, bytes)
        }
    };
    let text = String::from_utf8_lossy(doc).into_owned();
    crate::on_compiler_stack(|| {
        let mut session = Session::with_environment(Limits::default(), jai_wasm::lsp_environment());
        let uris = [
            DocumentUri::parse("file:///workspace/main.jai").expect("valid uri"),
            DocumentUri::parse("file:///workspace/b.jai").expect("valid uri"),
        ];
        if session.open(uris[0].clone(), 1, text.clone()).is_err() {
            return;
        }
        // The second document is the first half of the first one.
        let half = (0..=text.len() / 2)
            .rev()
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(0);
        let _ = session.open(uris[1].clone(), 1, text[..half].to_string());
        let mut script = Script {
            session,
            uris,
            versions: [1, 1],
            u: Unstructured::new(&ops),
        };
        for _ in 0..MAX_STEPS {
            if script.u.is_empty() || script.step().is_err() {
                break;
            }
        }
    });
}
