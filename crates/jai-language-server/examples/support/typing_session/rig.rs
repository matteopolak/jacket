//! The rig: a session, the checker's copy of every document and the journal of edits.

use super::{
    Choices, Clock, Config, Diagnostic, DocumentUri, History, Hover, Input, Limits, Location,
    Position, Query, Range, Report, SemanticToken, Session, Snapshot, Step, TextChange, answer,
    any_omits_literal, balanced, byte_of, identifier_starts, omits_literal, pos_of, probe_at,
    sanity, snapshot_difference, sorted_debug, stale_damage, usage_lint,
};

/// A byte range of the document that edits stay inside: `start` and `orig_end` in the original
/// text, `end` in the current one. It starts at a line break and ends before one (or is the end of
/// the file), so text that follows it cannot be swallowed by a comment or string left open in it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Region {
    pub(super) start: usize,
    pub(super) orig_end: usize,
    pub(super) end: usize,
}

pub(super) struct Doc {
    pub(super) uri: DocumentUri,
    pub(super) orig: String,
    pub(super) text: String,
    pub(super) version: i32,
    pub(super) region: Option<Region>,
    /// Every text the server was sent and what was asked of it after each, for replaying a failure.
    pub(super) history: Vec<Step>,
    /// Procedure bodies of the original text, as regions.
    pub(super) targets: Vec<Region>,
    pub(super) baseline: Snapshot,
}

#[derive(Clone, Debug)]
pub(super) struct Edit {
    pub(super) at: usize,
    pub(super) removed: String,
    pub(super) inserted: String,
}

pub(super) struct Rig<'a, C: Choices> {
    pub(super) input: &'a Input<'a>,
    pub(super) config: Config,
    pub(super) choices: C,
    pub(super) session: Session,
    pub(super) clock: Clock,
    pub(super) docs: Vec<Doc>,
    pub(super) journal: Vec<(usize, Vec<Edit>)>,
    pub(super) trace: Vec<String>,
    pub(super) report: Report,
    pub(super) counter: usize,
    pub(super) scenario: usize,
    /// The document whose newest history step the logged requests belong to.
    pub(super) last_step: Option<usize>,
    pub(super) initial_log: Vec<Query>,
    pub(super) hard_failures: usize,
}

/// Byte ranges of `#string NAME` bodies: source text held in a string (a program a metaprogram
/// compiles, say), where an edit changes what compile-time code does, not just what is typed.
pub(super) fn here_strings(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = text[from..].find("#string ") {
        let start = from + at;
        let rest = &text[start + "#string ".len()..];
        let name = rest.split(|c: char| c.is_whitespace()).next().unwrap_or("");
        let body = rest.find('\n').map_or(rest.len(), |n| n + 1);
        let end = rest[body..]
            .lines()
            .scan(body, |offset, line| {
                let at = *offset;
                *offset += line.len() + 1;
                Some((at, line))
            })
            .find(|(_, line)| !name.is_empty() && line.trim() == name)
            .map_or(text.len(), |(at, _)| start + "#string ".len() + at);
        out.push((start, end));
        from = end.max(start + 1);
    }
    out
}

/// Procedure bodies: a line at column 0 holding `name :: (` and ending in `{`, up to the next line
/// that is just `}`. The region runs from the header's line break to the break before that `}`.
pub(super) fn procedure_bodies(text: &str) -> Vec<Region> {
    let strings = here_strings(text);
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        let start = offset;
        offset += line.len();
        let header = trimmed.ends_with('{')
            && !strings.iter().any(|&(a, b)| a <= start && start < b)
            && trimmed.contains(" :: (")
            && !trimmed.starts_with([' ', '\t', '#', '/']);
        match open {
            None if header && line.ends_with('\n') => open = Some(offset - 1),
            Some(from) if trimmed == "}" && start > from => {
                out.push(Region {
                    start: from,
                    orig_end: start - 1,
                    end: start - 1,
                });
                open = None;
            }
            _ => {}
        }
    }
    out
}

impl<'a, C: Choices> Rig<'a, C> {
    pub(super) fn open_session(input: &Input<'_>, texts: &[String]) -> Result<Session, String> {
        let mut session = Session::with_environment(Limits::default(), (input.environment)());
        for (uri, text) in &input.setup {
            session
                .open(uri.clone(), 1, text.clone())
                .map_err(|e| format!("{e:?}"))?;
        }
        session.set_auto_import(false);
        for ((uri, _), text) in input.docs.iter().zip(texts) {
            session
                .open(uri.clone(), 1, text.clone())
                .map_err(|e| format!("{e:?}"))?;
        }
        Ok(session)
    }

