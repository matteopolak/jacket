//! Native transport only. The library and wasm caller never touch stdio.
/// How long input must be quiet before the diagnostics of the edits so far are computed and
/// published: a quarter of the last publication's time, within these bounds. Background checks
/// stop for newly received input at compiler/linter interruption points. Waiting for quiet input
/// coalesces a typing burst before starting that work, while queued requests are answered first.
#[cfg(not(target_arch = "wasm32"))]
const DIAGNOSTICS_DEBOUNCE: std::ops::RangeInclusive<std::time::Duration> =
    std::time::Duration::from_millis(15)..=std::time::Duration::from_millis(200);

/// How often an idle server looks for compile-time service answers it is waiting for.
#[cfg(not(target_arch = "wasm32"))]
const SERVICE_POLL: std::time::Duration = std::time::Duration::from_millis(25);

/// Reconciliation gets a service opportunity even when request traffic never becomes idle.
#[cfg(not(target_arch = "wasm32"))]
const INPUT_RECONCILE_POLL: std::time::Duration = std::time::Duration::from_millis(25);

/// Open the optional native trace before serving; disabled tracing allocates no recorder.
#[cfg(not(target_arch = "wasm32"))]
fn native_trace() -> std::io::Result<jai_language_server::trace::Trace> {
    Ok(
        if jacket_config::env::LSP_TRACE.get().as_deref() == Some("1") {
            let destination = jacket_config::env::LSP_TRACE_FILE
                .os()
                .ok_or_else(|| std::io::Error::other("LSP trace destination required"))?;
            let trace = jai_language_server::trace::Trace::open_new(
                std::path::Path::new(&destination),
                16384,
            )
            .map_err(|_| std::io::Error::other("cannot initialize LSP trace"))?;
            jai_language_server::trace::install(trace.clone())
                .map_err(|_| std::io::Error::other("LSP trace already initialized"))?;
            trace
        } else {
            jai_language_server::trace::Trace::disabled()
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn serve(untrusted: bool) -> Result<u8, Box<dyn std::error::Error>> {
    let limits = jai_language_server::Limits::default();
    let trace = native_trace()?;
    let mut session =
        jai_language_server::JsonSession::with_environment(limits, native_environment());
    use crossbeam_channel::{RecvTimeoutError, TryRecvError};
    use jai_language_server::transport::{HostTransport, Input};
    let transport = HostTransport::stdio(limits);
    session.defer_publications(true);
    session.defer_compiles(true);
    if untrusted {
        session.set_trusted(false);
    }
    let mut last_publication = std::time::Duration::from_millis(40);
    let mut warm_needed = false;
    let mut quiet_since = std::time::Instant::now();
    let mut reconcile_due = std::time::Instant::now() + INPUT_RECONCILE_POLL;
    let mut held = None;
    let mut shutdown_deadline = None;
    #[cfg(unix)]
    let mut compiles = 0;
    let mut finish_attempted = false;
    let result = (|| -> Result<u8, Box<dyn std::error::Error>> {
        loop {
            if transport.failures.try_recv().is_ok() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "LSP output disconnected",
                )
                .into());
            }
            #[cfg(unix)]
            {
                let (started, _) = session.compile_counts();
                if started != compiles {
                    compiles = started;
                    jai_language_server::large_alloc::release_free_memory();
                }
            }
            if session.services_waiting() {
                transport.send(session.service_arrivals_messages()?)?;
            }
            jaic::interrupt::request(false);
            // This runs between bounded input batches, not just on an idle timeout.
            // The slice checks cancellation after its first operation to avoid starvation.
            if session.has_input_epoch_work() && std::time::Instant::now() >= reconcile_due {
                transport.send(session.reconcile_input_epoch_messages()?)?;
                reconcile_due = std::time::Instant::now()
                    + if session.input_epoch_hydration_pending() {
                        std::time::Duration::ZERO
                    } else {
                        INPUT_RECONCILE_POLL
                    };
            }
            let quiet_due = quiet_since
                + (last_publication / 4)
                    .clamp(*DIAGNOSTICS_DEBOUNCE.start(), *DIAGNOSTICS_DEBOUNCE.end());
            let wait = if session.publications_pending() || session.compile_pending() {
                Some(quiet_due.saturating_duration_since(std::time::Instant::now()))
            } else if warm_needed || session.has_background_work() {
                Some(std::time::Duration::ZERO)
            } else if session.services_waiting() {
                Some(SERVICE_POLL)
            } else {
                None
            };
            let wait = if session.has_input_epoch_work() {
                let remaining = reconcile_due.saturating_duration_since(std::time::Instant::now());
                Some(wait.map_or(remaining, |wait| wait.min(remaining)))
            } else {
                wait
            };
            let wait = if let Some(deadline) = shutdown_deadline {
                let now = std::time::Instant::now();
                if now >= deadline {
                    finish_attempted = true;
                    transport.finish()?;
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "LSP exit did not follow shutdown",
                    )
                    .into());
                }
                let remaining = deadline - now;
                Some(wait.map_or(remaining, |wait| wait.min(remaining)))
            } else {
                wait
            };
            let received = match held.take() {
                Some(arrival) => Ok(arrival),
                None => {
                    if let Some(wait) = wait {
                        crossbeam_channel::select! {
                            recv(transport.arrivals) -> arrival => arrival.map_err(|_| RecvTimeoutError::Disconnected),
                            recv(transport.failures) -> _ => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "LSP output disconnected").into()),
                            default(wait) => Err(RecvTimeoutError::Timeout),
                        }
                    } else {
                        crossbeam_channel::select! {
                            recv(transport.arrivals) -> arrival => arrival.map_err(|_| RecvTimeoutError::Disconnected),
                            recv(transport.failures) -> _ => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "LSP output disconnected").into()),
                        }
                    }
                }
            };
            let arrival = match received {
                Ok(arrival) => arrival,
                Err(RecvTimeoutError::Disconnected) => {
                    transport.send(session.flush_publications_messages()?)?;
                    finish_attempted = true;
                    transport.finish()?;
                    return Ok(session.exit_status().unwrap_or(1));
                }
                Err(RecvTimeoutError::Timeout) => {
                    // Clear before checking queued arrivals; the reader queues then interrupts.
                    jaic::interrupt::request(false);
                    match transport.arrivals.try_recv() {
                        Ok(arrival) => arrival,
                        Err(TryRecvError::Disconnected) => continue,
                        Err(TryRecvError::Empty) => {
                            // A reconciliation wake must not bypass the independent typing
                            // debounce or restart it indefinitely on each short timer.
                            if (session.publications_pending() || session.compile_pending())
                                && std::time::Instant::now() < quiet_due
                            {
                                continue;
                            }
                            if session.publications_pending() {
                                let started = std::time::Instant::now();
                                let messages =
                                    session.flush_publications_unless_interrupted_messages()?;
                                if !messages.is_empty() {
                                    last_publication = started.elapsed();
                                }
                                transport.send(messages)?;
                            } else if session.compile_pending() {
                                transport.send(session.warm_pending_messages()?)?;
                            } else if warm_needed {
                                warm_needed = !session.warm_up_unless_interrupted();
                            } else if session.has_background_work() {
                                transport.send(session.background_work_messages()?)?;
                            }
                            continue;
                        }
                    }
                }
            };
            warm_needed = true;
            quiet_since = std::time::Instant::now();
            let frame = arrival.frame();
            let Some(input) = transport.take(arrival)? else {
                transport.send(session.flush_publications_messages()?)?;
                finish_attempted = true;
                transport.finish()?;
                return Ok(session.exit_status().unwrap_or(1));
            };
            let mut messages = Vec::new();
            let mut frames = trace.enabled().then(Vec::new);
            let mut bytes;
            match input {
                Input::Ignored {
                    ..
                } => {
                    let _dispatch = jai_language_server::trace::dispatch(frame);
                    continue;
                }
                Input::Message {
                    message,
                    bytes: length,
                } => {
                    bytes = length;
                    messages.push(message);
                    if let (Some(frames), Some(frame)) = (&mut frames, frame) {
                        frames.push(frame);
                    }
                }
                Input::Rejected {
                    rejected, ..
                } => {
                    let _dispatch = jai_language_server::trace::dispatch(frame);
                    if let Some(reply) = session.handle_rejected(rejected) {
                        transport.send_error(reply)?;
                    }
                    continue;
                }
            }
            // Both count and decoded body bytes bound lookahead. A held routing token owns
            // at most one already-bounded next message in the Connection channel.
            let mut admitted = 1;
            while admitted < 64 {
                let Ok(arrival) = transport.arrivals.try_recv() else {
                    break;
                };
                match arrival {
                    arrival @ (jai_language_server::transport::Arrival::Message {
                        bytes: length,
                        ..
                    }
                    | jai_language_server::transport::Arrival::Ignored {
                        bytes: length,
                        ..
                    }) if bytes
                        .checked_add(length)
                        .is_some_and(|n| n <= limits.message_bytes.saturating_mul(2)) =>
                    {
                        bytes += length;
                        admitted += 1;
                        let frame = arrival.frame();
                        match transport.take(arrival)? {
                            Some(Input::Message {
                                message, ..
                            }) => {
                                messages.push(message);
                                if let (Some(frames), Some(frame)) = (&mut frames, frame) {
                                    frames.push(frame);
                                }
                            }
                            Some(Input::Ignored {
                                ..
                            }) => {
                                let _dispatch = jai_language_server::trace::dispatch(frame);
                            }
                            _ => unreachable!("only message/ignored lookahead admitted"),
                        }
                    }
                    other => {
                        held = Some(other);
                        break;
                    }
                }
            }
            let emit = |output| {
                transport.send(output).map_err(|_| {
                    jai_language_server::ProtocolError::Serialization(
                        "LSP output disconnected".into(),
                    )
                })
            };
            if let Some(frames) = frames {
                session.dispatch_messages_traced(messages, &trace, frames, emit)?;
            } else {
                session.dispatch_messages(messages, emit)?;
            }
            if session.shutdown_requested() && shutdown_deadline.is_none() {
                shutdown_deadline =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(30));
            }
            if let Some(status) = session.exit_status() {
                finish_attempted = true;
                transport.finish()?;
                return Ok(status);
            }
        }
    })();
    export_trace(&trace, &transport, finish_attempted)?;
    result
}

