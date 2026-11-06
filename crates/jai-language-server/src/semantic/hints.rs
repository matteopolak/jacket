//! Query-local render observations, accepted separately from compiler resolution inputs.
use super::{Analysis, Buffers, CheckedInputs, Reads};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
pub(super) struct HintCapture {
    depth: Cell<usize>,
    proof: RefCell<Option<(Reads, Buffers)>>,
    epoch: RefCell<Option<Rc<super::input_epoch::InputEpochFs>>>,
    unavailable: Cell<bool>,
}

pub(crate) struct HintQuery(Rc<HintCapture>);

impl Drop for HintQuery {
    fn drop(&mut self) {
        let depth = self.0.depth.get() - 1;
        self.0.depth.set(depth);
        if depth == 0 {
            *self.0.proof.borrow_mut() = None;
            if let Some(epoch) = self.0.epoch.borrow_mut().take() {
                epoch.leave_hint_query();
            }
        }
    }
}

impl HintCapture {
    /// Nested stages share one proof; the outer guard clears it even during unwinding.
    pub(super) fn begin(self: &Rc<Self>) -> HintQuery {
        if self.depth.get() == 0 {
            *self.proof.borrow_mut() = None;
            *self.epoch.borrow_mut() = None;
            self.unavailable.set(false);
        }
        self.depth.set(self.depth.get() + 1);
        HintQuery(self.clone())
    }

    pub(super) fn observer(&self, analysis: &Analysis) -> Option<super::overlay::OverlayFs> {
        let mut observer = analysis.overlay.separate_observer();
        if let Some(owner) = &analysis.input_epoch {
            let Some(parent) = owner.upgrade() else {
                self.unavailable.set(true);
                return None;
            };
            let existing = self.epoch.borrow().clone();
            let fork = existing.or_else(|| parent.hint_fork());
            let Some(fork) = fork.filter(|fork| fork.hint_query_available()) else {
                self.unavailable.set(true);
                return None;
            };
            observer.base = fork.clone();
            if self.epoch.borrow().is_none() {
                fork.enter_hint_query();
                *self.epoch.borrow_mut() = Some(fork);
            }
        }
        Some(observer)
    }

    /// Preserve every batch's first responses against the query's original overlay view.
    pub(super) fn add(&self, analysis: &Analysis, reads: &Reads) {
        if reads.is_empty() {
            return;
        }
        let mut proof = self.proof.borrow_mut();
        let (all, _) =
            proof.get_or_insert_with(|| (Reads::default(), Rc::new(analysis.files.clone())));
        all.merge(reads);
    }

    /// Validate derived output without changing or retaining compiler input authority.
    pub(super) fn matches(&self, analysis: &Analysis, fs: &dyn jaic::sema::FileSystem) -> bool {
        if self.unavailable.get() {
            return false;
        }
        let epoch = self.epoch.borrow();
        if epoch
            .as_ref()
            .is_some_and(|fork| !fork.accept_hint_inputs())
        {
            return false;
        }
        self.proof.borrow().as_ref().is_none_or(|(reads, files)| {
            // The output combines compiled facts and render-time reads. Their first
            // responses must agree even if separate replays could each succeed.
            if !analysis.reads.borrow().disk.agrees_with(&reads.disk) {
                return false;
            }
            if let Some(fork) = epoch.as_ref() {
                CheckedInputs::capture_hint(analysis, Rc::new(reads.clone()), files, fork)
                    .matches_selected(analysis, &**fork)
            } else {
                CheckedInputs::capture_with_files(analysis, Rc::new(reads.clone()), files)
                    .matches_selected(analysis, fs)
            }
        })
    }
}

impl Analysis {
    /// Hold `let _hints = analysis.hint_query()` across callback and final acceptance.
    pub(crate) fn hint_query(&self) -> HintQuery {
        self.hints.begin()
    }

    /// The separate render proof, after semantic proof validation at this query boundary.
    pub(crate) fn hints_current(&self, fs: &dyn jaic::sema::FileSystem) -> bool {
        self.hints.matches(self, fs)
    }
}
