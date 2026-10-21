//! Advancing a workspace one step at a time and resuming failed loads.
use super::output::{Written, finish_custom_link, write_output};
use super::registry::{WASI_RUNTIME_IMPORT, new_compiler, phase, record_event, wants_wasi_runtime};
use super::settings::ReportSeverity;
use super::state::{
    EVENT_COMPLETE, EVENT_DEBUG_DUMP, EVENT_ERROR, EVENT_FAILED_IMPORT, EVENT_FILE, EVENT_IMPORT,
    EVENT_PERFORMANCE_REPORT, EVENT_TYPECHECKED, Event, FailedLoad, INTERCEPT_REPORT_POLYMORPHS,
    INTERCEPT_REPORT_RUNS, PHASE_ALL_SOURCE_CODE_PARSED, PHASE_TYPECHECKED_ALL_WE_CAN,
    SharedWorkspaces, Stage,
};
use crate::records::Records;
use crate::sema::{Compiler, ProgramSource};

/// Advance workspace `id` by one step, queueing the messages it produces:
/// Open → (load, run directives) Checked → (more sources, or generate code
/// and write output) Done. Errors are reported and end in a failed COMPLETE.
///
/// `budget` is the interpreter block budget of whoever drives the workspace (the metaprogram's
/// interpreter, or the top-level compiler). The workspace's compile-time code draws from it
/// and hands back what is left, so one budget bounds the whole compilation however many
/// workspaces a metaprogram creates.
pub(super) fn step(
    shared: &SharedWorkspaces,
    id: i64,
    budget: &mut Option<u64>,
) -> Result<(), String> {
    if shared.borrow().current.contains(&id) {
        return Err(format!(
            "compile-time workspace dependency cycle: workspace {id} is already active"
        ));
    }
    if shared.borrow_mut().ws(id)?.link.is_some() {
        return finish_custom_link(shared, id);
    }
    if shared.borrow_mut().ws(id)?.failed_load.is_some() && resume_failed_load(shared, id)? {
        return Ok(());
    }
    let (stage, mut compiler, pending, modifications) = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        if ws.stage == Stage::Done {
            return Ok(());
        }
        (
            ws.stage,
            ws.compiler.take(),
            std::mem::take(&mut ws.pending),
            std::mem::take(&mut ws.modifications),
        )
    };
    let mut lowering = false;
    let mut compiler = match compiler.take() {
        Some(c) => c,
        None => new_compiler(shared, id)?,
    };
    compiler.ct.interp.block_budget = *budget;
    let mut events = Vec::new();
    // The registry borrow is released while the compiler runs: its
    // compile-time code may call back into the registry.
    shared.borrow_mut().current.push(id);
    let modified = {
        let reg = shared.borrow();
        modifications
            .iter()
            .try_for_each(|(body, stmts)| compiler.modify_procedure(&reg.records, *body, stmts))
    }
    .and_then(|()| compiler.relower_modified());
    let mut pending = pending;
    let original_sources = pending.clone();
    if stage == Stage::Open {
        let reg = shared.borrow();
        let settings = &reg.list[id as usize].settings;
        if wants_wasi_runtime(settings.llvm_target) {
            pending.push(ProgramSource::String(WASI_RUNTIME_IMPORT.into()));
        }
    }
    let result = modified.and_then(|()| match stage {
        Stage::Open => compiler.begin_sources(&pending).map(|()| {
            events.push(phase(PHASE_ALL_SOURCE_CODE_PARSED));
            events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
            Stage::Checked
        }),
        Stage::Checked if !pending.is_empty() => pending
            .iter()
            .try_for_each(|source| compiler.add_source(source))
            .and_then(|()| compiler.settle())
            .map(|()| {
                events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
                Stage::Checked
            }),
        // Out of sources: lower what is reachable first; what that reports may make
        // the metaprogram add more before the program is generated.
        Stage::Checked => {
            compiler.lower_reachable();
            lowering = true;
            Ok(Stage::Checked)
        }
        Stage::Done => unreachable!(),
    });
    let intercepted = shared.borrow_mut().ws(id)?.intercepted;
    let full = |shared: &SharedWorkspaces| {
        shared.borrow().typechecked_use.get() & crate::source::TYPECHECKED_NAMED != 0
    };
    let mut file_events = Vec::new();
    let mut reported = false;
    if intercepted {
        // The registry's records are taken out while the compiler exports:
        // resolving declarations may run compile-time code.
        let mut records = Records::lend(&mut shared.borrow_mut().records);
        for (kind, record) in compiler.export_file_events(&mut records) {
            file_events.push(record_event(
                if kind == crate::sema::code_export::message_kind::IMPORT {
                    EVENT_IMPORT
                } else {
                    EVENT_FILE
                },
                record,
            ));
        }
        // After the final lowering too: procedure bodies are reported once lowered.
        if matches!(result, Ok(Stage::Checked | Stage::Done))
            && let Some(message) = compiler.export_typechecked_with(&mut records, full(shared))
        {
            reported = true;
            // Before TYPECHECKED_ALL_WE_CAN, after any files the export loaded.
            for (kind, record) in compiler.export_file_events(&mut records) {
                file_events.push(record_event(
                    if kind == crate::sema::code_export::message_kind::IMPORT {
                        EVENT_IMPORT
                    } else {
                        EVENT_FILE
                    },
                    record,
                ));
            }
            let at = events.len().saturating_sub(1);
            events.insert(at, record_event(EVENT_TYPECHECKED, message));
        }
        Records::give_back(&mut shared.borrow_mut().records, records);
    }
    // Lowering reported more: the metaprogram gets another TYPECHECKED_ALL_WE_CAN to
    // add code. Otherwise the program is complete.
    let result = match result {
        Ok(_) if lowering && reported => {
            events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
            Ok(Stage::Checked)
        }
        Ok(_) if lowering => {
            let finished = compiler.finish_program().map(|()| Stage::Done);
            // Checking what nothing reaches lowered more bodies: they are reported too, as the
            // last TYPECHECKED message.
            if finished.is_ok() && intercepted {
                let mut records = Records::lend(&mut shared.borrow_mut().records);
                if let Some(message) = compiler.export_typechecked_with(&mut records, full(shared))
                {
                    events.push(record_event(EVENT_TYPECHECKED, message));
                }
                Records::give_back(&mut shared.borrow_mut().records, records);
            }
            finished
        }
        other => other,
    };
    shared.borrow_mut().current.pop();
    *budget = compiler.ct.interp.block_budget;
    let mut failed = false;
    let mut held = None;
    let mut compiled = false;
    let next = match result {
        Ok(Stage::Done) => {
            // The procedures declared `#dump` are lowered; their listings go out before the
            // program is written.
            if intercepted {
                for text in compiler.debug_dumps() {
                    events.push(Event {
                        kind: EVENT_DEBUG_DUMP,
                        ints: Vec::new(),
                        strings: vec![text.into_bytes()],
                    });
                }
            }
            match write_output(shared, id, &mut compiler, &mut events)? {
                Written::Done(ok) => {
                    failed = !ok;
                    compiled = true;
                    Stage::Done
                }
                // The metaprogram links; `finish_custom_link` ends the workspace.
                Written::AwaitingLink => Stage::Checked,
            }
        }
        Ok(next) => next,
        Err(d) => {
            let text = compiler.render(&d);
            // An editor compile (see `WorkspaceObserver`) keeps the error with its facts.
            if let Some(ide) = compiler.ed.facts_mut() {
                ide.errors.push((*d).clone());
            }
            let mut imports = Vec::new();
            if intercepted {
                // The imports that found nothing, then the error itself.
                let mut records = Records::lend(&mut shared.borrow_mut().records);
                imports = compiler.export_failed_imports(&mut records);
                Records::give_back(&mut shared.borrow_mut().records, records);
                for &record in &imports {
                    events.push(record_event(EVENT_FAILED_IMPORT, record));
                }
            }
            if !imports.is_empty() && stage == Stage::Open {
                // The metaprogram may `provide_import` what was missing: the error waits for
                // its answer (`resume_failed_load`).
                held = Some(text);
                Stage::Open
            } else {
                (shared.borrow_mut().env.report)(ReportSeverity::Error, &text);
                failed = true;
                if intercepted {
                    events.push(Event {
                        kind: EVENT_ERROR,
                        ints: Vec::new(),
                        strings: Vec::new(),
                    });
                }
                Stage::Done
            }
        }
    };
    if next == Stage::Done {
        if compiled {
            events.extend(performance_event(shared, id, &mut compiler));
        }
        events.push(Event {
            kind: EVENT_COMPLETE,
            ints: vec![failed as i64],
            strings: Vec::new(),
        });
    }
    file_events.extend(events);
    {
        let mut reg = shared.borrow_mut();
        // A metaprogram that asks for type tables may ask after `COMPLETE`: a finished
        // workspace keeps its compiler then, unless an observer takes it.
        let keep = reg.type_table_used && reg.env.observer.is_none();
        let ws = reg.ws(id)?;
        ws.events.extend(file_events);
        ws.failed |= failed;
        ws.stage = next;
        if let Some(text) = held {
            let offers = ws.provided.len();
            ws.failed_load = Some(FailedLoad {
                text,
                sources: original_sources,
                compiler,
                offers,
            });
            return Ok(());
        }
        if next != Stage::Done || keep {
            ws.compiler = Some(compiler);
            return Ok(());
        }
    }
    let observer = shared.borrow_mut().env.observer.take();
    if let Some(mut o) = observer {
        o.finished(compiler, failed);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(())
}

