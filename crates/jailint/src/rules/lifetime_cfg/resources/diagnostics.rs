//! Lifetime diagnostics and on-demand witnesses, separate from fixed-point state.
use super::{
    CfgOp, Cx, DEAD, Event, Finding, HashMap, HashSet, LIVE, LifetimeBody, Model, Op, Origin,
    Reports, Span, VecDeque, contains, inside, kills,
};
use crate::Edit;
use crate::rules::{finding, suggesting};

#[derive(Clone, Copy)]
pub(super) struct Proof<'a> {
    pub(super) cx: &'a Cx<'a>,
    pub(super) body: &'a LifetimeBody,
    pub(super) model: &'a Model,
    pub(super) kind: Origin,
    pub(super) seed: usize,
    pub(super) origin: Span,
    pub(super) region: Option<usize>,
    pub(super) relevant: &'a HashMap<usize, usize>,
    pub(super) states: &'a [Option<Vec<u8>>],
}

pub(super) fn emit(proof: Proof<'_>, out: &mut Reports) {
    let Proof {
        cx,
        body,
        model,
        kind,
        origin,
        region,
        relevant,
        states,
        ..
    } = proof;
    let index = |cell: usize| relevant.get(&cell).copied();
    let frees: Vec<_> = model
        .blocks
        .iter()
        .filter_map(|b| match b.op {
            CfgOp::Resource(Op::Free {
                span, ..
            }) => Some(span),
            _ => None,
        })
        .collect();
    for (i, block) in model.blocks.iter().enumerate() {
        let Some(state) = &states[i] else {
            continue;
        };
        let primary = match block.op {
            CfgOp::Use(_, span)
            | CfgOp::Resource(
                Op::Read {
                    span, ..
                }
                | Op::Free {
                    span, ..
                }
                | Op::FreeRegion {
                    span, ..
                }
                | Op::Return {
                    span, ..
                }
                | Op::Escape {
                    span, ..
                },
            ) => Some(span),
            _ => None,
        };
        let mapped = if primary.is_some_and(|span| span.file != cx.file) {
            let Some(site) = block.site.filter(|s| s.file == cx.file) else {
                out.external_primary += 1;
                continue;
            };
            Some(site)
        } else {
            None
        };
        let before = [
            out.use_after_free.len(),
            out.double_free.len(),
            out.returned_local_address.len(),
            out.temp_storage_escape.len(),
        ];
        match &block.op {
            CfgOp::Use(cell, at)
            | CfgOp::Resource(Op::Read {
                cell,
                span: at,
            }) if index(*cell).is_some_and(|c| super::super::dead(cx, state[c]))
                && !frees.iter().any(|&span| contains(span, *at)) =>
            {
                let witnesses = witnesses(model, states, relevant, i, kind, region, cx);
                if witnesses.is_empty() {
                    continue;
                }
                if keeps_use(cx, model, &block.op, &witnesses, *at) {
                    continue;
                }
                if matches!(kind, Origin::Stack(_) | Origin::Statement(_)) {
                    let mut f = finding(*at,"this pointer refers to stack storage whose lifetime has ended".into(),Some("keep the value in the enclosing scope, or copy it into caller-owned storage".into()));
                    label(&mut f, origin, &witnesses);
                    if let Some(site) = mapped {
                        map_finding(&mut f, *at, site);
                    }
                    push(&mut out.returned_local_address, f);
                    continue;
                }
                let name = cx.compiler.src.sources.snippet(*at).trim();
                let mut f = finding(
                    *at,
                    format!("`{name}` is used after the call above freed it"),
                    Some(
                        "free it after its last use, or give it a new value before using it again"
                            .into(),
                    ),
                );
                if matches!(kind, Origin::Temporary) {
                    f.message = format!("`{name}` is used after its temporary storage was reset");
                    f.help = Some(
                        "use it before the reset, or copy it into owned storage before resetting"
                            .into(),
                    );
                }
                if matches!(kind, Origin::Until(_)) {
                    f.message =
                        format!("`{name}` is used after its foreign valid-until contract expired");
                    f.help = Some("copy the borrowed data before the next provider call, or obtain a new result".into());
                }
                if cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                    && state[index(*cell).unwrap()] != DEAD
                {
                    f.message =
                        format!("strict mode: `{name}` may be used after release along a bad path");
                }
                if witnesses.len() > 1 && state[index(*cell).unwrap()] == DEAD {
                    f.message.push_str(&format!(
                        "; every path passes one of these {} invalidating calls",
                        witnesses.len()
                    ));
                }
                label(&mut f, origin, &witnesses);
                push(&mut out.use_after_free, f);
            }
            CfgOp::Resource(Op::Free {
                cell,
                span,
                callee,
                ..
            }) if index(*cell).is_some_and(|c| super::super::dead(cx, state[c])) => {
                let witnesses = witnesses(model, states, relevant, i, kind, region, cx);
                if witnesses.is_empty() {
                    continue;
                }
                if witnesses.iter().all(|(j, _)| {
                    matches!(
                        model.blocks[*j].op,
                        CfgOp::Resource(Op::Free {
                            contents: true,
                            ..
                        })
                    )
                }) && matches!(
                    block.op,
                    CfgOp::Resource(Op::Free {
                        contents: false,
                        ..
                    })
                ) {
                    continue;
                }
                if let CfgOp::Resource(Op::Free {
                    contents: true,
                    fields: Some(now),
                    ..
                }) = &block.op
                    && let Some(before) = released_members(model, &witnesses)
                    && !now.iter().any(|f| before.contains(f))
                {
                    continue;
                }
                let name = free_name(cx, body, *span);
                let same = witnesses.iter().all(|(j,_)| matches!(model.blocks[*j].op, CfgOp::Resource(Op::Free {callee: previous,..}) if previous == *callee));
                let mut f = if same && span.file == cx.file && statement_call(body, *span) {
                    let (start, end) = cx.statement_removal(*span);
                    suggesting(
                        *span,
                        format!("`{name}` is freed a second time"),
                        format!(
                            "remove the second call, or assign `{name}` a new value in between"
                        ),
                        "remove the second free",
                        vec![Edit {
                            start,
                            end,
                            text: String::new(),
                        }],
                    )
                } else {
                    finding(
                        *span,
                        format!("`{name}` is freed a second time"),
                        Some(format!(
                            "remove the second call, or assign `{name}` a new value in between"
                        )),
                    )
                };
                label(&mut f, origin, &witnesses);
                push(&mut out.double_free, f);
            }
            CfgOp::Resource(Op::FreeRegion {
                cell,
                span,
                ..
            }) if index(*cell).is_some_and(|c| super::super::dead(cx, state[c])) => {
                let witnesses = witnesses(model, states, relevant, i, kind, region, cx);
                if witnesses.is_empty() || witnesses.iter().all(|(_, site)| *site == *span) {
                    continue;
                }
                // The contents went first; freeing the memory that held them is a different release.
                if witnesses.iter().all(|(j, _)| {
                    matches!(
                        model.blocks[*j].op,
                        CfgOp::Resource(Op::Free {
                            contents: true,
                            ..
                        })
                    )
                }) {
                    continue;
                }
                let mut f = finding(*span,"this call frees storage that was already released".into(),Some("remove the second call, or initialize the released field with a new allocation first".into()));
                label(&mut f, origin, &witnesses);
                push(&mut out.double_free, f);
            }
            CfgOp::Resource(
                Op::Return {
                    cell,
                    span,
                    ..
                }
                | Op::Escape {
                    cell,
                    span,
                    ..
                },
            ) => proof.escape(i, *cell, span, out),
            _ => {}
        }
        if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
            proof.decorate(i, before, out);
        }
        if let Some(site) = mapped {
            let raw = primary.unwrap();
            for (findings, from) in [
                &mut out.use_after_free,
                &mut out.double_free,
                &mut out.returned_local_address,
                &mut out.temp_storage_escape,
            ]
            .into_iter()
            .zip(before)
            {
                for finding in &mut findings[from..] {
                    map_finding(finding, raw, site);
                }
            }
        }
    }
}

