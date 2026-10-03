//! Derive Runtime_Support source parameters from actual workspace build policy.
use crate::BuildSettings;
use jai_modules::RuntimeSupportParameters;
use jai_types::{BacktraceOnCrash, BuildOutputKind, RuntimeSupportMode};

impl BuildSettings {
    pub fn runtime_support_parameters(&self) -> RuntimeSupportParameters {
        let (define_system_entry_point, define_initialization) = match self.runtime_support {
            RuntimeSupportMode::Auto => (self.output_kind == BuildOutputKind::Executable, true),
            RuntimeSupportMode::EntryPointAndInitialization => (true, true),
            RuntimeSupportMode::InitializationOnly => (false, true),
            RuntimeSupportMode::Omit => (false, false),
        };
        RuntimeSupportParameters {
            define_system_entry_point,
            define_initialization,
            enable_backtrace_on_crash: define_system_entry_point
                && self.backtrace_on_crash == BacktraceOnCrash::On,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_uses_output_kind_and_backtrace_requires_entry_point() {
        for output_kind in [
            BuildOutputKind::None,
            BuildOutputKind::Executable,
            BuildOutputKind::DynamicLibrary,
            BuildOutputKind::StaticLibrary,
            BuildOutputKind::Object,
        ] {
            let settings = BuildSettings {
                output_kind,
                ..Default::default()
            };
            let parameters = settings.runtime_support_parameters();
            assert!(parameters.define_initialization);
            assert_eq!(
                parameters.define_system_entry_point,
                output_kind == BuildOutputKind::Executable
            );
            assert_eq!(
                parameters.enable_backtrace_on_crash,
                parameters.define_system_entry_point
            );
        }
    }
    #[test]
    fn explicit_modes_override_output_kind() {
        for output_kind in [BuildOutputKind::Executable, BuildOutputKind::Object] {
            for (runtime_support, entry, init) in [
                (RuntimeSupportMode::EntryPointAndInitialization, true, true),
                (RuntimeSupportMode::InitializationOnly, false, true),
                (RuntimeSupportMode::Omit, false, false),
            ] {
                for backtrace_on_crash in [BacktraceOnCrash::Off, BacktraceOnCrash::On] {
                    let settings = BuildSettings {
                        output_kind,
                        runtime_support,
                        backtrace_on_crash,
                        ..Default::default()
                    };
                    assert_eq!(
                        settings.runtime_support_parameters(),
                        RuntimeSupportParameters {
                            define_system_entry_point: entry,
                            define_initialization: init,
                            enable_backtrace_on_crash: entry
                                && backtrace_on_crash == BacktraceOnCrash::On
                        }
                    );
                }
            }
        }
    }
}
