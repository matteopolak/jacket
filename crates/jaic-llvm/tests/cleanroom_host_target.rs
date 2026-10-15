//! Emitted code preserves compiler-host tags when executing on a different target.
#[path = "support/backend_execution.rs"]
mod backend;

#[test]
fn host_constants_execute_native_and_wasm_at_o0_and_o2() {
    use jaic::sema::{Options, TargetCpu, TargetOs};
    let host = Options::host();
    let os = match host.os {
        TargetOs::Windows => "WINDOWS",
        TargetOs::Linux => "LINUX",
        TargetOs::MacOS => "MACOS",
        TargetOs::Wasm => "WASM",
    };
    let cpu = match host.cpu {
        TargetCpu::X64 => "X64",
        TargetCpu::Arm64 => "ARM64",
        TargetCpu::Wasm => "CUSTOM",
    };
    let source = format!(
        r#"
        #assert BUILD_OS == .{os} && BUILD_CPU == .{cpu};
        #program_export "host_target_check"
        host_target_check :: () -> s32 #c_call {{
            if BUILD_OS != .{os} || BUILD_CPU != .{cpu} return 1;
            #if OS == .WASM {{
                if CPU != .CUSTOM return 2;
                if OS == BUILD_OS && CPU == BUILD_CPU return 3;
            }} else {{
                if OS != BUILD_OS || CPU != BUILD_CPU return 4;
            }}
            return 0;
        }}
        main :: () {{}}
    "#
    );
    backend::execute(
        "host-target",
        "host_target_check",
        &source,
        &[(
            "/lib/prelude/module.jai",
            "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }",
        )],
        None,
    );
}