/// Whether a use survives a call that released only an argument's contents: the value itself stays valid, and so do
/// the members that call did not free.
fn keeps_use(
    cx: &Cx<'_>,
    model: &Model,
    op: &CfgOp,
    witnesses: &[(usize, Span)],
    at: Span,
) -> bool {
    let contents = witnesses.iter().any(|(j, _)| {
        matches!(
            model.blocks[*j].op,
            CfgOp::Resource(Op::Free {
                contents: true,
                ..
            })
        )
    });
    let text = &cx.compiler.src.sources.get(at.file).text;
    let suffix = &text[at.end as usize..];
    if contents
        && !matches!(op, CfgOp::Resource(Op::Read { .. }))
        && (suffix.trim_start().starts_with(".* =")
            || (!suffix.starts_with('.') && !suffix.starts_with('[')))
    {
        return true;
    }
    released_members(model, witnesses)
        .zip(used_member(suffix, true).or_else(|| used_member(&text[at.start as usize..], false)))
        .is_some_and(|(freed, member)| !freed.iter().any(|f| f.as_str() == member))
}

impl Proof<'_> {
    fn decorate(&self, at: usize, before: [usize; 4], out: &mut Reports) {
        if before
            == [
                out.use_after_free.len(),
                out.double_free.len(),
                out.returned_local_address.len(),
                out.temp_storage_escape.len(),
            ]
        {
            return;
        }
        let (CfgOp::Use(cell, _)
        | CfgOp::Resource(
            Op::Read {
                cell, ..
            }
            | Op::Free {
                cell, ..
            }
            | Op::FreeRegion {
                cell, ..
            }
            | Op::Return {
                cell, ..
            }
            | Op::Escape {
                cell, ..
            },
        )) = self.model.blocks[at].op
        else {
            return;
        };
        let Some(&dense) = self.relevant.get(&cell) else {
            return;
        };
        let witnesses = witnesses(
            self.model,
            self.states,
            self.relevant,
            at,
            self.kind,
            self.region,
            self.cx,
        );
        let spans: Vec<_> = witnesses.iter().map(|(_, s)| *s).collect();
        let (decisions, count) = super::super::witness::strict_path(
            self.model,
            self.states,
            self.relevant,
            self.seed,
            at,
            dense,
            &spans,
        );
        let may = self.states[at]
            .as_ref()
            .is_some_and(|s| !matches!(s[dense], LIVE | DEAD));
        for (findings, start) in [
            &mut out.use_after_free,
            &mut out.double_free,
            &mut out.returned_local_address,
            &mut out.temp_storage_escape,
        ]
        .into_iter()
        .zip(before)
        {
            for f in &mut findings[start..] {
                if may {
                    f.fix = None;
                    if !f.message.starts_with("strict mode:") {
                        f.message =
                            format!("strict mode: possible lifetime violation: {}", f.message);
                    }
                }
                f.message
                    .push_str(&format!("; shortest bad path shown ({count})"));
                for &(span, taken) in &decisions {
                    f.secondary.push((
                        span,
                        format!("shortest bad path takes this condition as {taken}"),
                    ));
                }
                super::super::witness::targets(self.cx, self.model, &spans, f);
                if witnesses.iter().any(|(i, _)| {
                    matches!(
                        self.model.blocks[*i].op,
                        CfgOp::Resource(Op::MayFree { .. } | Op::MayRegion { .. })
                    )
                }) {
                    f.message.push_str("; release is possible, not guaranteed");
                }
            }
        }
    }

    fn escape(&self, i: usize, cell: usize, span: &Span, out: &mut Reports) {
        let Self {
            cx,
            body,
            model,
            kind,
            origin,
            region,
            relevant,
            states,
            ..
        } = *self;
        let block = &model.blocks[i];
        let state = states[i].as_ref().unwrap();
        let index = |cell| relevant.get(&cell).copied();
        let returning = matches!(block.op, CfgOp::Resource(Op::Return { .. }));
        let Some(cell) = index(cell) else {
            return;
        };
        if if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
            state[cell] & (LIVE | DEAD) == 0
        } else {
            !matches!(state[cell], LIVE | DEAD)
        } {
            return;
        }

        // Temporary allocations remain live until reset. A root run's readable
        // stack result is frozen immediately, before another activation reuses it.
        if cx.run.lifetimes.get().is_some_and(|s| {
            matches!(kind, Origin::Temporary)
                && state[cell] & DEAD == 0
                && s.compile_time_only.contains(&body.procedure)
                || returning
                    && matches!(kind, Origin::Stack(_) | Origin::Statement(_))
                    && s.frozen_returns.contains(&body.procedure)
        }) {
            return;
        }
        if matches!(kind, Origin::Stack(_) | Origin::Statement(_)) {
            if !returning && entrypoint_storage(cx, body, kind) {
                return;
            }
            if let CfgOp::Resource(Op::Escape {
                target,
                store,
                ..
            }) = block.op
                && !self.escape_survives(i, target, store, kind, Retention::AllPaths)
            {
                return;
            }
            let name = stack_name(cx, body, origin);
            let message = if returning {
                let source = cx.compiler.src.sources.snippet(origin).trim();
                let how = if source.ends_with(".data") && !source.starts_with('*') {
                    "the data of"
                } else {
                    "the address of"
                };
                format!(
                    "this returns {how} the local `{name}`, which is gone once the procedure returns"
                )
            } else {
                format!("the address of the local `{name}` escapes into storage that outlives it")
            };
            let mut f = finding(*span, message, Some("return the value itself, take the memory from an allocator (`New`, `alloc`), or have the caller pass the place to fill".into()));
            f.secondary
                .push((origin, "stack storage borrowed here".into()));
            push(&mut out.returned_local_address, f);
        } else if matches!(kind, Origin::Temporary)
            && self.temporary_store_retained(i, &block.op)
            && !matches!(
                block.op,
                CfgOp::Resource(Op::Escape {
                    temporary_target: true,
                    ..
                })
            )
            && !matches!(
                block.op,
                CfgOp::Resource(Op::Escape {
                    parameter_target: true,
                    ..
                })
            )
            && (!returning || !inferred_temporary_return(cx, body, model, &block.op))
            && (!returning
                || !(crate::rules::lifetime_movement::notes(cx.compiler, body.procedure)
                    .any(|n| n.text.trim() == "TempAllocates")
                    || crate::rules::lifetime_movement::result_indices(
                        cx.compiler,
                        body.procedure,
                        "TempAllocates",
                    )
                    .contains(&match block.op {
                        CfgOp::Resource(Op::Return {
                            result, ..
                        }) => result,
                        _ => usize::MAX,
                    })))
        {
            let name = expression_name(cx, body, *span);
            let (message, help) = if returning {
                (
                    format!(
                        "`{name}` holds temporary storage, which is reset, but it is returned here"
                    ),
                    "copy it (`copy_string`), or declare this procedure `@TempAllocates` so its callers know",
                )
            } else {
                (
                    format!(
                        "`{name}` holds temporary storage, which is reset, but it is kept in {}",
                        if matches!(
                            block.op,
                            CfgOp::Resource(Op::Escape {
                                pointer: true,
                                ..
                            })
                        ) {
                            "a place reached through a pointer"
                        } else {
                            "a global"
                        }
                    ),
                    "copy it first (`copy_string`) so it outlives the reset",
                )
            };
            let mut f = finding(*span, message, Some(help.into()));
            f.secondary
                .push((origin, "temporary storage allocated here".into()));
            push(&mut out.temp_storage_escape, f);
        } else if super::super::dead(cx, state[cell]) {
            let witnesses = witnesses(model, states, relevant, i, kind, region, cx);
            let mut f = finding(*span,"this returns storage released by deferred cleanup".into(),Some("transfer ownership to the caller, or return a copy made outside the released storage".into()));
            label(&mut f, origin, &witnesses);
            push(&mut out.use_after_free, f);
        }
    }
}

