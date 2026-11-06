//! Measure output construction, cap walks and the standard LSP writer on captured results.
//!
//! `cargo run --release -p jai-language-server --example wire_bench -- PAYLOAD.json [REPEATS]`
//! Input is a JSON array of `{ "method": "textDocument/…", "result": ... }`.
//! Capture and validate results independently. The file and payload are never printed.
use lsp_server::{Message, RequestId, Response};
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;
use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const MAX_ITEMS: usize = 64;
const MAX_REPEATS: usize = 100;
const MAX_INPUT: u64 = 64 * 1024 * 1024;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static REQUESTS: AtomicUsize = AtomicUsize::new(0);
static REQUESTED: AtomicUsize = AtomicUsize::new(0);

struct DiagnosticAllocator;

#[global_allocator]
static ALLOCATOR: DiagnosticAllocator = DiagnosticAllocator;

fn allocated(bytes: usize) {
    REQUESTS.fetch_add(1, Ordering::Relaxed);
    REQUESTED.fetch_add(bytes, Ordering::Relaxed);
    let now = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(now, Ordering::Relaxed);
}

// SAFETY: Every pointer and layout is passed unchanged to System; the atomic
// bookkeeping neither allocates nor panics and does not change allocator ownership.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for DiagnosticAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: GlobalAlloc gives this call the same valid layout required by System.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            allocated(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: GlobalAlloc gives this call the same valid layout required by System.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            allocated(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: The pointer and allocation layout are forwarded unchanged to System.
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, size: usize) -> *mut u8 {
        // SAFETY: The pointer, old layout and new size satisfy the caller's GlobalAlloc contract.
        let new = unsafe { System.realloc(ptr, old, size) };
        if !new.is_null() {
            LIVE.fetch_sub(old.size(), Ordering::Relaxed);
            allocated(size);
        }
        new
    }
}

#[derive(Clone, Copy, Default)]
struct Sample {
    nanos: u128,
    requests: usize,
    requested: usize,
    peak_extra: usize,
}

fn measured<T>(f: impl FnOnce() -> T) -> (T, Sample) {
    let live = LIVE.load(Ordering::Relaxed);
    PEAK.store(live, Ordering::Relaxed);
    let requests = REQUESTS.load(Ordering::Relaxed);
    let requested = REQUESTED.load(Ordering::Relaxed);
    let start = Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    let sample = Sample {
        nanos: elapsed.as_nanos(),
        requests: REQUESTS.load(Ordering::Relaxed) - requests,
        requested: REQUESTED.load(Ordering::Relaxed) - requested,
        peak_extra: PEAK.load(Ordering::Relaxed).saturating_sub(live),
    };
    (value, sample)
}

#[derive(Serialize)]
struct Versioned<'a> {
    jsonrpc: &'static str,
    #[serde(flatten)]
    message: &'a Message,
}

impl<'a> Versioned<'a> {
    fn new(message: &'a Message) -> Self {
        Self {
            jsonrpc: "2.0",
            message,
        }
    }
}

#[derive(Default)]
struct Counter {
    bytes: usize,
}

impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("byte overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn count(message: &Message) -> usize {
    let mut counter = Counter::default();
    serde_json::to_writer(&mut counter, &Versioned::new(message)).expect("count serialization");
    counter.bytes
}

#[derive(Deserialize)]
struct Item {
    method: String,
    result: Box<RawValue>,
}

struct Items {
    repeats: usize,
}

impl<'de> DeserializeSeed<'de> for Items {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        struct ItemsVisitor {
            repeats: usize,
        }
        impl<'de> Visitor<'de> for ItemsVisitor {
            type Value = ();
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an array of captured LSP results")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
                let mut n = 0;
                while let Some(item) = seq.next_element::<Item>()? {
                    n += 1;
                    if n > MAX_ITEMS {
                        return Err(serde::de::Error::custom("too many items"));
                    }
                    run_item(item, self.repeats).map_err(serde::de::Error::custom)?;
                }
                if n == 0 {
                    return Err(serde::de::Error::custom("empty capture"));
                }
                Ok(())
            }
        }
        d.deserialize_seq(ItemsVisitor {
            repeats: self.repeats,
        })
    }
}

fn run_item(item: Item, repeats: usize) -> Result<(), String> {
    macro_rules! typed {
        ($ty:ty) => {{
            let value: $ty = serde_json::from_str(item.result.get())
                .map_err(|_| "invalid typed result".to_owned())?;
            measure_result(&item.method, &value, &item.result, repeats)
        }};
    }
    match item.method.as_str() {
        "textDocument/semanticTokens/full" => {
            typed!(<gen_lsp_types::SemanticTokensRequest as gen_lsp_types::Request>::Result)
        }
        "textDocument/documentSymbol" => typed!(gen_lsp_types::DocumentSymbolResponse),
        "textDocument/completion" => typed!(gen_lsp_types::CompletionResponse),
        "textDocument/diagnostic" => {
            typed!(<gen_lsp_types::DocumentDiagnosticRequest as gen_lsp_types::Request>::Result)
        }
        _ => Err("unsupported method".into()),
    }
}

