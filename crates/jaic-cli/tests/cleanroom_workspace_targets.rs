//! Workspace OS/CPU settings must select the actual emitted object format.
#![cfg(feature = "llvm")]

use std::path::Path;
use std::process::Command;

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn check_object(path: &Path, os: &str, cpu: &str) {
    let bytes = std::fs::read(path).expect("actual compiler object output");
    assert!(
        bytes.len() > 32,
        "empty/truncated object: {}",
        path.display()
    );
    match os {
        "LINUX" => {
            assert_eq!(&bytes[..4], b"\x7fELF", "{}", path.display());
            assert_eq!(bytes[4], 2, "ELF64");
            assert_eq!(bytes[5], 1, "little-endian ELF");
            assert_eq!(u16_at(&bytes, 16), 1, "ET_REL");
            assert_eq!(
                u16_at(&bytes, 18),
                if cpu == "X64" {
                    62
                } else {
                    183
                }
            );
        }
        "MACOS" => {
            assert_eq!(u32_at(&bytes, 0), 0xfeed_facf, "Mach-O64");
            assert_eq!(
                u32_at(&bytes, 4),
                if cpu == "X64" {
                    0x0100_0007
                } else {
                    0x0100_000c
                }
            );
            assert_eq!(u32_at(&bytes, 12), 1, "MH_OBJECT");
        }
        "WINDOWS" => {
            assert_eq!(
                u16_at(&bytes, 0),
                if cpu == "X64" {
                    0x8664
                } else {
                    0xaa64
                }
            );
            assert!(u16_at(&bytes, 2) > 0, "COFF sections");
        }
        "WASM" => assert_eq!(&bytes[..8], b"\0asm\x01\0\0\0"),
        _ => unreachable!(),
    }
}

#[test]
fn workspace_options_emit_the_requested_object_format_and_machine() {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("workspace-targets-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for (os, cpu) in [
        ("LINUX", "X64"),
        ("LINUX", "ARM64"),
        ("MACOS", "X64"),
        ("MACOS", "ARM64"),
        ("WINDOWS", "X64"),
        ("WINDOWS", "ARM64"),
        ("WASM", "CUSTOM"),
    ] {
        let name = format!("object-{os}-{cpu}");
        let source = directory.join(format!("{name}.jai"));
        let output = directory.join(format!("{name}.o"));
        let text = format!(
            r#"C :: #import "Compiler";
#run,stallable {{
    C.set_build_options_dc(.{{do_output=false}});
    child := C.compiler_create_workspace("object target regression");
    options := C.get_build_options(child);
    options.os_target = .{os};
    options.cpu_target = .{cpu};
    options.output_type = .OBJECT_FILE;
    options.output_path = "{}";
    options.output_executable_name = "{name}.o";
    C.set_build_options(options, child);
    C.compiler_begin_intercept(child);
    C.add_build_string("main :: () -> s32 {{ return 17; }}", child);
    while true {{
        message := C.compiler_wait_for_message();
        if !message {{ C.compiler_report("missing child completion"); break; }}
        if message.workspace == child && message.kind == .COMPLETE {{
            complete := cast(*C.Message_Complete) message;
            if complete.error_code != .NONE C.compiler_report("object child failed");
            break;
        }}
    }}
    C.compiler_end_intercept(child);
}}
main :: () {{}}
"#,
            directory.display().to_string().replace('\\', "/")
        );
        std::fs::write(&source, text).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&source)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{os}-{cpu}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        check_object(&output, os, cpu);
    }
}