fn push(out: &mut Vec<Finding>, f: Finding) {
    if !out
        .iter()
        .any(|old| old.start == f.start && old.end == f.end)
    {
        out.push(f);
    }
}

fn label(f: &mut Finding, origin: Span, witnesses: &[(usize, Span)]) {
    f.secondary.push((origin, "storage derived here".into()));
    for &(_, site) in witnesses {
        f.secondary.push((site, "storage invalidated here".into()));
    }
}

fn witnesses(
    model: &Model,
    states: &[Option<Vec<u8>>],
    relevant: &HashMap<usize, usize>,
    use_at: usize,
    kind: Origin,
    region: Option<usize>,
    cx: &Cx,
) -> Vec<(usize, Span)> {
    let index = |c: usize| relevant.get(&c).copied();
    let mut incoming = vec![Vec::new(); model.blocks.len()];
    for (i, b) in model.blocks.iter().enumerate() {
        for &to in &b.next {
            incoming[to].push(i);
        }
    }
    let mut queue: VecDeque<_> = incoming[use_at].iter().copied().collect();
    let mut seen = HashSet::default();
    let mut result = Vec::new();
    while let Some(i) = queue.pop_front() {
        if !seen.insert(i) {
            continue;
        }
        let Some(state) = &states[i] else {
            continue;
        };
        if let CfgOp::Resource(op) = &model.blocks[i].op
            && kills(cx, op, kind, region, &state[..=relevant.len() * 2], &index)
        {
            let (Op::Free {
                span: site, ..
            }
            | Op::FreeRegion {
                span: site, ..
            }
            | Op::MayFree {
                span: site, ..
            }
            | Op::MayRegion {
                span: site, ..
            }
            | Op::Advance(_, site)
            | Op::MayAdvance(_, site)
            | Op::MayReset(site)
            | Op::MayRestore(_, site)
            | Op::Reset(site)
            | Op::Restore(_, site)
            | Op::ScopeEnd(_, site)
            | Op::StatementEnd(site)) = *op
            else {
                unreachable!()
            };
            result.push((i, site));
            continue;
        }
        queue.extend(&incoming[i]);
    }
    result.sort_by_key(|(_, s)| (s.file.0, s.start, s.end));
    result.dedup_by_key(|(_, s)| *s);
    result
}

