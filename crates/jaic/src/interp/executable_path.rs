//! `jaic run`'s answer to "where is my executable": the program asks the OS
//! (`_NSGetExecutablePath`, `readlink("/proc/self/exe")`, `GetModuleFileNameW(null)`), which
//! would name `jaic` itself. With `Interp::run_executable` set, these calls instead report the
//! executable `jaic build` would write, so a program that finds its data next to itself
//! (`join(path_strip_filename(get_path_of_running_executable()), "../assets")`) runs the same
//! either way. Compile-time code keeps the real answer (the compiler, as with any compiler).
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
            _ => return None,
        };
        Some(result.map(|value| vec![value]))
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
