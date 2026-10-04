//! Scalar-only ABI for the jaic-backed playground run. Mirrors the other exports: the host
//! pushes bytes per channel and reads the JSON result back byte by byte, so no raw pointers
//! cross the boundary.
//!
//! Channels for `jai_play_push`: 0 = file path, 1 = file contents, 2 = main file path.
use crate::play;
use std::cell::RefCell;
use std::collections::BTreeMap;

const INPUT_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Default)]
struct PlayState {
    files: BTreeMap<String, Vec<u8>>,
    path: Vec<u8>,
    content: Vec<u8>,
    main: Vec<u8>,
    total: usize,
    output: Vec<u8>,
    error: Vec<u8>,
}

thread_local! {
    static STATE: RefCell<PlayState> = RefCell::new(PlayState::default());
}

fn with<T>(f: impl FnOnce(&mut PlayState) -> T) -> T {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

impl PlayState {
    fn fail(&mut self, message: &str) -> u32 {
        self.error = message.as_bytes().to_vec();
        1
    }
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_reset() -> u32 {
    with(|s| {
        *s = PlayState::default();
        0
    })
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_push(channel: u32, byte: u32) -> u32 {
    with(|s| {
        let Ok(byte) = u8::try_from(byte) else {
            return s.fail("input is not a byte");
        };
        s.total += 1;
        if s.total > INPUT_LIMIT {
            return s.fail("input byte limit exceeded");
        }
        match channel {
            0 => s.path.push(byte),
            1 => s.content.push(byte),
            2 => s.main.push(byte),
            _ => return s.fail("unknown input channel"),
        }
        0
    })
}

/// Commit the pending path/content pair as one workspace file.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_finish_file() -> u32 {
    with(|s| {
        let Ok(path) = String::from_utf8(std::mem::take(&mut s.path)) else {
            return s.fail("file paths must be UTF-8");
        };
        let content = std::mem::take(&mut s.content);
        s.files.insert(path, content);
        0
    })
}

/// Compile and run. Zero means the JSON result is ready (compile errors included in it).
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_run() -> u32 {
    with(|s| {
        let Ok(main) = String::from_utf8(std::mem::take(&mut s.main)) else {
            return s.fail("main path must be UTF-8");
        };
        let files = std::mem::take(&mut s.files);
        s.output = play::run(&files, &main).to_json().into_bytes();
        s.files = files;
        0
    })
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_output_len() -> u32 {
    with(|s| u32::try_from(s.output.len()).unwrap_or(u32::MAX))
}

/// Out-of-range indices return 256, outside the byte range.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_output_byte(index: u32) -> u32 {
    with(|s| s.output.get(index as usize).copied().map_or(256, u32::from))
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_error_len() -> u32 {
    with(|s| u32::try_from(s.error.len()).unwrap_or(u32::MAX))
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_error_byte(index: u32) -> u32 {
    with(|s| s.error.get(index as usize).copied().map_or(256, u32::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(channel: u32, text: &str) {
        for b in text.bytes() {
            assert_eq!(jai_play_push(channel, u32::from(b)), 0);
        }
    }

    #[test]
    fn scalar_abi_round_trip() {
        jai_play_reset();
        push(0, "main.jai");
        push(
            1,
            "#import \"Basic\";\nmain :: () { print(\"Hello, %!\\n\", 42); }\n",
        );
        assert_eq!(jai_play_finish_file(), 0);
        push(2, "main.jai");
        assert_eq!(jai_play_run(), 0);
        let bytes: Vec<u8> = (0..jai_play_output_len())
            .map(|i| jai_play_output_byte(i) as u8)
            .collect();
        let json = String::from_utf8(bytes).unwrap();
        assert!(json.contains("\"stdout\":\"Hello, 42!\\n\""), "{json}");
        assert_eq!(jai_play_output_byte(u32::MAX), 256);
    }
}
