//! Compile only independently authored source and newly emitted LLVM modules.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-float-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn run(&self, source: &str) -> std::process::ExitStatus {
        let source_path = self.0.join("main.jai");
        fs::write(&source_path, source).unwrap();
        let graph =
            jai_modules::ModuleGraph::load(&source_path, jai_modules::GraphOptions::default())
                .unwrap();
        let program = jai_sema::resolve_graph(&graph).unwrap();
        let ir = jai_codegen::emit(&program).unwrap();
        let llvm_path = self.0.join("main.ll");
        fs::write(&llvm_path, ir).unwrap();
        let executable = self.0.join("main");
        let mut command = native_tools::clang_command();
        command.arg(&llvm_path).arg("-o").arg(&executable);
        if cfg!(target_os = "linux") {
            command.arg("-lm");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Command::new(executable).status().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn float_source_calls_storage_comparisons_and_contextual_rounding_execute_natively() {
    let fixture = Fixture::new();
    let status = fixture.run(
        r#"
        global: float64 = 2.5;
        CONSTANT :: cast(float32) 1.0000000596046448;
        TINY :: 1e-40;
        tiny_global := 1e-40;
        add :: (a: float32, b: float32) -> float32 { return a+b; }
        remainder32 :: (a:float32,b:float32)->float32 {return a%b;}
        remainder64 :: (a:float64,b:float64)->float64 {return a%b;}
        wider :: (x:float64=2.0) -> float64 { return x*global; }
        main :: () -> int {
            rounded:float32=1.0000000596046448;
            if rounded != 0h3f800001 return 1;
            if CONSTANT != rounded return 11;
            if wider()!=5.0 return 12;
            if remainder32(5.5,2.0) != 1.5 return 13;
            if remainder64(5.5,2.0) != 1.5 return 14;
            x:float32=16777216.0;
            x=add(x,1.0);
            if x!=16777216.0 return 2;
            y:float64=16777216.0;
            y+=1.0;
            if y!=16777217.0 return 3;
            if wider(2.0)!=5.0 return 4;
            nan:float32=0h7fbfffff;
            if nan==nan return 5;
            if !(nan!=nan) return 6;
            if nan<0.0 return 7;
            zero:float64=0h8000000000000000;
            if zero!=0.0 return 8;
            if cast(s8) -128.9 != -128 return 9;
            if cast(u8) -0.9 != 0 return 10;
            if cast(s8) 127.9 != 127 return 15;
            if cast(u8) 255.9 != 255 return 16;
            if cast(s64) 0hc3e0000000000000 != -9223372036854775808 return 17;
            if cast(u64) 0h43efffffffffffff != 18446744073709549568 return 18;
            unsigned:u64=18446744073709551615;
            if cast(float64) unsigned != 0h43f0000000000000 return 19;
            signed:s64=-9223372036854775808;
            if cast(float32) signed != 0hdf000000 return 20;
            tiny := 1e-40;
            tiny_named := TINY;
            if tiny != cast(float64) 1e-40 return 21;
            if tiny_named != tiny || tiny_global != tiny return 22;
            if tiny == cast(float64) (cast(float32) 1e-40) return 23;
            contextual:float32=1e-40;
            if contextual==0.0 return 24;
            return 0;
        }
    "#,
    );
    assert_eq!(status.code(), Some(0));
}

#[test]
fn invalid_float_to_integer_casts_trap_before_llvm_conversion() {
    for (target, value) in [
        ("s64", "0h7ff8000000000001"),
        ("s64", "0h7ff0000000000000"),
        ("s64", "9223372036854775808.0"),
        ("s8", "-129.0"),
        ("u8", "-1.0"),
        ("u8", "256.0"),
        ("u64", "0h43f0000000000000"),
    ] {
        let fixture = Fixture::new();
        let status = fixture.run(&format!(
            "main :: () -> int {{ x:float64={value}; converted := cast({target}) x; return 0; }}"
        ));
        assert!(!status.success(), "invalid cast succeeded: {value}");
    }
}

#[test]
fn float_initializer_coercions_agree_for_globals_and_locals() {
    for expression in ["cast(s32) 1", "cast(float64) 1.0"] {
        for source in [
            format!("value:float32={expression}; main :: () {{}}"),
            format!("main :: () {{value:float32={expression};}}"),
        ] {
            let fixture = Fixture::new();
            let source_path = fixture.0.join("main.jai");
            fs::write(&source_path, &source).unwrap();
            let graph =
                jai_modules::ModuleGraph::load(&source_path, jai_modules::GraphOptions::default())
                    .unwrap();
            assert!(
                jai_sema::resolve_graph(&graph).is_err(),
                "invalid implicit conversion accepted: {source}"
            );
        }
    }
}

#[test]
fn source_run_float_results_materialize_as_typed_ieee_constants() {
    let fixture = Fixture::new();
    let status = fixture.run(
        r#"
        narrow :: () -> float32 {return 1.0000000596046448;}
        nan_value :: () -> float32 {return 0h7fbfffff;}
        zero_value :: () -> float64 {return 0h8000000000000000;}
        NARROW :: #run narrow();
        NAN_VALUE :: #run nan_value();
        ZERO_VALUE :: #run zero_value();
        main :: () -> int {
            if NARROW!=0h3f800001 return 1;
            if NAN_VALUE==NAN_VALUE return 2;
            x:float64=ZERO_VALUE;
            if x!=0.0 return 3;
            zero_bits := cast(*u64) *x;
            if zero_bits.* != 0x8000000000000000 return 4;
            n:float32=NAN_VALUE;
            nan_bits := cast(*u32) *n;
            if nan_bits.* != 0x7fbfffff return 5;
            return 0;
        }
    "#,
    );
    assert_eq!(status.code(), Some(0));
}

#[test]
fn named_decimal_constants_bind_directly_in_each_storage_and_call_context() {
    let fixture = Fixture::new();
    let status = fixture.run(
        r#"
        PRECISE :: 1.0000000596046448;
        ALIAS :: PRECISE + 0.0;
        narrow_global:float32=ALIAS;
        wide_global:float64=ALIAS;
        take :: (x:float32)->float32 {return x;}
        main :: ()->int {
            LOCAL :: ALIAS;
            narrow:float32=LOCAL;
            wide:float64=LOCAL;
            if narrow!=0h3f800001 return 1;
            if narrow_global!=narrow return 2;
            if wide_global!=wide return 3;
            if cast(float32) wide!=1.0 return 4;
            if take(ALIAS)!=narrow return 5;
            return 0;
        }
    "#,
    );
    assert_eq!(status.code(), Some(0));
}

#[test]
fn repeated_source_aliases_remain_compact_and_round_in_both_contexts() {
    let mut source = String::from("K0 :: 0.125;\n");
    for index in 1..=128 {
        source.push_str(&format!("K{index} :: K{} + K{};\n", index - 1, index - 1));
    }
    source.push_str(
        r#"
        narrow :: ()->float32 {return K128;}
        wide :: ()->float64 {return K128;}
        NARROW :: #run narrow();
        WIDE :: #run wide();
        main :: ()->int {
            a:float32=K128;
            b:float64=K128;
            if a!=0h7e000000 || NARROW!=a return 1;
            if b!=0h47c0000000000000 || WIDE!=b return 2;
            return 0;
        }
        "#,
    );
    assert_eq!(Fixture::new().run(&source).code(), Some(0));
}
