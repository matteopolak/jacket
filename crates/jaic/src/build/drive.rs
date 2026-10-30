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
use crate::sema::{Compiler, Modification, ProgramSource};
use crate::source::Diagnostic;

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
    nested: bool,
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
    let Some(StepInputs {
        stage,
        compiler,
        pending,
        modifications,
    }) = take_step_inputs(shared, id)?
    else {
        return Ok(());
    };
    let mut compiler = match compiler {
        Some(c) => c,
        None => new_compiler(shared, id)?,
    };
    compiler.ct.interp.block_budget = *budget;
    compiler.ct.interp.nested_eval = nested;
    // The registry borrow is released while the compiler runs: its
    // compile-time code may call back into the registry.
    shared.borrow_mut().current.push(id);
    let original_sources = pending.clone();
    let mut events = Vec::new();
    let (result, lowering) = advance(
        shared,
        id,
        &mut compiler,
        stage,
        (pending, modifications),
        &mut events,
    );
    // Read after the compiler ran: its compile-time code may have asked for interception.
    let intercepted = shared.borrow_mut().ws(id)?.intercepted;
    let (mut file_events, reported) =
        export_checked_events(shared, &mut compiler, &result, intercepted, &mut events);
    let result = finish_lowering(
        shared,
        &mut compiler,
        result,
        (lowering, reported, intercepted),
        &mut events,
    );
    shared.borrow_mut().current.pop();
    *budget = compiler.ct.interp.block_budget;
    let outcome = settle(
        shared,
        id,
        &mut compiler,
        (result, stage, intercepted),
        &mut events,
    )?;
    file_events.extend(events);
    store_step(shared, id, compiler, outcome, file_events, original_sources)
}

/// What the compiler of one step produced once its result was settled.
struct Outcome {
    next: Stage,
    failed: bool,
    /// The error text of a load that failed on imports a metaprogram may still provide.
    held: Option<String>,
}

/// What a step takes out of the registry: the workspace's stage, its compiler (none before the
/// first step), the sources queued since the last step and its procedure modifications.
struct StepInputs {
    stage: Stage,
    compiler: Option<Box<Compiler>>,
    pending: Vec<ProgramSource>,
    modifications: Vec<Modification>,
}

/// `None` when the workspace is finished.
fn take_step_inputs(shared: &SharedWorkspaces, id: i64) -> Result<Option<StepInputs>, String> {
    let mut reg = shared.borrow_mut();
    let ws = reg.ws(id)?;
    if ws.stage == Stage::Done {
        return Ok(None);
    }
    Ok(Some(StepInputs {
        stage: ws.stage,
        compiler: ws.compiler.take(),
        pending: std::mem::take(&mut ws.pending),
        modifications: std::mem::take(&mut ws.modifications),
    }))
}

/// Apply the modifications and move the compiler through `stage`. The flag says the compiler
/// ran out of sources and lowered what is reachable, so the program may now be complete.
fn advance(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: &mut Compiler,
    stage: Stage,
    (mut pending, modifications): (Vec<ProgramSource>, Vec<Modification>),
    events: &mut Vec<Event>,
) -> (Result<Stage, Box<Diagnostic>>, bool) {
    let modified = modifications
        .into_iter()
        .try_for_each(|m| compiler.modify_procedure(m))
        .and_then(|()| compiler.relower_modified());
    if stage == Stage::Open {
        let reg = shared.borrow();
        let settings = &reg.list[id as usize].settings;
        if wants_wasi_runtime(settings.llvm_target) {
            pending.push(ProgramSource::String(WASI_RUNTIME_IMPORT.into()));
        }
    }
    let mut lowering = false;
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
    if let Ok(mut reg) = shared.try_borrow_mut()
        && reg.record_files
    {
        reg.read_files.extend(
            compiler
                .src
                .sources
                .files()
                .map(std::path::Path::to_path_buf),
        );
    }
    (result, lowering)
}

/// Whether the metaprogram asked for the full `TYPECHECKED` records (it names the message).
fn typechecked_in_full(shared: &SharedWorkspaces) -> bool {
    shared.borrow().typechecked_use.get() & crate::source::TYPECHECKED_NAMED != 0
}

