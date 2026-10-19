//! Compiler messages: interception, the event stream and `report`.

#[cfg(target_arch = "wasm32")]
use super::state::{INTERCEPT_REPORT_POLYMORPHS, INTERCEPT_REPORT_RUNS};
use super::{Call, Interp, MetaOp, ReportSeverity, Rets, Stage, TOP_LEVEL_WORKSPACE, Trap, step};

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let trap = Trap::new;
    let shared = cx.shared;
    let arg = |i: usize| cx.arg(i);
    let text = |interp: &Interp, i: usize| cx.text(interp, i);
    let return_string =
        |interp: &mut Interp, bytes: &[u8], i: usize| cx.return_string(interp, bytes, i);
    match op {
        MetaOp::BeginIntercept => {
            #[cfg(target_arch = "wasm32")]
            if arg(1) as i64 & (INTERCEPT_REPORT_RUNS | INTERCEPT_REPORT_POLYMORPHS) != 0 {
                return Err(trap("performance reporting requires a host monotonic clock; this WASM compiler host does not provide one".into()));
            }
            let mut reg = shared.borrow_mut();
            let ws = reg.ws(arg(0) as i64).map_err(trap)?;
            ws.intercepted = true;
            ws.intercept_flags = arg(1) as i64;
            Ok(Rets::default())
        }
        MetaOp::EndIntercept => {
            shared
                .borrow_mut()
                .ws(arg(0) as i64)
                .map_err(trap)?
                .intercepted = false;
            Ok(Rets::default())
        }
        MetaOp::NextEvent => {
            let id = arg(0) as i64;
            if id == shared.borrow().current_id() || id == TOP_LEVEL_WORKSPACE {
                // A workspace cannot wait on its own compilation.
                return Ok(Rets::one(0));
            }
            loop {
                let mut reg = shared.borrow_mut();
                let ws = reg.ws(id).map_err(trap)?;
                if let Some(event) = ws.events.pop_front() {
                    let kind = event.kind;
                    reg.event = event;
                    return Ok(Rets::one(kind as u64));
                }
                if ws.stage == Stage::Done {
                    return Ok(Rets::one(0));
                }
                drop(reg);
                let mut budget = interp.block_budget;
                let stepped = step(shared, id, &mut budget);
                interp.block_budget = budget;
                stepped.map_err(trap)?;
            }
        }
        MetaOp::EventInt => {
            let reg = shared.borrow();
            let at = arg(0) as usize;
            match reg.event.ints.get(at) {
                Some(&value) => Ok(Rets::one(value as u64)),
                None => Err(Trap::new(format!("the message has no integer field {at}"))),
            }
        }
        MetaOp::EventString => {
            let at = arg(0) as usize;
            let Some(bytes) = shared.borrow().event.strings.get(at).cloned() else {
                return Err(Trap::new(format!("the message has no string field {at}")));
            };
            return_string(interp, &bytes, 1)?;
            Ok(Rets::default())
        }
        MetaOp::Report => {
            let (message, file) = (text(interp, 0)?, text(interp, 1)?);
            // `mode`: `Report` in the Compiler module (ERROR, ERROR_CONTINUABLE, WARNING, INFO).
            let (line, column, mode) = (arg(2), arg(3), arg(4));
            let location = if file.is_empty() {
                String::new()
            } else {
                format!("{file}:{line}:{column}: ")
            };
            if mode == 0 {
                let mut t = trap(message.trim_end().to_string());
                t.reported = Some(Box::new((file, line as u32, column as u32)));
                return Err(t);
            }
            let severity = match mode {
                1 => ReportSeverity::Error,
                2 => ReportSeverity::Warning,
                _ => ReportSeverity::Info,
            };
            let label = match severity {
                ReportSeverity::Error => "error",
                ReportSeverity::Warning => "warning",
                ReportSeverity::Info => "info",
            };
            let rendered = format!("{location}{label}: {}", message.trim_end());
            let mut reg = shared.borrow_mut();
            (reg.env.report)(severity, &rendered);
            if mode == 1 {
                // A continuable error: the metaprogram goes on, but the build has failed.
                let current = reg.current_id();
                reg.ws(current).map_err(trap)?.failed = true;
            }
            Ok(Rets::default())
        }
        _ => unreachable!("not a messages primitive"),
    }
}
