//! Compiler-host constants must remain independent of the selected output target.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use std::rc::Rc;

fn os_name(os: TargetOs) -> &'static str {
    match os {
        TargetOs::Windows => "WINDOWS",
        TargetOs::Linux => "LINUX",
        TargetOs::MacOS => "MACOS",
        TargetOs::Wasm => "WASM",
    }
}

fn cpu_name(cpu: TargetCpu) -> &'static str {
    match cpu {
        TargetCpu::X64 => "X64",
        TargetCpu::Arm64 => "ARM64",
        TargetCpu::Wasm => "CUSTOM",
    }
}

#[test]
fn build_constants_identify_compiler_host_for_every_selected_target() {
    let host = Options::host();
    for os in [
        TargetOs::Windows,
        TargetOs::Linux,
        TargetOs::MacOS,
        TargetOs::Wasm,
    ] {
        for cpu in [TargetCpu::X64, TargetCpu::Arm64, TargetCpu::Wasm] {
            let mut options = Options::host();
            options.os = os;
            options.cpu = cpu;
            options.runtime_support = false;
            options.preload = Some("/lib/prelude/module.jai".into());
            let mut fs = VirtualFs::default();
            fs.insert("/lib/prelude/module.jai", "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }".as_bytes());
            let mut c = Compiler::new(options, Rc::new(fs));
            let source = format!(
                r#"
                #assert OS == .{target_os};
                #assert CPU == .{target_cpu};
                #assert BUILD_OS == .{host_os};
                #assert BUILD_CPU == .{host_cpu};
                #assert type_of(OS) == type_of(BUILD_OS);
                #assert type_of(CPU) == type_of(BUILD_CPU);
                #assert (OS != BUILD_OS || CPU != BUILD_CPU) == {cross};
                main :: () {{}}
                "#,
                target_os = os_name(os),
                target_cpu = cpu_name(cpu),
                host_os = os_name(host.os),
                host_cpu = cpu_name(host.cpu),
                cross = os != host.os || cpu != host.cpu,
            );
            if let Err(error) = c.compile_sources(&[ProgramSource::String(source)]) {
                panic!("{}", error.render(&c.sources));
            }
        }
    }
}
