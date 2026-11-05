//! Bounded opt-in native trace. Records contain no protocol identifiers or user content.
use serde::Serialize;
use std::cell::Cell;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

/// Fixed method families. Unknown or client-specific methods use `Other`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Request,
    Notification,
    Response,
    Rejected,
    Ignored,
    Lifecycle,
    Navigation,
    Hover,
    References,
    LinkedEditing,
    Rename,
    Diagnostics,
    Completion,
    Change,
    Other,
}

/// Only these transitions can be recorded; no client-provided strings are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Decoded,
    SendStart,
    SendReturned,
    OwnerDequeued,
    DispatchStart,
    DispatchEnd,
    OutputEnqueueStart,
    OutputEnqueued,
    WriteStart,
    WriteDone,
    WriteFailed,
    JobStart,
    JobEnd,
}

/// In-process correlation only; numbers never contain JSON-RPC IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "ordinal", rename_all = "snake_case")]
pub enum Origin {
    Frame(u64),
    Job(u64),
    None,
}

/// A captured native frame's identity and monotonic decode timestamp.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub ordinal: u64,
    pub category: Category,
    pub bytes: usize,
    pub decoded_ns: u64,
}

/// Final outcome at the actual compile transaction boundary, not inferred from interruption.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Accepted,
    Abandoned,
    Unknown,
}

#[derive(Serialize)]
struct Record {
    elapsed_ns: u64,
    origin: Origin,
    parent: Option<Origin>,
    output_ordinal: Option<u64>,
    stage: Stage,
    category: Category,
    bytes: usize,
    outcome: Option<Outcome>,
    wall_ns: Option<u64>,
    thread_cpu_ns: Option<u64>,
}

impl Record {
    fn new(origin: Origin, stage: Stage, category: Category, bytes: usize) -> Self {
        Self {
            elapsed_ns: 0,
            origin,
            parent: None,
            output_ordinal: None,
            stage,
            category,
            bytes,
            outcome: None,
            wall_ns: None,
            thread_cpu_ns: None,
        }
    }
}

#[derive(Serialize)]
struct Export<'a> {
    recorded: usize,
    buffer_bytes: usize,
    dropped: u64,
    late: u64,
    incomplete: bool,
    records: &'a [Record],
}

/// Loss accounting observed at export; a nonzero count makes the trace incomplete.
#[derive(Clone, Copy, Debug)]
pub struct Summary {
    pub recorded: usize,
    /// Allocated record-vector capacity only; excludes recorder overhead and RSS.
    pub buffer_bytes: usize,
    pub dropped: u64,
    pub late: u64,
    pub incomplete: bool,
}

struct Inner {
    start: Instant,
    next: AtomicU64,
    closed: AtomicBool,
    dropped: AtomicU64,
    late: AtomicU64,
    records: Mutex<Vec<Record>>,
    capacity: usize,
    buffer_bytes: usize,
    file: Mutex<Option<File>>,
}

/// A cloneable recorder. Disabled handles allocate nothing and read no clocks.
#[derive(Clone, Default)]
pub struct Trace(Option<Arc<Inner>>);

static GLOBAL: OnceLock<Trace> = OnceLock::new();
thread_local! { static CURRENT: Cell<Origin> = const { Cell::new(Origin::None) }; }

/// The installed recorder, if tracing was enabled by the native host.
pub fn global() -> Option<&'static Trace> {
    GLOBAL.get().filter(|trace| trace.enabled())
}

/// Install the native recorder exactly once; unsupported/browser hosts do not call this.
pub fn install(trace: Trace) -> Result<(), Trace> {
    GLOBAL.set(trace)
}

/// Restores the previous dispatch origin even during unwinding.
///
/// ```compile_fail
/// fn requires_send<T: Send>() {}
/// requires_send::<jai_language_server::trace::OriginGuard>();
/// ```
pub struct OriginGuard(Origin, std::marker::PhantomData<std::rc::Rc<()>>);

impl Drop for OriginGuard {
    fn drop(&mut self) {
        CURRENT.with(|current| current.set(self.0));
    }
}

/// Set the origin on the calling thread for the duration of one actual dispatch.
pub fn scoped_origin(origin: Origin) -> OriginGuard {
    OriginGuard(
        CURRENT.with(|current| current.replace(origin)),
        std::marker::PhantomData,
    )
}

/// Scope one actual native dispatch, recording both ends without exposing protocol IDs.
///
/// For example, `let _guard = dispatch(frame);` keeps the origin through output emission.
pub fn dispatch(frame: Option<Frame>) -> Option<DispatchGuard> {
    dispatch_on(global()?, frame?)
}

