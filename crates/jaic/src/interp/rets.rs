//! Return-value storage for a call.

/// A call's results: up to four inline. A `Vec` per call was a malloc and free on every call.
#[derive(Debug, Default)]
pub struct Rets {
    len: usize,
    inline: [u64; 4],
    spill: Vec<u64>,
}

impl Rets {
    pub(super) fn collect(values: impl ExactSizeIterator<Item = u64>) -> Self {
        let mut rets = Rets {
            len: values.len(),
            ..Default::default()
        };
        if rets.len <= 4 {
            for (slot, v) in rets.inline.iter_mut().zip(values) {
                *slot = v;
            }
        } else {
            rets.spill = values.collect();
        }
        rets
    }

    pub fn into_vec(self) -> Vec<u64> {
        if self.len <= 4 {
            self.inline[..self.len].to_vec()
        } else {
            self.spill
        }
    }
}

impl From<Vec<u64>> for Rets {
    fn from(values: Vec<u64>) -> Self {
        if values.len() <= 4 {
            return Rets::collect(values.into_iter());
        }
        Rets {
            len: values.len(),
            inline: [0; 4],
            spill: values,
        }
    }
}

impl Rets {
    /// A single result.
    pub fn one(value: u64) -> Self {
        Rets {
            len: 1,
            inline: [value, 0, 0, 0],
            spill: Vec::new(),
        }
    }
}

impl std::ops::Deref for Rets {
    type Target = [u64];

    fn deref(&self) -> &[u64] {
        if self.len <= 4 {
            &self.inline[..self.len]
        } else {
            &self.spill
        }
    }
}
