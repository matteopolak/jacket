//! Writing a workspace's output, custom link steps and finishing all workspaces.
use super::drive::{performance_event, step};
use super::meta_op::TOP_LEVEL_WORKSPACE;
use super::registry::phase;
use super::settings::{BuildSettings, PendingLink, ReportSeverity, shown_path};
use super::state::{
    EVENT_COMPLETE, EVENT_PHASE, Event, PHASE_ALL_TARGET_CODE_BUILT, PHASE_POST_WRITE_EXECUTABLE,
    PHASE_PRE_WRITE_EXECUTABLE, PHASE_READY_FOR_CUSTOM_LINK_COMMAND, SharedWorkspaces, Stage,
};
use crate::builtin_keys::OutputType;
use crate::sema::Compiler;
use std::path::PathBuf;

/// What [`write_output`] did.
pub(super) enum Written {
    /// Whether the output was written (or not wanted).
    Done(bool),
    /// Objects are written and `READY_FOR_CUSTOM_LINK_COMMAND` queued: the metaprogram links.
    AwaitingLink,
}

/// Code generation is done: queue the write phases and call the backend.
pub(super) fn write_output(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: &mut Compiler,
    events: &mut Vec<Event>,
) -> Result<Written, String> {
    events.push(phase(PHASE_ALL_TARGET_CODE_BUILT));
    let settings = shared.borrow_mut().ws(id)?.settings.clone();
    if !settings.do_output || settings.output_type == OutputType::NoOutput {
        return Ok(Written::Done(true));
    }
    let output = output_path(&settings, compiler);
    let name = output.display().to_string().into_bytes();
    events.push(Event {
        kind: EVENT_PHASE,
        ints: vec![PHASE_PRE_WRITE_EXECUTABLE, 0],
        strings: vec![name.clone()],
    });
    // Output preparation prunes checked-but-unreferenced bodies and freezes
    // runtime globals for emission. A check-only embedder keeps this compiler
    // for further semantic queries; there is no output IR to prepare.
    if shared.borrow().env.backend.is_some() {
        compiler
            .prepare_compiled_output()
            .map_err(|error| compiler.render(&error))?;
    }
    let links = matches!(
        settings.output_type,
        OutputType::Executable | OutputType::DynamicLibrary
    );
    if settings.use_custom_link_command && links {
        let mut reg = shared.borrow_mut();
        if let Some(backend) = reg.env.backend.as_mut() {
            let objects = backend.write_objects(&compiler.program, &settings, &output);
            return match objects {
                Ok(o) => {
                    let mut strings: Vec<Vec<u8>> = Vec::new();
                    for list in [&o.objects, &o.system_libraries, &o.user_libraries] {
                        strings.extend(list.iter().map(|s| s.clone().into_bytes()));
                    }
                    strings.push(name);
                    events.push(Event {
                        kind: EVENT_PHASE,
                        ints: vec![
                            PHASE_READY_FOR_CUSTOM_LINK_COMMAND,
                            o.objects.len() as i64,
                            0,
                            o.system_libraries.len() as i64,
                            o.user_libraries.len() as i64,
                        ],
                        strings,
                    });
                    reg.ws(id)?.link = Some(PendingLink {
                        exit_code: None,
                        output: output.display().to_string(),
                    });
                    Ok(Written::AwaitingLink)
                }
                Err(message) => {
                    (reg.env.report)(
                        ReportSeverity::Error,
                        &format!("error: writing {}: {message}", output.display()),
                    );
                    Ok(Written::Done(false))
                }
            };
        }
    }
    let written = {
        let mut reg = shared.borrow_mut();
        match reg.env.backend.as_mut() {
            Some(backend) => backend.write_output(&compiler.program, &settings, &output),
            // No backend: checking only.
            None => {
                if let Some(hint) = reg.env.unwritten_output_hint.clone() {
                    let name = reg.ws(id)?.name.clone();
                    let kind = match settings.output_type {
                        OutputType::DynamicLibrary => "a dynamic library",
                        OutputType::StaticLibrary => "a static library",
                        OutputType::ObjectFile => "an object file",
                        _ => "an executable",
                    };
                    (reg.env.report)(
                        ReportSeverity::Warning,
                        &format!(
                            "warning: `jaic -check` does not write {} (workspace `{name}` asks for \
                         {kind})\n\
                         help: `jaic {main}` (or `jaic {main} -interp`) writes it",
                            shown_path(&output),
                            main = hint.main_file,
                        ),
                    );
                }
                Ok(())
            }
        }
    };
    if let Err(message) = &written {
        (shared.borrow_mut().env.report)(
            ReportSeverity::Error,
            &format!("error: writing {}: {message}", output.display()),
        );
    }
    events.push(Event {
        kind: EVENT_PHASE,
        ints: vec![PHASE_POST_WRITE_EXECUTABLE, written.is_err() as i64, 0],
        strings: vec![name],
    });
    Ok(Written::Done(written.is_ok()))
}