    pub(super) fn new(input: &'a Input<'a>, config: Config, choices: C) -> Result<Self, String> {
        let texts: Vec<String> = input.docs.iter().map(|(_, t)| t.clone()).collect();
        let session = Self::open_session(input, &texts)?;
        let mut clock = Clock::new(config.budget, config.hang);
        clock.recording = true;
        let docs = input
            .docs
            .iter()
            .map(|(uri, text)| Doc {
                uri: uri.clone(),
                orig: text.clone(),
                text: text.clone(),
                version: 1,
                region: None,
                history: Vec::new(),
                targets: procedure_bodies(text),
                baseline: Snapshot::take(&mut clock, &session, uri, text, config.full_probes),
            })
            .collect();
        // Asking again must change nothing: the second snapshot is the baseline, and a difference
        // from the first is oracle h's.
        let mut docs: Vec<Doc> = docs;
        let mut asked_twice = Vec::new();
        for d in &mut docs {
            let again = Snapshot::take(&mut clock, &session, &d.uri, &d.orig, config.full_probes);
            if let Some(message) = snapshot_difference(&d.baseline, &again) {
                asked_twice.push((d.uri.as_str().to_string(), message));
            }
            d.baseline = again;
        }
        let mut rig = Self {
            input,
            config,
            choices,
            session,
            clock,
            docs,
            journal: Vec::new(),
            trace: Vec::new(),
            report: Report::default(),
            counter: 0,
            scenario: 0,
            last_step: None,
            initial_log: Vec::new(),
            hard_failures: 0,
        };
        for (uri, message) in asked_twice {
            rig.report.failures.push(format!(
                "oracle h in {uri}: the same requests answered differently the second time:\n{message}"
            ));
        }
        Ok(rig)
    }

    /// File the requests logged so far under the version they were made on.
    pub(super) fn flush_log(&mut self) {
        let queries: Vec<Query> = std::mem::take(&mut self.clock.log);
        match self.last_step {
            Some(doc) => {
                if let Some(step) = self.docs[doc].history.last_mut() {
                    step.queries.extend(queries);
                }
            }
            None => self.initial_log.extend(queries),
        }
    }

    pub(super) fn failed(&self) -> bool {
        self.hard_failures >= self.config.max_failures
    }

    pub(super) fn fail(&mut self, oracle: &str, doc: usize, detail: String) {
        if oracle != "h" {
            self.hard_failures += 1;
        }
        let recent: Vec<_> = self.trace.iter().rev().take(6).rev().cloned().collect();
        let text: String = self.docs[doc].text.chars().take(1500).collect();
        if oracle == "d" {
            self.flush_log();
            self.report.histories.push((
                doc,
                History {
                    initial: self.initial_log.clone(),
                    steps: self.docs[doc].history.clone(),
                },
            ));
        }
        self.report.failures.push(format!(
            "oracle {oracle} in {} (scenario {}, version {}, {} edits so far):\n{detail}\nlast edits:\n  {}\ntext:\n{text}",
            self.docs[doc].uri.as_str(),
            self.scenario,
            self.report.versions,
            self.trace.len(),
            recent.join("\n  ")
        ));
    }

    pub(super) fn at_baseline(&self) -> bool {
        self.docs.iter().all(|d| d.text == d.orig)
    }

    // -- editing ---------------------------------------------------------------------------

    /// Send `edits` as one `didChange`, in order, and record them in the journal.
    pub(super) fn apply(&mut self, doc: usize, edits: Vec<Edit>) {
        self.apply_inner(doc, &edits);
        self.journal.push((doc, edits));
    }