fn free_name(cx: &Cx, body: &LifetimeBody, site: Span) -> String {
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                span,
                args,
                ..
            } if *span == site => args
                .iter()
                .find(|a| a.formal == 0)
                .map(|a| cx.compiler.src.sources.snippet(a.span).trim().to_owned()),
            _ => None,
        })
        .unwrap_or_else(|| "storage".into())
}

fn stack_name(cx: &Cx, body: &LifetimeBody, origin: Span) -> String {
    let value = cx
        .compiler
        .src
        .sources
        .snippet(origin)
        .trim()
        .trim_start_matches('*');
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Local {
                name, ..
            } if value.strip_prefix(name.as_str()).is_some_and(|rest| {
                rest.is_empty() || rest.starts_with('.') || rest.starts_with('[')
            }) =>
            {
                Some(name.as_str().to_owned())
            }
            _ => None,
        })
        .unwrap_or_else(|| value.into())
}

fn statement_call(body: &LifetimeBody, site: Span) -> bool {
    let mut found = false;
    if let Some(block) = &body.syntax.body {
        jaic::ast::walk::walk(&block.stmts, &mut |node, _| {
            if let Some(jaic::ast::StmtKind::Expr(e)) = node.stmt().map(|s| &s.kind) {
                found |= e.span == site;
            }
            true
        });
    }
    found
}


