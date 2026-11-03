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
                let contents = witnesses.iter().any(|(j, _)| {
                    matches!(
                        model.blocks[*j].op,
                        CfgOp::Resource(Op::Free {
                            contents: true,
                            ..
                        })
                    )
                });
                let suffix = &cx.compiler.src.sources.get(at.file).text[at.end as usize..];
                if contents
                    && !matches!(block.op, CfgOp::Resource(Op::Read { .. }))
                    && (suffix.trim_start().starts_with(".* =")
                        || (!suffix.starts_with('.') && !suffix.starts_with('[')))
                {
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

        if matches!(kind, Origin::Stack(_) | Origin::Statement(_)) {
            if let CfgOp::Resource(Op::Escape {
                target,
                store,
                ..
            }) = block.op
                && !escape_survives(cx, model, i, target, store, kind)
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


fn escape_survives(
    cx: &Cx,
    model: &Model,
    at: usize,
    target: usize,
    store: bool,
    kind: Origin,
) -> bool {
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
            CfgOp::Barrier => kept = false,
            CfgOp::Resource(Op::Exit) => {
                if !kept {
                    return false;
                }
                escaped = true;
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
        queue.extend(model.blocks[i].next.iter().map(|&i| (i, kept)));
    }
    escaped
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
