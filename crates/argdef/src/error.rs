//! What can be wrong with a command line.

/// Which kind of mistake an [`Error`] is, so a tool can word its own message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// An option nothing declares.
    Unknown,
    /// An option that takes a value was last.
    Missing,
    /// A value or argument that was refused.
    Invalid,
}

/// A command-line mistake: a message, then help lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    /// The option involved (`-timing`), when there is one.
    pub subject: String,
    pub message: String,
    pub help: Vec<String>,
}

impl Error {
    /// A refused value or argument.
    pub fn invalid(message: impl Into<String>) -> Self {
        Error {
            kind: ErrorKind::Invalid,
            subject: String::new(),
            message: message.into(),
            help: Vec::new(),
        }
    }

    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    /// An argument that is not UTF-8 where only an `os` option's value may be anything.
    pub(crate) fn not_utf8(arg: &std::ffi::OsStr, os_options: &[&str]) -> Self {
        let mut error = Error::invalid(format!("argument `{}` is not valid UTF-8", arg.display()));
        if let Some((last, first)) = os_options.split_last() {
            let list = if first.is_empty() {
                (*last).to_string()
            } else {
                format!("{} and {last}", first.join(", "))
            };
            error = error.help(format!("only the values of {list} may be any bytes"));
        }
        error
    }

    pub(crate) fn unknown(name: &str) -> Self {
        Error {
            kind: ErrorKind::Unknown,
            subject: name.to_string(),
            message: format!("unknown option `{name}`"),
            help: Vec::new(),
        }
    }

    pub(crate) fn missing(name: &str, needs: &str) -> Self {
        let needs = if needs.is_empty() {
            "a value"
        } else {
            needs
        };
        Error {
            kind: ErrorKind::Missing,
            subject: name.to_string(),
            message: format!("`{name}` needs {needs}"),
            help: Vec::new(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {
}
