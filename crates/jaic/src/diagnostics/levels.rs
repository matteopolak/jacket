//! How loud a warning is: the `-A`, `-W` and `-D` flags of `jaic` and `jailint` and the settings
//! that mirror them.
//!
//! A [`Level`] is `allow` (say nothing), `warn` (print it, the default) or `deny` (print it as an
//! error and fail). [`Levels`] is a list of flags, applied in the order they were written, where a
//! later flag wins. A flag names a code (`format-missing-argument`), `leniencies` (every code that
//! marks a form the official compiler rejects) or `warnings` (every warning).
use super::{Code, Diagnostic, Severity};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Allow,
    Warn,
    Deny,
}

impl Level {
    pub fn parse(text: &str) -> Option<Level> {
        match text {
            "allow" => Some(Level::Allow),
            "warn" => Some(Level::Warn),
            "deny" => Some(Level::Deny),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Level::Allow => "allow",
            Level::Warn => "warn",
            Level::Deny => "deny",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Warnings,
    Leniencies,
    Code(Code),
}

/// The levels the command line sets for the compiler's warnings.
#[derive(Clone, Debug, Default)]
pub struct Levels {
    settings: Vec<(Target, Level)>,
}

impl Levels {
    /// The flags `(name, level)` in the order they were written. An unknown name, or a name that
    /// is an error (which `-A` and `-W` cannot soften), is refused with a message to show.
    pub fn parse(flags: &[(String, Level)]) -> Result<Levels, String> {
        let mut settings = Vec::new();
        for (name, level) in flags {
            let target = match name.as_str() {
                "warnings" => Target::Warnings,
                "leniencies" => Target::Leniencies,
                other => match Code::parse(other) {
                    Some(code) => {
                        if code.severity() == Severity::Error && *level != Level::Deny {
                            return Err(format!(
                                "`{other}` is an error, so it cannot be set to {}\nhelp: only warnings can be allowed or warned; `-D {other}` is already the case",
                                level.as_str()
                            ));
                        }
                        Target::Code(code)
                    }
                    None => return Err(unknown(other)),
                },
            };
            settings.push((target, *level));
        }
        Ok(Levels {
            settings,
        })
    }

    /// The level of `code`: the last flag that names it, or `warn`.
    pub fn level(&self, code: Code) -> Level {
        self.settings
            .iter()
            .rev()
            .find(|(target, _)| match target {
                Target::Warnings => code.severity() == Severity::Warning,
                Target::Leniencies => code.is_leniency(),
                Target::Code(named) => *named == code,
            })
            .map_or(Level::Warn, |(_, level)| *level)
    }

    /// `d` as the levels have it: `None` when it is allowed, an error when it is denied.
    pub fn apply(&self, mut d: Diagnostic) -> Option<Diagnostic> {
        if d.severity != Severity::Warning {
            return Some(d);
        }
        match self.level(d.code) {
            Level::Allow => None,
            Level::Warn => Some(d),
            Level::Deny => {
                d.severity = Severity::Error;
                Some(d)
            }
        }
    }
}

fn unknown(name: &str) -> String {
    let names = Code::ALL
        .iter()
        .filter(|code| code.severity() == Severity::Warning)
        .map(|code| code.name())
        .chain(["warnings", "leniencies"]);
    match crate::suggest::closest(name, names) {
        Some(near) => format!("unknown warning `{name}`\nhelp: did you mean `{near}`?"),
        None => format!(
            "unknown warning `{name}`\nhelp: `jacket explain` lists the codes; `warnings` and `leniencies` name groups"
        ),
    }
}
