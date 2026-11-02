# Iprof_Trace

Records what Iprof measures as a timeline: every zone invocation per frame, nested, in nanoseconds, plus a small binary file format (`.jipt`) to save and load it. Iprof's own reports don't change. `@experimental`. [traceview](../tools/dogfood.md) reads the files.

```jai
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);
#import "Extensions/Iprof_Trace";

recorder: Recorder;
start_capture(*recorder);           // before opening zones
// ... zones as usual; each I.update(true) closes a frame ...
stop_capture(*recorder);            // with every zone closed
save_trace(*recorder.trace, "run.jipt");
free_trace(*recorder.trace);

trace, ok := load_trace("run.jipt");
defer free_trace(*trace);
```

## How it works

- Iprof's private zone and frame code calls a tiny observer (`observer/module.jai`) with each clock sample it already takes: zone enter, zone exit, frame end. With no recorder attached that's one null check per call.
- `start_capture` attaches the recorder. It copies zone names and records spans in preorder, each with a parent index, start and duration relative to its frame. Recursive calls stay separate spans; an invocation still open at a frame boundary is split across both frames.
- Capture begins at the first sample. Paused updates and `update(false)` don't end frames. `stop_capture` drops the unfinished frame.
- `format.jai` writes and validates the file: a magic with the version, then names and frames as little-endian 64-bit fields. `decode_trace` rejects truncated input, bad trees and overflowing durations; files are capped at 64 MiB.

## How to change it

- Capture and frame splitting live in `module.jai`, serialization in `format.jai`. A format change needs a new magic version and round-trip plus malformed-input tests.
- One Iprof instance, one profiling thread and one recorder at a time. Keep the `Recorder` at a fixed address while it's attached. Capture storage grows until you stop, so keep captures bounded.
- New public declarations stay in this extension; the hook in Iprof's internals is private.

Tests: `jacket test --path stdlib/Extensions/Iprof_Trace`. Depends on `Basic`, `File` and `String`; a client imports `Iprof` to produce samples.
