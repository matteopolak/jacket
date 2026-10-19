//! A language-server session over a small workspace of up to three files that load one another
//! (`main.jai`, `a.jai`, `b.jai`), driven by a script of edits in any file and cross-file renames,
//! checked against oracles that need no expected answer:
//!
//! - a rename of a name in an error-free workspace, applied to every file it touches, leaves the
//!   workspace error-free, and renaming back restores every file byte for byte;
//! - after the edits, the diagnostics of every file equal those of a fresh session opened on the
//!   same texts (no answer depends on the order the edits arrived in).
//!
//! The input is up to three NUL-separated files, then optionally a NUL and the script's bytes;
//! without the script (a plain seed) it comes from a hash of the files. A failed check panics.
use crate::jailint_fix::{apply, offset};
use arbitrary::{Result, Unstructured};
use jai_language_server::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Limits, Range, Session, TextChange,
};

const NAMES: [&str; 3] = ["main.jai", "a.jai", "b.jai"];
/// Edits, renames and checks per input; each request compiles the workspace again.
const MAX_STEPS: usize = 10;
/// The name a rename gives; no program of the fuzzer's uses it.
/// Rename round trips tried before the first edit.
const RENAMES: usize = 3;
const FRESH: &str = "fzrn_0";

fn uri(doc: usize) -> DocumentUri {
    DocumentUri::parse(&format!("file:///workspace/{}", NAMES[doc])).expect("valid uri")
}

struct Workspace {
    session: Session,
    docs: usize,
    versions: [i32; 3],
}

