# Traceview

A small native viewer for Iprof recordings: frame duration bars, a flame view with actual invocation offsets, per-zone calls/self/total/average/maximum, case-sensitive search, and a second frame as a comparison baseline. Times are nanoseconds. Recursive invocations count individually in totals; self time subtracts direct children.

```sh
export T=/Volumes/LodestoneScratch/jai-targets/codex-traceview
export JAIC_STDLIB=$PWD/stdlib PATH=$T/target/debug:$PATH
jaic examples/traceview/tools/record/main.jai -exe "$T/record"
"$T/record" "$T/work.jipt"
jaic examples/traceview/main.jai -exe "$T/traceview"
"$T/traceview" "$T/work.jipt"
"$T/traceview" "$T/work.jipt" --smoke --output "$T/frame.png"
(cd examples/traceview && jacket test -j 3)
sh examples/traceview/tools/smoke.sh
```

Click a frame bar, or Previous/Next, to select a frame. Use selected as baseline pins it; Swap frames reverses the comparison. Click any table heading to sort; clicking it again reverses the order. Search filters zone names in the table. The flame view scrolls vertically and retains the full call tree even when searching. Δ calls/total is selected minus baseline; zones present only in the baseline remain visible with zero selected calls.

Flags: required trace path, `--selected N` and `--compare N` (zero-based, default 0), `--search TEXT`, `--smoke`, `--output PATH` (default `traceview.png`). `--help` describes them. Exit 2 means invalid arguments, unreadable/malformed input, empty trace, or out-of-range frame. Exit 1 means UI/render failure.

## Recording

`tools/record/main.jai` imports Iprof in manual mode and records four nested frames. For another workload import `Extensions/Iprof_Trace`, keep a `Recorder` at a stable address, call `start_capture(*recorder)` before opening zones, then use Iprof normally and call `update(true)` for each frame. At a closed-zone boundary call `stop_capture`, `save_trace(*recorder.trace, path)`, and `free_trace`. Only one profiling thread and one recorder are supported. Frames are completed by unpaused `update(true)` calls; `update(false)` does not split the timeline. An unfinished frame is discarded at stop. The first frame begins at the first observed zone/frame sample, so idle time before that sample is absent. Zones open across a frame boundary are split and continued at offset zero in the next frame.

Iprof's existing reports are unchanged. A private runtime observer forwards samples from its native monotonic clock to the optional extension. With no recorder attached it does no allocation. This avoids adding public API to Iprof, which mirrors an existing API.

## Format

JIPT v1 is portable little endian. The magic is the eight bytes `JIPT 01 00 00 00`; two u64s give name count and frame count. Each name is a u64 byte length followed by bytes. Each frame has u64 duration in ns and u64 span count. Each span stores four u64s: zone name index, parent span index plus one (zero for a root), start ns, end ns. Spans are in preorder. Empty frames and zero-duration spans are valid. The loader rejects unknown versions, truncation, trailing bytes, invalid zone/parent indices, overlap, and children outside their parents. Accepted files are limited to 64 MiB; durations and the sum of inclusive span times in a frame must fit a signed 64-bit nanosecond value. Recordings are held in memory, so keep captures bounded.

## How to change it

- `main.jai`: command-line options and lifetime ownership.
- `model/module.jai`: pure flame layout, frame normalization, filtering and stable sorting. Extend `Sort_Column` and the heading list together.
- `view.jai`: UI declaration and clipped custom paint callbacks on `Extensions/UI`. The same build procedure runs headless and in the window.
- `smoke.jai`: headless control interactions, render and summary; `tools/smoke.sh` checks real recorder/viewer output and exit codes.
- `stdlib/Extensions/Iprof_Trace`: recording, owned trace model, binary codec and validation. Version the magic before changing the wire format. Runtime hook calls live in Iprof's private implementation, not its public interface.

Dependencies: `Extensions/Args`, `Extensions/UI` (WebGPU, FreeType and platform windows), `Extensions/Iprof_Trace`, Iprof for the recorder, and Basic/String/File/System. The data/model tests need no GPU; screenshots need an offscreen WebGPU adapter. The smoke reports a clean PNG skip when no adapter exists; set `REQUIRE_GPU=1` to require rendering. No assets or third-party recordings are committed.