enum Retention {
    AllPaths,
    AnyPath,
    UntilProcessExit,
}

impl Proof<'_> {
    fn temporary_store_retained(&self, at: usize, op: &CfgOp) -> bool {
        let CfgOp::Resource(Op::Escape {
            target,
            store,
            ..
        }) = op
        else {
            return true;
        };
        // A concrete sink must be cleared before reset or before returning to a caller.
        // The process entry has no caller after its final exit, but resets still matter.
        self.escape_survives(
            at,
            *target,
            *store,
            Origin::Temporary,
            if entrypoint_activation(self.cx, self.body) {
                Retention::UntilProcessExit
            } else {
                Retention::AnyPath
            },
        )
    }

    fn escape_survives(
        &self,
        at: usize,
        target: usize,
        store: bool,
        kind: Origin,
        retention: Retention,
    ) -> bool {
        let cx = self.cx;
        let model = self.model;
        let any_path = !matches!(retention, Retention::AllPaths);
        let process_exit = matches!(retention, Retention::UntilProcessExit);
        let target_place = match model.cells.values[target] {
            super::Cell::Place(place) => Some(place),
            super::Cell::Value(_) => None,
        };
        let mut start = at;
        if store {
            // The escape observation precedes this store's conservative memory barrier.
            // Begin checking retention after the actual destination copy, not its preparation.
            loop {
                let [next] = model.blocks[start].next.as_slice() else {
                    return false;
                };
                start = *next;
                if matches!(&model.blocks[start].op,CfgOp::Copy(pairs) if pairs.iter().any(|(t,_)| *t == target))
                {
                    break;
                }
            }
        }
        let mut seen = HashSet::default();
        let mut queue: VecDeque<_> = model.blocks[start]
            .next
            .iter()
            .map(|&i| (i, true))
            .collect();
        let mut escaped = false;
        while let Some((i, mut kept)) = queue.pop_front() {
            if !seen.insert((i, kept)) {
                continue;
            }
            match model.blocks[i].op {
                CfgOp::Resource(Op::Escape {
                    target: other, ..
                }) if other == target => kept = false,
                CfgOp::Replace(place)
                    if target_place.is_some_and(|p| model.places.contains(place, p)) =>
                {
                    kept = false;
                }
                CfgOp::Barrier if !any_path => kept = false,
                CfgOp::Resource(Op::Reset(_)) if matches!(kind, Origin::Temporary) => {
                    if kept {
                        return true;
                    }
                    continue;
                }
                CfgOp::Resource(Op::MayReset(_))
                    if matches!(kind, Origin::Temporary)
                        && cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                        && kept =>
                {
                    return true;
                }
                CfgOp::Resource(Op::Restore(_, _) | Op::MayRestore(_, _))
                    if process_exit
                        && matches!(kind, Origin::Temporary)
                        && kept
                        && self.restore_invalidates(i) =>
                {
                    return true;
                }
                CfgOp::Resource(Op::Exit) => {
                    if process_exit {
                        continue;
                    }
                    if !kept && !any_path {
                        return false;
                    }
                    if kept && any_path {
                        return true;
                    }
                    escaped |= kept;
                    continue;
                }
                CfgOp::Resource(Op::ScopeEnd(parent, _)) if matches!(kind,Origin::Stack(child) if inside(cx,child,parent)) =>
                {
                    if !kept {
                        return false;
                    }
                    escaped = true;
                    continue;
                }
                CfgOp::Resource(Op::StatementEnd(span)) if matches!(kind,Origin::Statement(at) if contains(span,at)) =>
                {
                    if !kept {
                        return false;
                    }
                    escaped = true;
                    continue;
                }
                _ => {}
            }
            if any_path && !kept {
                // An overwritten origin cannot become retained again on this path.
                continue;
            }
            queue.extend(model.blocks[i].next.iter().map(|&i| (i, kept)));
        }
        escaped
    }

    fn restore_invalidates(&self, at: usize) -> bool {
        let CfgOp::Resource(op) = &self.model.blocks[at].op else {
            return false;
        };
        self.states[at].as_ref().is_some_and(|state| {
            super::kills(
                self.cx,
                op,
                self.kind,
                self.region,
                &state[..=self.relevant.len() * 2],
                &|cell| self.relevant.get(&cell).copied(),
            )
        })
    }
}


