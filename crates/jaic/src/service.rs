//! Compile-time services: long-lived worker processes that compile-time code asks questions.
//!
//! A service is a Jai program (`serve` in `Extensions/Service`) that reads framed requests on
//! stdin and answers each on stdout. [`Services`] starts one `jaic <file> -interp` worker per
//! service file the first time it is called, keeps it for later calls, and restarts it when the
//! service's sources (or `jaic` itself) change. Compile-time code reaches it through the
//! `compile_time_service` hook (`Host::service`); native builds wait for the answer, the language
//! server takes [`ServiceStatus::Pending`] and republishes when the answer arrives.
//!
//! Wire format, both directions: an 8-byte little-endian length, then that many bytes. An answer
//! starts with one status byte ([`ServiceStatus`]) before its body.
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How a call ended. The numbers are what `compile_time_service` returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceStatus {
    /// The body is the service's answer.
    Ok = 0,
    /// The answer is not ready yet (editor only); the caller is asked again later.
    Pending = 1,
    /// The service cannot answer here (no database, no `jaic`, untrusted, a sandbox without
    /// services). The body says why. Compile-time code should treat the result as unknown.
    Unavailable = 2,
    /// The service failed: it crashed, timed out, ran out of memory or reported an error. The
    /// body is the message.
    Failed = 3,
}

impl ServiceStatus {
    fn from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            0 => Self::Ok,
            1 => Self::Pending,
            2 => Self::Unavailable,
            3 => Self::Failed,
            _ => return None,
        })
    }
}

/// A status and its body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceReply {
    pub status: ServiceStatus,
    pub body: Vec<u8>,
}

impl ServiceReply {
    #[must_use]
    pub fn new(status: ServiceStatus, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(ServiceStatus::Unavailable, message.into().into_bytes())
    }

    #[must_use]
    pub fn failed(message: impl Into<String>) -> Self {
        Self::new(ServiceStatus::Failed, message.into().into_bytes())
    }

    #[must_use]
    pub fn pending() -> Self {
        Self::new(ServiceStatus::Pending, Vec::new())
    }
}

/// Limits and the `jaic` to run workers with.
#[derive(Clone, Debug)]
pub struct ServiceConfig {
    /// The `jaic` executable; `None` finds one ([`find_jaic`]).
    pub jaic: Option<PathBuf>,
    /// How long one request may take before the worker is stopped.
    pub timeout: Duration,
    /// A worker with no request for this long is stopped.
    pub idle: Duration,
    /// Each worker's allocation cap (`JAIC_MEMORY_LIMIT`), in bytes.
    pub memory_bytes: u64,
    /// After a crash the service is not restarted for this long, doubling per crash up to
    /// `backoff_max`; a good answer resets it.
    pub backoff_min: Duration,
    pub backoff_max: Duration,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            jaic: None,
            timeout: Duration::from_secs(30),
            idle: Duration::from_secs(600),
            memory_bytes: 1 << 30,
            backoff_min: Duration::from_secs(1),
            backoff_max: Duration::from_secs(60),
        }
    }
}

/// Finds the `jaic` to run workers with: this process when it is `jaic`, else one next to this
/// executable (the language server ships beside it), else the first on `PATH`.
#[must_use]
pub fn find_jaic() -> Option<PathBuf> {
    let name = format!("jaic{}", std::env::consts::EXE_SUFFIX);
    let exe = std::env::current_exe().ok();
    if let Some(exe) = &exe {
        if exe.file_name().is_some_and(|n| n == name.as_str()) {
            return Some(exe.clone());
        }
        if let Some(sibling) = exe.parent().map(|dir| dir.join(&name))
            && sibling.is_file()
        {
            return Some(sibling);
        }
    }
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(&name))
        .find(|candidate| candidate.is_file())
}

/// Resolves a service path the way the hook does: relative paths start at `base`.
#[must_use]
pub fn resolve(service: &str, base: Option<&Path>) -> PathBuf {
    let path = Path::new(service);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match base {
        Some(base) => base.join(path),
        None => std::env::current_dir().unwrap_or_default().join(path),
    }
}

/// What a sandboxed host asks for answers (the language server's provider memoizes them and
/// answers [`ServiceStatus::Pending`] while one is computed).
pub trait ServiceProvider {
    fn ask(&self, service: &Path, request: &[u8]) -> ServiceReply;