    pub(super) fn apply_inner(&mut self, doc: usize, edits: &[Edit]) {
        self.flush_log();
        self.last_step = Some(doc);
        let d = &mut self.docs[doc];
        let mut changes = Vec::new();
        for e in edits {
            let end = e.at + e.removed.len();
            assert_eq!(&d.text[e.at..end], e.removed, "checker edit out of sync");
            changes.push(TextChange {
                range: Some(Range {
                    start: pos_of(&d.text, e.at),
                    end: pos_of(&d.text, end),
                }),
                range_length: None,
                text: e.inserted.clone(),
            });
            d.text.replace_range(e.at..end, &e.inserted);
            if let Some(r) = &mut d.region {
                assert!(e.at >= r.start && end <= r.end, "edit left its region");
                r.end = r.end - e.removed.len() + e.inserted.len();
            }
            self.trace.push(format!(
                "doc {doc} at {} remove {:?} insert {:?}",
                e.at, e.removed, e.inserted
            ));
        }
        d.version += 1;
        d.history.push(Step {
            text: d.text.clone(),
            queries: Vec::new(),
        });
        let (uri, version) = (d.uri.clone(), d.version);
        if let Err(e) = self.session.change(&uri, version, &changes) {
            self.fail("f", doc, format!("change rejected: {e:?}"));
        }
        self.report.versions += 1;
        self.counter += 1;
    }

    pub(super) fn undo_one(&mut self) -> bool {
        let Some((doc, edits)) = self.journal.pop() else {
            return false;
        };
        let inverse: Vec<Edit> = edits
            .iter()
            .rev()
            .map(|e| Edit {
                at: e.at,
                removed: e.inserted.clone(),
                inserted: e.removed.clone(),
            })
            .collect();
        self.apply_inner(doc, &inverse);
        self.forget_regions();
        true
    }

    /// A document back at its original text has no region (the next scenario picks one).
    pub(super) fn forget_regions(&mut self) {
        for d in &mut self.docs {
            if d.text == d.orig {
                d.region = None;
            }
        }
        if self.at_baseline() {
            self.journal.clear();
        }
    }

    pub(super) fn insert(&mut self, doc: usize, at: usize, text: &str) {
        self.apply(
            doc,
            vec![Edit {
                at,
                removed: String::new(),
                inserted: text.into(),
            }],
        );
    }

    pub(super) fn remove(&mut self, doc: usize, at: usize, len: usize) {
        let removed = self.docs[doc].text[at..at + len].to_string();
        self.apply(
            doc,
            vec![Edit {
                at,
                removed,
                inserted: String::new(),
            }],
        );
    }

    // -- checks ----------------------------------------------------------------------------

    /// Run after every version.
    pub(super) fn after_edit(&mut self) {
        if self.failed() {
            return;
        }
        self.check_light();
        if self.config.diff_every > 0 && self.counter.is_multiple_of(self.config.diff_every) {
            self.check_fresh();
        }
        if self.at_baseline() {
            self.check_baseline();
        }
        for slow in std::mem::take(&mut self.clock.slow) {
            self.fail("a", 0, format!("slow request: {slow}"));
        }
    }

    pub(super) fn region_text(&self, doc: usize) -> Option<&str> {
        let d = &self.docs[doc];
        d.region.map(|r| &d.text[r.start..r.end])
    }

    pub(super) fn all_balanced(&self) -> bool {
        (0..self.docs.len()).all(|i| self.region_text(i).is_none_or(balanced))
    }

    /// Map a byte of the original text to the current one; `None` inside the region.
    pub(super) fn map_byte(&self, doc: usize, byte: usize, is_end: bool) -> Option<usize> {
        let d = &self.docs[doc];
        let Some(r) = d.region else {
            return Some(byte);
        };
        if byte < r.start || (is_end && byte == r.start) {
            Some(byte)
        } else if byte >= r.orig_end {
            Some(byte - r.orig_end + r.end)
        } else {
            None
        }
    }

    pub(super) fn map_position(&self, doc: usize, p: Position, is_end: bool) -> Option<Position> {
        let d = &self.docs[doc];
        let byte = byte_of(&d.orig, p)?;
        let mapped = self.map_byte(doc, byte, is_end)?;
        Some(pos_of(&d.text, mapped))
    }

    pub(super) fn map_range(&self, doc: usize, r: Range) -> Option<Range> {
        Some(Range {
            start: self.map_position(doc, r.start, false)?,
            end: self.map_position(doc, r.end, true)?,
        })
    }

    pub(super) fn map_location(&self, l: &Location) -> Option<Location> {
        let Some(target) = self.docs.iter().position(|d| d.uri.as_str() == l.uri) else {
            return Some(l.clone());
        };
        Some(Location {
            uri: l.uri.clone(),
            range: self.map_range(target, l.range)?,
        })
    }

