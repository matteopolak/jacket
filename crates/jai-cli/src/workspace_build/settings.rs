//! Seed explicit CLI bootstrap settings through the same typed driver requests.
use crate::Error;
use jai_driver::CompilerSession;

pub(super) fn seed_runtime_settings(
    session: &mut CompilerSession,
    bootstrap: &jai_driver::modules::BootstrapOptions,
) -> Result<(), Error> {
    use jai_types::{BacktraceOnCrash, RuntimeSupportMode};
    use jai_vm::{BuildOption, CompilerEffects, CompilerRequest, EffectOutcome};
    let Some(runtime) = &bootstrap.runtime_support else {
        return Ok(());
    };
    let flags = runtime.parameters;
    let mode = match (flags.define_system_entry_point, flags.define_initialization) {
        (true, true) => RuntimeSupportMode::EntryPointAndInitialization,
        (false, true) => RuntimeSupportMode::InitializationOnly,
        (false, false) => RuntimeSupportMode::Omit,
        (true, false) => {
            return Err(Error::Arguments(
                "Runtime_Support entry point requires initialization",
            ));
        }
    };
    if flags.enable_backtrace_on_crash && !flags.define_system_entry_point {
        return Err(Error::Arguments(
            "Runtime_Support backtrace requires its entry point",
        ));
    }
    session.begin();
    for option in [
        BuildOption::RuntimeSupport(mode),
        BuildOption::BacktraceOnCrash(if flags.enable_backtrace_on_crash {
            BacktraceOnCrash::On
        } else {
            BacktraceOnCrash::Off
        }),
    ] {
        match session.request(CompilerRequest::SetBuildOption {
            workspace: session.root(),
            option,
        }) {
            EffectOutcome::Ready(_) => {}
            EffectOutcome::Pending(_) => {
                session
                    .finish(false)
                    .map_err(|error| Error::Source(error.to_string()))?;
                return Err(Error::Source(
                    "initial Runtime_Support settings are awaiting compiler input".into(),
                ));
            }
            EffectOutcome::Rejected(error) => {
                session
                    .finish(false)
                    .map_err(|error| Error::Source(error.to_string()))?;
                return Err(Error::Source(error.to_string()));
            }
        }
    }
    session
        .finish(true)
        .map_err(|error| Error::Source(error.to_string()))
}
