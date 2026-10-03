//! Parse source bootstrap policy before loading any module graph.
use crate::Error;
use jai_driver::modules::{
    BootstrapOptions, GraphOptions, PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters,
    RuntimeSupportSource,
};
use std::{
    env,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub struct SourceConfiguration {
    pub graph: GraphOptions,
    pub bootstrap: BootstrapOptions,
    pub compile_time_limits: jai_vm::Limits,
}
enum SourceSelection {
    Disabled,
    Search,
    File(PathBuf),
}
impl SourceSelection {
    fn parse(value: &OsStr) -> Result<Self, Error> {
        match value.to_str() {
            Some("off") => Ok(Self::Disabled),
            Some("search") => Ok(Self::Search),
            Some("") => Err(Error::Arguments(
                "source bootstrap selection cannot be empty",
            )),
            _ => Ok(Self::File(PathBuf::from(value))),
        }
    }
    fn prelude(self) -> PreludeSource {
        match self {
            Self::Disabled => PreludeSource::Disabled,
            Self::Search => PreludeSource::Search,
            Self::File(path) => PreludeSource::File(path),
        }
    }
}
impl SourceConfiguration {
    pub fn from_environment() -> Result<Self, Error> {
        let mut configuration = Self::parse(|key| env::var_os(key))?;
        for directory in &mut configuration.graph.import_dirs {
            *directory = canonical(directory)?;
            if !directory.is_dir() {
                return Err(Error::Source(format!(
                    "source module root {} is not a directory",
                    directory.display()
                )));
            }
        }
        if let PreludeSource::File(path) = &mut configuration.bootstrap.prelude {
            *path = source_file(path)?;
        }
        if let Some(runtime) = &mut configuration.bootstrap.runtime_support {
            if let RuntimeSupportSource::File(path) = &mut runtime.source {
                *path = source_file(path)?;
            }
        }
        Ok(configuration)
    }
    fn parse(environment: impl Fn(&str) -> Option<OsString>) -> Result<Self, Error> {
        let mut graph = GraphOptions {
            import_dirs: environment("JAI_RS_MODULE_PATH")
                .map_or_else(Vec::new, |paths| env::split_paths(&paths).collect()),
        };
        let standard_library = environment("JAI_RS_STDLIB");
        if let Some(root) = &standard_library {
            if root.is_empty() {
                return Err(Error::Arguments(
                    "JAI_RS_STDLIB requires a source module directory",
                ));
            }
            graph.import_dirs.push(PathBuf::from(root));
        }
        let prelude = environment("JAI_RS_PRELOAD").map_or_else(
            || {
                Ok(if standard_library.is_some() {
                    PreludeSource::Search
                } else {
                    PreludeSource::Disabled
                })
            },
            |value| SourceSelection::parse(&value).map(SourceSelection::prelude),
        )?;
        let runtime_support = match environment("JAI_RS_RUNTIME_SUPPORT") {
            None => None,
            Some(value) => match SourceSelection::parse(&value)? {
                SourceSelection::Disabled => None,
                selection => {
                    let parameters = RuntimeSupportParameters {
                        define_system_entry_point: flag(&environment, "JAI_RS_RUNTIME_ENTRY")?,
                        define_initialization: flag(&environment, "JAI_RS_RUNTIME_INITIALIZATION")?,
                        enable_backtrace_on_crash: flag(&environment, "JAI_RS_RUNTIME_BACKTRACE")?,
                        temporary_storage_size: 32768,
                    };
                    Some(RuntimeSupportOptions {
                        source: match selection {
                            SourceSelection::Search => RuntimeSupportSource::Search,
                            SourceSelection::File(path) => RuntimeSupportSource::File(path),
                            SourceSelection::Disabled => unreachable!(),
                        },
                        parameters,
                    })
                }
            },
        };
        if runtime_support.is_some() && prelude == PreludeSource::Disabled {
            return Err(Error::Arguments(
                "Runtime_Support source requires Preload bootstrap",
            ));
        }
        if let Some(runtime) = &runtime_support {
            if runtime.parameters.define_system_entry_point
                && !runtime.parameters.define_initialization
            {
                return Err(Error::Arguments(
                    "Runtime_Support entry point requires initialization",
                ));
            }
            if runtime.parameters.enable_backtrace_on_crash
                && !runtime.parameters.define_system_entry_point
            {
                return Err(Error::Arguments(
                    "Runtime_Support backtrace requires its entry point",
                ));
            }
        }
        Ok(Self {
            compile_time_limits: crate::compile_time_limits::parse(&environment)?,
            graph,
            bootstrap: BootstrapOptions {
                prelude,
                runtime_support,
            },
        })
    }
}
fn canonical(path: &std::path::Path) -> Result<PathBuf, Error> {
    path.canonicalize().map_err(|cause| Error::Io {
        path: path.into(),
        cause,
    })
}
fn source_file(path: &std::path::Path) -> Result<PathBuf, Error> {
    let path = canonical(path)?;
    if !path.is_file() {
        return Err(Error::Source(format!(
            "bootstrap source {} is not a file",
            path.display()
        )));
    }
    Ok(path)
}
fn flag(environment: &impl Fn(&str) -> Option<OsString>, key: &str) -> Result<bool, Error> {
    match environment(key).as_deref().and_then(OsStr::to_str) {
        Some("true" | "1") => Ok(true),
        Some("false" | "0") => Ok(false),
        _ => Err(Error::Source(format!(
            "{key} must be explicitly set to true/false or 1/0 when Runtime_Support is selected"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn configuration(values: &[(&str, &str)]) -> Result<SourceConfiguration, Error> {
        SourceConfiguration::parse(|key| {
            values
                .iter()
                .find_map(|(name, value)| (*name == key).then(|| OsString::from(value)))
        })
    }
    #[test]
    fn explicit_standard_library_requires_preload_and_off_remains_explicit() {
        assert_eq!(
            configuration(&[]).unwrap().bootstrap.prelude,
            PreludeSource::Disabled
        );
        let selected = configuration(&[("JAI_RS_STDLIB", "stdlib")]).unwrap();
        assert_eq!(selected.bootstrap.prelude, PreludeSource::Search);
        assert_eq!(selected.graph.import_dirs, [PathBuf::from("stdlib")]);
        assert_eq!(
            configuration(&[("JAI_RS_STDLIB", "stdlib"), ("JAI_RS_PRELOAD", "off")])
                .unwrap()
                .bootstrap
                .prelude,
            PreludeSource::Disabled
        );
        assert!(configuration(&[("JAI_RS_STDLIB", "")]).is_err());
    }
    #[test]
    fn runtime_source_requires_checked_explicit_flags_and_preload() {
        assert!(
            configuration(&[
                ("JAI_RS_PRELOAD", "search"),
                ("JAI_RS_RUNTIME_SUPPORT", "search")
            ])
            .is_err()
        );
        let selected = configuration(&[
            ("JAI_RS_PRELOAD", "search"),
            ("JAI_RS_RUNTIME_SUPPORT", "runtime.jai"),
            ("JAI_RS_RUNTIME_ENTRY", "0"),
            ("JAI_RS_RUNTIME_INITIALIZATION", "true"),
            ("JAI_RS_RUNTIME_BACKTRACE", "false"),
        ])
        .unwrap();
        let runtime = selected.bootstrap.runtime_support.unwrap();
        assert_eq!(
            runtime.source,
            RuntimeSupportSource::File("runtime.jai".into())
        );
        assert_eq!(
            runtime.parameters,
            RuntimeSupportParameters {
                define_system_entry_point: false,
                define_initialization: true,
                enable_backtrace_on_crash: false,
                temporary_storage_size: 32768,
            }
        );
    }
}
