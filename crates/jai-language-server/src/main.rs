//! Native transport only. The library and wasm caller never touch stdio.
/// How long input must be quiet before the diagnostics of the edits so far are computed and
/// published. A compile of a large program takes up to seconds and cannot be interrupted, so
/// starting one on every `didChange` made the completion queued behind it wait for it.
#[cfg(not(target_arch = "wasm32"))]
const DIAGNOSTICS_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(200);

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use std::io::{Read, Write};
    fn serve() -> Result<u8, Box<dyn std::error::Error>> {
        let limits = jai_language_server::Limits::default();
        let mut session =
            jai_language_server::JsonSession::with_environment(limits, native_environment());
        let mut decoder = jai_language_server::framing::FrameDecoder::new(limits.message_bytes);
        session.defer_publications(true);
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
        let mut compiles = 0;
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
            // Diagnostics for edits wait until no input has come for `DIAGNOSTICS_DEBOUNCE`, so
            // requests that are queued (completion, hover) are answered first and a burst of
            // keystrokes costs one compile.
            let received = if session.publications_pending() {
                match input.recv_timeout(DIAGNOSTICS_DEBOUNCE) {
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        for response in session.flush_publications()? {
                            output.write_all(&jai_language_server::framing::encode(&response))?;
                        }
                        output.flush()?;
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(Vec::new()),
                    Ok(received) => received,
                }
            } else {
                input.recv().unwrap_or(Ok(Vec::new()))
            };
            let chunk = received?;
            if chunk.is_empty() {
                decoder.finish()?;
                for response in session.flush_publications()? {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
                return Ok(session.exit_status().unwrap_or(1));
            }
            for message in decoder.push(&chunk)? {
                for response in session.handle_json(&message)? {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
                if let Some(status) = session.exit_status() {
                    for response in session.flush_publications()? {
                        output.write_all(&jai_language_server::framing::encode(&response))?;
                    }
                    output.flush()?;
                    return Ok(status);
                }
            }
            // A message over the size limit was skipped unread: say so rather than go quiet.
            for length in decoder.take_oversized() {
                let notice = jai_language_server::message_too_large(length, limits.message_bytes);
                output.write_all(&jai_language_server::framing::encode(&notice))?;
                output.flush()?;
            }
        }
    }
    if let Some(code) = command_line() {
        return code;
    }
    // Type checking recurses on the syntax tree: give it the compiler's stack (as `jaic` does),
    // not the main thread's 8 MiB.
    let worker = std::thread::Builder::new()
        .name("jailsp".into())
        .stack_size(1 << 30)
        .spawn(|| serve().map_err(|error| error.to_string()));
    match worker.map(|handle| handle.join()) {
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
usage: jailsp [--stdio]

The Jai language server. An editor starts it and exchanges Language Server Protocol messages
with it over stdin and stdout; it takes no files on the command line.

options:
  --stdio        talk over stdin and stdout (the default and only transport)
  --help, -h     show this text
  --version, -V  show the version

To check a file from a terminal, use `jaic check file.jai`; for lints, `jailint file.jai`.
";

/// Handles the command line: `Some(status)` when jailsp should exit without serving.
#[cfg(not(target_arch = "wasm32"))]
fn command_line() -> Option<std::process::ExitCode> {
    use std::io::IsTerminal;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--stdio" => {}
            "--help" | "-h" => {
                print!("{USAGE}");
                return Some(std::process::ExitCode::SUCCESS);
            }
            "--version" | "-V" => {
                println!("jailsp {}", env!("CARGO_PKG_VERSION"));
                return Some(std::process::ExitCode::SUCCESS);
            }
            file if !file.starts_with('-') => {
                eprintln!(
                    "error: jailsp takes no files: it is a language server that an editor starts"
                );
                eprintln!("help: to check `{file}` from a terminal, use `jaic check {file}`");
                return Some(std::process::ExitCode::from(2));
            }
            other => {
                eprintln!("error: unknown option `{other}`");
                eprintln!("help: `jailsp --help` lists the options");
                return Some(std::process::ExitCode::from(2));
            }
        }
    }
    if std::io::stdin().is_terminal() {
        eprintln!(
            "note: jailsp is waiting for Language Server Protocol messages on stdin; an editor normally starts it (Ctrl-D quits)"
        );
    }
    None
}

/// Large blocks go back to the operating system when freed (see `large_alloc`).
#[cfg(unix)]
#[global_allocator]
static ALLOCATOR: jai_language_server::large_alloc::LargeBlocksToOs =
    jai_language_server::large_alloc::LargeBlocksToOs;

/// Modules come from disk: the `modules` folder next to the main file, then the stdlib
/// (`JAIC_STDLIB`, else `stdlib/` next to the executable, else the repository's).
#[cfg(not(target_arch = "wasm32"))]
fn native_environment() -> jai_language_server::Environment {
    use std::path::PathBuf;
    let stdlib = jaic::stdlib_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"));
    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        eprintln!("warning: {message}");
    }
    jai_language_server::Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |main| {
            let mut options = jaic::sema::Options::host();
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
}