/// Export outside request intervals, retaining incomplete tails after aborted transport work.
#[cfg(not(target_arch = "wasm32"))]
fn export_trace(
    trace: &jai_language_server::trace::Trace,
    transport: &jai_language_server::transport::HostTransport,
    finish_attempted: bool,
) -> std::io::Result<()> {
    if trace.enabled() {
        // Never join a reader whose client kept stdin open or repeat a failed writer drain.
        if !finish_attempted {
            let _ = transport.finish();
        }
        trace
            .finish_output(transport.trace_quiesced())
            .map_err(|_| std::io::Error::other("cannot export LSP trace"))?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    let untrusted = match command_line() {
        Err(code) => return code,
        Ok(untrusted) => untrusted,
    };
    // Type checking recurses on the syntax tree: give it the compiler's stack (as `jaic` does),
    // not the main thread's 8 MiB.
    let worker = std::thread::Builder::new()
        .name("jailsp".into())
        .stack_size(1 << 30)
        .spawn(move || serve(untrusted).map_err(|error| error.to_string()));
    match worker.map(std::thread::JoinHandle::join) {
        Ok(Ok(Ok(code))) => std::process::ExitCode::from(code),
        Ok(Ok(Err(error))) => {
            eprintln!("error: jailsp stopped: {error}");
            eprintln!(
                "note: jailsp reads Language Server Protocol messages (with `Content-Length` headers) on stdin; an editor starts it"
            );
            std::process::ExitCode::FAILURE
        }
        _ => std::process::ExitCode::from(101),
    }
}

const USAGE: &str = "\
usage: jailsp [--stdio] [--untrusted]

The Jai language server. An editor starts it and exchanges Language Server Protocol messages
with it over stdin and stdout; it takes no files on the command line.

options:
  --stdio        talk over stdin and stdout (the default and only transport)
  --untrusted    start without workspace trust: run no compile-time code (`#run`, build
                 metaprograms, computed `#insert`) from the workspace until the client sends
                 `jai/setTrusted` with `{\"trusted\": true}`
  --help, -h     show this text
  --version, -V  show the version

To check a file from a terminal, use `jaic file.jai -check`; for lints, `jailint file.jai`.
";

/// Handles the command line: `Err(status)` when jailsp should exit without serving, else
/// whether to start untrusted.
#[cfg(not(target_arch = "wasm32"))]
fn command_line() -> Result<bool, std::process::ExitCode> {
    use std::io::IsTerminal;
    let mut untrusted = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--stdio" => {}
            "--untrusted" => untrusted = true,
            "--help" | "-h" => {
                print!("{USAGE}");
                return Err(std::process::ExitCode::SUCCESS);
            }
            "--version" | "-V" => {
                println!("jailsp {}", env!("CARGO_PKG_VERSION"));
                return Err(std::process::ExitCode::SUCCESS);
            }
            file if !file.starts_with('-') => {
                eprintln!(
                    "error: jailsp takes no files: it is a language server that an editor starts"
                );
                eprintln!("help: to check `{file}` from a terminal, use `jaic {file} -check`");
                return Err(std::process::ExitCode::from(2));
            }
            other => {
                eprintln!("error: unknown option `{other}`");
                eprintln!("help: `jailsp --help` lists the options");
                return Err(std::process::ExitCode::from(2));
            }
        }
    }
    if std::io::stdin().is_terminal() {
        eprintln!(
            "note: jailsp is waiting for Language Server Protocol messages on stdin; an editor normally starts it (Ctrl-D quits)"
        );
    }
    Ok(untrusted)
}

/// Large blocks go back to the operating system when freed (see `large_alloc`).
#[cfg(unix)]
#[global_allocator]
static ALLOCATOR: jai_language_server::large_alloc::LargeBlocksToOs =
    jai_language_server::large_alloc::LargeBlocksToOs;

/// Modules come from disk: the `modules` folder next to the main file, then the stdlib
/// (`JAIC_STDLIB`, else `stdlib/` next to the executable, else the repository's). Compile-time
/// services run with the `jaic` next to this executable, else the first on `PATH`.
#[cfg(not(target_arch = "wasm32"))]
fn native_environment() -> jai_language_server::Environment {
    let stdlib = jaic::default_stdlib_dir();
    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        eprintln!("warning: {message}");
    }
    // Compile-time services run as separate `jaic` processes; only trusted compiles ask them.
    let services = jaic::service::Services::new(jaic::service::ServiceConfig::default());
    jai_language_server::Environment {
        services: Some(std::rc::Rc::new(
            jai_language_server::services::EditorServices::new(services),
        )),
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |fs, main| {
            let inputs = jaic::project::ProgramInputs::for_file(fs, main, &stdlib);
            jaic::sema::Options::for_program(fs, main, &inputs)
        }),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
}