fn print_stats(method: &str, phase: &str, samples: &mut [Sample]) {
    samples.sort_unstable_by_key(|s| s.nanos);
    let mid = samples.len() / 2;
    let median = if samples.len().is_multiple_of(2) {
        u128::midpoint(samples[mid - 1].nanos, samples[mid].nanos)
    } else {
        samples[mid].nanos
    };
    println!(
        "method={method} phase={phase} n={} min_ns={} median_ns={} max_ns={} max_requests={} max_requested_bytes={} max_peak_live_extra_bytes={}",
        samples.len(),
        samples[0].nanos,
        median,
        samples[samples.len() - 1].nanos,
        samples.iter().map(|s| s.requests).max().unwrap(),
        samples.iter().map(|s| s.requested).max().unwrap(),
        samples.iter().map(|s| s.peak_extra).max().unwrap()
    );
}

fn measure_result<T: Serialize>(
    method: &str,
    typed: &T,
    captured: &RawValue,
    repeats: usize,
) -> Result<(), String> {
    let mut to_value = [Sample::default(); MAX_REPEATS];
    let mut one_count = to_value;
    let mut two_counts = to_value;
    let mut writer = to_value;
    let initial = serde_json::to_value(typed).map_err(|_| "typed serialization failed")?;
    let original: serde_json::Value =
        serde_json::from_str(captured.get()).map_err(|_| "invalid captured result")?;
    if initial != original {
        return Err("typed result changes captured payload".into());
    }
    drop(original);
    let message = Message::Response(Response {
        id: RequestId::from(1),
        response_result: Ok(initial),
    });
    let expected_body = count(&message);
    let output_limit = jai_language_server::Limits::default().output_bytes;
    if expected_body > output_limit {
        println!(
            "method={method} status=capped body_bytes={expected_body} limit_bytes={output_limit}"
        );
        return Ok(());
    }
    println!("method={method} status=accepted limit_bytes={output_limit}");
    let mut expected_frame = Vec::new();
    message
        .write(&mut expected_frame)
        .map_err(|_| "writer failed")?;
    let expected_body_bytes =
        serde_json::to_vec(&Versioned::new(&message)).map_err(|_| "expected body failed")?;
    let expected_header = format!("Content-Length: {expected_body}\r\n\r\n");
    if !expected_frame.starts_with(expected_header.as_bytes())
        || expected_frame.len() != expected_header.len() + expected_body
        || expected_frame[expected_header.len()..] != expected_body_bytes
    {
        return Err("count and writer frame disagree".into());
    }
    for i in 0..repeats {
        let (value, s) = measured(|| serde_json::to_value(typed));
        let value = value.map_err(|_| "typed serialization failed")?;
        if let Message::Response(response) = &message
            && response.response_result.as_ref().ok() != Some(&value)
        {
            return Err("typed value changed".into());
        }
        drop(value);
        to_value[i] = s;
        let (bytes, s) = measured(|| count(&message));
        if bytes != expected_body {
            return Err("count changed".into());
        }
        one_count[i] = s;
        let ((a, b), s) = measured(|| (count(&message), count(&message)));
        if a != expected_body || b != expected_body {
            return Err("duplicate count changed".into());
        }
        two_counts[i] = s;
        let (sink, s) = measured(|| {
            let mut sink = Counter::default();
            message.write(&mut sink).expect("writer");
            sink
        });
        if sink.bytes != expected_frame.len() {
            return Err("frame length changed".into());
        }
        let mut check = Vec::new();
        message
            .write(&mut check)
            .map_err(|_| "verification write failed")?;
        if check != expected_frame {
            return Err("frame output changed".into());
        }
        writer[i] = s;
    }
    println!(
        "method={method} body_bytes={expected_body} framed_bytes={}",
        expected_frame.len()
    );
    print_stats(method, "typed_to_value", &mut to_value[..repeats]);
    print_stats(method, "one_count", &mut one_count[..repeats]);
    print_stats(method, "two_counts", &mut two_counts[..repeats]);
    print_stats(method, "library_write", &mut writer[..repeats]);
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected capture file")?;
    let repeats = args
        .next()
        .map_or(Ok(7), |n| n.to_string_lossy().parse::<usize>())?;
    if args.next().is_some() || !(1..=MAX_REPEATS).contains(&repeats) {
        return Err("invalid repeat count".into());
    }
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > MAX_INPUT {
        return Err("capture too large".into());
    }
    let mut reader = std::io::BufReader::new(file).take(MAX_INPUT + 1);
    let mut de = serde_json::Deserializer::from_reader(&mut reader);
    Items {
        repeats,
    }
    .deserialize(&mut de)?;
    de.end()?;
    Ok(())
}

fn main() {
    if run().is_err() {
        eprintln!("wire benchmark input or serialization error");
        std::process::exit(1);
    }
}
