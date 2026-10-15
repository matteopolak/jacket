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
            session: JsonSession::with_environment(Limits::browser(), lsp_environment()),
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
pub fn lsp_environment() -> Environment {
    let fs = Rc::new(virtual_fs(&BTreeMap::new()));
    let option_fs = fs.clone();
    Environment {
        fs,
        options: Box::new(move |main| {
            let mut options = Options::host();
            options.os = TargetOs::Wasm;
            options.cpu = TargetCpu::Wasm;
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), PathBuf::from(STDLIB_ROOT)];
            options.preload = Some(
                jaic::find_preload(&*option_fs, std::path::Path::new(STDLIB_ROOT))
                    .unwrap_or_else(|| PathBuf::from(STDLIB_ROOT).join(jaic::PRELOAD_ENTRY)),
            );
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
        let text = concat!(
            "#import \"Basic\";\n",
            "main :: () {\n",
            "    count := 3;\n",
            "    print(\"%\", count);\n",
            "    \n",
            "}\n",
            "Pair :: struct { a: u8; b: s32; }\n",
        );
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        let open = format!(
            concat!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"#,
                r#""uri":"file:///jai-script/main.jai","languageId":"jai","version":1,"#,
                r#""text":"{escaped}"}}}}}}"#,
            ),
            escaped = escaped,
        );
        send(&mut bridge, &open);
        let hover = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":3,"character":18}}}"#,
            ),
        );
        assert!(hover.contains("count: s64"), "{hover}");
        let layout = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":4,"method":"textDocument/hover","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":6,"character":1}}}"#,
            ),
        );
        assert!(
            layout.contains("size 8, align 4 (3 bytes of padding)"),
            "{layout}"
        );
        let completion = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":4,"character":4}}}"#,
            ),
        );
        assert!(completion.contains("\"label\":\"count\""), "{completion}");
        assert!(completion.contains("\"label\":\"print\""), "{completion}");
    }

    #[test]
    fn expansions_inlay_hints_and_format_strings_work_in_the_bridge() {
        let mut bridge = Bridge::default();
        send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{"#,
                r#""textDocument":{"hover":{"contentFormat":["markdown","plaintext"]}}}}}"#,
            ),
        );
        let text = concat!(
            "#import \"Basic\";\n",
            "LIMIT :: #run 6 * 7;\n",
            "main :: () {\n",
            "    count := 3;\n",
            "    #insert \"twice := count * 2;\";\n",
            "    print(\"% %\\n\", count, twice);\n",
            "}\n",
        );
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        let open = format!(
            concat!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"#,
                r#""uri":"file:///jai-script/main.jai","languageId":"jai","version":1,"#,
                r#""text":"{escaped}"}}}}}}"#,
            ),
            escaped = escaped,
        );
        send(&mut bridge, &open);
        // A format string with too few arguments is the compiler's warning.
        let bad = r##"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///jai-script/bad.jai","languageId":"jai","version":1,"text":"#import \"Basic\";\nmain :: () { print(\"% %\\n\", 1); }\n"}}}"##;
        let published = send(&mut bridge, bad);
        assert!(
            published.contains("incorrect number of arguments"),
            "{published}"
        );
        let hints = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/inlayHint","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""range":{"start":{"line":0,"character":0},"end":{"line":7,"character":0}}}}"#,
            ),
        );
        assert!(hints.contains("\"label\":\": s64\""), "{hints}");
        let expansion = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":3,"method":"jai/expansion","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":4,"character":6}}}"#,
            ),
        );
        assert!(expansion.contains("twice := count * 2;"), "{expansion}");
        let hover = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":4,"method":"textDocument/hover","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":5,"character":12}}}"#,
            ),
        );
        // The client asked for Markdown: the argument and its type are a code span.
        assert!(hover.contains(r#""kind":"markdown""#), "{hover}");
        assert!(hover.contains("`count: s64`"), "{hover}");
        assert!(hover.contains("`twice: s64`"), "{hover}");
        // `#import "Basic"` goes to the bundled stdlib, readable through `jai/source`.
        let definition = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":5,"method":"textDocument/definition","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":0,"character":10}}}"#,
            ),
        );
        assert!(
            definition.contains(r#""uri":"file:///stdlib/Basic/module.jai""#),
            "{definition}"
        );
        let links = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":6,"method":"textDocument/documentLink","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"}}}"#,
            ),
        );
        assert!(
            links.contains(r#""target":"file:///stdlib/Basic/module.jai""#),
            "{links}"
        );
        let source = send(
            &mut bridge,
            r#"{"jsonrpc":"2.0","id":7,"method":"jai/source","params":{"uri":"file:///stdlib/Basic/module.jai"}}"#,
        );
        let messages: Vec<serde_json::Value> = serde_json::from_str(&source).unwrap();
        let response = messages.iter().find(|message| message["id"] == 7).unwrap();
        assert_eq!(
            response["result"].as_str().unwrap(),
            include_str!("../../../stdlib/Basic/module.jai"),
            "jai/source must return the complete canonical public facade"
        );
    }

    #[test]
    fn asm_blocks_get_instruction_completion_and_hover_in_the_bridge() {
        let mut bridge = Bridge::default();
        send(
            &mut bridge,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        // An unterminated block, as it is while being typed.
        let open = concat!(
            r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"#,
            r#""uri":"file:///jai-script/main.jai","languageId":"jai","version":1,"#,
            r#""text":"main :: () {\n    total := 1;\n    #asm AVX2 {\n        vpaddd v:, v, v;\n        vpad\n}\n"}}}"#,
        );
        send(&mut bridge, open);
        let completion = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":4,"character":12}}}"#,
            ),
        );
        assert!(
            completion.contains(r#""label":"vpaddd""#)
                && completion.contains("vpaddd dst: vec, a: vec, b: vec/mem (AVX2)")
                && completion.contains("Add packed dwords"),
            "{completion}"
        );
        let hover = send(
            &mut bridge,
            concat!(
                r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"#,
                r#""textDocument":{"uri":"file:///jai-script/main.jai"},"#,
                r#""position":{"line":3,"character":10}}}"#,
            ),
        );
        assert!(
            hover.contains("Requires AVX2") && hover.contains(r#""id":3"#),
            "{hover}"
        );
    }
}
