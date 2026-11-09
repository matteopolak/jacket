//! What a call frees below its argument, without freeing the argument itself.
//!
//! A procedure that frees `h.name`, `array_reset`s `h.items` or hands `h.storage` to its
//! allocator releases what `h` owns. `h` stays valid and can be filled again, so this is
//! kept apart from a whole-argument release (`free(p)`), which ends `p` itself. The body says
//! it: no declaration note is needed.
use super::{Kind, Path, Root, Step, Summary, descriptor_storage};
use jaic::intern::Sym;

/// Storage owned through one parameter that every normal return has freed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Contents {
    /// The members of the argument that hold the freed storage, each as the chain of member
    /// names down to the freed pointer (`[bar, data]` for `free(a.bar.data)`), or `None` when
    /// the freed storage is reached some other way (through a pointer argument, an element).
    pub fields: Option<Vec<Vec<Sym>>>,
    /// The call also writes over every place it freed, so the argument is usable again.
    pub reset: bool,
}

impl Summary {
    /// The storage below parameter `formal` that every normal return frees, if any.
    ///
    /// Only definite frees count: a call that frees on some paths, or only a temporary of its
    /// own, releases nothing the caller can rely on. A whole argument or a descriptor's backing
    /// storage is not "contents"; those are the plain releases.
    ///
    /// ```jai
    /// clear :: (r: *Registry) { free(r.label); r.label = ""; }  // contents: [label], reset
    /// drop  :: (r: *Registry) { free(r.label); }               // contents: [label]
    /// ```
    pub(in crate::rules) fn released_contents(&self, formal: usize) -> Option<Contents> {
        let freed: Vec<&Path> = self
            .guaranteed
            .keys()
            .filter(|e| e.kind == Kind::Free && e.guard.is_none())
            .map(|e| &e.target)
            .filter(|p| {
                p.root == Root::Parameter(formal)
                    && !p.steps.is_empty()
                    && !descriptor_storage(&p.steps)
            })
            .collect();
        if freed.is_empty() {
            return None;
        }
        let mut fields = Some(Vec::new());
        for path in &freed {
            let chain: Vec<Sym> = path
                .steps
                .iter()
                .map_while(|step| match step {
                    Step::Field(name) => Some(*name),
                    _ => None,
                })
                .collect();
            match &mut fields {
                Some(chains) if !chain.is_empty() => {
                    if !chains.contains(&chain) {
                        chains.push(chain);
                    }
                }
                _ => fields = None,
            }
        }
        let written: Vec<&Path> = self
            .may
            .keys()
            .filter(|e| e.kind == Kind::Write && e.guard.is_none())
            .map(|e| &e.target)
            .filter(|p| p.root == Root::Parameter(formal))
            .collect();
        let reset = freed
            .iter()
            .all(|path| written.iter().any(|w| path.steps.starts_with(&w.steps)));
        Some(Contents {
            fields,
            reset,
        })
    }

    /// The parameters whose contents are released, with what is known about each.
    pub(in crate::rules) fn released_contents_all(&self) -> Vec<(usize, Contents)> {
        let mut formals: Vec<usize> = self
            .guaranteed
            .keys()
            .filter_map(|e| match e.target.root {
                Root::Parameter(n) if e.kind == Kind::Free => Some(n),
                _ => None,
            })
            .collect();
        formals.sort_unstable();
        formals.dedup();
        formals
            .into_iter()
            .filter_map(|n| Some((n, self.released_contents(n)?)))
            .collect()
    }
}