    /// Some answer was [`ServiceStatus::Pending`] and has not arrived yet.
    fn waiting(&self) -> bool {
        false
    }

    /// Takes in the answers that arrived since the last call: `true` when any did, so what was
    /// compiled while they were pending should be compiled again.
    fn take_arrivals(&self) -> bool {
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{Services, Ticket, fingerprint};

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{ServiceConfig, ServiceReply, ServiceStatus, find_jaic};
    use crate::fxhash::HashMap;
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, Command, Stdio};
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant, SystemTime};

    /// Requests larger than this are refused, and answers larger than this end the worker.
    const MAX_FRAME: u64 = 256 << 20;

    /// How much of a worker's stderr is kept for crash messages.
    const STDERR_TAIL: usize = 4096;

    /// The workers of every service this process has called. Cheap to clone; clones share them.
    #[derive(Clone)]
    pub struct Services {
        config: Arc<ServiceConfig>,
        managers: Arc<Mutex<HashMap<PathBuf, Sender<Job>>>>,
    }

    struct Job {
        request: Vec<u8>,
        reply: Sender<ServiceReply>,
    }

    /// A submitted request whose answer may not have arrived yet.
    pub struct Ticket(Receiver<ServiceReply>);

    impl Ticket {
        /// The answer, if it has arrived. A lost worker thread reads as a failure.
        pub fn poll(&self) -> Option<ServiceReply> {
            match self.0.try_recv() {
                Ok(reply) => Some(reply),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(ServiceReply::failed("the service's manager stopped"))
                }
            }
        }

        /// Waits up to `limit` for the answer.
        pub fn wait(&self, limit: Duration) -> Option<ServiceReply> {
            match self.0.recv_timeout(limit) {
                Ok(reply) => Some(reply),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => {
                    Some(ServiceReply::failed("the service's manager stopped"))
                }
            }
        }
    }

    impl Services {
        #[must_use]
        pub fn new(config: ServiceConfig) -> Self {
            Self {
                config: Arc::new(config),
                managers: Arc::default(),
            }
        }

        /// The process-wide services native builds use.
        pub fn global() -> &'static Self {
            static GLOBAL: std::sync::OnceLock<Services> = std::sync::OnceLock::new();
            GLOBAL.get_or_init(|| Self::new(ServiceConfig::default()))
        }

        #[must_use]
        pub fn config(&self) -> &ServiceConfig {
            &self.config
        }

        /// Queues `request` for `service` (an absolute path) and returns at once.
        pub fn submit(&self, service: &Path, request: Vec<u8>) -> Ticket {
            let (reply, answer) = mpsc::channel();
            let mut job = Job {
                request,
                reply,
            };
            let mut managers = self
                .managers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // A manager that stopped (idle) dropped its receiver: start a new one.
            if let Some(sender) = managers.get(service) {
                match sender.send(job) {
                    Ok(()) => return Ticket(answer),
                    Err(mpsc::SendError(back)) => job = back,
                }
            }
            let (sender, jobs) = mpsc::channel();
            let _ = sender.send(job);
            managers.insert(service.to_path_buf(), sender);
            let manager = Manager::new(service.to_path_buf(), self.config.clone());
            let spawned = std::thread::Builder::new()
                .name("jaic-service".into())
                .spawn(move || manager.run(&jobs));
            if spawned.is_err() {
                managers.remove(service);
            }
            Ticket(answer)
        }

        /// Asks `service` and waits for the answer (a native build).
        pub fn call(&self, service: &Path, request: Vec<u8>) -> ServiceReply {
            let ticket = self.submit(service, request);
            // The manager enforces the timeout itself; the slack covers starting the worker.
            ticket
                .wait(self.config.timeout * 2 + Duration::from_secs(5))
                .unwrap_or_else(|| ServiceReply::failed("the service did not answer"))
        }
    }

    /// One service's worker and its restart state, owned by the thread that serves its jobs.
    struct Manager {
        path: PathBuf,
        config: Arc<ServiceConfig>,
        worker: Option<Worker>,
        fingerprint: Option<u64>,
        checked: Option<Instant>,
        backoff: Duration,
        retry_at: Option<Instant>,
        last_failure: String,
    }

    struct Worker {
        child: Child,
        stdin: ChildStdin,
        answers: Receiver<Result<ServiceReply, String>>,
        stderr: Arc<Mutex<Vec<u8>>>,
    }

    impl Drop for Worker {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    impl Manager {
        fn new(path: PathBuf, config: Arc<ServiceConfig>) -> Self {
            Self {
                path,
                config,
                worker: None,
                fingerprint: None,
                checked: None,
                backoff: Duration::ZERO,
                retry_at: None,
                last_failure: String::new(),
            }
        }

        fn run(mut self, jobs: &Receiver<Job>) {
            loop {
                match jobs.recv_timeout(self.config.idle) {
                    Ok(job) => {
                        let reply = self.answer(job.request);
                        let _ = job.reply.send(reply);
                    }
                    // Idle or nobody left to ask: the worker goes with `self`.
                    Err(_) => return,
                }
            }
        }

        fn answer(&mut self, request: Vec<u8>) -> ServiceReply {
            if request.len() as u64 > MAX_FRAME {
                return ServiceReply::failed("the request is larger than 256 MiB");
            }
            self.restart_if_changed();
            if let Some(at) = self.retry_at {
                let now = Instant::now();
                if now < at {
                    return ServiceReply::failed(format!(
                        "the service failed and is not restarted for {} s: {}",
                        (at - now).as_secs() + 1,
                        self.last_failure,
                    ));
                }
            }
            if self.worker.is_none() {
                match self.spawn() {
                    Ok(worker) => self.worker = Some(worker),
                    Err(reply) => return reply,
                }
            }
            let worker = self.worker.as_mut().expect("spawned above");
            let mut frame = (request.len() as u64).to_le_bytes().to_vec();
            frame.extend_from_slice(&request);
            if worker
                .stdin
                .write_all(&frame)
                .and_then(|()| worker.stdin.flush())
                .is_err()
            {
                return self.crashed("the service stopped before reading the request");
            }
            match worker.answers.recv_timeout(self.config.timeout) {
                Ok(Ok(reply)) => {
                    self.backoff = Duration::ZERO;
                    self.retry_at = None;
                    reply
                }
                Ok(Err(message)) => self.crashed(&message),
                Err(RecvTimeoutError::Timeout) => self.crashed(&format!(
                    "the service did not answer within {} s",
                    self.config.timeout.as_secs()
                )),
                Err(RecvTimeoutError::Disconnected) => self.crashed("the service stopped"),
            }
        }

        /// Stops the worker after a failure and backs off before the next start.
        fn crashed(&mut self, reason: &str) -> ServiceReply {
            let mut message = reason.to_string();
            if let Some(mut worker) = self.worker.take() {
                let _ = worker.child.kill();
                let status = worker.child.wait().ok();
                if status
                    .and_then(|s| s.code())
                    .is_some_and(|c| c == i32::from(crate::memory_limit::EXIT_CODE))
                {
                    message = format!(
                        "the service used more than {} MiB",
                        self.config.memory_bytes >> 20
                    );
                }
                let tail = worker.stderr.lock().map(|b| b.clone()).unwrap_or_default();
                let tail = String::from_utf8_lossy(&tail);
                let tail = tail.trim();
                if !tail.is_empty() {
                    message.push_str(":\n");
                    message.push_str(tail);
                }
            }
            self.backoff = if self.backoff.is_zero() {
                self.config.backoff_min
            } else {
                (self.backoff * 2).min(self.config.backoff_max)
            };
            self.retry_at = Some(Instant::now() + self.backoff);
            self.last_failure.clone_from(&message);
            ServiceReply::failed(message)
        }

        /// Restarts the worker (and forgets a crash) when the service's sources or `jaic`
        /// changed. Checked at most once a second.
        fn restart_if_changed(&mut self) {
            if self
                .checked
                .is_some_and(|at| at.elapsed() < Duration::from_secs(1))
            {
                return;
            }
            self.checked = Some(Instant::now());
            let now = fingerprint(&self.path, self.jaic().as_deref());
            if self.fingerprint.is_some_and(|before| before != now) {
                self.worker = None;
                self.backoff = Duration::ZERO;
                self.retry_at = None;
            }
            self.fingerprint = Some(now);
        }

        fn jaic(&self) -> Option<PathBuf> {
            self.config.jaic.clone().or_else(find_jaic)
        }

        fn spawn(&self) -> Result<Worker, ServiceReply> {
            let Some(jaic) = self.jaic() else {
                return Err(ServiceReply::unavailable(
                    "no `jaic` was found to run the service with (put it on PATH or next to this program)",
                ));
            };
            if !self.path.is_file() {
                return Err(ServiceReply::failed(format!(
                    "service `{}` does not exist",
                    self.path.display()
                )));
            }
            let mut command = Command::new(&jaic);
            command
                .arg(&self.path)
                .arg("-interp")
                .env(
                    crate::memory_limit::ENV_VAR,
                    self.config.memory_bytes.to_string(),
                )
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            if let Some(dir) = self.path.parent() {
                command.current_dir(dir);
            }
            let mut child = command.spawn().map_err(|error| {
                ServiceReply::unavailable(format!("could not start `{}`: {error}", jaic.display()))
            })?;
            let stdin = child.stdin.take().expect("piped");
            let mut stdout = child.stdout.take().expect("piped");
            let mut stderr = child.stderr.take().expect("piped");
            let tail = Arc::new(Mutex::new(Vec::new()));
            let kept = tail.clone();
            std::thread::spawn(move || {
                let mut chunk = [0u8; 4096];
                while let Ok(n) = stderr.read(&mut chunk) {
                    if n == 0 {
                        return;
                    }
                    let mut tail = kept
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    tail.extend_from_slice(&chunk[..n]);
                    if tail.len() > STDERR_TAIL {
                        let cut = tail.len() - STDERR_TAIL;
                        tail.drain(..cut);
                    }
                }
            });
            let (answers, received) = mpsc::channel();
            std::thread::spawn(move || {
                loop {
                    let answer = read_answer(&mut stdout);
                    let stop = answer.is_err();
                    if answers.send(answer).is_err() || stop {
                        return;
                    }
                }
            });
            Ok(Worker {
                child,
                stdin,
                answers: received,
                stderr: tail,
            })
        }
    }

    fn read_answer(from: &mut impl Read) -> Result<ServiceReply, String> {
        let mut length = [0u8; 8];
        from.read_exact(&mut length)
            .map_err(|_| "the service stopped".to_string())?;
        let length = u64::from_le_bytes(length);
        if length == 0 || length > MAX_FRAME {
            return Err(format!(
                "the service sent a malformed answer ({length} bytes)"
            ));
        }
        let mut body = vec![0u8; length as usize];
        from.read_exact(&mut body)
            .map_err(|_| "the service stopped mid-answer".to_string())?;
        let status = ServiceStatus::from_byte(body[0])
            .filter(|s| *s != ServiceStatus::Pending)
            .ok_or_else(|| format!("the service sent an unknown status {}", body[0]))?;
        body.remove(0);
        Ok(ServiceReply {
            status,
            body,
        })
    }

    /// Modification times and sizes of the `.jai` files under the service's folder (two levels
    /// down, skipping hidden folders) and of `jaic`: when it changes, the service is restarted
    /// and remembered answers are dropped.
    pub fn fingerprint(service: &Path, jaic: Option<&Path>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let stamp = |path: &Path, hasher: &mut std::collections::hash_map::DefaultHasher| {
            if let Ok(meta) = std::fs::metadata(path) {
                path.hash(hasher);
                meta.len().hash(hasher);
                meta.modified()
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .hash(hasher);
            }
        };
        if let Some(jaic) = jaic {
            stamp(jaic, &mut hasher);
        }
        let mut dirs: Vec<(PathBuf, u32)> = service
            .parent()
            .map(|d| vec![(d.to_path_buf(), 0)])
            .unwrap_or_default();
        while let Some((dir, depth)) = dirs.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            entries.sort();
            for path in entries {
                let hidden = path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'));
                if path.is_dir() {
                    if depth < 2 && !hidden {
                        dirs.push((path, depth + 1));
                    }
                } else if path.extension().is_some_and(|e| e == "jai") {
                    stamp(&path, &mut hasher);
                }
            }
        }
        hasher.finish()
    }
}
