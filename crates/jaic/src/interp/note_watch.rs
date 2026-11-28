//! Which notes compile-time code reads, for the editor ([note readers](../../../../docs/compiler/note-readers.md)).
//!
//! The record builder hands a metaprogram each `Code_Note` with its text in bytes of its own.
//! While a watch is on, the builder registers those bytes here, and the interpreter reports
//! an access that touches them: a load (in the dispatch loop that watches blocks), a copy, and
//! a `memcmp` that found them equal. A comparison that differs, or that a length check skipped,
//! is not a read: the code was looking for another note. Accesses by foreign code are not seen.
//!
//! Off (`Interp::note_watch` is `None`) outside the language server and linter, so a batch
//! compile pays nothing.
use crate::fxhash::HashSet;
use std::sync::Arc;

/// Where a note is written: its file as the program spells it (`paths::program_path`) and the
/// one-based line and byte column of its start and end. The same place in every workspace
/// compiler, so a note one compiler exported can be found in another that read it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NotePlace {
    pub path: Arc<str>,
    pub start: (u32, u32),
    pub end: (u32, u32),
}

/// A location the interpreter runs at: file number, line, column.
pub type Loc = (u32, u32, u32);

/// One read: the note, and where the reading code ran, innermost procedure first.
#[derive(Clone, Debug)]
pub struct NoteRead {
    pub note: NotePlace,
    pub stack: Vec<Loc>,
}

/// How many callers of the reading code a read keeps: enough to get out of library helpers.
const STACK: usize = 8;

#[derive(Default)]
pub struct NoteWatch {
    /// Text bytes of each note handed to the program, `(start, end, index into places)`,
    /// sorted by start. Each note's text is its own allocation, so they do not overlap.
    ranges: Vec<(u64, u64, u32)>,
    places: Vec<NotePlace>,
    /// Lowest start and highest end of `ranges`: most accesses are outside.
    lo: u64,
    hi: u64,
    /// Reads not yet taken (`take`).
    reads: Vec<NoteRead>,
    /// `(note, innermost location)` pairs already read: a loop reads each byte once per run.
    seen: HashSet<(u32, Option<Loc>)>,
}

impl NoteWatch {
    /// The `len` bytes at `start` are the text of the note at `place`.
    pub fn add(&mut self, start: u64, len: u64, place: NotePlace) {
        if len == 0 {
            return;
        }

        let end = start + len;
        let at = self.ranges.partition_point(|r| r.0 < start);

        if self.ranges.get(at).is_some_and(|r| r.0 == start) {
            return;
        }

        let index = self.places.len() as u32;

        self.places.push(place);
        self.ranges.insert(at, (start, end, index));

        if self.ranges.len() == 1 {
            (self.lo, self.hi) = (start, end);
        } else {
            self.lo = self.lo.min(start);
            self.hi = self.hi.max(end);
        }
    }

    /// Whether `addr..addr + len` may touch a note: the cheap test before `touched`.
    #[inline(always)]
    pub fn near(&self, addr: u64, len: u64) -> bool {
        addr < self.hi && addr.wrapping_add(len) > self.lo
    }

    /// The notes `addr..addr + len` touches.
    fn touched(&self, addr: u64, len: u64) -> impl Iterator<Item = u32> + '_ {
        let end = addr.saturating_add(len);
        // The first range that ends after `addr`: ranges are disjoint and sorted, so their
        // ends are sorted too.
        let from = self.ranges.partition_point(|r| r.1 <= addr);

        self.ranges[from..]
            .iter()
            .take_while(move |r| r.0 < end)
            .map(|r| r.2)
    }

    /// Code running at `stack` (innermost first) read `addr..addr + len`.
    pub(super) fn read(&mut self, addr: u64, len: u64, stack: &[Loc]) {
        let notes: Vec<u32> = self.touched(addr, len).collect();

        for note in notes {
            if self.seen.insert((note, stack.first().copied())) {
                self.reads.push(NoteRead {
                    note: self.places[note as usize].clone(),
                    stack: stack.iter().take(STACK).copied().collect(),
                });
            }
        }
    }

    /// The reads since the last call.
    pub fn take(&mut self) -> Vec<NoteRead> {
        std::mem::take(&mut self.reads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(line: u32) -> NotePlace {
        NotePlace {
            path: "/a.jai".into(),
            start: (line, 1),
            end: (line, 5),
        }
    }

    #[test]
    fn an_access_reads_the_notes_it_overlaps_once_per_site() {
        let mut w = NoteWatch::default();

        w.add(200, 4, place(2));
        w.add(100, 4, place(1));
        assert!(!w.near(50, 10) && !w.near(204, 1) && w.near(103, 1));
        w.read(103, 100, &[(0, 9, 1)]);
        w.read(101, 1, &[(0, 9, 1)]);
        w.read(150, 4, &[(0, 9, 1)]);
        let lines: Vec<u32> = w.take().iter().map(|r| r.note.start.0).collect();

        assert_eq!(lines, [1, 2]);
        w.read(200, 1, &[(0, 10, 1)]);
        assert_eq!(w.take().len(), 1);
    }
}
