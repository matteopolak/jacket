//! Which compile-time code read which notes ([note readers](../../../../docs/compiler/note-readers.md)).
//!
//! While `Compiler::ed` records, every compile-time run watches the text of the `Code_Note`s
//! handed to it (`interp::note_watch`). After the run the reads become `NoteReader`s: the
//! note's place, and the reading code's location in the first of its callers that is in a
//! recorded file (so a read inside a library helper lands on the user's call). A note and the
//! code that read it are usually in different workspace compilers, so `share_note_reads`
//! gives every compiler of a check the readers all of them found.
use crate::fxhash::HashSet;
use crate::interp::note_watch::{NotePlace, NoteRead};
use crate::sema::Compiler;
use crate::source::{FileId, Span};
use std::sync::Arc;

/// Compile-time code that read a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteReader {
    pub note: NotePlace,
    /// The reading code's file (as its compiler shows the path), that file's text, and the
    /// byte offset it is at.
    pub path: String,
    pub text: Arc<str>,
    pub offset: u32,
}

impl NoteReader {
    /// What tells two readers apart (the text goes with the path).
    fn key(&self) -> (NotePlace, String, u32) {
        (self.note.clone(), self.path.clone(), self.offset)
    }
}

/// Readers kept per analysis: a metaprogram that walks every note of a big program stays
/// bounded.
const MAX_READERS: usize = 16384;

impl Compiler {
    /// Turn on the note watch of this compiler's interpreter when it records for an editor.
    pub(crate) fn ide_watch_notes(&mut self) {
        if self.ed.is_live() {
            self.ct.interp.watch_notes();
        }
    }

    /// Record the reads the interpreter saw since the last call.
    pub(crate) fn ide_note_reads(&mut self) {
        let Some(watch) = self.ct.interp.note_watch.as_mut() else {
            return;
        };
        let reads = watch.take();
        for read in reads {
            if let Some(reader) = self.note_reader(&read)
                && let Some(ide) = self.ed.facts_mut()
                && ide.note_readers.len() < MAX_READERS
                && ide.note_reader_keys.insert(reader.key())
            {
                ide.note_readers.push(reader);
            }
        }
    }

    /// Where `read` happened: the innermost location in a recorded file, else the innermost.
    fn note_reader(&mut self, read: &NoteRead) -> Option<NoteReader> {
        let files = self.src.sources.len();
        let mut site = None;
        for &loc in read.stack.iter().filter(|l| (l.0 as usize) < files) {
            site.get_or_insert(loc);
            if self.ide_wants(FileId(loc.0)) {
                site = Some(loc);
                break;
            }
        }
        let (file, line, col) = site?;
        let source = self.src.sources.get(FileId(file));
        Some(NoteReader {
            note: read.note.clone(),
            path: source.path.to_string(),
            text: source.text.clone(),
            offset: source.offset_of(line, col),
        })
    }

    /// The readers of notes written in `file`, with where each note is.
    pub fn ide_note_readers_in(&self, file: FileId) -> Vec<(Span, &NoteReader)> {
        let Some(ide) = self.ed.facts() else {
            return Vec::new();
        };
        let source = self.src.sources.get(file);
        if source.path.as_file().is_none() {
            return Vec::new();
        }
        let path = crate::paths::program_path(&source.path.to_string());
        ide.note_readers
            .iter()
            .filter(|r| *r.note.path == *path)
            .map(|r| {
                let span = Span {
                    file,
                    start: source.offset_of(r.note.start.0, r.note.start.1),
                    end: source.offset_of(r.note.end.0, r.note.end.1),
                };
                (span, r)
            })
            .collect()
    }

    /// The compile-time code that read the note at `offset` of `file`.
    pub fn ide_note_readers_at(&self, file: FileId, offset: u32) -> Vec<NoteReader> {
        self.ide_note_readers_in(file)
            .into_iter()
            .filter(|(span, _)| span.start <= offset && offset < span.end)
            .map(|(_, r)| r.clone())
            .collect()
    }
}

/// Give each of `compilers` (those of one check: the root and the workspaces its metaprogram
/// made) the note readers all of them recorded.
pub fn share_note_reads(compilers: &mut [&mut Compiler]) {
    let mut keys = HashSet::default();
    let mut all: Vec<NoteReader> = Vec::new();
    for c in compilers.iter() {
        if let Some(ide) = c.ed.facts() {
            for reader in &ide.note_readers {
                if keys.insert(reader.key()) {
                    all.push(reader.clone());
                }
            }
        }
    }
    for c in compilers.iter_mut() {
        if let Some(ide) = c.ed.facts_mut() {
            ide.note_readers.clone_from(&all);
            ide.note_reader_keys.clone_from(&keys);
        }
    }
}
