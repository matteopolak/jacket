//! Moving offsets between the text a compile saw and the text of the document now.

/// How one text became another: they share a prefix and a suffix, and what lies between was
/// edited. Offsets outside the edited stretch move by the change in length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shift {
    prefix: usize,
    old_tail: usize,
    new_tail: usize,
}

impl Default for Shift {
    fn default() -> Self {
        Self::none()
    }
}

impl Shift {
    /// The text is as it was.
    pub const fn none() -> Self {
        Self {
            prefix: usize::MAX,
            old_tail: 0,
            new_tail: 0,
        }
    }

    /// The longest common prefix and suffix of `old` and `new`.
    pub fn between(old: &[u8], new: &[u8]) -> Self {
        if old == new {
            return Self::none();
        }
        let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
        let room = old.len().min(new.len()) - prefix;
        let suffix = old
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take(room)
            .take_while(|(a, b)| a == b)
            .count();
        Self {
            prefix,
            old_tail: old.len() - suffix,
            new_tail: new.len() - suffix,
        }
    }

    pub fn is_none(&self) -> bool {
        self.prefix == usize::MAX
    }

    /// Where `new` (an offset in the current text) was in the old one, `None` inside the edit.
    pub fn to_old(self, new: usize) -> Option<usize> {
        if self.is_none() || new <= self.prefix {
            Some(new)
        } else if new >= self.new_tail {
            Some(new - self.new_tail + self.old_tail)
        } else {
            None
        }
    }

    /// Like `to_old`, an offset inside the edit becoming the start of it.
    pub fn to_old_or_edit(self, new: usize) -> usize {
        self.to_old(new).unwrap_or(self.prefix)
    }

    /// Where `old` (an offset in the old text) is now, `None` inside what was edited away.
    pub fn to_new(self, old: usize) -> Option<usize> {
        if self.is_none() || old < self.prefix {
            Some(old)
        } else if old >= self.old_tail {
            Some(old - self.old_tail + self.new_tail)
        } else {
            None
        }
    }

    /// Move a complete unchanged span. Its end stays before an insertion at that boundary;
    /// its start follows an insertion before the span. Spans crossing an edit are omitted.
    pub fn to_new_range(self, start: usize, end: usize) -> Option<(usize, usize)> {
        if self.is_none() || end <= self.prefix {
            Some((start, end))
        } else if start >= self.old_tail {
            Some((
                start - self.old_tail + self.new_tail,
                end - self.old_tail + self.new_tail,
            ))
        } else {
            None
        }
    }

    /// The unchanged current span's coordinates in the checked text.
    pub fn to_old_range(self, start: usize, end: usize) -> Option<(usize, usize)> {
        Self {
            old_tail: self.new_tail,
            new_tail: self.old_tail,
            ..self
        }
        .to_new_range(start, end)
    }

    /// Like `to_new`, an offset inside the edit staying where it is.
    pub fn to_new_or_same(self, old: usize) -> usize {
        self.to_new(old).unwrap_or(old)
    }
}

#[cfg(test)]
mod tests {
    use super::Shift;

    #[test]
    fn offsets_move_past_the_edit() {
        let shift = Shift::between(b"abc def", b"abc\nx def");
        assert_eq!(shift.to_old(2), Some(2));
        assert_eq!(shift.to_old(7), Some(5));
        assert_eq!(shift.to_old(4), None);
        assert_eq!(shift.to_old_or_edit(4), 3);
        assert_eq!(shift.to_new(5), Some(7));
        assert_eq!(shift.to_new(0), Some(0));
        // What starts where text was inserted moves with it.
        assert_eq!(shift.to_new(3), Some(5));
    }

    #[test]
    fn the_same_text_moves_nothing() {
        let shift = Shift::between(b"abc", b"abc");
        assert!(shift.is_none());
        assert_eq!(shift.to_old(3), Some(3));
        assert_eq!(shift.to_new(3), Some(3));
    }
}