impl Workspace {
    fn text(&self, doc: usize) -> String {
        self.session
            .document_text(&uri(doc))
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn errors(&self) -> Vec<Diagnostic> {
        (0..self.docs)
            .flat_map(|d| self.session.diagnostics(&uri(d)).unwrap_or_default())
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect()
    }

    fn replace(&mut self, doc: usize, range: Option<Range>, text: String) {
        let version = self.versions[doc] + 1;
        let change = TextChange {
            range,
            range_length: None,
            text,
        };
        if self.session.change(&uri(doc), version, &[change]).is_ok() {
            self.versions[doc] = version;
        }
    }

    /// Changes `doc` by `edits`, each as one incremental change from the end of the text
    /// backwards so the positions stay valid.
    fn apply_edits(&mut self, doc: usize, edits: &[jai_language_server::TextEdit]) {
        let text = self.text(doc);
        self.replace(doc, None, apply(&text, edits));
    }

    /// Renames the identifier at `at` in `doc` to `new`; the document and byte offset of the same
    /// name afterwards, or `None` when it cannot be renamed.
    fn rename(&mut self, doc: usize, at: usize, new: &str) -> Option<(usize, usize)> {
        let text = self.text(doc);
        let position = crate::position(&text, at);
        let range = self.session.prepare_rename(&uri(doc), position).ok()??;
        let edits = self.session.rename(&uri(doc), position, new).ok()??;
        let start = offset(&text, range.start);
        let mut shift = 0isize;
        for (target, list) in &edits {
            let Some(d) = (0..self.docs).find(|&d| uri(d).as_str() == target) else {
                continue;
            };
            if d == doc {
                let before = self.text(d);
                for e in list {
                    let (a, b) = (offset(&before, e.range.start), offset(&before, e.range.end));
                    if a < start {
                        shift += e.new_text.len() as isize - (b.max(a) - a) as isize;
                    }
                }
            }
            self.apply_edits(d, list);
        }
        Some((doc, (start as isize + shift) as usize))
    }
}

/// Byte offsets of identifier starts in `text`.
fn identifiers(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut previous_word = false;
    for (i, c) in text.char_indices() {
        let word = c.is_alphanumeric() || c == '_';
        if word && !previous_word && !c.is_ascii_digit() {
            found.push(i);
        }
        previous_word = word;
    }
    found
}

fn rename_round_trip(w: &mut Workspace, u: &mut Unstructured) -> Result<()> {
    if !w.errors().is_empty() {
        return Ok(());
    }
    let doc = u.int_in_range(0..=w.docs - 1)?;
    let ids = identifiers(&w.text(doc));
    if ids.is_empty() {
        return Ok(());
    }
    let at = *u.choose(&ids)?;
    let before: Vec<String> = (0..w.docs).map(|d| w.text(d)).collect();
    let name_end = before[doc][at..]
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map_or(before[doc].len(), |n| at + n);
    let old = before[doc][at..name_end].to_string();
    let Some((doc2, at2)) = w.rename(doc, at, FRESH) else {
        return Ok(());
    };
    let after: Vec<String> = (0..w.docs).map(|d| w.text(d)).collect();
    if std::env::var_os("JAI_FUZZ_VERBOSE").is_some() {
        eprintln!("renamed `{old}` in {doc}");
    }
    let errors = w.errors();
    assert!(
        errors.is_empty(),
        "renaming `{old}` at {doc}:{at} to `{FRESH}` breaks the workspace: {errors:?}\n{before:#?}\n{after:#?}"
    );
    let Some(_) = w.rename(doc2, at2, &old) else {
        panic!("`{FRESH}` cannot be renamed back to `{old}`\n{before:#?}\n{after:#?}");
    };
    let restored: Vec<String> = (0..w.docs).map(|d| w.text(d)).collect();
    assert!(
        restored == before,
        "renaming `{old}` and back changes the files:\n{before:#?}\n{after:#?}\n{restored:#?}"
    );
    Ok(())
}

fn edit(w: &mut Workspace, u: &mut Unstructured) -> Result<()> {
    let doc = u.int_in_range(0..=w.docs - 1)?;
    let text = w.text(doc);
    let char_start = |u: &mut Unstructured| -> Result<usize> {
        let mut at = u.int_in_range(0..=text.len())?;
        while !text.is_char_boundary(at) {
            at -= 1;
        }
        Ok(at)
    };
    let start = char_start(u)?;
    let mut end = (start + u.int_in_range(0..=6)?).min(text.len());
    while !text.is_char_boundary(end) {
        end += 1;
    }
    let new = if u.ratio(1, 3)? {
        String::new()
    } else {
        crate::lsp_edits::snippet(u)?
    };
    let range = Range {
        start: crate::position(&text, start),
        end: crate::position(&text, end),
    };
    w.replace(doc, Some(range), new);
    Ok(())
}

fn fresh_diagnostics(texts: &[String]) -> Vec<Vec<Diagnostic>> {
    let mut session = Session::with_environment(Limits::default(), jai_wasm::lsp_environment());
    for (d, text) in texts.iter().enumerate() {
        let _ = session.open(uri(d), 1, text.clone());
    }
    (0..texts.len())
        .map(|d| session.diagnostics(&uri(d)).unwrap_or_default())
        .collect()
}

fn run(files: &[String], ops: &[u8]) {
    let mut session = Session::with_environment(Limits::default(), jai_wasm::lsp_environment());
    for (d, text) in files.iter().enumerate() {
        if session.open(uri(d), 1, text.clone()).is_err() {
            return;
        }
    }
    let mut w = Workspace {
        session,
        docs: files.len(),
        versions: [1; 3],
    };
    let mut u = Unstructured::new(ops);
    for index in 0..MAX_STEPS {
        // Renames first, while the workspace still compiles; the edits that follow break it.
        let step = match u.int_in_range(0..=9) {
            Ok(_) if index < RENAMES => 6,
            Ok(step) => step,
            Err(_) => break,
        };
        if std::env::var_os("JAI_FUZZ_VERBOSE").is_some() {
            eprintln!("step {index}: {step}");
        }
        let done = match step {
            0..=5 => edit(&mut w, &mut u),
            6..=8 => rename_round_trip(&mut w, &mut u),
            _ => {
                let _ = w.session.diagnostics(&uri(0));
                Ok(())
            }
        };
        if std::env::var_os("JAI_FUZZ_VERBOSE").is_some() {
            eprintln!("--- after step {index}:\n{}", w.text(0));
        }
        if done.is_err() {
            break;
        }
    }
    let texts: Vec<String> = (0..w.docs).map(|d| w.text(d)).collect();
    let fresh = fresh_diagnostics(&texts);
    let incremental: Vec<Vec<Diagnostic>> = (0..w.docs)
        .map(|d| w.session.diagnostics(&uri(d)).unwrap_or_default())
        .collect();
    // A literal or comment that stopped closing is left out of the analysis on purpose
    // ("Incomplete lexical edit"), which a fresh session cannot know.
    if incremental
        .iter()
        .chain(&fresh)
        .flatten()
        .any(|d| d.code == DiagnosticCode::Lexer)
    {
        return;
    }
    for (d, expected) in fresh.iter().enumerate() {
        let got = &incremental[d];
        assert!(
            got == expected,
            "the diagnostics of {} differ from a fresh session's:\n{got:#?}\n{expected:#?}\n{texts:#?}",
            NAMES[d]
        );
    }
}

pub fn lsp_workspace(data: &[u8]) {
    let mut parts: Vec<&[u8]> = data.split(|&b| b == 0).collect();
    let script = if parts.len() > 3 {
        let rest = parts.split_off(3);
        rest.join(&0u8)
    } else {
        // A plain seed: derive the script from the files (splitmix64 over a hash).
        let mut state = data.iter().fold(0x9e37_79b9_7f4a_7c15u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        });
        (0..128)
            .flat_map(|_| {
                state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = state;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                (z ^ (z >> 31)).to_le_bytes()
            })
            .collect()
    };
    let files: Vec<String> = parts
        .iter()
        .map(|p| String::from_utf8_lossy(p).replace('\r', ""))
        .collect();
    // Compile-time code runs again on every edit, up to the interpreter's budget.
    if files
        .iter()
        .any(|f| f.contains("#run") || f.contains("#insert"))
    {
        return;
    }
    crate::on_compiler_stack(|| run(&files, &script));
}
