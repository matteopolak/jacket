// Internal bridge between Iprof and its optional timeline recorder.
// @experimental

// Receives enter (0), exit (1), or frame (2), using Iprof's monotonic clock.
Listener :: #type (data: *void, kind: u8, name: string, ticks: s64, seconds_per_tick: float64);

// The optional process-wide listener. Only the profiling thread may change it.
listener: Listener;
// Opaque storage owned by the listener; valid until it is detached.
listener_data: *void;

// Sends a clock sample without changing Iprof's aggregate accounting.
emit :: (kind: u8, name: string, ticks: s64, seconds_per_tick: float64) {
    if listener listener(listener_data, kind, name, ticks, seconds_per_tick);
}