    /// Whether a range of `text` touches `lo..hi`.
    pub(super) fn touches(text: &str, range: Range, lo: usize, hi: usize) -> bool {
        let (Some(a), Some(b)) = (byte_of(text, range.start), byte_of(text, range.end)) else {
            return true;
        };
        if a == b {
            return lo < a && a <= hi;
        }
        a < hi && b > lo
    }

    /// Whether a current-text range touches the region of `doc`.
    pub(super) fn in_region(&self, doc: usize, range: Range) -> bool {
        let d = &self.docs[doc];
        d.region
            .is_some_and(|r| Self::touches(&d.text, range, r.start, r.end))
    }

    /// Whether a diagnostic range touches the region or the header line above it: a missing return
    /// or an unused parameter is reported there, on what the body is checked against.
    pub(super) fn near_region(&self, doc: usize, range: Range, original: bool) -> bool {
        let d = &self.docs[doc];
        let Some(r) = d.region else {
            return false;
        };
        let (text, end) = if original {
            (&d.orig, r.orig_end)
        } else {
            (&d.text, r.end)
        };
        let line = text[..r.start].rfind('\n').map_or(0, |i| i + 1);
        Self::touches(text, range, line, end)
    }

    pub(super) fn check_light(&mut self) {
        let texts: Vec<_> = self
            .docs
            .iter()
            .map(|d| (d.uri.clone(), d.text.clone()))
            .collect();
        for (i, (uri, text)) in texts.iter().enumerate() {
            let seen = self.session.document_text(uri).map(str::to_owned);
            if seen.as_deref() != Ok(text.as_str()) {
                self.fail(
                    "f",
                    i,
                    "the server's text differs from the edited text".into(),
                );
                return;
            }
            let diagnostics = self.clock.run(Query::Diagnostics(uri.clone()), || {
                self.session.diagnostics(uri)
            });
            match diagnostics {
                Ok(list) => {
                    if let Some(message) = sanity(text, &list) {
                        self.fail("e", i, message);
                    }
                }
                Err(e) => self.fail("e", i, format!("diagnostics failed: {e:?}")),
            }
            self.cursor_queries(i);
        }
        if self.config.outside && self.all_balanced() && !self.failed() {
            for i in 0..self.docs.len() {
                if let Some(message) = self.outside_difference(i) {
                    self.fail("b", i, message);
                    return;
                }
            }
        }
    }

    /// Requests at the edit site, only to be survived (answers there may be anything).
    pub(super) fn cursor_queries(&mut self, doc: usize) {
        let d = &self.docs[doc];
        let Some(r) = d.region else {
            return;
        };
        let (uri, text) = (d.uri.clone(), d.text.clone());
        let at = r.start + (self.counter * 7) % (r.end - r.start + 1);
        let at = text.floor_char_boundary(at);
        let p = pos_of(&text, at);
        let s = &self.session;
        for query in [
            Query::Hover(uri.clone(), p),
            Query::Definition(uri.clone(), p),
            Query::Completion(uri.clone(), p),
            Query::SignatureHelp(uri.clone(), p),
        ] {
            self.clock.run(query.clone(), || query.execute(s));
        }
    }

