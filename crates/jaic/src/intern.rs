//! Global string interner. Symbols are cheap copyable handles compared by id.
use crate::fxhash::HashMap;
use std::cell::RefCell;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sym(u32);

#[derive(Default)]
struct Interner {
    map: HashMap<&'static str, Sym>,
    names: Vec<&'static str>,
    /// Spellings already leaked by this thread's jobs (see `begin_job`), so a spelling met in
    /// many files is leaked once.
    leaked: crate::fxhash::HashSet<&'static str>,
    job: bool,
}

impl Interner {
    /// Under `JAIC_SYM_SCRAMBLE`, skips a few numbers before a new symbol (`knobs::sym_scramble`).
    fn scramble(&mut self) {
        if let Some(seed) = crate::knobs::sym_scramble() {
            let gap = crate::knobs::scramble_gap(seed, self.names.len() as u64, 4);
            self.names.extend((0..gap).map(|_| ""));
        }
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::default());
}

impl Sym {
    pub fn intern(text: &str) -> Sym {
        INTERNER.with(|i| {
            let mut i = i.borrow_mut();
            if let Some(&s) = i.map.get(text) {
                return s;
            }
            // Leaked once per distinct spelling; the set of identifiers is bounded by source size.
            let leaked: &'static str = if i.job {
                if let Some(&known) = i.leaked.get(text) {
                    known
                } else {
                    let new: &'static str = Box::leak(text.to_owned().into_boxed_str());
                    i.leaked.insert(new);
                    new
                }
            } else {
                Box::leak(text.to_owned().into_boxed_str())
            };
            i.scramble();
            let sym = Sym(i.names.len() as u32);
            i.names.push(leaked);
            i.map.insert(leaked, sym);
            sym
        })
    }

    /// Interns a spelling that is already leaked, without copying it again.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn intern_static(text: &'static str) -> Sym {
        INTERNER.with(|i| {
            let mut i = i.borrow_mut();
            if let Some(&s) = i.map.get(text) {
                return s;
            }
            i.scramble();
            let sym = Sym(i.names.len() as u32);
            i.names.push(text);
            i.map.insert(text, sym);
            sym
        })
    }

    pub fn as_str(self) -> &'static str {
        INTERNER.with(|i| i.borrow().names[self.0 as usize])
    }

    pub fn index(self) -> u32 {
        self.0
    }
}

/// Starts a parse job on a worker thread: this thread's symbols restart at zero, so the job's
/// spellings, in first-seen order, are exactly what a serial parse would have interned.
/// `end_job` returns them; `Remap` carries the symbols over to the main interner.
#[cfg(not(target_arch = "wasm32"))]
pub fn begin_job() {
    INTERNER.with(|i| {
        let mut i = i.borrow_mut();
        i.map.clear();
        i.names.clear();
        i.job = true;
    });
}

/// Ends a job started by `begin_job` and returns its spellings by local symbol index.
#[cfg(not(target_arch = "wasm32"))]
pub fn end_job() -> Vec<&'static str> {
    INTERNER.with(|i| {
        let mut i = i.borrow_mut();
        i.map.clear();
        std::mem::take(&mut i.names)
    })
}

/// Maps a job's local symbols to the calling thread's interner.
#[cfg(not(target_arch = "wasm32"))]
pub struct Remap(Vec<Sym>);

#[cfg(not(target_arch = "wasm32"))]
impl Remap {
    /// Interns `spellings` in order, as the serial parse would have.
    pub fn new(spellings: &[&'static str]) -> Remap {
        Remap(spellings.iter().map(|&s| Sym::intern_static(s)).collect())
    }

    pub fn apply(&self, sym: Sym) -> Sym {
        self.0[sym.0 as usize]
    }
}

impl fmt::Debug for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl fmt::Display for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Convenience: `sym!("name")` interns a literal.
#[macro_export]
macro_rules! sym {
    ($s:expr) => {
        $crate::intern::Sym::intern($s)
    };
}
