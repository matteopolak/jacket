//! `ARCHITECTURE` names the CPU family exactly, where `CPU` shares `.CUSTOM` between targets.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic::target::Target;
use std::rc::Rc;

const PRELUDE: &str = "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; } Architecture :: enum u32 { X64; ARM64; X86; WASM32; WASM64; }";

fn check(triple: &str, member: &str) {
    let target = Target::parse(triple).unwrap();
    let mut options = Options::host();
    options.os = target.os;
    options.cpu = target.cpu;
    options.layout = target.data_layout();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", PRELUDE.as_bytes());
    let mut c = Compiler::new(options, Rc::new(fs));
    let source = format!("#assert ARCHITECTURE == .{member}; main :: () {{}}");
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{triple}: {}", e.render(&c.src.sources));
    }
}

#[test]
fn the_architecture_constant_names_each_target() {
    check("x86_64-unknown-linux-gnu", "X64");
    check("aarch64-unknown-linux-gnu", "ARM64");
    check("i686-unknown-linux-gnu", "X86");
    check("wasm32-wasi", "WASM32");
    check("wasm64-wasi", "WASM64");
}