    /// Oracle b for one document: baseline answers outside the region, mapped through the edits.
    pub(super) fn outside_difference(&mut self, doc: usize) -> Option<String> {
        self.report.outside_comparisons += 1;
        let d = &self.docs[doc];
        let (uri, text) = (d.uri.clone(), d.text.clone());
        let base = &d.baseline;
        // A rotating window of the baseline probes.
        let n = base.probes.len();
        let take = self.config.probes.min(n);
        let from = if n == 0 {
            0
        } else {
            (self.counter * 5) % n
        };
        let mut found = None;
        for k in 0..take {
            let probe = &base.probes[(from + k) % n];
            let Some(byte) = self.map_byte(doc, probe.byte, false) else {
                continue;
            };
            let (Ok(old_def), Ok(old_hover)) = (&probe.definition, &probe.hover) else {
                continue;
            };
            let expected: Option<Vec<Location>> =
                old_def.iter().map(|l| self.map_location(l)).collect();
            let Some(expected) = expected else {
                continue;
            };
            let now = probe_at(&mut self.clock, &self.session, &uri, &text, byte);
            if now.definition.as_ref() != Ok(&expected) {
                found = Some(format!(
                    "definition at {:?} (was {:?}) changed:\n  baseline {:?}\n  now      {:?}",
                    now.position, probe.position, expected, now.definition
                ));
                break;
            }
            let expected_hover = old_hover.as_ref().map(|h| {
                self.map_range(doc, h.range).map(|range| Hover {
                    contents: h.contents.clone(),
                    range,
                })
            });
            let comparable = match expected_hover {
                None => Some(None),
                Some(Some(h)) => Some(Some(h)),
                Some(None) => None,
            };
            let poly = super::snapshot::in_poly_struct(&text, byte);
            let differs = |expected: &Option<Hover>, now: &Result<Option<Hover>, String>| {
                if poly {
                    // The fallback to the syntax is what an unchecked struct shows.
                    return match (expected, now) {
                        (Some(a), Ok(Some(b))) => {
                            a.range != b.range
                                || (b.contents.value.contains("Source syntax declaration")
                                    && !a.contents.value.contains("Source syntax declaration"))
                        }
                        (None, Ok(None)) => false,
                        _ => true,
                    };
                }
                now.as_ref() != Ok(expected)
            };
            if let Some(expected_hover) = comparable
                && differs(&expected_hover, &now.hover)
            {
                found = Some(format!(
                    "hover at {:?} (was {:?}) changed:\n  baseline {:?}\n  now      {:?}",
                    now.position, probe.position, expected_hover, now.hover
                ));
                break;
            }
        }
        if found.is_some() {
            return found;
        }
        self.outside_lists(doc)
    }

    pub(super) fn outside_lists(&mut self, doc: usize) -> Option<String> {
        let d = &self.docs[doc];
        let (uri, text) = (d.uri.clone(), d.text.clone());
        let session = &self.session;
        let diagnostics = self.clock.run(Query::Diagnostics(uri.clone()), || {
            answer(session.diagnostics(&uri))
        });
        let tokens = self.clock.run(Query::Tokens(uri.clone()), || {
            answer(session.semantic_tokens(&uri))
        });
        let d = &self.docs[doc];
        if let (Ok(now), Ok(was)) = (&diagnostics, &d.baseline.diagnostics) {
            let expected: Vec<_> = was
                .iter()
                .filter(|x| !usage_lint(x) && !self.near_region(doc, x.range, true))
                .filter_map(|x| {
                    Some(Diagnostic {
                        range: self.map_range(doc, x.range)?,
                        ..x.clone()
                    })
                })
                .collect();
            let actual: Vec<_> = now
                .iter()
                .filter(|x| !usage_lint(x) && !self.near_region(doc, x.range, false))
                .cloned()
                .collect();
            if sorted_debug(&expected) != sorted_debug(&actual) {
                return Some(format!(
                    "diagnostics outside the edited region changed:\n  baseline {expected:?}\n  now      {actual:?}"
                ));
            }
        }
        if let (Ok(now), Ok(was)) = (&tokens, &d.baseline.tokens) {
            let expected: Vec<_> = was
                .iter()
                .filter_map(|t| {
                    Some(SemanticToken {
                        position: self.map_position(doc, t.position, false)?,
                        ..*t
                    })
                })
                .collect();
            let actual: Vec<_> = now
                .iter()
                .filter(|t| {
                    let end = Position {
                        line: t.position.line,
                        character: t.position.character + t.length,
                    };
                    !self.in_region(
                        doc,
                        Range {
                            start: t.position,
                            end,
                        },
                    )
                })
                .copied()
                .collect();
            if sorted_debug(&expected) != sorted_debug(&actual) {
                let (x, y) = (sorted_debug(&expected), sorted_debug(&actual));
                let first = x
                    .iter()
                    .find(|t| !y.contains(t))
                    .map(|t| format!("lost {t}"))
                    .or_else(|| {
                        y.iter()
                            .find(|t| !x.contains(t))
                            .map(|t| format!("gained {t}"))
                    });
                return Some(format!(
                    "semantic tokens outside the edited region changed: {first:?} ({} -> {})",
                    expected.len(),
                    actual.len()
                ));
            }
        }
        let _ = text;
        None
    }