fn expression_name(cx: &Cx, body: &LifetimeBody, span: Span) -> String {
    let mut source = span;
    if let Some(syntax) = &body.syntax.body {
        jaic::ast::walk::walk(&syntax.stmts, &mut |node, _| {
            if let Some(expr) = node.expr()
                && expr.span == span
                && let jaic::ast::ExprKind::Call {
                    callee, ..
                } = &expr.kind
            {
                source = callee.span;
            }
            true
        });
    }
    cx.compiler.src.sources.snippet(source).trim().to_owned()
}


fn map_finding(finding: &mut Finding, raw: Span, site: Span) {
    finding.start = site.start as usize;
    finding.end = site.end as usize;
    finding.fix = None;
    finding
        .secondary
        .push((raw, "expanded code uses the storage here".into()));
}

fn inferred_temporary_return(cx: &Cx, body: &LifetimeBody, model: &Model, op: &CfgOp) -> bool {
    let CfgOp::Resource(Op::Return {
        result,
        cell,
        ..
    }) = op
    else {
        return false;
    };
    let Some(summary) = cx
        .run
        .lifetimes
        .get()
        .and_then(|s| s.procs.get(&body.procedure))
    else {
        return false;
    };
    if summary.may_returns.contains(result) {
        return true;
    }
    if summary
        .returns
        .get(result)
        .is_some_and(|(p, g)| g.is_none() && p.root == super::Root::Temporary)
    {
        return true;
    }
    let super::Cell::Place(place) = model.cells.values[*cell] else {
        return false;
    };
    let mut prefix = Vec::new();
    for projection in &model.places.paths[place].1 {
        match projection {
            crate::rules::lifetime_places::Projection::Field(name) => {
                prefix.push(crate::rules::lifetime_summary::Step::Field(*name));
            }
            crate::rules::lifetime_places::Projection::Element(index) => {
                prefix.push(crate::rules::lifetime_summary::Step::Element(*index));
            }
            _ => break,
        }
        if summary
            .return_fields
            .get(&(*result, prefix.clone()))
            .is_some_and(|p| p.root == super::Root::Temporary)
            || summary
                .may_return_fields
                .contains(&(*result, prefix.clone()))
        {
            return true;
        }
    }
    let steps = model.places.paths[place]
        .1
        .iter()
        .map(|p| match p {
            crate::rules::lifetime_places::Projection::Field(n) => {
                Some(crate::rules::lifetime_summary::Step::Field(*n))
            }
            crate::rules::lifetime_places::Projection::Element(n) => {
                Some(crate::rules::lifetime_summary::Step::Element(*n))
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    steps.is_some_and(|mut steps| {
        let known = |steps: &[crate::rules::lifetime_summary::Step]| {
            summary
                .may_return_fields
                .contains(&(*result, steps.to_vec()))
                || summary
                    .return_fields
                    .get(&(*result, steps.to_vec()))
                    .is_some_and(|p| p.root == super::Root::Temporary)
        };
        if known(&steps) {
            return true;
        }
        // Resource copies can introduce a descriptor's data child after typed
        // places were indexed. Recover its parent type from the retained root.
        if descriptor_data_place(cx, model, place) {
            steps.pop();
            return known(&steps);
        }
        false
    })
}

fn descriptor_data_place(cx: &Cx, model: &Model, place: usize) -> bool {
    use crate::rules::lifetime_places::Projection;
    let ((_, root_ty), path) = &model.places.paths[place];
    let mut ty = *root_ty;
    let Some((Projection::Field(name), prefix)) = path.split_last() else {
        return false;
    };
    if name.as_str() != "data" {
        return false;
    }
    for projection in prefix {
        ty = match projection {
            Projection::Field(name) => {
                let Some(st) = cx.compiler.ty.types.as_struct(ty) else {
                    return false;
                };
                let Some(field) = cx
                    .compiler
                    .ty
                    .types
                    .struct_info(st)
                    .fields
                    .iter()
                    .find(|f| f.name == Some(*name))
                else {
                    return false;
                };
                field.ty
            }
            Projection::Element(_) | Projection::DynamicElement(_) => {
                let jaic::types::TypeKind::Array {
                    elem, ..
                } = cx.compiler.ty.types.kind(ty)
                else {
                    return false;
                };
                *elem
            }
            Projection::Deref => {
                let Some(pointee) = cx.compiler.ty.types.pointee(ty) else {
                    return false;
                };
                pointee
            }
        };
    }
    matches!(
        cx.compiler.ty.types.kind(ty),
        jaic::types::TypeKind::String
            | jaic::types::TypeKind::Array {
                kind: jaic::types::ArrayKind::View | jaic::types::ArrayKind::Resizable,
                ..
            }
    )
}

/// Only the selected process entry's outer activation lasts until process exit.
/// A nested lexical scope, or a callable/recursive entry, still has an ordinary lifetime.
fn entrypoint_storage(cx: &Cx, body: &LifetimeBody, kind: Origin) -> bool {
    let Origin::Stack(scope) = kind else {
        return false;
    };
    let mut scope = &cx.compiler.ns.scopes[scope.0 as usize];
    if scope.kind == jaic::sema::scope::ScopeKind::Block
        && let Some(parent) = scope.parent
    {
        scope = &cx.compiler.ns.scopes[parent.0 as usize];
    }
    scope.kind == jaic::sema::scope::ScopeKind::Proc
        && scope.proc == Some(body.procedure)
        && entrypoint_activation(cx, body)
}

fn entrypoint_activation(cx: &Cx, body: &LifetimeBody) -> bool {
    let proc = &cx.compiler.pr.procs[body.procedure.0 as usize];
    let selected = cx.compiler.pr.entry_point == Some(body.procedure)
        || cx.compiler.pr.entry_point.is_none()
            && proc.name.as_str() == cx.compiler.options.entry_point
            && [cx.compiler.ns.main_module, cx.run.lifetime_entry_module]
                .contains(&Some(cx.compiler.ns.scopes[proc.scope.0 as usize].module));
    if !cx.compiler.options.runtime_entry_point
        || !selected
        || proc.export.is_some()
        || proc.sig.as_ref().is_none_or(|s| !s.params.is_empty())
    {
        return false;
    }
    let Some(facts) = cx.compiler.ed.facts() else {
        return false;
    };
    let summaries = cx.run.lifetimes.get().unwrap();
    !facts.lifetime_bodies.values().any(|caller| {
        caller
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .any(|e| match e {
                Event::Call {
                    callee, ..
                } => *callee == body.procedure,
                Event::IndirectCall {
                    callee,
                    allocator: false,
                    ..
                } => summaries
                    .targets
                    .get(caller.procedure, *callee)
                    .procedures
                    .contains(&body.procedure),
                _ => false,
            })
    })
}

/// The members that every witness released, when each one released only named members of its
/// argument.
fn released_members(model: &Model, witnesses: &[(usize, Span)]) -> Option<Vec<jaic::intern::Sym>> {
    let mut all = Vec::new();
    for (j, _) in witnesses {
        let CfgOp::Resource(Op::Free {
            contents: true,
            fields: Some(fields),
            ..
        }) = &model.blocks[*j].op
        else {
            return None;
        };
        all.extend(fields.iter().copied());
    }
    (!all.is_empty()).then_some(all)
}

/// The member read next in `text`: with `after`, `text` follows the used expression (`.names`);
/// otherwise it starts with a variable (`*r.names`).
fn used_member(text: &str, after: bool) -> Option<&str> {
    let ident = |t: &str| {
        t.find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(t.len())
    };
    let rest = if after {
        text.strip_prefix('.')?
    } else {
        let text = text.trim_start_matches(['*', '&']);
        text[ident(text)..].strip_prefix('.')?
    };
    let member = &rest[..ident(rest)];
    (!member.is_empty()).then_some(member)
}
