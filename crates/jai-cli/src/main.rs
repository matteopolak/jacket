mod backend;
mod compile_time_limits;
mod foreign_libraries;
mod native_dependencies;
mod native_paths;
mod native_tools;
mod source_check;
mod source_configuration;
mod source_warnings;
mod workspace_build;
use std::{
    env,
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
    process::ExitCode,
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
    EmitLlvm {
        source: PathBuf,
        output: Output,
        target: jai_codegen::target::TargetOptions,
    },
    EmitObject {
        source: PathBuf,
        output: PathBuf,
        target: jai_codegen::target::TargetOptions,
    },
    Build {
        source: PathBuf,
        output: PathBuf,
        target: jai_codegen::target::TargetOptions,
    },
}
impl Options {
    fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self, Error> {
        let action = args.next().ok_or(Error::Arguments(
            "usage: jai-rs <lex|parse|check|check-library|emit-llvm|emit-object|build> <file.jai> [output]",
        ))?;
        let source = PathBuf::from(args.next().ok_or(Error::Arguments("missing source file"))?);
        let remaining: Vec<_> = args.collect();
        let has_output = remaining
            .first()
            .is_some_and(|value| !value.to_string_lossy().starts_with('-'));
        let output = has_output.then(|| PathBuf::from(&remaining[0]));
        let native_args = &remaining[usize::from(has_output)..];
        let target = backend::target_options(native_args)?;
        if !native_args.is_empty()
            && !matches!(action.to_str(), Some("build" | "emit-llvm" | "emit-object"))
        {
            return Err(Error::Arguments(
                "native target flags require build, emit-llvm or emit-object",
            ));
        }
        match action.to_str() {
            Some("lex") if output.is_none() => Ok(Self::Lex(source)),
            Some("parse") if output.is_none() => Ok(Self::Parse(source)),
            Some("check") if output.is_none() => Ok(Self::Check(source)),
            Some("check-library") if output.is_none() => Ok(Self::CheckLibrary(source)),
            Some("emit-llvm") => Ok(Self::EmitLlvm {
                source,
                output: output.map_or(Output::Stdout, Output::File),
                target,
            }),
            Some("emit-object") => Ok(Self::EmitObject {
                source,
                output: output.ok_or(Error::Arguments("emit-object requires an output path"))?,
                target,
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
                Ok(Self::Build {
                    source,
                    output,
                    target,
                })
            }
            Some("lex" | "parse" | "check" | "check-library") => Err(Error::Arguments(
                "this command does not take an output path",
            )),
            _ => Err(Error::Arguments(
                "unknown command; expected lex, parse, check, check-library, emit-llvm, emit-object or build",
            )),
        }
    }
    fn target_options(&self) -> Option<&jai_codegen::target::TargetOptions> {
        match self {
            Self::EmitLlvm { target, .. }
            | Self::EmitObject { target, .. }
            | Self::Build { target, .. } => Some(target),
            _ => None,
        }
    }
    fn source(&self) -> &Path {
        match self {
            Self::Lex(p) | Self::Parse(p) | Self::Check(p) | Self::CheckLibrary(p) => p,
            Self::EmitLlvm { source, .. }
            | Self::EmitObject { source, .. }
            | Self::Build { source, .. } => source,
        }
    }
}
#[derive(Debug)]
enum Error {
    Arguments(&'static str),
    CompileTimeLimit(compile_time_limits::ParseError),
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
    OutputIo(std::io::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arguments(text) => f.write_str(text),
            Self::CompileTimeLimit(error) => fmt::Display::fmt(error, f),
            Self::Source(text) => f.write_str(text),
            Self::Codegen(e) => fmt::Display::fmt(e, f),
            Self::Io { path, cause } => write!(f, "{}: {cause}", path.display()),
            Self::ToolNotFound(p) => write!(f, "trusted native tool not found: {}", p.display()),
            Self::ReferenceTool(p) => {
                write!(f, "refusing to execute reference tool: {}", p.display())
            }
            Self::BackendFailed => f.write_str("LLVM compilation/linking failed"),
            Self::BackendIo(e) => write!(f, "trusted compiler: {e}"),
            Self::OutputIo(e) => write!(f, "compiler output: {e}"),
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
    let path = options.source().to_owned();
    let bytes = fs::read(&path).map_err(|cause| Error::Io {
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
    let default_target = jai_codegen::target::TargetOptions::default();
    let target = jai_codegen::target::NativeTarget::select(
        options.target_options().unwrap_or(&default_target),
    )
    .map_err(|error| Error::Source(error.to_string()))?;
    let sources = source_configuration::SourceConfiguration::from_environment()?;
    let native_options = options.target_options().unwrap_or(&default_target).clone();
    match options {
        Options::EmitLlvm { output, .. } => {
            return workspace_build::run(
                &path,
                sources.graph,
                sources.bootstrap,
                sources.compile_time_limits,
                &native_options,
                &target,
                workspace_build::ArtifactCommand::Llvm(output),
            );
        }
        Options::EmitObject { output, .. } => {
            return workspace_build::run(
                &path,
                sources.graph,
                sources.bootstrap,
                sources.compile_time_limits,
                &native_options,
                &target,
                workspace_build::ArtifactCommand::Object(output),
            );
        }
        Options::Build { output, .. } => {
            return workspace_build::run(
                &path,
                sources.graph,
                sources.bootstrap,
                sources.compile_time_limits,
                &native_options,
                &target,
                workspace_build::ArtifactCommand::Executable(output),
            );
        }
        _ => {}
    }
    let kind = if matches!(options, Options::CheckLibrary(_)) {
        source_check::CheckKind::Library
    } else {
        source_check::CheckKind::Application
    };
    source_check::run(&path, sources, &target, kind)
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
