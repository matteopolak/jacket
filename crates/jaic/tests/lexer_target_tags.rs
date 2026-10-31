//! The lexer's platform tags cover every target the compiler can select.
use jaic::lexer::{CPU_TAGS, OS_TAGS};
use jaic::target::{TargetCpu, TargetOs, cpu_member, os_member};

#[test]
fn target_members_are_known_tags() {
    for os in [
        TargetOs::Windows,
        TargetOs::Linux,
        TargetOs::MacOS,
        TargetOs::Wasm,
    ] {
        assert!(OS_TAGS.contains(&os_member(os)));
    }
    for cpu in [TargetCpu::X64, TargetCpu::Arm64, TargetCpu::Wasm] {
        assert!(CPU_TAGS.contains(&cpu_member(cpu)));
    }
}
