use std::{
    env,
    ffi::OsString,
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

enum Output {
    Stdout,
    File(PathBuf),
}
enum Options {
    Lex(PathBuf),
    Parse(PathBuf),
    Check(PathBuf),
    CheckLibrary(PathBuf),
    EmitLlvm { source: PathBuf, output: Output },
    Build { source: PathBuf, output: PathBuf },
}
impl Options {
    fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self, Error> {
        let action = args.next().ok_or(Error::Arguments(
            "usage: jai-rs <lex|parse|check|check-library|emit-llvm|build> <file.jai> [output]",
        ))?;
        let source = PathBuf::from(args.next().ok_or(Error::Arguments("missing source file"))?);
        let output = args.next().map(PathBuf::from);
        if args.next().is_some() {
            return Err(Error::Arguments("too many arguments"));
        }
        match action.to_str() {
            Some("lex") if output.is_none() => Ok(Self::Lex(source)),
            Some("parse") if output.is_none() => Ok(Self::Parse(source)),
            Some("check") if output.is_none() => Ok(Self::Check(source)),
            Some("check-library") if output.is_none() => Ok(Self::CheckLibrary(source)),
            Some("emit-llvm") => Ok(Self::EmitLlvm {
                source,
                output: output.map_or(Output::Stdout, Output::File),
            }),
            Some("build") => {
                let output = output.unwrap_or_else(|| {
                    let mut name = source
                        .file_stem()
                        .unwrap_or(source.as_os_str())
                        .to_os_string();
                    name.push(".jai-output");
                    PathBuf::from(name)
                });
                Ok(Self::Build { source, output })
            }
            Some("lex" | "parse" | "check" | "check-library") => Err(Error::Arguments(
                "this command does not take an output path",
            )),
            _ => Err(Error::Arguments(
                "unknown command; expected lex, parse, check, check-library, emit-llvm or build",
            )),
        }
    }
    fn source(&self) -> &Path {
        match self {
            Self::Lex(p) | Self::Parse(p) | Self::Check(p) | Self::CheckLibrary(p) => p,
            Self::EmitLlvm { source, .. } | Self::Build { source, .. } => source,
        }
    }
}
#[derive(Debug)]
enum Error {
    Arguments(&'static str),
    Io {
        path: PathBuf,
        cause: std::io::Error,
    },
    Source(String),
    Codegen(jai_codegen::Error),
    ToolNotFound(PathBuf),
    ReferenceTool(PathBuf),
    BackendFailed,
    BackendIo(std::io::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arguments(text) => f.write_str(text),
            Self::Source(text) => f.write_str(text),
            Self::Codegen(e) => fmt::Display::fmt(e, f),
            Self::Io { path, cause } => write!(f, "{}: {cause}", path.display()),
            Self::ToolNotFound(p) => write!(f, "trusted compiler not found: {}", p.display()),
            Self::ReferenceTool(p) => {
                write!(f, "refusing to execute reference tool: {}", p.display())
            }
            Self::BackendFailed => f.write_str("LLVM compilation/linking failed"),
            Self::BackendIo(e) => write!(f, "trusted compiler: {e}"),
        }
    }
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), Error> {
    let options = Options::parse(env::args_os().skip(1))?;
    let path = options.source();
    let bytes = fs::read(path).map_err(|cause| Error::Io {
        path: path.to_owned(),
        cause,
    })?;
    let source = jai_lexer::decode_source(&bytes).map_err(|e| Error::Source(e.to_string()))?;
    let render =
        |e: jai_source::Diagnostic| Error::Source(e.render(&path.to_string_lossy(), &source));
    if let Options::Lex(_) = options {
        let tokens = jai_lexer::lex(&source).map_err(render)?;
        println!("{} tokens (lexical stage only)", tokens.len() - 1);
        return Ok(());
    }
    if let Options::Parse(_) = options {
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert(path.to_owned(), source.into_owned());
        let mut symbols = jai_source::Symbols::default();
        let parsed =
            jai_syntax::parse_file(sources.get(id).expect("inserted source"), &mut symbols)
                .map_err(|diagnostic| Error::Source(diagnostic.render(&sources)))?;
        println!("{} items (syntax stage only)", parsed.items().len());
        return Ok(());
    }
    let graph_options = env::var_os("JAI_RS_MODULE_PATH").map_or_else(
        jai_driver::modules::GraphOptions::default,
        |paths| jai_driver::modules::GraphOptions {
            import_dirs: env::split_paths(&paths).collect(),
        },
    );
    let unit = jai_driver::CompilationUnit::load_with_options(path, graph_options)
        .map_err(|e| Error::Source(e.to_string()))?;
    if let Options::CheckLibrary(_) = options {
        unit.resolve_library()
            .map_err(|e| Error::Source(e.to_string()))?;
        println!("checked library {}", path.display());
        return Ok(());
    }
    let program = unit.resolve().map_err(|e| Error::Source(e.to_string()))?;
    match options {
        Options::Lex(_) | Options::Parse(_) | Options::CheckLibrary(_) => {}
        Options::Check(path) => println!("checked {}", path.display()),
        Options::EmitLlvm { output, .. } => {
            let ir = jai_codegen::emit(&program).map_err(Error::Codegen)?;
            match output {
                Output::Stdout => print!("{ir}"),
                Output::File(path) => {
                    fs::write(&path, ir).map_err(|cause| Error::Io { path, cause })?
                }
            }
        }
        Options::Build { output, .. } => {
            let ir = jai_codegen::emit(&program).map_err(Error::Codegen)?;
            let tool = trusted_compiler()?;
            let mut child = Command::new(tool)
                .args(["-x", "ir", "-", "-o"])
                .arg(&output)
                .stdin(Stdio::piped())
                .spawn()
                .map_err(Error::BackendIo)?;
            let written = child
                .stdin
                .take()
                .expect("piped stdin")
                .write_all(ir.as_bytes());
            let status = child.wait().map_err(Error::BackendIo)?;
            written.map_err(Error::BackendIo)?;
            if !status.success() {
                return Err(Error::BackendFailed);
            }
            println!("built {}", output.display());
        }
    }
    Ok(())
}
fn trusted_compiler() -> Result<PathBuf, Error> {
    let name = env::var_os("JAI_RS_CLANG").unwrap_or_else(|| "clang".into());
    let name = PathBuf::from(name);
    let candidate = if name.components().count() > 1 || name.is_absolute() {
        Some(name.clone())
    } else {
        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths)
                .map(|p| p.join(&name))
                .find(|p| p.is_file())
        })
    };
    let canonical = candidate
        .and_then(|p| p.canonicalize().ok())
        .ok_or(Error::ToolNotFound(name))?;
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../reference")
        .canonicalize()
        .ok();
    if reference.is_some_and(|r| canonical.starts_with(r)) {
        return Err(Error::ReferenceTool(canonical));
    }
    Ok(canonical)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn options(args: &[&str]) -> Result<Options, Error> {
        Options::parse(args.iter().map(OsString::from))
    }
    #[test]
    fn parse_commands_before_work() {
        assert!(matches!(
            options(&["emit-llvm", "input.jai"]),
            Ok(Options::EmitLlvm {
                output: Output::Stdout,
                ..
            })
        ));
        for args in [
            &["other", "x"][..],
            &["check", "x", "y"],
            &["check-library", "x", "y"],
            &["parse", "x", "y"],
            &["lex", "x", "y"],
            &["build"],
            &["build", "x", "y", "z"],
        ] {
            assert!(options(args).is_err());
        }
    }
}
