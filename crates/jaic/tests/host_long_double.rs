//! A `long double` computed by compile-time code has the host's format; the program being built
//! gets the same number in the target's format. The host is replaced (`Options::host`) so the two
//! formats differ whatever machine runs the test.
use jaic::sema::{Compiler, HostPlatform, Options, TargetCpu, TargetOs, VirtualFs};
use jaic::wide_float::WideFloat;
use std::{path::Path, rc::Rc};

const PRELUDE: &str = "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }";

/// Compile `source` for a Linux target whose `long double` is `target`, with compile-time code
/// running on a host whose `long double` is `host`.
fn compile(source: &str, host: Option<WideFloat>, target: Option<WideFloat>) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    fs.insert("/prelude/module.jai", PRELUDE.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude/module.jai".into());
    options.host = HostPlatform {
        os: TargetOs::Linux,
        cpu: TargetCpu::X64,
        long_double: host,
    };
    options.os = TargetOs::Linux;
    options.cpu = TargetCpu::Arm64;
    options.long_double = target;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.src.sources))
}

/// `2^-60`, computed by a run and again by the target compiler, which must agree. The sum with
/// one shows the bits past `float64`'s 53 survived the crossing.
const SOURCE: &str = "
Ld :: #jaic_type long_double;
half :: (n: s64) -> Ld { x: Ld = 1; for 1..n x /= 2; return x; }
FOLDED :: #run half(60);
#assert FOLDED == half(60);
#assert FOLDED + 1 != 1;
main :: () { tiny := half(60); ok := tiny == FOLDED; }
";

#[test]
fn x87_host_result_becomes_a_binary128_constant() {
    compile(SOURCE, Some(WideFloat::X87), Some(WideFloat::Binary128)).unwrap();
}

#[test]
fn binary128_host_result_becomes_an_x87_constant() {
    compile(SOURCE, Some(WideFloat::Binary128), Some(WideFloat::X87)).unwrap();
}

#[test]
fn wide_host_result_becomes_a_float64_where_the_target_has_no_wider_long_double() {
    compile(
        "
Ld :: #jaic_type long_double;
half :: (n: s64) -> Ld { x: Ld = 1; for 1..n x /= 2; return x; }
FOLDED :: #run half(60);
#assert FOLDED * 1152921504606846976.0 == 1.0;
main :: () { tiny: Ld = FOLDED; }
",
        Some(WideFloat::X87),
        None,
    )
    .unwrap();
}

#[test]
fn float64_host_result_becomes_a_wide_constant_for_a_wide_target() {
    compile(
        "
Ld :: #jaic_type long_double;
half :: (n: s64) -> Ld { x: Ld = 1; for 1..n x /= 2; return x; }
FOLDED :: #run half(60);
#assert FOLDED == half(60);
main :: () { tiny: Ld = FOLDED; }
",
        None,
        Some(WideFloat::Binary128),
    )
    .unwrap();
}

#[test]
fn a_long_double_inside_another_type_must_have_the_same_format() {
    let error = compile(
        "
Ld :: #jaic_type long_double;
Holder :: struct { value: Ld; }
make :: () -> Holder { h: Holder; h.value = 3; return h; }
HELD :: #run make();
main :: () {}
",
        Some(WideFloat::X87),
        Some(WideFloat::Binary128),
    )
    .expect_err("the struct holds the host's format");
    assert!(error.contains("different format"), "{error}");
}
