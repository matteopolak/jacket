//! The data a derive produces: the options of a struct and how to store into it.
use crate::error::Error;
use crate::parse;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// What an option is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A switch.
    Flag,
    /// Takes a value: `-out x`, `-out=x`, or glued when `attached`.
    Value,
    /// `+Name` and the arguments up to the next marker.
    Group,
    /// A marker (`-`) and every argument after it, up to `until`.
    Rest,
    /// A bare argument.
    Positional,
}

/// One option, as the derive declares it.
#[derive(Clone, Copy, Debug)]
pub struct Opt {
    /// Where to store: the position among a struct's options, flattened ones after its own.
    pub id: usize,
    pub kind: Kind,
    /// Spellings that are listed and suggested. For `Rest`, the marker.
    pub names: &'static [&'static str],
    /// Spellings accepted but neither listed nor suggested.
    pub aliases: &'static [&'static str],
    /// The value's name in help (`<name>`).
    pub value: &'static str,
    /// What a missing value should have been (`a file name`).
    pub needs: &'static str,
    /// The help label, when it is not the spellings.
    pub synopsis: &'static str,
    pub help: &'static str,
    pub section: &'static str,
    /// What completes the value: `file`, `dir` or nothing.
    pub hint: &'static str,
    /// A label for tools that look at options, not parse them.
    pub tag: &'static str,
    pub hidden: bool,
    /// Stop parsing once seen.
    pub exit: bool,
    pub attached: bool,
    /// The value is kept as the OS gave it, not decoded as UTF-8 (`FromOsArg`).
    pub os: bool,
    /// For `Rest`: the marker that ends it.
    pub until: &'static str,
    /// For `Group`: what a group name starts with.
    pub prefix: &'static str,
}

impl Opt {
    /// The fields a derive leaves at their defaults.
    pub const BLANK: Opt = Opt {
        id: 0,
        kind: Kind::Flag,
        names: &[],
        aliases: &[],
        value: "",
        needs: "",
        synopsis: "",
        help: "",
        section: "",
        hint: "",
        tag: "",
        hidden: false,
        exit: false,
        attached: false,
        os: false,
        until: "",
        prefix: "",
    };

    /// Whether `name` is one of its spellings.
    pub fn spelled(&self, name: &str) -> bool {
        self.names.contains(&name) || self.aliases.contains(&name)
    }

    /// Whether it is listed in help.
    pub fn listed(&self) -> bool {
        !self.hidden && !self.help.is_empty()
    }

    /// The label help gives it.
    pub fn label(&self) -> String {
        if !self.synopsis.is_empty() {
            return self.synopsis.to_string();
        }
        let names = self.names.join(", ");
        if self.kind == Kind::Value {
            format!("{names} <{}>", self.value)
        } else {
            names
        }
    }
}

/// What a struct says about the tool as a whole.
#[derive(Clone, Copy, Debug)]
pub struct Meta {
    pub name: &'static str,
    pub version: &'static str,
    pub usage: &'static str,
    /// The struct's first doc line.
    pub about: &'static str,
    /// The rest of its doc comment.
    pub description: &'static str,
    /// Section titles, in the order help lists them.
    pub sections: &'static [&'static str],
}

