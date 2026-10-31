# Platform differences

`crates/jaic/src/platform.rs` names what differs between the native compiler and the browser (wasm32) build, so ordinary code doesn't carry its own `cfg(target_arch = "wasm32")`.

## How it works

- `HAS_CLOCK` is false on wasm. `Stopwatch` is a real `Instant` natively and reads zero on wasm.
- Subsystems that exist on one side only are separate modules picked where they are declared: `parse_ahead`, `service`, the interpreter's `threads` versus `threads_inline`, `progress::start_ticker`, and `fiber`'s per-CPU assembly.
- The OS and CPU reported to `#run` code come from `compiler_host` in `sema/mod.rs`.

## How to change it

- A new "the browser lacks X" difference: add a constant or small wrapper to `platform.rs` and branch on it instead of adding a `cfg`.
- A native-only subsystem: a module with a wasm stub.
- A new CPU for fibers: its `.s` file and one arm in `fiber.rs`.
