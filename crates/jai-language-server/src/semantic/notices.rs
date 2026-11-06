//! Bounded historical compile-limit events, without source text or compiler ownership.
use super::queries::LimitHit;
use std::collections::VecDeque;

const BUDGET: usize = 16 << 20;

pub(super) struct Notices {
    enabled: bool,
    budget: usize,
    events: VecDeque<LimitHit>,
    payload_bytes: usize,
    dropped: usize,
}

impl Default for Notices {
    fn default() -> Self {
        Self {
            enabled: false,
            budget: BUDGET,
            events: VecDeque::new(),
            payload_bytes: 0,
            dropped: 0,
        }
    }
}

impl Notices {
    pub(super) fn enabled(&self) -> bool {
        self.enabled
    }

    /// A setting transition starts a new subscription; old events are not replayed to it.
    pub(super) fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            self.events = VecDeque::new();
            self.payload_bytes = 0;
            self.enabled = enabled;
        }
    }

    fn payload(event: &LimitHit) -> usize {
        event
            .location
            .uri
            .capacity()
            .saturating_add(event.message.capacity())
    }

    /// Queued event count, allocated queue/string bytes, and events retired under pressure.
    pub(super) fn counts(&self) -> (usize, usize, usize) {
        (
            self.events.len(),
            self.payload_bytes.saturating_add(
                self.events
                    .capacity()
                    .saturating_mul(std::mem::size_of::<LimitHit>()),
            ),
            self.dropped,
        )
    }

    fn retire_front(&mut self) {
        if let Some(event) = self.events.pop_front() {
            self.payload_bytes -= Self::payload(&event);
            self.dropped = self.dropped.saturating_add(1);
        }
    }

    /// Keep newest events within the byte budget; one oversized event is never retained.
    pub(super) fn extend(&mut self, events: impl IntoIterator<Item = LimitHit>) {
        if !self.enabled {
            return;
        }
        for event in events {
            let bytes = Self::payload(&event);
            if bytes.saturating_add(std::mem::size_of::<LimitHit>()) > self.budget {
                self.dropped = self.dropped.saturating_add(1);
                continue;
            }
            self.payload_bytes = self.payload_bytes.saturating_add(bytes);
            self.events.push_back(event);
            if self.counts().1 <= self.budget {
                continue;
            }
            while !self.events.is_empty()
                && self.payload_bytes.saturating_add(
                    self.events
                        .len()
                        .saturating_mul(std::mem::size_of::<LimitHit>()),
                ) > self.budget
            {
                self.retire_front();
            }
            // Release excess backing storage once, after choosing the oldest prefix to retire.
            self.events.shrink_to_fit();
            if self.counts().1 > self.budget {
                // An allocator may keep excess capacity despite shrink_to_fit. Never report
                // that allocation as within budget; retire the remaining queue in that case.
                self.dropped = self.dropped.saturating_add(self.events.len());
                self.events = VecDeque::new();
                self.payload_bytes = 0;
            }
        }
    }

    /// Drain owned historical frames, releasing all retained queue backing storage.
    pub(super) fn take(&mut self) -> VecDeque<LimitHit> {
        self.payload_bytes = 0;
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(ordinal: usize, message_bytes: usize) -> LimitHit {
        LimitHit {
            kind: crate::semantic::LimitKind::Budget,
            location: crate::Location {
                uri: format!("file:///project/{ordinal}.jai"),
                range: crate::Range::default(),
            },
            message: "x".repeat(message_bytes),
        }
    }

    #[test]
    fn pressure_retires_oldest_events_and_charges_backing_storage() {
        let mut queue = Notices {
            budget: 2_000,
            ..Default::default()
        };
        queue.set_enabled(true);
        queue.extend((0..20).map(|ordinal| event(ordinal, 128)));
        let (count, bytes, dropped) = queue.counts();
        assert!(count > 0 && count < 20);
        assert!(bytes <= 2_000);
        assert_eq!(count + dropped, 20);
        let retained = queue.take();
        assert!(retained.back().unwrap().location.uri.ends_with("/19.jai"));
        let first = 20 - count;
        assert_eq!(
            retained
                .iter()
                .map(|hit| hit.location.uri.clone())
                .collect::<Vec<_>>(),
            (first..20)
                .map(|ordinal| format!("file:///project/{ordinal}.jai"))
                .collect::<Vec<_>>()
        );
        assert_eq!(queue.counts(), (0, 0, dropped));
    }

    #[test]
    fn disabled_and_oversized_events_never_accumulate() {
        let mut queue = Notices {
            budget: 2_000,
            ..Default::default()
        };
        queue.extend([event(0, 10)]);
        assert_eq!(queue.counts(), (0, 0, 0));
        queue.set_enabled(true);
        queue.extend([event(1, 10), event(2, 2_001)]);
        assert_eq!(queue.counts().0, 1);
        assert_eq!(queue.counts().2, 1);
        queue.set_enabled(false);
        assert_eq!(queue.counts(), (0, 0, 1));
        queue.set_enabled(true);
        queue.extend([event(3, 10)]);
        assert!(
            queue
                .take()
                .front()
                .unwrap()
                .location
                .uri
                .ends_with("/3.jai")
        );
    }
}