    /// Oracle c: the original text again must give the baseline back exactly.
    pub(super) fn check_baseline(&mut self) {
        if self.failed() {
            return;
        }
        self.report.baseline_comparisons += 1;
        for i in 0..self.docs.len() {
            let d = &self.docs[i];
            let (uri, text) = (d.uri.clone(), d.text.clone());
            let now = Snapshot::take(
                &mut self.clock,
                &self.session,
                &uri,
                &text,
                self.config.full_probes,
            );
            if let Some(message) = snapshot_difference(&self.docs[i].baseline, &now) {
                self.fail(
                    "c",
                    i,
                    format!("after returning to the original text: {message}"),
                );
                return;
            }
        }
    }

    /// Oracle d: a session opened on the current text answers the same.
    pub(super) fn check_fresh(&mut self) {
        self.report.fresh_comparisons += 1;
        let texts: Vec<String> = self.docs.iter().map(|d| d.text.clone()).collect();
        let fresh = match Self::open_session(self.input, &texts) {
            Ok(s) => s,
            Err(e) => {
                self.fail("d", 0, format!("fresh session failed to open: {e}"));
                return;
            }
        };
        // What the facts of a broken declaration hold depends on which requests were made on this
        // text, so the fresh session is asked the same ones first: only state carried over from
        // earlier versions can then make it differ.
        for query in &self.clock.log {
            query.execute(&fresh);
        }
        let omits = any_omits_literal(&self.session, self.docs.iter().map(|d| &d.uri));
        for i in 0..self.docs.len() {
            let d = &self.docs[i];
            let (uri, text) = (d.uri.clone(), d.text.clone());
            let cap = self.config.full_probes;
            let now = Snapshot::take(&mut self.clock, &self.session, &uri, &text, cap);
            self.clock.recording = false;
            let reference = Snapshot::take(&mut self.clock, &fresh, &uri, &text, cap);
            self.clock.recording = true;
            if stale_damage(&now, &text) {
                self.fail(
                    "g",
                    i,
                    "reports an incomplete lexical edit for text that lexes".into(),
                );
                return;
            }
            if omits || omits_literal(&now) {
                continue;
            }
            if let Some(message) = snapshot_difference(&reference, &now) {
                self.fail(
                    "d",
                    i,
                    format!("differs from a fresh session (was fresh, now incremental): {message}"),
                );
                return;
            }
        }
    }

    // -- scenarios -------------------------------------------------------------------------

    pub(super) fn pick_doc(&mut self) -> usize {
        let n = self.docs.len();
        // Mostly the first document; the others get a third of the sessions between them.
        if n > 1 && self.choices.chance(1, 3) {
            1 + self.choices.below(n - 1)
        } else {
            0
        }
    }

    /// Give `doc` a region to edit in (a procedure body, or the end of the file).
    pub(super) fn ensure_region(&mut self, doc: usize) {
        if self.docs[doc].region.is_some() {
            return;
        }
        let targets = self.docs[doc].targets.len();
        let end = self.docs[doc].orig.len();
        let region = if targets > 0 && !self.choices.chance(1, 6) {
            self.docs[doc].targets[self.choices.below(targets)]
        } else {
            Region {
                start: end,
                orig_end: end,
                end,
            }
        };
        self.docs[doc].region = Some(region);
    }

    /// Offsets in the region where a line can be opened: the ends of its lines.
    pub(super) fn line_ends(&self, doc: usize) -> Vec<usize> {
        let d = &self.docs[doc];
        let r = d.region.expect("region");
        let mut out: Vec<usize> = (r.start..r.end)
            .filter(|&i| d.text.as_bytes()[i] == b'\n')
            .collect();
        out.push(r.end);
        out
    }

    pub(super) fn statement(&mut self) -> String {
        const SNIPPETS: &[&str] = &[
            "__ty := 1;",
            "__ty: int = 7;",
            "__ty := 1;\n    __ty += 2;",
            "if true { __ty := 3; }",
            "for 0..3 { __ty := it; }",
            "print(\"%\\n\", 1);",
            "{\n        __ty := 1;\n    }",
            "__ty :: (a: int) -> int { return a + 1; }",
            "__ty :: struct { a: int; b: float; }",
            "__ty :: enum { A; B; }",
            "__ty := cast(float) 2 * 1.5;",
            "x := __ty.a;",
            "// a comment",
            "/* block */ __ty := 1;",
            "s := \"text\\n\";",
            "if 1 == 1 { } else { }",
            "while true { break; }",
            "__ty := (1 + 2) * 3;",
            "a: [4] int;",
            "p := *__ty;",
        ];
        let name = format!("__ty{}", self.counter);
        let mut statement = SNIPPETS[self.choices.below(SNIPPETS.len())].replace("__ty", &name);
        // Sometimes a call or use of a name the file already resolves.
        if self.choices.chance(1, 3) {
            let words: Vec<&str> = self.docs[0]
                .baseline
                .probes
                .iter()
                .filter(|p| matches!(&p.definition, Ok(l) if !l.is_empty()))
                .map(|p| {
                    let rest = &self.docs[0].orig[p.byte..];
                    let end = rest
                        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .unwrap_or(rest.len());
                    &rest[..end]
                })
                .collect();
            if !words.is_empty() {
                let word = words[self.choices.below(words.len())];
                statement = if self.choices.chance(1, 2) {
                    format!("{word}(1, 2);")
                } else {
                    format!("{name} := {word};")
                };
            }
        }
        format!("    {statement}")
    }

