// The browser playground. Every wgpu* procedure is a host call the sandbox forwards to
// crates/jai-wasm/js/webgpu_host.mjs, except the two that run callbacks: JavaScript cannot call
// a Jai procedure, so the host queues finished requests and these call them
// (webgpu_dispatch_record in generated_wasm.jai).
//
// Callback modes, as far as the browser allows (docs/stdlib/webgpu.md):
// - .AllowProcessEvents: in wgpuInstanceProcessEvents, or in wgpuInstanceWaitAny on its future.
// - .WaitAnyOnly: only in wgpuInstanceWaitAny on its future.
// - .AllowSpontaneous: in either, and also when webgpu_present waited for a frame. JavaScript
//   cannot interrupt the program, so "spontaneous" means at those points only.
//
// A wait suspends the whole module (JSPI). When other Jai threads could run, the waiting thread
// first lets them (jai_sched_yield_for_wait) and then waits in short slices, so they keep going
// while it waits for the GPU or the next frame.

wgpuInstanceProcessEvents :: (instance: WGPUInstance) {
    // Give the page a turn so its promises settle, then run what finished.
    jai_sched_yield_for_wait();
    jai_webgpu_wait(null, 0, 0);
    webgpu_dispatch(null, 0, .PROCESS_EVENTS);
}

wgpuInstanceWaitAny :: (instance: WGPUInstance, futureCount: u64, futures: *WGPUFutureWaitInfo, timeoutNS: u64) -> WGPUWaitStatus {
    status := webgpu_wasm_wait(futures, futureCount, timeoutNS);
    webgpu_dispatch(futures, futureCount, .WAIT_ANY);
    return status;
}

#scope_module

// Filled by the host; see `pollRecord` in webgpu_host.mjs.
Webgpu_Callback_Record :: struct {
    kind: u32; // 1-based index of the callback in webgpu.yml
    mode: u32;
    future: u64;
    callback: *void;
    userdata1: *void;
    userdata2: *void;
    args: [4] u64;
    message_data: *u8;
    message_length: u64;
}

// Which finished callbacks a dispatch runs (the `how` of jai_webgpu_poll).
Dispatch :: enum u32 {
    PROCESS_EVENTS :: 0; // modes AllowProcessEvents and AllowSpontaneous
    WAIT_ANY :: 1; // the futures waited on (any mode), and AllowSpontaneous
    SPONTANEOUS :: 2; // AllowSpontaneous only
}

// While other threads run, a wait for the page lasts at most this long before they get a turn.
WAIT_SLICE_NS :: 2_000_000;
// A timeout that never ends.
WAIT_FOREVER :: 0xFFFF_FFFF_FFFF_FFFF;

webgpu_dispatch :: (futures: *WGPUFutureWaitInfo, count: u64, how: Dispatch) {
    record: Webgpu_Callback_Record;
    while jai_webgpu_poll(*record, futures, count, xx how) webgpu_dispatch_record(*record);
}

// Wait until one of `futures` completed or `timeout_ns` passed, letting other threads run.
webgpu_wasm_wait :: (futures: *WGPUFutureWaitInfo, count: u64, timeout_ns: u64) -> WGPUWaitStatus {
    left := timeout_ns;
    while true {
        if !jai_sched_yield_for_wait() return cast(WGPUWaitStatus) jai_webgpu_wait(futures, count, left);
        slice := ifx left < WAIT_SLICE_NS then left else WAIT_SLICE_NS;
        status := cast(WGPUWaitStatus) jai_webgpu_wait(futures, count, slice);
        if status != .TimedOut || left == slice return status;
        if left != WAIT_FOREVER left -= slice;
    }
    return .TimedOut;
}

// webgpu_present: wait for the page's next animation frame, then run spontaneous callbacks.
webgpu_wasm_next_frame :: () {
    frame: WGPUFutureWaitInfo;
    frame.future.id = jai_webgpu_request_frame();
    webgpu_wasm_wait(*frame, 1, WAIT_FOREVER);
    webgpu_dispatch(null, 0, .SPONTANEOUS);
}

// Host procedures (no library: the sandbox resolves them by name).
// Whether the page provides the host procedure `name` (0 when it has no host at all).
jai_host_provides :: (name: *u8, count: u64) -> u32 #foreign;
// Whether the page has WebGPU.
jai_webgpu_available :: () -> u32 #foreign;
// Write the next finished callback that `how` allows into `record`; 0 when none is ready. Also
// marks which of `futures` completed.
jai_webgpu_poll :: (record: *Webgpu_Callback_Record, futures: *WGPUFutureWaitInfo, count: u64, how: u32) -> u32 #foreign;
// Suspend until one of `futures` completed or `timeout_ns` passed (count 0: one turn of the page).
jai_webgpu_wait :: (futures: *WGPUFutureWaitInfo, count: u64, timeout_ns: u64) -> u32 #foreign;
// A future that completes at the page's next animation frame.
jai_webgpu_request_frame :: () -> u64 #foreign;
// The canvas size in pixels.
jai_canvas_size :: (width: *u32, height: *u32) #foreign;
// The interpreter's thread scheduler (not the page): let the other threads that can run now run
// first; returns whether any other thread may still run, so the caller should wait in slices.
jai_sched_yield_for_wait :: () -> u32 #foreign;