/// End a workspace whose metaprogram links it (`use_custom_link_command`): the linker's exit
/// code decides whether it failed. A metaprogram that waits for the next message without
/// saying the link is done gets an error instead of a workspace that never finishes.
pub(super) fn finish_custom_link(shared: &SharedWorkspaces, id: i64) -> Result<(), String> {
    let (link, compiler) = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        (ws.link.take().expect("a pending link"), ws.compiler.take())
    };
    let failed = match link.exit_code {
        Some(0) => false,
        Some(code) => {
            (shared.borrow_mut().env.report)(
                ReportSeverity::Error,
                &format!(
                    "error: the custom link command for {} failed with exit code {code}",
                    link.output
                ),
            );
            true
        }
        None => {
            (shared.borrow_mut().env.report)(
                ReportSeverity::Error,
                &format!(
                    "error: the metaprogram did not link {} after READY_FOR_CUSTOM_LINK_COMMAND\n\
                 help: call `compiler_custom_link_command_is_complete(workspace, exit_code)` once \
                 its link command has run",
                    link.output
                ),
            );
            true
        }
    };
    let mut compiler = compiler;
    let report = compiler
        .as_mut()
        .and_then(|compiler| performance_event(shared, id, compiler));
    {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        ws.events.push_back(Event {
            kind: EVENT_PHASE,
            ints: vec![
                PHASE_POST_WRITE_EXECUTABLE,
                failed as i64,
                link.exit_code.unwrap_or(-1),
            ],
            strings: vec![link.output.into_bytes()],
        });
        ws.events.extend(report);
        ws.events.push_back(Event {
            kind: EVENT_COMPLETE,
            ints: vec![failed as i64],
            strings: Vec::new(),
        });
        ws.failed |= failed;
        ws.stage = Stage::Done;
    }
    let observer = shared.borrow_mut().env.observer.take();
    if let (Some(mut o), Some(compiler)) = (observer, compiler) {
        o.finished(compiler, failed);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(())
}

/// Where a workspace's output goes: `output_path/output_executable_name`
/// (default name: the first source file's stem).
pub(super) fn output_path(settings: &BuildSettings, compiler: &Compiler) -> PathBuf {
    let name = if settings.output_executable_name.is_empty() {
        compiler
            .main_module
            .and_then(|m| compiler.files.iter().find(|f| f.module == m))
            .and_then(|f| f.path.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "output".into())
    } else {
        settings.output_executable_name.clone()
    };
    PathBuf::from(&settings.output_path).join(name)
}

/// Compile every workspace created but never driven by a message loop
/// (Jai compiles them after the metaprogram's `#run` returns). Their compile-time code draws
/// from `budget`, the top-level compiler's interpreter budget.
pub fn finish_all(shared: &SharedWorkspaces, budget: &mut Option<u64>) -> Result<(), String> {
    let mut id = TOP_LEVEL_WORKSPACE + 1;
    while id < shared.borrow().list.len() as i64 {
        // A workspace nobody added a file or string to has nothing to compile; building it
        // would link an executable without `main`.
        {
            let mut reg = shared.borrow_mut();
            let ws = &mut reg.list[id as usize];
            if ws.stage == Stage::Open && ws.pending.is_empty() {
                ws.stage = Stage::Done;
            }
        }
        while shared.borrow().list[id as usize].stage != Stage::Done {
            step(shared, id, budget)?;
        }
        shared.borrow_mut().list[id as usize].events.clear();
        id += 1;
    }
    Ok(())
}