    /// Type `text` character by character at `cursor`. With `autoclose` an opening bracket or quote
    /// inserts its pair and typing the closer steps over it, as editors do. Returns the cursor.
    pub(super) fn type_text(
        &mut self,
        doc: usize,
        mut cursor: usize,
        text: &str,
        autoclose: bool,
    ) -> usize {
        for ch in text.chars() {
            if self.failed() {
                break;
            }
            let after = self.docs[doc].text[cursor..].chars().next();
            if autoclose && matches!(ch, ')' | ']' | '}' | '"') && after == Some(ch) {
                cursor += 1;
                continue;
            }
            let closer = match ch {
                '(' => Some(')'),
                '[' => Some(']'),
                '{' => Some('}'),
                '"' => Some('"'),
                _ => None,
            };
            let mut typed = ch.to_string();
            if let Some(closer) = closer.filter(|_| autoclose) {
                typed.push(closer);
            }
            self.insert(doc, cursor, &typed);
            cursor += ch.len_utf8();
            self.after_edit();
        }
        cursor
    }

    /// Delete backwards from `cursor` down to `stop`, one character per version.
    pub(super) fn backspace(&mut self, doc: usize, mut cursor: usize, stop: usize) -> usize {
        while cursor > stop && !self.failed() {
            let text = &self.docs[doc].text;
            let ch = text[..cursor].chars().next_back().expect("character");
            let len = ch.len_utf8();
            let pair = matches!(ch, '(' | '[' | '{' | '"')
                && text[cursor..].starts_with(match ch {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    _ => '"',
                });
            let (at, total) = if pair {
                (cursor - len, len + 1)
            } else {
                (cursor - len, len)
            };
            self.remove(doc, at, total);
            cursor -= len;
            self.after_edit();
        }
        cursor
    }

    pub(super) fn scenario_type(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let site = if self.choices.chance(2, 3) {
            *ends.last().expect("end")
        } else {
            ends[self.choices.below(ends.len())]
        };
        // Open the new line first, as Enter does.
        self.insert(doc, site, "\n");
        self.after_edit();
        let statement = self.statement();
        let autoclose = self.choices.chance(1, 2);
        let cursor = self.type_text(doc, site + 1, &statement, autoclose);
        if self.choices.chance(1, 2) {
            // Backspace part of it, or all of it and the new line.
            let keep = if self.choices.chance(1, 2) {
                site + 1
            } else {
                site + 1 + (cursor - site - 1) / 2
            };
            let cursor = self.backspace(doc, cursor, keep);
            if keep == site + 1 && !self.failed() {
                self.backspace(doc, cursor, site);
            }
        }
    }

    pub(super) fn scenario_retype(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let lines: Vec<(usize, usize)> = ends
            .windows(2)
            .map(|w| (w[0] + 1, w[1]))
            .filter(|&(s, e)| s < e && !self.docs[doc].text[s..e].trim().is_empty())
            .collect();
        if lines.is_empty() {
            return self.scenario_type(doc);
        }
        let (mut start, mut end) = lines[self.choices.below(lines.len())];
        if self.choices.chance(1, 2) {
            // Just one token of the line.
            let text = &self.docs[doc].text;
            let words: Vec<(usize, usize)> = identifier_starts(&text[start..end])
                .into_iter()
                .map(|i| {
                    let rest = &text[start + i..end];
                    let len = rest
                        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .unwrap_or(rest.len());
                    (start + i, start + i + len)
                })
                .collect();
            if !words.is_empty() {
                (start, end) = words[self.choices.below(words.len())];
            }
        }
        let original = self.docs[doc].text[start..end].to_string();
        if self.choices.chance(1, 2) {
            self.remove(doc, start, end - start);
            self.after_edit();
        } else {
            self.backspace(doc, end, start);
        }
        self.type_text(doc, start, &original, false);
    }

