//! Native transport only. The library and wasm caller never touch stdio.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use std::io::{Read, Write};
    fn serve() -> Result<u8, Box<dyn std::error::Error>> {
        let limits = jai_language_server::Limits::default();
        let mut session =
            jai_language_server::JsonSession::with_environment(limits, native_environment());
        let mut decoder = jai_language_server::framing::FrameDecoder::new(limits.message_bytes);
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut input = stdin.lock();
        let mut output = stdout.lock();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let count = input.read(&mut chunk)?;
            if count == 0 {
                decoder.finish()?;
                return Ok(session.exit_status().unwrap_or(1));
            }
            for message in decoder.push(&chunk[..count])? {
                for response in session.handle_json(&message)? {
                    output.write_all(&jai_language_server::framing::encode(&response))?;
                }
                output.flush()?;
                if let Some(status) = session.exit_status() {
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
