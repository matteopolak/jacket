# Friction log

## Fixed in the tools

- Iprof kept aggregate frame history, but no invocation timestamps a viewer could load. Fixed by "feat(iprof-trace): record and load nested Iprof timelines": an optional observer in Iprof's private runtime forwards its existing clock samples to `Extensions/Iprof_Trace`. The extension owns the public recorder and portable file codec; Iprof gains no public declarations and its reports stay unchanged. Tests cover recursive zones split across frames, round trips and invalid files.

## Open

- Offscreen UI rendering still requires a WebGPU adapter. This Mac's sandbox reports no suitable Metal adapter. The smoke always checks recording, loading, UI controls, drawing commands and summaries; it skips PNG readback explicitly when no adapter exists. `REQUIRE_GPU=1` makes that a failure. A visible native window and rendered pixels remain unverified here.
- Iprof is process-global and single-threaded. Capture shares that constraint; the recorder must remain at a stable address and attach before opening any zones. Captures grow in memory until stopped; keep them short. Only unpaused `update(true)` completes a frame, and stopping drops unfinished work. The first frame starts at the first observed zone/frame sample, so idle time before that sample is absent.
- The timeline draws one bar per frame without horizontal zoom or virtualization. The flame view scrolls vertically; very short calls can become subpixel bars. This is intended for short captures, not multi-hour traces.