    pub(super) fn scenario_paste(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let site = ends[self.choices.below(ends.len())];
        let block = if self.choices.chance(1, 2) {
            // The region's own text again.
            let d = &self.docs[doc];
            let r = d.region.expect("region");
            d.text[r.start..r.end].to_string()
        } else {
            let first = self.statement();
            let second = self.statement();
            format!("\n{first}\n{second}")
        };
        if block.is_empty() {
            return;
        }
        self.insert(doc, site, &block);
        self.after_edit();
        if self.choices.chance(1, 2) {
            self.undo_one();
            self.after_edit();
        }
    }

    pub(super) fn scenario_unbalanced(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let site = ends[self.choices.below(ends.len())];
        let (open, close) = [
            ("{", "}"),
            ("(", ")"),
            ("[", "]"),
            ("\"", "\""),
            ("/*", "*/"),
            ("#string END\n", "\nEND"),
            ("/*", ""),
            ("\"", ""),
        ][self.choices.below(8)];
        // Open it at the end of a line, check, then close it at the end of the region.
        self.insert(doc, site, open);
        self.after_edit();
        if self.choices.chance(1, 2) {
            // Type something while it is open.
            let statement = self.statement();
            let cursor = site + open.len();
            self.type_text(doc, cursor, &format!("\n{statement}"), false);
        }
        if !close.is_empty() && !self.failed() {
            let end = self.docs[doc].region.expect("region").end;
            self.insert(doc, end, close);
            self.after_edit();
        }
    }

    pub(super) fn scenario_undo(&mut self) {
        let steps = 1 + self.choices.below(self.journal.len().max(1));
        for _ in 0..steps {
            if self.failed() || !self.undo_one() {
                break;
            }
            self.after_edit();
        }
    }

    /// Revert every document the way an editor's "revert file" does: replace it whole.
    pub(super) fn revert_whole(&mut self) {
        for i in 0..self.docs.len() {
            if self.docs[i].text == self.docs[i].orig {
                continue;
            }
            self.flush_log();
            self.last_step = Some(i);
            let d = &mut self.docs[i];
            d.version += 1;
            d.text = d.orig.clone();
            d.history.push(Step {
                text: d.text.clone(),
                queries: Vec::new(),
            });
            d.region = None;
            let (uri, version, text) = (d.uri.clone(), d.version, d.text.clone());
            let change = TextChange {
                range: None,
                range_length: None,
                text,
            };
            if let Err(e) = self.session.change(&uri, version, &[change]) {
                self.fail("f", i, format!("whole-document change rejected: {e:?}"));
            }
            self.trace
                .push(format!("doc {i} replaced by the original text"));
            self.report.versions += 1;
            self.counter += 1;
        }
        self.journal.clear();
        self.after_edit();
    }

    pub(super) fn run_scenario(&mut self) {
        let doc = self.pick_doc();
        match self.choices.below(10) {
            0..=3 => self.scenario_type(doc),
            4 | 5 => self.scenario_retype(doc),
            6 => self.scenario_paste(doc),
            7 | 8 => self.scenario_unbalanced(doc),
            _ => self.scenario_type(doc),
        }
        // Sometimes carry on from the edited text; otherwise go back.
        match self.choices.below(4) {
            0 => {}
            1 => self.scenario_undo(),
            2 => self.revert_whole(),
            _ => {
                while !self.failed() && self.undo_one() {
                    if self.choices.chance(1, 3) {
                        self.after_edit();
                    }
                }
                self.after_edit();
            }
        }
    }

    pub(super) fn run(mut self) -> Report {
        for scenario in 0..self.config.scenarios {
            if self.failed() || self.choices.exhausted() {
                break;
            }
            self.scenario = scenario;
            self.run_scenario();
        }
        if !self.failed() {
            // End where the document began, by the editor's undo.
            while self.undo_one() {
                if self.choices.chance(1, 4) {
                    self.after_edit();
                }
            }
            self.revert_whole();
            self.after_edit();
        }
        self.report
    }
}
