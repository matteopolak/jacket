//! On-demand cut witnesses and labelled source locations.
use super::{Cx, Finding, HashMap, Model, Op, Span, dead, finding, invalidation_cut};

pub(super) fn emit(
    cx: &Cx,
    model: &Model,
    candidate: usize,
    states: &[Option<Vec<u8>>],
    relevant: &HashMap<usize, usize>,
    region_name: &str,
    resizable_only: bool,
) -> Vec<Finding> {
    let blocks = &model.blocks;
    let chains = &model.chains;
    let (region, origin, element) = match blocks[candidate].op {
        Op::Borrow(_, region, origin) | Op::PointerLoad(_, _, region, origin) => {
            (region, origin, None)
        }
        Op::ElementBorrow(_, region, index, length, origin)
        | Op::LogicalBorrow(_, region, index, length, origin) => {
            (region, origin, Some((index, length)))
        }
        _ => return Vec::new(),
    };
    let mut reports: Vec<Finding> = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        let Op::Use(id, at) = block.op else {
            continue;
        };
        let source = at;
        let at = if at.file == cx.file {
            at
        } else if let Some(site) = block.site.filter(|s| s.file == cx.file) {
            site
        } else {
            continue;
        };
        let Some(&cell) = relevant.get(&id) else {
            continue;
        };
        if states[i].as_ref().is_none_or(|s| !dead(cx, s[cell])) {
            continue;
        }
        let growths = invalidation_cut(
            model,
            states,
            relevant,
            i,
            cell,
            region,
            if resizable_only {
                None
            } else {
                element
            },
        );
        if growths.is_empty() {
            continue;
        }
        let logical = !resizable_only && growths.iter().any(|at| {
            blocks
                .iter()
                .any(|b| matches!(b.op, Op::Mutate(_, m, span) if span == *at && element.is_some_and(|(s, len)| crate::rules::lifetime_movement::invalidates(m, s, len))))
        });
        let explanation = if cx.lifetime_mode == jacket_config::LifetimeMode::Strict
            && states[i].as_ref().is_some_and(|s| s[cell] != super::DEAD)
        {
            format!(
                "strict mode: `{}` may outlive the storage or element it refers to along a bad path",
                cx.src(at)
            )
        } else if logical {
            format!(
                "`{}` refers to an element in `{region_name}` whose identity is invalidated on every path to this use",
                cx.src(at)
            )
        } else if resizable_only && growths.len() == 1 {
            format!(
                "`{}` points into `{region_name}`, which `{}` may have moved by now",
                cx.src(at),
                cx.src(growths[0]).split('(').next().unwrap_or("the call")
            )
        } else if growths.len() == 1 {
            format!(
                "`{}` points into `{region_name}`, which may be resized: {}",
                cx.src(at),
                cx.compiler.src.sources.snippet(growths[0]).trim()
            )
        } else {
            format!(
                "`{}` points into `{region_name}`; every path to this use passes one of {} invalidating calls",
                cx.src(at),
                growths.len()
            )
        };
        let mut report = finding(
            at,
            explanation,
            Some(
                if logical {
                    "take the pointer again after the operation, or keep a stable key instead"
                } else if resizable_only {
                    "take the pointer again after the array has grown, or keep the index instead"
                } else {
                    "take the pointer again after growth, or store an index instead"
                }
                .into(),
            ),
        );
        report.label = Some(
            if logical {
                "pointer used here after its element was moved or removed"
            } else {
                "pointer used here after its storage was invalidated"
            }
            .into(),
        );
        if source != at {
            report
                .secondary
                .push((source, "expanded code uses the pointer here".into()));
        }
        report
            .secondary
            .push((origin, "pointer derived here".into()));
        super::witness::targets(cx, model, &growths, &mut report);
        if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
            let (decisions, count) =
                super::witness::strict_path(model, states, relevant, candidate, i, cell, &growths);
            report
                .message
                .push_str(&format!("; shortest bad path shown ({count})"));
            for (site, taken) in decisions {
                report.secondary.push((
                    site,
                    format!("shortest bad path takes this condition as {taken}"),
                ));
            }
            if growths.iter().any(|s| {
                blocks
                    .iter()
                    .any(|b| matches!(b.op, Op::MayReach(_, at) if at == *s))
            }) {
                report.message.push_str(
                    "; unknown or foreign code may invalidate storage reachable from its arguments",
                );
            }
        }
        for growth in growths {
            if let Some(chain) = chains.get(&(growth, region)) {
                for &site in chain {
                    if site == Span::NONE {
                        report
                            .message
                            .push_str(" (inferred chain truncated in the middle)");
                    } else if site != growth {
                        report
                            .secondary
                            .push((site, "invalidation inferred through this call".into()));
                    }
                }
            }
            report.secondary.push((
                growth,
                if logical {
                    format!("`{region_name}` element moved or removed here")
                } else {
                    format!("`{region_name}` storage invalidated here")
                },
            ));
        }
        if !reports.iter().any(|f| {
            f.start >= at.start as usize
                && f.end <= at.end as usize
                && f.secondary == report.secondary
        }) {
            reports.push(report);
        }
    }
    // The caller merges all candidates for this rule.
    reports
}