/// What the parser hands a struct to store.
#[derive(Clone, Copy, Debug)]
pub enum Input<'a> {
    Flag,
    Value(&'a str),
    /// A value of an `os` option.
    Os(&'a OsStr),
    /// A group's `Name`, without its prefix.
    Start(&'a str),
    /// A group's argument, a `Rest` argument or a positional.
    Arg(&'a str),
    /// A `Rest` marker was seen.
    Begin,
}

/// A struct of options; `#[derive(Args)]` writes this.
pub trait Args: Default + Sized {
    /// How many options it declares, the flattened ones' included.
    const COUNT: usize;

    const META: Meta;

    /// Append its options, numbered from `offset`.
    fn options(offset: usize, out: &mut Vec<Opt>);

    /// Store `input` for the option `id`, spelt `name`.
    fn apply(&mut self, id: usize, name: &str, input: Input<'_>) -> Result<(), Error>;

    /// The options and the tool's description.
    fn spec() -> Spec {
        let mut opts = Vec::new();
        Self::options(0, &mut opts);
        Spec {
            meta: Self::META,
            opts,
        }
    }

    /// Read `args` (the command line without the program name).
    ///
    /// ```
    /// use argdef::Args;
    ///
    /// #[derive(Args, Default)]
    /// #[args(name = "demo", usage = "demo [options] file")]
    /// struct Demo {
    ///     /// print more
    ///     #[arg(name = "-verbose", section = "Options")]
    ///     verbose: bool,
    ///     /// the output file
    ///     #[arg(name = "-out", value = "file", hint = "file")]
    ///     out: Option<String>,
    /// }
    ///
    /// let args = ["-verbose", "-out", "a.txt"].map(String::from);
    /// let demo = Demo::parse(&args).unwrap();
    /// assert!(demo.verbose);
    /// assert_eq!(demo.out.as_deref(), Some("a.txt"));
    /// assert!(Demo::parse(&["-nope".to_string()]).is_err());
    /// ```
    fn parse<S: AsRef<OsStr>>(args: &[S]) -> Result<Self, Error> {
        parse::parse(&Self::spec(), args)
    }
}

/// A value's type: how an option's text becomes one.
pub trait FromArg: Sized {
    /// `name` is how the option was spelt, for the message.
    fn from_arg(name: &str, text: &str) -> Result<Self, Error>;
}

impl FromArg for String {
    fn from_arg(_: &str, text: &str) -> Result<Self, Error> {
        Ok(text.to_string())
    }
}

impl FromArg for PathBuf {
    fn from_arg(_: &str, text: &str) -> Result<Self, Error> {
        Ok(PathBuf::from(text))
    }
}

macro_rules! numbers {
    ($($ty:ty),*) => {$(
        impl FromArg for $ty {
            fn from_arg(name: &str, text: &str) -> Result<Self, Error> {
                text.parse().map_err(|_| {
                    Error::invalid(format!("`{name}` takes a whole number, not `{text}`"))
                        .help(format!("for example `{name} 4`"))
                })
            }
        }
    )*};
}

numbers!(usize, u8, u16, u32, u64, i32, i64);

/// A value's type when the option keeps what the OS gave (`os`): a path, or an argument for
/// another program.
pub trait FromOsArg: Sized {
    fn from_os_arg(name: &str, value: &OsStr) -> Result<Self, Error>;
}

impl FromOsArg for OsString {
    fn from_os_arg(_: &str, value: &OsStr) -> Result<Self, Error> {
        Ok(value.to_os_string())
    }
}

impl FromOsArg for PathBuf {
    fn from_os_arg(_: &str, value: &OsStr) -> Result<Self, Error> {
        Ok(PathBuf::from(value))
    }
}

/// A `+Name` and the arguments after it, in a struct.
pub trait Group {
    fn start(name: &str) -> Self;
    fn push(&mut self, arg: &str);
}

/// The options of a tool.
#[derive(Clone, Debug)]
pub struct Spec {
    pub meta: Meta,
    pub opts: Vec<Opt>,
}

/// One argument that [`Spec::scan`] read.
#[derive(Clone, Debug)]
pub struct Token<'a> {
    /// Its position in the arguments.
    pub index: usize,
    /// The option as spelt, without an inline value.
    pub name: &'a str,
    pub opt: Option<Opt>,
    /// A value option's value, when it is UTF-8.
    pub value: Option<String>,
    /// Whether the value was part of the argument (`-out=x`, `-Dx`), not the next one.
    pub inline: bool,
    /// How many arguments it took: two for a value in the next argument.
    pub width: usize,
}

/// The options among some arguments, up to the first marker.
#[derive(Clone, Debug)]
pub struct Scan<'a> {
    pub tokens: Vec<Token<'a>>,
    /// Where a group or rest marker ended the options.
    pub boundary: Option<usize>,
    /// A value option with no value, which ended the scan.
    pub error: Option<Error>,
}

impl Spec {
    /// The option spelt `name` (not a marker or a positional).
    pub fn find(&self, name: &str) -> Option<&Opt> {
        self.opts
            .iter()
            .find(|o| matches!(o.kind, Kind::Flag | Kind::Value) && o.spelled(name))
    }

    /// Split `arg` into the option it spells and its inline value.
    pub fn split<'a>(&self, arg: &'a str) -> (&'a str, Option<String>, Option<&Opt>) {
        if let Some(opt) = self.find(arg) {
            return (arg, None, Some(opt));
        }
        if let Some((name, value)) = arg.split_once('=')
            && let Some(opt) = self.find(name).filter(|o| o.kind == Kind::Value)
        {
            return (name, Some(value.to_string()), Some(opt));
        }
        for opt in self
            .opts
            .iter()
            .filter(|o| o.kind == Kind::Value && o.attached)
        {
            for name in opt.names.iter().chain(opt.aliases) {
                // `-Dwarnings=x` is not `-D` with the value `warnings=x`.
                if arg.len() > name.len() && arg.starts_with(name) && !arg.contains('=') {
                    return (
                        &arg[..name.len()],
                        Some(arg[name.len()..].to_string()),
                        Some(opt),
                    );
                }
            }
        }
        (arg, None, None)
    }

    /// The `Rest` option that `arg` starts, or the group `arg` opens.
    pub(crate) fn marker(&self, arg: &str) -> Option<&Opt> {
        self.opts.iter().find(|o| match o.kind {
            Kind::Rest => o.names.contains(&arg),
            Kind::Group => arg.len() > o.prefix.len() && arg.starts_with(o.prefix),
            _ => false,
        })
    }

    /// Read the options in `args` without storing them: for a tool that only inspects a command
    /// line. It stops at a marker, and at an option missing its value.
    ///
    /// ```
    /// use argdef::Args;
    ///
    /// #[derive(Args, Default)]
    /// #[args(name = "demo", usage = "demo [options]")]
    /// struct Demo {
    ///     /// print more
    ///     #[arg(name = "-verbose", section = "Options")]
    ///     verbose: bool,
    ///     /// the output file
    ///     #[arg(name = "-out", value = "file", hint = "file")]
    ///     out: Option<String>,
    /// }
    ///
    /// let args = ["-verbose", "-out", "a.txt", "x"].map(String::from);
    /// let scan = Demo::spec().scan(&args);
    /// assert_eq!(scan.tokens.len(), 3); // `x` is a token with no option
    /// assert_eq!(scan.tokens[1].value.as_deref(), Some("a.txt"));
    /// assert!(scan.error.is_none());
    /// ```
    pub fn scan<'a, S: AsRef<OsStr>>(&self, args: &'a [S]) -> Scan<'a> {
        let mut scan = Scan {
            tokens: Vec::new(),
            boundary: None,
            error: None,
        };
        let mut index = 0;
        while let Some(os) = args.get(index) {
            // An argument that is not UTF-8 is a value or an operand, never an option.
            let Some(arg) = os.as_ref().to_str() else {
                scan.tokens.push(Token {
                    index,
                    name: "",
                    opt: None,
                    value: None,
                    inline: false,
                    width: 1,
                });
                index += 1;
                continue;
            };
            if self.marker(arg).is_some() {
                scan.boundary = Some(index);
                break;
            }
            let (name, inline, opt) = self.split(arg);
            let mut token = Token {
                index,
                name,
                opt: opt.copied(),
                value: inline.clone(),
                inline: inline.is_some(),
                width: 1,
            };
            if let Some(opt) = opt.filter(|o| o.kind == Kind::Value)
                && inline.is_none()
            {
                let Some(value) = args.get(index + 1) else {
                    scan.error = Some(Error::missing(name, opt.needs));
                    break;
                };
                token.value = value.as_ref().to_str().map(String::from);
                token.width = 2;
            }
            index += token.width;
            scan.tokens.push(token);
        }
        scan
    }
}
