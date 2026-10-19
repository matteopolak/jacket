//! The compiler's one thread pool, for CPU-bound fork-join work.
//!
//! Scanning function bodies, generating code units and linting files in parallel all borrow
//! threads from here instead of starting their own, so the machine is never asked for more than
//! one pool's worth at once. The size is [`configure`]d once (the `-threads` flag, or a tool's own
//! job count) and defaults to the core count.
//!
//! Not on the pool, on purpose: the parse-ahead workers (`parse_ahead.rs`), which wait on a
//! condition variable for the crawl to find files and would hold pool threads idle; and the
//! interpreter's threads, which are Jai program threads (`interp/threads.rs`).
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The size [`configure`] set, 0 until then.
static CONFIGURED: AtomicUsize = AtomicUsize::new(0);

static POOL: OnceLock<ThreadPool> = OnceLock::new();

/// Checking and code generation recurse on the syntax tree and the IR, so pool threads get the
/// compiler's large stack (address space only; pages are touched as needed).
const WORKER_STACK: usize = 1 << 30;

/// Set how many threads the pool has. Ignored once the pool exists (the first [`get`]).
pub fn configure(threads: usize) {
    CONFIGURED.store(threads.max(1), Ordering::Relaxed);
}

/// The pool's size: what was configured, else `-threads`, else the number of cores.
pub fn threads() -> usize {
    let configured = CONFIGURED.load(Ordering::Relaxed);
    if configured != 0 {
        return configured;
    }
    crate::knobs::get().threads.map_or_else(cores, |n| n.max(1))
}

/// The number of cores, 1 where the platform cannot say (and on WebAssembly).
pub fn cores() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
}

/// The pool, started on first use.
///
/// # Panics
/// When the operating system refuses to start any thread.
pub fn get() -> &'static ThreadPool {
    POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .num_threads(threads())
            .stack_size(WORKER_STACK)
            .thread_name(|i| format!("jaic-pool-{i}"))
            .build()
            .expect("start the compiler's thread pool")
    })
}

/// Run `task(0)` to `task(count - 1)` on the pool while `local` runs on the calling thread
/// (work that must stay there, such as using a value that cannot cross threads); returns the
/// tasks' results in order once all are done and `local` has returned.
///
/// A task that panics panics here too, after the others finish.
pub fn units<R: Send>(
    count: usize,
    task: impl Fn(usize) -> R + Sync,
    local: impl FnOnce(),
) -> Vec<R> {
    let slots: Vec<std::sync::Mutex<Option<R>>> =
        (0..count).map(|_| std::sync::Mutex::new(None)).collect();
    get().in_place_scope(|scope| {
        for (index, slot) in slots.iter().enumerate() {
            let task = &task;
            scope.spawn(move |_| {
                let result = task(index);
                *slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(result);
            });
        }
        local();
    });
    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .expect("the scope ran every task")
        })
        .collect()
}
