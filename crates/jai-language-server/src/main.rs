//! Native transport only. The library and wasm caller never touch stdio.
/// How long input must be quiet before the diagnostics of the edits so far are computed and
/// published: a quarter of the last publication's time, within these bounds. A compile cannot be
/// interrupted, so starting one on every `didChange` made the completion queued behind it wait
/// for it; but edits that arrive while one runs are read together afterwards, so a short wait is
/// enough for a small program (whose compile is quick) and only a large one needs the long one.
#[cfg(not(target_arch = "wasm32"))]
const DIAGNOSTICS_DEBOUNCE: std::ops::RangeInclusive<std::time::Duration> =
    std::time::Duration::from_millis(15)..=std::time::Duration::from_millis(200);

/// How often an idle server looks for compile-time service answers it is waiting for.
#[cfg(not(target_arch = "wasm32"))]
const SERVICE_POLL: std::time::Duration = std::time::Duration::from_millis(25);

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use std::io::{Read, Write};
    fn serve(untrusted: bool) -> Result<u8, Box<dyn std::error::Error>> {
        let limits = jai_language_server::Limits::default();
        let mut session =
            jai_language_server::JsonSession::with_environment(limits, native_environment());
        let mut decoder = jai_language_server::framing::FrameDecoder::new(limits.message_bytes);
        session.defer_publications(true);
        if untrusted {
            session.set_trusted(false);
        }
        let stdout = std::io::stdout();
        let mut output = stdout.lock();
        // A thread reads stdin so the loop below can wait for more input with a timeout.
        let (chunks, input) = std::sync::mpsc::channel::<std::io::Result<Vec<u8>>>();
        std::thread::spawn(move || {
            let mut stdin = std::io::stdin().lock();
            let mut chunk = vec![0u8; 64 * 1024];
            loop {
                let sent = match stdin.read(&mut chunk) {
                    Ok(count) => {
                        let done = count == 0;
                        (chunks.send(Ok(chunk[..count].to_vec())).is_err(), done)
                    }
                    Err(error) => (chunks.send(Err(error)).is_err(), true),
                };
                if sent.0 || sent.1 {
                    return;
                }
            }
        });
        #[cfg(unix)]
        let mut compiles = 0;
        // How long the last publication took, which sets how long to wait for more edits.
        let mut last_publication = std::time::Duration::from_millis(40);
        let mut warm_needed = false;
        loop {
            // A compile (or the eviction it caused) freed many small blocks: hand their pages
            // back while the editor is idle (see `large_alloc`).
            #[cfg(unix)]
            {
                let (started, _) = session.compile_counts();
                if started != compiles {
                    compiles = started;
                    jai_language_server::large_alloc::release_free_memory();
                }
            }
            // Compile-time service answers that compiles read as pending: once one is in, the
            // open files are checked again (published after the usual debounce).
            if session.services_waiting() {
                for message in session.service_arrivals()? {
                    output.write_all(&jai_language_server::framing::encode(&message))?;
                }
                output.flush()?;
            }
            // Diagnostics for edits wait until no input has come for `DIAGNOSTICS_DEBOUNCE`, so
            // requests that are queued (completion, hover) are answered first and a burst of
            // keystrokes costs one compile.
            let received = if session.publications_pending() {
                let wait = (last_publication / 4)
                    .clamp(*DIAGNOSTICS_DEBOUNCE.start(), *DIAGNOSTICS_DEBOUNCE.end());
                match input.recv_timeout(wait) {
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        let started = std::time::Instant::now();
                        let published = session.flush_publications()?;
                        last_publication = started.elapsed();
                        for response in published {
                            output.write_all(&jai_language_server::framing::encode(&response))?;
                        }
                        output.flush()?;
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(Vec::new()),
                    Ok(received) => received,
                }
            } else if warm_needed || session.has_background_work() {
                // Idle after a message: read the indexes completion will want, then do the
                // work requests left for later one piece at a time, unless input is waiting.
                match input.try_recv() {
                    Ok(received) => received,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        if warm_needed {
                            warm_needed = false;
                            session.warm_up();
                        } else {
                            for message in session.background_work()? {
                                output
                                    .write_all(&jai_language_server::framing::encode(&message))?;
                            }
                            output.flush()?;
                        }
                        continue;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Ok(Vec::new()),
                }
            } else if session.services_waiting() {
                match input.recv_timeout(SERVICE_POLL) {
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(Vec::new()),
                    Ok(received) => received,
                }
            } else {
                input.recv().unwrap_or(Ok(Vec::new()))
            };
            warm_needed = true;
            let chunk = received?;
            if chunk.is_empty() {
                decoder.finish()?;
                for response in session.flush_publications()? {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
                return Ok(session.exit_status().unwrap_or(1));
            }
            // Whatever else has already arrived is read with it, so a cancellation can reach a
            // request that is still waiting its turn.
            let mut messages = decoder.push(&chunk)?;
            let mut ended = false;
            while let Ok(more) = input.try_recv() {
                let more = more?;
                if more.is_empty() {
                    ended = true;
                    break;
                }
                messages.extend(decoder.push(&more)?);
            }
            for responses in session.handle_batch(&messages)? {
                for response in responses {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
            }
            if let Some(status) = session.exit_status() {
                for response in session.flush_publications()? {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
                return Ok(status);
            }
            if ended {
                decoder.finish()?;
                for response in session.flush_publications()? {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
                return Ok(session.exit_status().unwrap_or(1));
            }
            // A message over the size limit was skipped unread: say so rather than go quiet.
            for length in decoder.take_oversized() {
                let notice = jai_language_server::message_too_large(length, limits.message_bytes);
                output.write_all(&jai_language_server::framing::encode(&notice))?;
                output.flush()?;
            }
        }
    }
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
    use std::path::PathBuf;
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
        options: Box::new(move |main| {
            let mut options = jaic::sema::Options::host();
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            // A file inside a standard library is analysed against that library.
            let stdlib = jaic::enclosing_stdlib(&jaic::sema::NativeFs, main)
                .unwrap_or_else(|| stdlib.clone());
            options.import_paths = vec![dir.join("modules"), stdlib.clone()];
            options.set_stdlib(&jaic::sema::NativeFs, &stdlib);
            options
        }),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
}
