//! Scalar-only ABI for the jaic-backed playground run. Mirrors the other exports: the host
//! pushes bytes per channel and reads the JSON result back byte by byte, so no raw pointers
//! cross the boundary.
//!
//! Channels for `jai_play_push`: 0 = file path, 1 = file contents, 2 = main file path,
//! 3 = program arguments (each argument ends with a NUL byte; none pushed passes no `argv`),
//! 4 = standard input (the whole of it, when fixed; otherwise the page supplies it as the program
//! reads, see `host_bridge`).
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
    args: Vec<u8>,
    stdin: Vec<u8>,
    fixed_stdin: bool,
    total: usize,
    output: Vec<u8>,
    error: Vec<u8>,
    limits: play::PlayOptions,
}

thread_local! {
    static STATE: RefCell<PlayState> = RefCell::new(PlayState::default());
    /// Last panic message. Kept apart from `STATE` because a panic can happen while that is
    /// borrowed.
    static PANIC: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Record panic messages so the host can report why the module trapped (wasm32 panics abort, so
/// the module is unusable afterwards, but its memory can still be read).
fn install_panic_hook() {
    if !cfg!(target_arch = "wasm32") {
        return; // native keeps the default hook
    }
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let text = info.to_string();
            PANIC.with(|p| {
                if let Ok(mut p) = p.try_borrow_mut() {
                    *p = text;
                }
            });
        }));
    });
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
    install_panic_hook();
    with(|s| {
        *s = PlayState {
            limits: s.limits,
            ..PlayState::default()
        };
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
            3 => s.args.push(byte),
            4 => {
                s.stdin.push(byte);
                s.fixed_stdin = true;
            }
            _ => return s.fail("unknown input channel"),
        }
        0
    })
}

/// Whether this build takes program arguments (channel 3 of `jai_play_push`) and standard input
/// (channel 4, or the page's `jai_stdin_read`); a page checks for the export before using them.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_accepts_arguments() -> u32 {
    1
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

/// Bound the next runs to `thousands * 1000` interpreter blocks; 0 removes the bound. Survives
/// `jai_play_reset`.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_set_budget(thousands: u32) -> u32 {
    with(|s| {
        s.limits.budget = (thousands != 0).then(|| u64::from(thousands) * 1000);
        0
    })
}

/// Render the next runs' errors with ANSI colour and box drawing (`1`) or as plain text (`0`,
/// the default). Survives `jai_play_reset`.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_set_styled(styled: u32) -> u32 {
    with(|s| {
        s.limits.styled = styled != 0;
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
        let raw = std::mem::take(&mut s.args);
        let mut pieces: Vec<&[u8]> = raw.split(|&b| b == 0).collect();
        // Every argument ends with a NUL, so the last piece is the empty rest.
        pieces.pop();
        let Ok(args) = pieces
            .into_iter()
            .map(|arg| String::from_utf8(arg.to_vec()))
            .collect::<Result<Vec<_>, _>>()
        else {
            return s.fail("arguments must be UTF-8");
        };
        let io = play::PlayIo {
            args,
            stdin: std::mem::take(&mut s.fixed_stdin).then(|| std::mem::take(&mut s.stdin)),
        };
        s.output = play::run_with_io(&files, &main, s.limits, &io)
            .to_json()
            .into_bytes();
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

/// Length of the last recorded panic message (0 when none).
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_panic_len() -> u32 {
    PANIC.with(|p| u32::try_from(p.borrow().len()).unwrap_or(u32::MAX))
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_play_panic_byte(index: u32) -> u32 {
    PANIC.with(|p| {
        p.borrow()
            .as_bytes()
            .get(index as usize)
            .copied()
            .map_or(256, u32::from)
    })
}

/// Zeroed, 16-byte aligned memory the page fills for the running program (mapped buffer
/// ranges, strings it returns); 0 when it cannot be had. See `host_bridge::host_alloc`.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_host_alloc(size: usize) -> usize {
    crate::host_bridge::host_alloc(size)
}

/// Release what `jai_host_alloc` returned (the size is not needed).
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_host_free(address: usize, _size: usize) {
    crate::host_bridge::host_free(address);
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

    #[test]
    fn arguments_and_fixed_input_reach_the_program() {
        jai_play_reset();
        push(0, "main.jai");
        push(
            1,
            concat!(
                "#import \"Basic\";\n#import \"POSIX\";\n",
                "main :: () {\n",
                "    args := get_command_line_arguments();\n",
                "    for args print(\"[%]\", it);\n",
                "    buf: [64] u8;\n",
                "    n := read(0, buf.data, 64);\n",
                "    print(\" %:%\", n, string.{ n, buf.data });\n",
                "}\n",
            ),
        );
        assert_eq!(jai_play_finish_file(), 0);
        push(2, "main.jai");
        push(3, "main\0two words\0\0");
        push(4, "hi\n");
        assert_eq!(jai_play_run(), 0);
        let bytes: Vec<u8> = (0..jai_play_output_len())
            .map(|i| jai_play_output_byte(i) as u8)
            .collect();
        let json = String::from_utf8(bytes).unwrap();
        assert!(
            json.contains("\"stdout\":\"[main][two words][] 3:hi\\n\""),
            "{json}"
        );
    }
}
