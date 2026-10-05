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
        let mut chunk = [0u8; 4096];
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
        }
    }
    match serve() {
        Ok(code) => std::process::ExitCode::from(code),
        Err(error) => {
            eprintln!("jai-lsp: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
/// Modules come from disk: the `modules` folder next to the main file, then the stdlib
/// (`JAIC_STDLIB`, else the repository's).
#[cfg(not(target_arch = "wasm32"))]
fn native_environment() -> jai_language_server::Environment {
    use std::path::PathBuf;
    let stdlib = std::env::var_os("JAIC_STDLIB").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        PathBuf::from,
    );
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
