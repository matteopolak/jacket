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
            let leaked: &'static str = Box::leak(text.to_owned().into_boxed_str());
            let sym = Sym(i.names.len() as u32);
            i.names.push(leaked);
            i.map.insert(leaked, sym);
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
