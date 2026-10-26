# Platform differences

## What it is

`crates/jaic/src/platform.rs` names what differs between the native compiler and the browser (wasm32) build, so ordinary code does not carry its own `cfg(target_arch = "wasm32")`.

## How it works

- `HAS_CLOCK` is false on wasm. `Stopwatch` is a real `Instant` natively and reads zero on wasm; `sema/perf.rs` and the interpreter's call timing use it. Reporting performance to a metaprogram (`begin_intercept`) is refused when there is no clock.
- Whole subsystems that exist on one side only are separate modules chosen where they are declared, each with a counterpart: `parse_ahead` and `service` (`mod imp` / `mod native`), the interpreter's `threads` (native baton scheduler) versus `threads_inline` (wasm and the sandbox), `progress::start_ticker`, and `fiber`'s per-CPU assembly files (`crates/jaic/src/fiber/<cpu>-<abi>.s`, picked by `cfg(all(target_arch, target_os))` in `fiber.rs`).
- Which OS and CPU the host reports to `#run` code is `compiler_host` in `sema/mod.rs`; it carries the `compile_error!` guards for unsupported hosts.

## How to change it

A new "the browser build lacks X" difference: add a constant or small wrapper to `platform.rs` and branch on it, instead of a new `cfg`. A new native-only subsystem: a module with a wasm stub, selected at its declaration. A new CPU for fibers: add its `.s` file and one arm in `fiber.rs`.

## Configuration

Selected at compile time by the target (`wasm32-unknown-unknown` for the browser build); no flags.

## Dependencies

None beyond `std`.
