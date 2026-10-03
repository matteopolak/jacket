//! Source-only parser probe, independent of semantic/backend build readiness.
use std::{env, fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), String> {
    let mut arguments = env::args_os().skip(1);
    let path = PathBuf::from(arguments.next().ok_or("usage: source-check <file.jai>")?);
    if arguments.next().is_some() {
        return Err("usage: source-check <file.jai>".into());
    }
    let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let text = jai_lexer::decode_source(&bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .into_owned();
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert(path.clone(), text);
    let mut symbols = jai_source::Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols)
        .map_err(|error| error.render(&sources))?;
    println!(
        "{}: {} top-level items (syntax only)",
        path.display(),
        parsed.items().len()
    );
    Ok(())
}
