//! Native transport only. The library and wasm caller never touch stdio.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use std::io::{Read, Write};
    fn serve() -> Result<u8, Box<dyn std::error::Error>> {
        let limits = jai_language_server::Limits::default();
        let mut session = jai_language_server::JsonSession::new(limits);
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
#[cfg(target_arch = "wasm32")]
fn main() {
}