fn file_event(kind: i64, record: i64) -> Event {
    record_event(
        if kind == crate::sema::code_export::message_kind::IMPORT {
            EVENT_IMPORT
        } else {
            EVENT_FILE
        },
        record,
    )
}

/// For a workspace the metaprogram intercepts: the file and import events the step loaded,
/// and the `TYPECHECKED` message (inserted before the last phase event). Returns the file
/// events, which go out first, and whether `TYPECHECKED` was sent.
fn export_checked_events(
    shared: &SharedWorkspaces,
    compiler: &mut Compiler,
    result: &Result<Stage, Box<Diagnostic>>,
    intercepted: bool,
    events: &mut Vec<Event>,
) -> (Vec<Event>, bool) {
    let mut file_events = Vec::new();
    let mut reported = false;
    if !intercepted {
        return (file_events, reported);
    }
    // The registry's records are taken out while the compiler exports:
    // resolving declarations may run compile-time code.
    let mut records = Records::lend(&mut shared.borrow_mut().records);
    for (kind, record) in compiler.export_file_events(&mut records) {
        file_events.push(file_event(kind, record));
    }
    // After the final lowering too: procedure bodies are reported once lowered.
    if matches!(result, Ok(Stage::Checked | Stage::Done))
        && let Some(message) =
            compiler.export_typechecked_with(&mut records, typechecked_in_full(shared))
    {
        reported = true;
        // Before TYPECHECKED_ALL_WE_CAN, after any files the export loaded.
        for (kind, record) in compiler.export_file_events(&mut records) {
            file_events.push(file_event(kind, record));
        }
        let at = events.len().saturating_sub(1);
        events.insert(at, record_event(EVENT_TYPECHECKED, message));
    }
    Records::give_back(&mut shared.borrow_mut().records, records);
    (file_events, reported)
}

/// After lowering what is reachable: if that reported more, the metaprogram gets another
/// `TYPECHECKED_ALL_WE_CAN` to add code; otherwise the program is generated.
fn finish_lowering(
    shared: &SharedWorkspaces,
    compiler: &mut Compiler,
    result: Result<Stage, Box<Diagnostic>>,
    (lowering, reported, intercepted): (bool, bool, bool),
    events: &mut Vec<Event>,
) -> Result<Stage, Box<Diagnostic>> {
    match result {
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
                if let Some(message) =
                    compiler.export_typechecked_with(&mut records, typechecked_in_full(shared))
                {
                    events.push(record_event(EVENT_TYPECHECKED, message));
                }
                Records::give_back(&mut shared.borrow_mut().records, records);
            }
            finished
        }
        other => other,
    }
}

/// Turn the step's result into the next stage: write the output of a finished program, or
/// report the error (or hold it back for imports the metaprogram may still provide).
fn settle(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: &mut Compiler,
    (result, stage, intercepted): (Result<Stage, Box<Diagnostic>>, Stage, bool),
    events: &mut Vec<Event>,
) -> Result<Outcome, String> {
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
            match write_output(shared, id, compiler, events)? {
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
            events.extend(performance_event(shared, id, compiler));
        }
        events.push(Event {
            kind: EVENT_COMPLETE,
            ints: vec![failed as i64],
            strings: Vec::new(),
        });
    }
    Ok(Outcome {
        next,
        failed,
        held,
    })
}

/// Put the step's events and stage back in the registry, and the compiler with them unless the
/// workspace is finished (an observer takes a finished compiler).
fn store_step(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: Box<Compiler>,
    outcome: Outcome,
    file_events: Vec<Event>,
    original_sources: Vec<ProgramSource>,
) -> Result<(), String> {
    let Outcome {
        next,
        failed,
        held,
        ..
    } = outcome;
    {
        let mut reg = shared.borrow_mut();
        // A metaprogram that asks for type tables may ask after `COMPLETE`: a finished
        // workspace keeps its compiler then; an observer gets it when the metaprogram is done
        // (`hand_over_kept`).
        let keep = reg.type_table_used;
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
        // The program the embedder runs keeps its compiler.
        let run = reg.run_target == Some(id) && reg.run_ready && !failed;
        let ws = reg.ws(id)?;
        if next != Stage::Done || keep || run {
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