/// The `PERFORMANCE_REPORT` message of a workspace that compiled, when its metaprogram asked for
/// one with `DO_PERFORMANCE_REPORT_RUNS` or `_POLYMORPHS`: sent after everything else and right
/// before `COMPLETE`.
pub(super) fn performance_event(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: &mut Compiler,
) -> Option<Event> {
    let flags = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id).ok()?;
        if !ws.intercepted {
            return None;
        }
        ws.intercept_flags
    };
    let runs = flags & INTERCEPT_REPORT_RUNS != 0;
    let polymorphs = flags & INTERCEPT_REPORT_POLYMORPHS != 0;
    if !runs && !polymorphs {
        return None;
    }
    let mut records = Records::lend(&mut shared.borrow_mut().records);
    let record = compiler.export_performance_report(&mut records, runs, polymorphs);
    Records::give_back(&mut shared.borrow_mut().records, records);
    Some(record_event(EVENT_PERFORMANCE_REPORT, record))
}

/// A workspace held back by a failed import (see [`FailedLoad`]) is asked for its answer: when the
/// metaprogram has offered anything since, the load is tried again from scratch (true if it
/// was not and the workspace is finished instead); otherwise the error is reported and
/// followed by `ERROR` and `COMPLETE`.
fn resume_failed_load(shared: &SharedWorkspaces, id: i64) -> Result<bool, String> {
    let held = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        match ws.failed_load.take() {
            Some(held) if ws.provided.len() > held.offers => {
                ws.pending = held.sources;
                ws.compiler = None;
                return Ok(false);
            }
            other => other,
        }
    };
    let Some(held) = held else {
        return Ok(false);
    };
    (shared.borrow_mut().env.report)(ReportSeverity::Error, &held.text);
    {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        ws.events.push_back(Event {
            kind: EVENT_ERROR,
            ints: Vec::new(),
            strings: Vec::new(),
        });
        ws.events.push_back(Event {
            kind: EVENT_COMPLETE,
            ints: vec![1],
            strings: Vec::new(),
        });
        ws.failed = true;
        ws.stage = Stage::Done;
    }
    let observer = shared.borrow_mut().env.observer.take();
    if let Some(mut o) = observer {
        o.finished(held.compiler, true);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(true)
}
