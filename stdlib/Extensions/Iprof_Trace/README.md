# Iprof timeline extension

Optional capture and JIPT v1 persistence for Iprof. Import it alongside Iprof; `start_capture` attaches to the profiler's private clock observer, `update(true)` completes frames, and `stop_capture` detaches at a closed-zone boundary. `save_trace` writes completed frames; `load_trace` and `decode_trace` return owned storage released by `free_trace`.

The recorder uses the exact samples used in aggregate accounting. It copies names, records preorder spans, and splits open invocations at frame boundaries. Reports are unchanged. Capture starts at the first observed sample, excludes paused/false updates as frame boundaries, and discards the last incomplete frame. One Iprof instance, profiling thread and recorder are supported; keep its address stable and capture lengths bounded.

Change `module.jai` for capture ownership and frame splitting, `format.jai` for serialization/validation, and the internal `observer` bridge for sample transport. The private Iprof `runtime-zones.jai` and `frame-history.jai` call the observer after sampling their native clock. New public declarations remain under Extensions. Changing the format requires a new magic version and round-trip/malformed input tests. See `examples/traceview/README.md` for the complete wire format and recorder invocation.

There are no flags or environment variables. Files are limited to 64 MiB; frame durations and summed inclusive spans must fit signed 64-bit ns. Capture storage itself is unbounded until stopped. Dependencies: Basic, File, String; a client must import Iprof to produce samples. The observer has no native dependencies of its own. `jacket test -j 3 Extensions/Iprof_Trace` from `stdlib/` checks round trips, recursion across frames, truncation and invalid trees.
