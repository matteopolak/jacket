//! Shell completion scripts from a spec.
use crate::error::Error;
use crate::spec::{FromArg, Kind, Opt, Spec};

/// A shell a completion script can be written for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    /// Every shell, as `-completions` names them.
    pub const NAMES: &'static str = "bash, zsh or fish";
}

impl FromArg for Shell {
    fn from_arg(name: &str, text: &str) -> Result<Self, Error> {
        match text {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err(
                Error::invalid(format!("unknown shell `{text}` for `{name}`"))
                    .help(format!("use {}", Shell::NAMES)),
            ),
        }
    }
}

/// The spellings of an option that completion offers.
fn spellings(opt: &Opt) -> impl Iterator<Item = &'static str> {
    opt.names.iter().chain(opt.aliases).copied()
}

fn first_line(opt: &Opt) -> &'static str {
    opt.help.lines().next().unwrap_or_default()
}

impl Spec {
    /// A script that completes this tool's options in `shell`, for the command `meta.name`.
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
    /// let script = Demo::spec().completions(argdef::Shell::Fish);
    /// assert!(script.contains("complete -c demo -o verbose"));
    /// assert!(script.contains("complete -c demo -o out -r -F"));
    /// ```
    pub fn completions(&self, shell: Shell) -> String {
        let options: Vec<&Opt> = self
            .opts
            .iter()
            .filter(|o| matches!(o.kind, Kind::Flag | Kind::Value) && !o.hidden)
            .collect();

        match shell {
            Shell::Bash => self.bash(&options),
            Shell::Zsh => self.zsh(&options),
            Shell::Fish => self.fish(&options),
        }
    }

    fn bash(&self, options: &[&Opt]) -> String {
        let name = self.meta.name;
        let function = format!("_{}", name.replace('-', "_"));
        let words: Vec<&str> = options.iter().flat_map(|o| spellings(o)).collect();
        let mut cases = String::new();

        for opt in options.iter().filter(|o| o.kind == Kind::Value) {
            let action = match opt.hint {
                "file" => format!("{function}_reply compgen -f -- \"$cur\""),
                "dir" => format!("{function}_reply compgen -d -- \"$cur\""),
                _ => "COMPREPLY=()".to_string(),
            };
            let pattern: Vec<&str> = spellings(opt).collect();

            cases.push_str(&format!(
                "        {})\n            {action}\n            return ;;\n",
                pattern.join("|")
            ));
        }

        // `read` into the array keeps names with spaces and glob characters whole (`mapfile` needs
        // bash 4). Bash splits `-name=value` at the `=`, which then is a word of its own.
        format!(
            "# bash completion for {name}: source it, or save it as /etc/bash_completion.d/{name}\n\
             {function}_reply() {{\n\
             \x20   local line\n\
             \x20   COMPREPLY=()\n\
             \x20   while IFS= read -r line; do COMPREPLY+=(\"$line\"); done < <(\"$@\")\n\
             }}\n\
             {function}() {{\n\
             \x20   local cur prev\n\
             \x20   cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n\
             \x20   prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n\
             \x20   if [[ \"$cur\" == \"=\" ]]; then cur=\"\"; fi\n\
             \x20   if [[ \"$prev\" == \"=\" ]]; then prev=\"${{COMP_WORDS[COMP_CWORD-2]}}\"; fi\n\
             \x20   case \"$prev\" in\n{cases}    esac\n\
             \x20   if [[ \"$cur\" == -* ]]; then\n\
             \x20       COMPREPLY=($(compgen -W \"{}\" -- \"$cur\"))\n\
             \x20       return\n\
             \x20   fi\n\
             \x20   {function}_reply compgen -f -- \"$cur\"\n\
             }}\n\
             complete -o filenames -F {function} {name}\n",
            words.join(" ")
        )
    }

    fn zsh(&self, options: &[&Opt]) -> String {
        let name = self.meta.name;
        let escape = |text: &str| {
            text.replace('\'', "'\\''")
                .replace('[', "\\[")
                .replace(']', "\\]")
        };
        let mut specs = String::new();

        for opt in options {
            for spelling in spellings(opt) {
                let help = escape(first_line(opt));
                let action = match (opt.kind, opt.hint) {
                    (Kind::Value, "dir") => format!(":{}:_files -/", opt.value),
                    (Kind::Value, "file") => format!(":{}:_files", opt.value),
                    (Kind::Value, _) => format!(":{}: ", opt.value),
                    _ => String::new(),
                };

                specs.push_str(&format!("    '{spelling}[{help}]{action}' \\\n"));

                if opt.kind == Kind::Value {
                    // `-name=value`, and `-Dvalue` for an option that takes its value glued on.
                    specs.push_str(&format!("    '{spelling}=[{help}]{action}' \\\n"));

                    if opt.attached {
                        specs.push_str(&format!("    '{spelling}-[{help}]{action}' \\\n"));
                    }
                }
            }
        }

        format!(
            "#compdef {name}\n# zsh completion for {name}: save it as _{name} in a directory of $fpath\n\
             _arguments \\\n{specs}    '*:file:_files'\n"
        )
    }

    fn fish(&self, options: &[&Opt]) -> String {
        let name = self.meta.name;
        let mut out = format!(
            "# fish completion for {name}: save it as ~/.config/fish/completions/{name}.fish\n"
        );

        for opt in options {
            for spelling in spellings(opt) {
                let flag = if let Some(long) = spelling.strip_prefix("--") {
                    format!("-l {long}")
                } else {
                    let short = spelling.trim_start_matches('-');

                    if short.chars().count() == 1 {
                        format!("-s {short}")
                    } else {
                        format!("-o {short}")
                    }
                };
                let value = match (opt.kind, opt.hint) {
                    (Kind::Value, "file") => " -r -F",
                    (Kind::Value, "dir") => " -r -f -a '(__fish_complete_directories)'",
                    (Kind::Value, _) => " -x",
                    _ => "",
                };
                let help = first_line(opt).replace('\\', "\\\\").replace('\'', "\\'");

                out.push_str(&format!("complete -c {name} {flag}{value} -d '{help}'\n"));
            }
        }

        out
    }
}
