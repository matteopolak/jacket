//! Independent bounded byte channels for the shared language server; no pointers or IO.
use crate::play::{STDLIB_ROOT, virtual_fs};
use jai_language_server::{Environment, JsonSession, Limits};
use jaic::sema::{Options, TargetCpu, TargetOs};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
const INPUT_LIMIT: usize = 1024 * 1024;
const OUTPUT_LIMIT: usize = 2 * 1024 * 1024;
struct Bridge {
    session: JsonSession,
    input: Vec<u8>,
    output: Vec<u8>,
    diagnostic: Vec<u8>,
}
impl Default for Bridge {
    fn default() -> Self {
        Self {
            session: JsonSession::with_environment(Limits::default(), environment()),
            input: Vec::new(),
            output: Vec::new(),
            diagnostic: Vec::new(),
        }
    }
}
impl Bridge {
    fn fail(&mut self, message: impl ToString) -> u32 {
        self.input.clear();
        self.output.clear();
        self.diagnostic = message.to_string().into_bytes();
        1
    }
    fn push(&mut self, byte: u32) -> u32 {
        let Ok(byte) = u8::try_from(byte) else {
            return self.fail("language input is not a byte");
        };
        if self.input.len() >= INPUT_LIMIT {
            return self.fail("language message byte limit exceeded");
        }
        self.input.push(byte);
        0
    }
    fn dispatch(&mut self) -> u32 {
        let bytes = std::mem::take(&mut self.input);
        let text = match std::str::from_utf8(&bytes) {
            Ok(text) => text,
            Err(_) => return self.fail("language request must be UTF-8"),
        };
        let messages = match self.session.handle_json(text) {
            Ok(messages) => messages,
            Err(error) => return self.fail(error),
        };
        let Some(length) = messages
            .iter()
            .try_fold(2usize, |length, message| {
                length.checked_add(message.len())?.checked_add(1)
            })
            .filter(|length| *length <= OUTPUT_LIMIT)
        else {
            return self.fail("language response byte limit exceeded");
        };
        let mut output = Vec::with_capacity(length);
        output.push(b'[');
        for (index, message) in messages.iter().enumerate() {
            if index != 0 {
                output.push(b',');
            }
            output.extend_from_slice(message.as_bytes());
        }
        output.push(b']');
        self.output = output;
        self.diagnostic.clear();
        0
    }
}
/// Hover and completion type-check against the bundled stdlib, as the playground compiles.
fn environment() -> Environment {
    Environment {
        fs: Rc::new(virtual_fs(&BTreeMap::new())),
        options: Box::new(|main| {
            let mut options = Options::host();
            options.os = TargetOs::Wasm;
            options.cpu = TargetCpu::Wasm;
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), PathBuf::from(STDLIB_ROOT)];
            options.preload = Some(PathBuf::from(format!("{STDLIB_ROOT}/module.jai")));
            options
        }),
    }
}
thread_local! {
    static STATE: RefCell<Bridge> = RefCell::new(Bridge::default());
}
/// Run `f` on the bridge; 1 if it is already in use (a re-entrant call).
fn with<T>(fallback: T, f: impl FnOnce(&mut Bridge) -> T) -> T {
    STATE.with(|state| match state.try_borrow_mut() {
        Ok(mut bridge) => f(&mut bridge),
        Err(_) => fallback,
    })
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_reset() -> u32 {
    with(1, |bridge| {
        *bridge = Bridge::default();
        0
    })
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_begin() -> u32 {
    with(1, |bridge| {
        bridge.input.clear();
        bridge.output.clear();
        bridge.diagnostic.clear();
        0
    })
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_push(byte: u32) -> u32 {
    with(1, |bridge| bridge.push(byte))
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_dispatch() -> u32 {
    with(1, |bridge| bridge.dispatch())
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_output_len() -> u32 {
    with(0, |bridge| bridge.output.len() as u32)
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_output_byte(index: u32) -> u32 {
    with(256, |bridge| {
        bridge
            .output
            .get(index as usize)
            .copied()
            .map_or(256, u32::from)
    })
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_diagnostic_len() -> u32 {
    with(0, |bridge| bridge.diagnostic.len() as u32)
}
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn jai_lsp_diagnostic_byte(index: u32) -> u32 {
    with(256, |bridge| {
        bridge
            .diagnostic
            .get(index as usize)
            .copied()
            .map_or(256, u32::from)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_json_core_and_pointer_free_limits_are_paired() {
        let mut bridge = Bridge::default();
        let request =
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
        for byte in request.bytes() {
            assert_eq!(bridge.push(u32::from(byte)), 0);
        }
        assert_eq!(bridge.dispatch(), 0);
        let output = std::str::from_utf8(&bridge.output).unwrap();
        assert!(output.contains("capabilities"));
        assert!(output.contains("\"id\":1"));
        assert_eq!(bridge.push(256), 1);
        assert!(bridge.input.is_empty());
        bridge.input = vec![b' '; INPUT_LIMIT];
        assert_eq!(bridge.push(u32::from(b' ')), 1);
        assert!(bridge.input.is_empty());
        bridge.input = vec![0xff];
        assert_eq!(bridge.dispatch(), 1);
        assert!(
            std::str::from_utf8(&bridge.diagnostic)
                .unwrap()
                .contains("UTF-8")
        );
    }

    fn send(bridge: &mut Bridge, message: &str) -> String {
        for byte in message.bytes() {
            assert_eq!(bridge.push(u32::from(byte)), 0);
        }
        assert_eq!(bridge.dispatch(), 0);
        String::from_utf8(bridge.output.clone()).unwrap()
    }

    #[test]
    fn hover_and_completion_use_the_bundled_stdlib() {
        let mut bridge = Bridge::default();
        send(
            &mut bridge,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        send(
            &mut bridge,
            r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        );
        let text = "#import \"Basic\";\nmain :: () {\n    count := 3;\n    print(\"%\", count);\n    \n}\n";
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        let open = format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///jai-script/main.jai","languageId":"jai","version":1,"text":"{escaped}"}}}}}}"#
        );
        send(&mut bridge, &open);
        let hover = send(
            &mut bridge,
            r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///jai-script/main.jai"},"position":{"line":3,"character":18}}}"#,
        );
        assert!(hover.contains("count: s64"), "{hover}");
        let completion = send(
            &mut bridge,
            r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///jai-script/main.jai"},"position":{"line":4,"character":4}}}"#,
        );
        assert!(completion.contains("\"label\":\"count\""), "{completion}");
        assert!(completion.contains("\"label\":\"print\""), "{completion}");
    }
}
