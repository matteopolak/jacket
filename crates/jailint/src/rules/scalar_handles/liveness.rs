use super::{Expr, HandleEffect, HashSet, NativeKind, Site, State, Token};

const MAX_FRAMES: usize = 16;

impl State {
    fn tokens(&self) -> Vec<Token> {
        let mut out = HashSet::default();
        for x in self
            .values
            .values()
            .chain(self.slots.values())
            .chain(self.returns.iter().map(|(_, _, x)| x))
            .chain(self.probes.values())
        {
            if let Expr::Fresh(t) = x {
                out.insert(t.clone());
            }
        }
        out.into_iter().collect()
    }

    fn work(&mut self, count: usize) -> bool {
        let Some(work) = self.liveness_work.checked_add(count) else {
            self.liveness_exhausted = true;
            return false;
        };
        self.liveness_work = work;
        if work > 65536 || jaic::interrupt::should_stop() {
            self.liveness_exhausted = true;
            return false;
        }
        true
    }

    pub(super) fn frame_for(&mut self, token: Token, site: Site) {
        if self.invalid.contains(&token) {
            return;
        }
        let frames = self.frames.entry(token.clone()).or_default();
        if !frames.contains(&site) {
            if frames.len() < MAX_FRAMES {
                frames.push(site);
            } else {
                self.invalid.insert(token);
            }
        }
    }

    /// Private scalar storage preserves bits, not an OS object's global lifetime.
    pub(super) fn global(&mut self, site: Site) {
        if !self.global_sites.contains(&site) {
            if self.global_sites.len() < MAX_FRAMES {
                self.global_sites.push(site.clone());
            } else {
                self.global_unknown = true;
            }
        }
        let tokens = self.tokens();
        if !self.work(tokens.len()) {
            return;
        }
        for token in tokens {
            self.frame_for(token, site.clone());
        }
    }

    pub(super) fn consume_kind(&mut self, kind: NativeKind) {
        let tokens = self.tokens();
        if !self.work(tokens.len()) {
            return;
        }
        for token in tokens {
            if token.kind == kind {
                self.invalid.insert(token);
            }
        }
        self.global_consumed.insert(kind);
    }

    pub(super) fn unknown_global(&mut self) {
        let tokens = self.tokens();
        if !self.work(tokens.len()) {
            return;
        }
        self.invalid.extend(tokens);
        self.global_unknown = true;
    }

    pub(super) fn apply_effect(&mut self, effect: HandleEffect, consumed: bool) {
        let tokens = self.tokens();
        if !self.work(tokens.len()) {
            return;
        }
        // Test aliases against one pre-effect state; iteration order must not
        // make closing A erase the known distinction between live A and B.
        let invalid: Vec<_> = tokens
            .into_iter()
            .filter(|token| {
                !self.invalid.contains(token)
                    && if let Some(kind) = effect.kind {
                        kind == token.kind && super::guards::may_alias(self, token, &effect.value)
                    } else {
                        // An opaque argument has no native namespace contract. An
                        // explicitly passed acquisition loses its live witness; other
                        // possible aliases are covered by the call's global frame.
                        matches!(&effect.value,Expr::Fresh(value) if value==token)
                    }
            })
            .collect();
        self.invalid.extend(invalid);
        let effects = if consumed {
            &mut self.consumed
        } else {
            &mut self.uncertain
        };
        if !effects.contains(&effect) {
            effects.push(effect);
        }
    }
}
