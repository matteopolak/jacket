//! Stable scalar ABI: status zero means success; diagnostics remain owned by Rust.
use crate::bridge::{Bridge, state};

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_reset() -> u32 {
    match state().lock() { Ok(mut bridge) => { *bridge = Bridge::default(); 0 }, Err(_) => 1 }
}
/// Channels: 0 = source bytes, 1 = current argument, 2 = relative source filename.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_push(channel: u32, byte: u32) -> u32 {
    match state().lock() { Ok(mut bridge) => bridge.push(channel, byte), Err(_) => 1 }
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_finish_source() -> u32 {
    match state().lock() { Ok(mut bridge) => bridge.finish_source(), Err(_) => 1 }
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_finish_argument() -> u32 {
    match state().lock() { Ok(mut bridge) => bridge.finish_argument(), Err(_) => 1 }
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_run(fuel: u32) -> u32 {
    match state().lock() { Ok(mut bridge) => bridge.run(fuel), Err(_) => 1 }
}
/// Consult jai_script_has_result first; zero itself is a valid exit code.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_has_result() -> u32 {
    state().lock().ok().map_or(0, |bridge| u32::from(bridge.result.is_some()))
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_exit_code() -> i64 {
    state().lock().ok().and_then(|bridge| bridge.result).map_or(0, |result| result.exit_code)
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_steps() -> u64 {
    state().lock().ok().and_then(|bridge| bridge.result).map_or(0, |result| result.statistics.steps)
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_diagnostic_len() -> u32 {
    state().lock().ok().map_or(0, |bridge| u32::try_from(bridge.diagnostic.len()).unwrap_or(u32::MAX))
}
/// Out-of-range indices return 256, outside the byte range.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_script_diagnostic_byte(index: u32) -> u32 {
    state().lock().ok().and_then(|bridge| bridge.diagnostic.get(index as usize).copied()).map_or(256, u32::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn append(channel: u32, bytes: &[u8]) { for byte in bytes { assert_eq!(jai_script_push(channel, u32::from(*byte)), 0); } }
    fn diagnostic() -> String { String::from_utf8((0..jai_script_diagnostic_len()).map(|i| jai_script_diagnostic_byte(i) as u8).collect()).unwrap() }
    // One fixture owns the process-global ABI state throughout all cases.
    #[test]
    fn native_bridge_uses_real_source_interpreter_and_reports_errors() {
        assert_eq!(jai_script_reset(), 0);
        append(0, b"main :: () -> int { if #compile_time return 900; return 42; }");
        assert_eq!(jai_script_finish_source(), 0);
        assert_eq!(jai_script_run(1_000_000), 0, "{}", diagnostic());
        assert_eq!(jai_script_has_result(), 1);
        assert_eq!(jai_script_exit_code(), 42);
        assert!(jai_script_steps() > 0);
        jai_script_reset();
        append(0, b"main :: (args: []string) -> int { if args[0] != \"two words\" return 1; return args.count + 41; }");
        jai_script_finish_source(); append(1, b"two words"); jai_script_finish_argument();
        assert_eq!(jai_script_run(1_000_000), 0, "{}", diagnostic());
        assert_eq!(jai_script_exit_code(), 42);
        jai_script_reset(); append(0, b"main :: () -> int { return missing; }"); jai_script_finish_source();
        assert_eq!(jai_script_run(1_000_000), 1);
        assert_eq!(jai_script_has_result(), 0);
        assert!(diagnostic().contains("missing"));
        assert_eq!(jai_script_diagnostic_byte(u32::MAX), 256);
        assert_eq!(jai_script_push(9, 65), 1);
        assert_eq!(jai_script_push(0, 300), 1);
    }
}
