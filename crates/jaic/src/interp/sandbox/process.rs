//! Output, exiting, and process-level procedures.

use super::*;

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
                let bytes = if len == 0 {
                    Vec::new()
                } else {
                    unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec()
                };
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
            LibcFn::Getenv => 0,
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
