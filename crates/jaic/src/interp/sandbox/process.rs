//! Output, exiting, and process-level procedures.

use super::{EINVAL, EPERM, Host, LibcFn, SandboxHost, read_bytes};

impl SandboxHost {
    /// Program output, exiting, and the process-level procedures a sandboxed run answers with
    /// fixed values or refuses.
    pub(super) fn process_call(
        &mut self,
        fun: LibcFn,
        arg: impl Fn(usize) -> u64,
    ) -> Option<Result<u64, String>> {
        Some(Ok(match fun {
            LibcFn::WasmWriteString => {
                let (len, ptr, to_stderr) = (arg(0) as usize, arg(1), arg(2) & 1 != 0);
                let bytes = read_bytes(ptr, len);
                self.write(&bytes, to_stderr);
                0
            }
            LibcFn::Exit | LibcFn::ImmediateExit => {
                return Some(Err(format!("exit({})", arg(0) as i32)));
            }
            LibcFn::Abort => return Some(Err("abort()".into())),
            // Threads are scheduled by the interpreter; these only run when no thread exists.
            LibcFn::PthreadMutexattrInit
            | LibcFn::PthreadMutexattrDestroy
            | LibcFn::PthreadMutexattrSettype
            | LibcFn::PthreadAttrInit
            | LibcFn::PthreadAttrDestroy
            | LibcFn::PthreadAttrSetstacksize
            | LibcFn::PthreadMutexLock
            | LibcFn::PthreadMutexUnlock
            | LibcFn::PthreadMutexInit
            | LibcFn::PthreadMutexDestroy => 0,
            LibcFn::Isatty => 0,
            LibcFn::Getenv => self.getenv(arg(0)),
            // Whether the embedding page offers a host procedure (stdlib/Extensions/WebGPU): not here. The
            // browser playground's host (jai-wasm `host_bridge`) asks its page instead.
            LibcFn::JaiHostProvides => 0,
            LibcFn::WasmDebugBreak => return Some(Err("debug_break() was called".into())),
            LibcFn::SchedYield => 0,
            // One processor, 4 KiB pages: the browser sandbox is a single cooperative thread.
            LibcFn::Sysconf => match arg(0) as i32 {
                83 | 84 => 1,
                30 => 4096,
                _ => self.set_errno(EINVAL),
            },
            LibcFn::Getpid => 1,
            // Secure random bytes, at most 256 per call like the C function. Elsewhere (the
            // browser engine) the embedding host answers.
            #[cfg(unix)]
            LibcFn::Getentropy => match host_entropy(arg(1) as usize) {
                Some(bytes) => {
                    super::write_bytes(arg(0), &bytes);
                    0
                }
                None => self.set_errno(EINVAL),
            },
            // Nothing a sandboxed program runs may start another process. It fails as a
            // refused call does, so code with a fallback (a build stamping its version from
            // `git`) carries on, and code that called it holding a lock lets go of it.
            LibcFn::Pipe
            | LibcFn::Pipe2
            | LibcFn::Socketpair
            | LibcFn::Fork
            | LibcFn::Vfork
            | LibcFn::Execv
            | LibcFn::Execve
            | LibcFn::Execvp => self.set_errno(EPERM),
            LibcFn::PosixSpawn | LibcFn::PosixSpawnp => EPERM,
            // Nor share memory with one (the memory debugger's visualizer channel).
            LibcFn::ShmOpen | LibcFn::ShmUnlink => self.set_errno(EPERM),
            _ => return None,
        }))
    }
}

impl SandboxHost {
    /// `getenv`: unset unless the embedder allows the real environment (read-only, so asking
    /// twice gives the same answer while the variable is unchanged).
    fn getenv(&mut self, name: u64) -> u64 {
        if !self.real_environment {
            return 0;
        }
        let name = super::cstr(name);
        let Ok(value) = std::env::var(&name) else {
            return 0;
        };
        let key = (name, value);
        if let Some(&text) = self.environment_texts.get(&key) {
            return text;
        }
        let text = self.alloc(key.1.len() + 1);
        if text != 0 {
            super::write_bytes(text, key.1.as_bytes());
            self.environment_texts.insert(key, text);
        }
        text
    }
}

/// `len` bytes from the operating system's secure random source, or none when `len` is over the
/// C function's limit or the source cannot be read.
#[cfg(unix)]
fn host_entropy(len: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    if len > 256 {
        return None;
    }
    let mut bytes = vec![0; len];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .ok()?;
    Some(bytes)
}