pub(crate) fn dispatch_on(trace: &Trace, frame: Frame) -> Option<DispatchGuard> {
    if !trace.enabled() {
        return None;
    }
    let origin = Origin::Frame(frame.ordinal);
    let origin_guard = scoped_origin(origin);
    trace.event(origin, Stage::DispatchStart, frame.category, frame.bytes);
    Some(DispatchGuard {
        trace: trace.clone(),
        frame,
        _origin: origin_guard,
    })
}

/// Restores the previous origin after recording the end of this same-thread dispatch.
pub struct DispatchGuard {
    trace: Trace,
    frame: Frame,
    _origin: OriginGuard,
}

impl Drop for DispatchGuard {
    fn drop(&mut self) {
        self.trace.event(
            Origin::Frame(self.frame.ordinal),
            Stage::DispatchEnd,
            self.frame.category,
            self.frame.bytes,
        );
    }
}

/// Current calling-thread origin; unsolicited output is `None`.
pub fn current_origin() -> Origin {
    CURRENT.with(Cell::get)
}

fn nanos(duration: std::time::Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

impl Trace {
    /// Construct a disabled handle without allocating.
    pub fn disabled() -> Self {
        Self(None)
    }

    /// Check before categorizing messages or reading a clock.
    pub fn enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Create a fresh regular file and a fixed-size recorder; existing destinations are rejected.
    /// Example: `Trace::open_new(path, 8192)`.
    pub fn open_new(path: &Path, capacity: usize) -> io::Result<Self> {
        if capacity == 0 || capacity > 65536 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid trace capacity",
            ));
        }
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trace destination is not a regular file",
            ));
        }
        let records = Vec::<Record>::with_capacity(capacity);
        let buffer_bytes = records.capacity() * std::mem::size_of::<Record>();
        Ok(Self(Some(Arc::new(Inner {
            start: Instant::now(),
            next: AtomicU64::new(1),
            closed: AtomicBool::new(false),
            dropped: AtomicU64::new(0),
            late: AtomicU64::new(0),
            records: Mutex::new(records),
            capacity,
            buffer_bytes,
            file: Mutex::new(Some(file)),
        }))))
    }

    /// Assign an internal ordinal at completed frame decode; disabled does no clock work.
    pub fn frame(&self, category: Category, bytes: usize) -> Option<Frame> {
        let inner = self.0.as_ref()?;
        let frame = Frame {
            ordinal: inner.next.fetch_add(1, Ordering::Relaxed),
            category,
            bytes,
            decoded_ns: nanos(inner.start.elapsed()),
        };
        let record = Record::new(
            Origin::Frame(frame.ordinal),
            Stage::Decoded,
            category,
            bytes,
        );
        self.record(record, Some(frame.decoded_ns));
        Some(frame)
    }

    /// Allocate an internal output ordinal, including for each member of a response group.
    pub fn output_ordinal(&self) -> Option<u64> {
        self.0
            .as_ref()
            .map(|i| i.next.fetch_add(1, Ordering::Relaxed))
    }

    /// Record a transition with no output ordinal.
    pub fn event(&self, origin: Origin, stage: Stage, category: Category, bytes: usize) {
        self.record(Record::new(origin, stage, category, bytes), None);
    }

    /// Record an output transition with its unique output ordinal and dispatch/job origin.
    pub fn output_event(
        &self,
        origin: Origin,
        output: u64,
        stage: Stage,
        category: Category,
        bytes: usize,
    ) {
        let mut record = Record::new(origin, stage, category, bytes);
        record.output_ordinal = Some(output);
        self.record(record, None);
    }

    fn record(&self, mut record: Record, elapsed_ns: Option<u64>) {
        let Some(inner) = &self.0 else {
            return;
        };
        if inner.closed.load(Ordering::Acquire) {
            inner.late.fetch_add(1, Ordering::Relaxed);
            return;
        }
        record.elapsed_ns = elapsed_ns.unwrap_or_else(|| nanos(inner.start.elapsed()));
        let Ok(mut records) = inner.records.try_lock() else {
            inner.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        if inner.closed.load(Ordering::Acquire) {
            inner.late.fetch_add(1, Ordering::Relaxed);
        } else if records.len() == inner.capacity {
            inner.dropped.fetch_add(1, Ordering::Relaxed);
        } else {
            records.push(record);
        }
    }

    /// Begin a compile on the calling thread. Finish it on the same thread at its acceptance boundary.
    pub fn job_start(&self) -> Option<JobTimer> {
        let inner = self.0.as_ref()?;
        let ordinal = inner.next.fetch_add(1, Ordering::Relaxed);
        let parent = current_origin();
        let mut record = Record::new(Origin::Job(ordinal), Stage::JobStart, Category::Other, 0);
        record.parent = Some(parent);
        self.record(record, None);
        Some(JobTimer {
            trace: self.clone(),
            ordinal,
            parent,
            start: Instant::now(),
            cpu: cpu_start(),
            finished: false,
            _same_thread: std::marker::PhantomData,
        })
    }

    /// Freeze recording and write one bounded JSON document after transport completion.
    /// Slow local files may block; the caller's supervisor must bound shutdown.
    pub fn finish_output(&self, producers_quiesced: bool) -> io::Result<Option<Summary>> {
        let Some(inner) = &self.0 else {
            return Ok(None);
        };
        if inner.closed.swap(true, Ordering::AcqRel) {
            return Err(io::Error::other("trace already closed"));
        }
        let mut file = inner
            .file
            .try_lock()
            .map_err(|_| io::Error::other("trace file busy"))?;
        let file = file
            .as_mut()
            .ok_or_else(|| io::Error::other("trace file unavailable"))?;
        let dropped = inner.dropped.load(Ordering::Acquire);
        let late = inner.late.load(Ordering::Acquire);
        let records = inner
            .records
            .try_lock()
            .ok()
            .map(|mut records| std::mem::take(&mut *records));
        let incomplete = !producers_quiesced || records.is_none() || dropped != 0 || late != 0;
        let records = records.unwrap_or_default();
        let summary = Summary {
            recorded: records.len(),
            buffer_bytes: inner.buffer_bytes,
            dropped,
            late,
            incomplete,
        };
        serde_json::to_writer(
            &mut *file,
            &Export {
                recorded: summary.recorded,
                buffer_bytes: summary.buffer_bytes,
                dropped,
                late,
                incomplete,
                records: &records,
            },
        )
        .map_err(io::Error::other)?;
        use std::io::Write;
        file.write_all(b"\n")?;
        Ok(Some(summary))
    }

    /// Events attempted after freeze, including attempts after the exported snapshot.
    pub fn late_events(&self) -> u64 {
        self.0
            .as_ref()
            .map_or(0, |i| i.late.load(Ordering::Relaxed))
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
type CpuStart = Option<cpu_time::ThreadTime>;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn cpu_start() -> CpuStart {
    cpu_time::ThreadTime::try_now().ok()
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn cpu_elapsed(start: &CpuStart) -> Option<u64> {
    let before = start.as_ref()?.as_duration();
    let after = cpu_time::ThreadTime::try_now().ok()?.as_duration();
    after.checked_sub(before).map(nanos)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
type CpuStart = ();

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn cpu_start() -> CpuStart {
    ()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn cpu_elapsed(_: &CpuStart) -> Option<u64> {
    None
}

/// A same-thread compile span. Drop without an explicit outcome records `Unknown`.
///
/// ```compile_fail
/// fn requires_send<T: Send>() {}
/// requires_send::<jai_language_server::trace::JobTimer>();
/// ```
pub struct JobTimer {
    trace: Trace,
    pub ordinal: u64,
    pub parent: Origin,
    start: Instant,
    cpu: CpuStart,
    finished: bool,
    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl JobTimer {
    /// Record the actual acceptance outcome and calling-thread CPU, if available.
    pub fn finish(mut self, outcome: Outcome) {
        self.finish_inner(outcome);
    }

    fn finish_inner(&mut self, outcome: Outcome) {
        if self.finished {
            return;
        }
        self.finished = true;
        let mut record = Record::new(Origin::Job(self.ordinal), Stage::JobEnd, Category::Other, 0);
        record.parent = Some(self.parent);
        record.outcome = Some(outcome);
        record.wall_ns = Some(nanos(self.start.elapsed()));
        record.thread_cpu_ns = cpu_elapsed(&self.cpu);
        self.trace.record(record, None);
    }
}

impl Drop for JobTimer {
    fn drop(&mut self) {
        self.finish_inner(Outcome::Unknown);
    }
}

#[cfg(test)]
fn test_sequence() -> u64 {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_does_not_allocate_ordinals_or_emit() {
        let trace = Trace::disabled();
        assert!(trace.frame(Category::Hover, 10).is_none());
        assert!(trace.job_start().is_none());
        assert!(trace.finish_output(true).unwrap().is_none());
    }

    #[test]
    fn bounded_trace_reports_overflow_and_closed_events() {
        let (trace, path) = temporary_trace(1);
        let frame = trace.frame(Category::Hover, 10).unwrap();
        trace.event(
            Origin::Frame(frame.ordinal),
            Stage::DispatchStart,
            Category::Hover,
            0,
        );
        let summary = trace.finish_output(true).unwrap().unwrap();
        assert_eq!(summary.recorded, 1);
        assert_eq!(summary.dropped, 1);
        trace.event(
            Origin::Frame(frame.ordinal),
            Stage::DispatchEnd,
            Category::Hover,
            0,
        );
        assert_eq!(trace.late_events(), 1);
        let output = std::fs::read_to_string(&path).unwrap();
        assert!(output.contains("\"dropped\":1"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn file_is_fresh_and_trace_has_only_fixed_fields() {
        let (trace, path) = temporary_trace(8);
        assert!(Trace::open_new(&path, 8).is_err());
        let frame = trace.frame(Category::Other, 12).unwrap();
        trace.event(
            Origin::Frame(frame.ordinal),
            Stage::DispatchStart,
            Category::Other,
            12,
        );
        trace.finish_output(true).unwrap();
        let output = std::fs::read_to_string(&path).unwrap();
        assert!(output.contains("\"category\":\"other\""));
        assert!(!output.contains(path.to_str().unwrap()));
        assert!(!output.contains("payload"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unquiesced_producers_and_unknown_job_are_explicit() {
        let (trace, path) = temporary_trace(8);
        {
            let _job = trace.job_start().unwrap();
        }
        let summary = trace.finish_output(false).unwrap().unwrap();
        assert!(summary.incomplete);
        let output = std::fs::read_to_string(&path).unwrap();
        assert!(output.contains("\"outcome\":\"unknown\""));
        assert!(output.contains("\"incomplete\":true"));
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn existing_symlink_is_never_followed() {
        let (trace, target) = temporary_trace(8);
        let link = target.with_extension("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(Trace::open_new(&link, 8).is_err());
        trace.finish_output(true).unwrap();
        std::fs::remove_file(link).unwrap();
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn dispatch_preserves_origin_bytes_and_restores_nested_scope() {
        let (trace, path) = temporary_trace(8);
        let frame = trace.frame(Category::Hover, 37).unwrap();
        assert_eq!(frame.bytes, 37);
        {
            let _outer = scoped_origin(Origin::Job(9));
            {
                let _dispatch = dispatch_on(&trace, frame).unwrap();
                assert_eq!(current_origin(), Origin::Frame(frame.ordinal));
            }
            assert_eq!(current_origin(), Origin::Job(9));
        }
        assert_eq!(current_origin(), Origin::None);
        let summary = trace.finish_output(true).unwrap().unwrap();
        assert_eq!(summary.recorded, 3);
        assert!(summary.buffer_bytes >= 8 * std::mem::size_of::<Record>());
        let output: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(output["buffer_bytes"], summary.buffer_bytes);
        assert_eq!(output["records"][1]["stage"], "dispatch_start");
        assert_eq!(output["records"][2]["stage"], "dispatch_end");
        assert_eq!(output["records"][2]["bytes"], 37);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn dispatch_unwind_restores_origin_and_records_one_end() {
        let (trace, path) = temporary_trace(8);
        let frame = trace.frame(Category::Hover, 19).unwrap();
        let _outer = scoped_origin(Origin::Job(17));
        assert!(dispatch_on(&Trace::disabled(), frame).is_none());
        assert_eq!(current_origin(), Origin::Job(17));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _dispatch = dispatch_on(&trace, frame).unwrap();
            assert_eq!(current_origin(), Origin::Frame(frame.ordinal));
            panic!("test unwind");
        }));
        assert!(result.is_err());
        assert_eq!(current_origin(), Origin::Job(17));
        trace.finish_output(true).unwrap();
        let output: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let records = output["records"].as_array().unwrap();
        assert_eq!(
            records
                .iter()
                .filter(|record| record["stage"] == "dispatch_end")
                .count(),
            1
        );
        std::fs::remove_file(path).unwrap();
    }

    fn temporary_trace(capacity: usize) -> (Trace, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "lsp-trace-test-{}-{}",
            std::process::id(),
            super::test_sequence()
        ));
        (Trace::open_new(&path, capacity).unwrap(), path)
    }
}
