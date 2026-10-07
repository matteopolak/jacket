//! `jaic run`'s answer to "where is my executable": the program asks the OS
//! (`_NSGetExecutablePath`, `readlink("/proc/self/exe")`, `GetModuleFileNameW(null)`), which
//! would name `jaic` itself. With `Interp::run_executable` set, these calls instead report the
//! executable `jaic build` would write, so a program that finds its data next to itself
//! (`join(path_strip_filename(get_path_of_running_executable()), "../assets")`) runs the same
//! either way. Compile-time code keeps the real answer (the compiler, as with any compiler).
//!
//! The same goes for the command line on Windows: Runtime_Support reads the arguments again
//! from `GetCommandLineW` (the C runtime's `argv` is in the ANSI code page), which would give
//! `jaic run file.jai -- ...`. While `run_executable` is set it gets `Interp::run_arguments`
//! quoted so that `CommandLineToArgvW` splits them back unchanged.
use super::{Interp, Res};
use crate::ir::Ty;

impl Interp {
    /// The executable-path queries `run_executable` answers. `None`: not one of them (or no
    /// substitution), so the call goes to the OS.
    pub(super) fn executable_path_foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
    ) -> Option<Res<Vec<u64>>> {
        let path = self.run_executable.clone()?;
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        let result = match symbol {
            // int _NSGetExecutablePath(char *buf, uint32_t *size): -1 and the size needed
            // when the buffer is too small.
            "_NSGetExecutablePath" => (|| {
                let needed = path.len() as u64 + 1;
                let size = self.load(Ty::I32, arg(1))?;
                if arg(0) == 0 || size < needed {
                    self.store(Ty::I32, arg(1), needed)?;
                    return Ok(u32::MAX as u64);
                }
                self.write_bytes(arg(0), path.as_bytes())?;
                self.store(Ty::I8, arg(0) + path.len() as u64, 0)?;
                Ok(0)
            })(),
            // ssize_t readlink(const char *path, char *buf, size_t cap): no terminator, cut
            // to `cap`.
            "readlink" if self.c_string(arg(0)).ok()? == "/proc/self/exe" => (|| {
                let bytes = &path.as_bytes()[..path.len().min(arg(2) as usize)];
                self.write_bytes(arg(1), bytes)?;
                Ok(bytes.len() as u64)
            })(),
            // DWORD GetModuleFileNameW(HMODULE module, WCHAR *buf, DWORD cap) for the
            // executable (module null): when cut, `cap` characters and the result `cap`.
            "GetModuleFileNameW" if arg(0) == 0 => (|| {
                let wide: Vec<u16> = path.encode_utf16().collect();
                let cap = arg(2) as usize;
                if cap == 0 {
                    return Ok(0);
                }
                let fits = wide.len() < cap;
                let count = if fits {
                    wide.len()
                } else {
                    cap - 1
                };
                for (i, unit) in wide[..count].iter().enumerate() {
                    self.store(Ty::I16, arg(1) + 2 * i as u64, *unit as u64)?;
                }
                self.store(Ty::I16, arg(1) + 2 * count as u64, 0)?;
                Ok(if fits {
                    wide.len() as u64
                } else {
                    cap as u64
                })
            })(),
            // LPWSTR GetCommandLineW(void): the program's arguments, not jaic's.
            "GetCommandLineW" => Ok(self.run_command_line()),
            // No arguments at all (`jaic run` without `--`) has no Windows spelling: an empty
            // command line splits into the executable's path. Report the split as failed, which
            // keeps the empty `argv` the program started with.
            "CommandLineToArgvW"
                if self.run_arguments.is_empty() && arg(0) == self.run_command_line() =>
            {
                self.store(Ty::I32, arg(1), 0).map(|()| 0)
            }
            _ => return None,
        };
        Some(result.map(|value| vec![value]))
    }

    fn run_command_line(&mut self) -> u64 {
        *self.run_command_line.get_or_insert_with(|| {
            let mut wide: Vec<u16> = windows_command_line(&self.run_arguments)
                .encode_utf16()
                .collect();
            wide.push(0);
            Box::leak(wide.into_boxed_slice()).as_ptr() as u64
        })
    }

    fn write_bytes(&self, address: u64, bytes: &[u8]) -> Res<()> {
        for (i, byte) in bytes.iter().enumerate() {
            self.store(Ty::I8, address + i as u64, *byte as u64)?;
        }
        Ok(())
    }

    fn c_string(&self, address: u64) -> Res<String> {
        let mut bytes = Vec::new();
        loop {
            let byte = self.load(Ty::I8, address + bytes.len() as u64)? as u8;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// `args` as one Windows command line that `CommandLineToArgvW` splits back into `args`. The
/// first argument (the program) is read up to the next quote with no escapes; the others treat
/// backslashes before a quote as escapes.
pub fn windows_command_line(args: &[String]) -> String {
    let mut line = String::new();
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            line.push(' ');
        }
        let plain = !arg.is_empty() && !arg.contains([' ', '\t', '\n', '\x0b', '"']);
        if plain {
            line.push_str(arg);
        } else if index == 0 {
            line.push('"');
            line.push_str(arg);
            line.push('"');
        } else {
            line.push('"');
            let mut backslashes = 0;
            for c in arg.chars() {
                match c {
                    '\\' => backslashes += 1,
                    '"' => {
                        line.extend(std::iter::repeat_n('\\', 2 * backslashes + 1));
                        line.push('"');
                        backslashes = 0;
                    }
                    _ => {
                        line.extend(std::iter::repeat_n('\\', backslashes));
                        line.push(c);
                        backslashes = 0;
                    }
                }
            }
            line.extend(std::iter::repeat_n('\\', 2 * backslashes));
            line.push('"');
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::windows_command_line;

    fn line(args: &[&str]) -> String {
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        windows_command_line(&args)
    }

    #[test]
    fn quotes_only_what_command_line_to_argv_would_split() {
        assert_eq!(line(&[r"C:\a\prog.jai", "x", "y"]), r"C:\a\prog.jai x y");
        assert_eq!(line(&[r"C:\my dir\p.jai", ""]), r#""C:\my dir\p.jai" """#);
        assert_eq!(line(&["p", "two words", r"dir\"]), r#"p "two words" dir\"#);
        assert_eq!(line(&["p", r"my dir\"]), r#"p "my dir\\""#);
        assert_eq!(
            line(&["p", r#"say "hi""#, r#"a\"b"#]),
            r#"p "say \"hi\"" "a\\\"b""#
        );
        assert_eq!(line(&["p", r"c:\path\file"]), r"p c:\path\file");
    }
}
