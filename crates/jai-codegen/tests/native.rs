//! Actual execution of newly generated code; never uses supplied native code.
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
struct Scratch(PathBuf);
static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execute(source: &str) -> i32 {
    execute_status(source)
        .code()
        .expect("test program terminated by a signal")
}
fn execute_status(source: &str) -> ExitStatus {
    let module = jai_syntax::parse(source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    let ir = jai_codegen::emit(&program).unwrap();
    let path = std::env::temp_dir().join(format!(
        "jai-rust-test-{}-{}-{}",
        std::process::id(),
        NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let scratch = Scratch(path);
    let executable = scratch.0.join("program");
    let mut compiler = Command::new("clang")
        .args(["-x", "ir", "-", "-o"])
        .arg(&executable)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("native tests require independently installed clang");
    compiler
        .stdin
        .take()
        .unwrap()
        .write_all(ir.as_bytes())
        .unwrap();
    let result = compiler.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "LLVM failed: {}\n{ir}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut process = Command::new(&executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated program exceeded 5 seconds; possible control-flow regression");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn recursive_calls_and_early_return() {
    assert_eq!(
        execute(
            "fact :: (n:int)->int { if n <= 1 return 1; return n*fact(n-1); } main :: ()->int { return fact(4); }"
        ),
        24
    );
}
#[test]
fn mutable_loop_and_nested_scope() {
    assert_eq!(
        execute(
            "main :: ()->int { sum := 0; n := 0; while n < 7 { { n := 42; sum = sum + n; } n = n + 1; } return sum / 7; }"
        ),
        42
    );
}
#[test]
fn both_branches_terminate() {
    assert_eq!(
        execute(
            "pick :: (x:int)->int { if x > 0 return 17; else return 29; } main :: ()->int { return pick(-2); }"
        ),
        29
    );
}
#[test]
fn default_initialization_and_bitwise_values() {
    assert_eq!(
        execute("main :: ()->int { n: int; n = (0xff & 0b111111) ^ 0x15; return n; }"),
        42
    );
}

#[test]
fn boolean_arguments_returns_and_default() {
    assert_eq!(
        execute(
            "flip :: (b:bool)->bool { return !b; } main :: ()->int { b: bool; b = flip(b); if b == true return 42; else return 1; }"
        ),
        42
    );
}

#[test]
fn nested_short_circuit_skips_nonterminating_calls() {
    assert_eq!(
        execute(
            "forever :: ()->bool { while true {} return true; } main :: ()->int { if (false && forever()) || (true || forever()) return 42; else return 1; }"
        ),
        42
    );
}

#[test]
fn void_call_and_void_entry() {
    assert_eq!(
        execute("f :: (b:bool) { if b return; } main :: () { f(true); }"),
        0
    );
}

#[test]
fn integer_truthiness_is_explicit_in_lowering() {
    assert_eq!(
        execute("main :: ()->int { n := -7; if n && !0 && !!n return 42; else return 1; }"),
        42
    );
}

#[test]
fn explicit_scalar_casts_and_compound_assignment() {
    assert_eq!(
        execute(
            "main :: ()->int { n := cast(int) cast(bool) -9; n *= 21; n <<= 1; n |= 3; n ^= 3; return n; }"
        ),
        40
    );
}

#[test]
fn boolean_compound_assignments_short_circuit() {
    assert_eq!(
        execute(
            "forever :: ()->bool { while true {} return true; } main :: ()->int { b := false; b &&= forever(); b ||= true; return cast(int) b * 42; }"
        ),
        42
    );
}

#[test]
fn hexadecimal_subtraction_without_whitespace() {
    assert_eq!(execute("main :: ()->int { return 0xFE-212; }"), 42);
}

#[test]
fn inclusive_ranges_and_iterator_shadowing() {
    assert_eq!(
        execute(
            "main :: ()->int { total := 0; it := 7; for 1..3 { total += it; for 10..11 total += it; } return total + it; }"
        ),
        76
    );
}

#[test]
fn reverse_ranges_and_empty_ranges() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; for < i: 1..3 n = n * 10 + i; for 5..4 n = 0; for < 5..4 n = 0; return n - 279; }"
        ),
        42
    );
}

#[test]
fn range_endpoints_are_snapshots_before_iterator_binding() {
    assert_eq!(
        execute(
            "main :: ()->int { it := 2; limit := 4; n := 0; for it..limit { limit = 0; n += it; } return n + it; }"
        ),
        11
    );
}

#[test]
fn range_continue_runs_step_and_named_jump_selects_outer_loop() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; for outer: 1..5 { if outer == 2 continue; for inner: 1..4 { if inner == 2 continue outer; n += outer; } } return n; }"
        ),
        13
    );
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; for outer: 1..5 { for inner: 1..4 { if outer == 3 && inner == 2 break outer; n += 1; } } return n; }"
        ),
        9
    );
}

#[test]
fn while_jumps_and_named_condition_values() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; while true { n += 1; if n < 4 continue; else break; } return n; }"
        ),
        4
    );
    assert_eq!(
        execute(
            "main :: ()->int { n := 4; total := 0; while value := n { n -= 1; if value == 2 continue value; total += value; } return total; }"
        ),
        8
    );
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; while outer := n < 10 { n += 1; while true { if n == 4 break outer; else continue outer; } } return n; }"
        ),
        4
    );
}

#[test]
fn mixed_returns_and_loop_exits_have_no_spurious_join() {
    assert_eq!(
        execute(
            "f :: (b:bool)->int { while true { if b return 17; else break; } return 29; } main :: ()->int { return f(false) + f(true); }"
        ),
        46
    );
}

#[test]
fn inclusive_ranges_stop_at_integer_boundaries_without_wrapping() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; max := 9223372036854775807; min := -9223372036854775807 - 1; for max-1..max n += 1; for < min..min+1 n += 1; for max..max n += 1; for < min..min n += 1; return n; }"
        ),
        6
    );
}

#[test]
fn deferred_cleanup_is_lifo_and_reads_values_on_exit() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; x := 1; { defer n = n * 10 + x; defer n = n * 10 + 2; x = 3; } return n; }"
        ),
        23
    );
}

#[test]
fn deferred_cleanup_runs_on_while_break_and_continue() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; while true { defer n += 1; if n < 3 continue; break; } return n; }"
        ),
        4
    );
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; while true { if n == 3 break; defer n += 1; continue; } return n; }"
        ),
        3
    );
}

#[test]
fn outer_loop_exits_unwind_only_the_crossed_scopes() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; { defer n += 100; for outer: 1..3 { defer n += 10; for inner: 1..3 { defer n += 1; if inner == 2 continue outer; } } } return n; }"
        ),
        136
    );
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; for outer: 1..3 { defer n += 10; for inner: 1..3 { defer n += 1; break outer; } } return n; }"
        ),
        11
    );
}

#[test]
fn return_values_are_captured_before_deferred_mutation() {
    assert_eq!(
        execute(
            "f :: ()->int { n := 7; defer n = 99; { defer n = 88; return n; } } main :: ()->int { return f(); }"
        ),
        7
    );
    assert_eq!(
        execute(
            "f :: ()->bool { b := true; defer b = false; return b; } main :: ()->int { return cast(int) f(); }"
        ),
        1
    );
    assert_eq!(execute("main :: () { x := 0; defer x += 1; return; }"), 0);
}

#[test]
fn conditional_defer_activation_and_nested_cleanup() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; if false { defer n = 99; } { defer { defer n = n * 10 + 1; n = 2; } } return n; }"
        ),
        21
    );
}

#[test]
fn deferred_names_capture_their_original_binding() {
    assert_eq!(
        execute("main :: ()->int { n := 0; x := 3; { defer n = x; x := 8; x += 1; } return n; }"),
        3
    );
}

#[test]
fn cleanup_can_use_its_own_loops_and_short_circuiting() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; { defer { for i: 1..4 { if i == 2 continue i; n += i; } } } return n; }"
        ),
        8
    );
    assert_eq!(
        execute(
            "forever :: ()->bool { while true {} return true; } main :: ()->int { b := false; { defer b ||= (false && forever()) || true; } return cast(int) b * 42; }"
        ),
        42
    );
}

#[test]
fn constants_resolve_forward_dependencies_and_local_scopes() {
    assert_eq!(
        execute("ANSWER :: LIMIT * 6; LIMIT :: 7; main :: ()->int { return ANSWER; }"),
        42
    );
    assert_eq!(
        execute(
            "N :: 7; main :: ()->int { result := 0; { N :: M + 1; M :: 5; result = N; } return result + N; }"
        ),
        13
    );
    assert_eq!(execute("main :: ()->int { return N; N : int : 42; }"), 42);
}

#[test]
fn mutable_globals_are_shared_by_procedures_and_recursion() {
    assert_eq!(
        execute(
            "count : int; increment :: ()->int { count += 1; return count; } main :: ()->int { increment(); increment(); return count + 40; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "count := 0; visit :: (n:int) { count += 1; if n > 0 visit(n-1); } main :: ()->int { visit(4); return count; }"
        ),
        5
    );
}

#[test]
fn boolean_globals_and_constant_initializers_preserve_types() {
    assert_eq!(
        execute(
            "LIMIT :: 7; count := LIMIT * 6; enabled : bool; turn_on :: () { enabled = true; } main :: ()->int { turn_on(); if enabled return count; else return 1; }"
        ),
        42
    );
}

#[test]
fn local_shadowing_preserves_global_storage_and_deferred_capture() {
    assert_eq!(
        execute(
            "count := 3; main :: ()->int { { count := 9; count += 1; } { defer count += 4; count := 100; count += 1; } return count; }"
        ),
        7
    );
}

#[test]
fn constant_short_circuiting_does_not_execute_invalid_arithmetic() {
    assert_eq!(
        execute("PASS :: true || (1 / 0); main :: ()->int { if PASS return 42; else return 1; }"),
        42
    );
}

#[test]
fn conditional_values_support_recursion_and_optional_then() {
    assert_eq!(
        execute(
            "factorial :: (n:int)->int { return ifx n <= 1 then 1 else n * factorial(n-1); } main :: ()->int { return factorial(5); }"
        ),
        120
    );
    assert_eq!(
        execute(
            "main :: ()->int { units := 0; sequence_length := 4; units += ifx sequence_length == 4 2 else 1; return units; }"
        ),
        2
    );
}

#[test]
fn conditional_branches_execute_only_the_selected_side() {
    assert_eq!(
        execute(
            "count := 0; condition :: ()->bool { count += 1; return true; } selected :: ()->int { count += 10; return 31; } skipped :: ()->int { count += 100; return 99; } main :: ()->int { value := ifx condition() then selected() else skipped(); return value + count; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "count := 0; selected :: ()->int { count += 1; return 41; } skipped :: ()->int { count += 100; return 99; } main :: ()->int { value := ifx false then skipped() else selected(); return value + count; }"
        ),
        42
    );
}

#[test]
fn conditional_defaults_and_truthiness_preserve_result_types() {
    assert_eq!(
        execute(
            "main :: ()->int { a := ifx 0 then 99; b := ifx -1 42; flag := ifx 0 then true; return a + b + cast(int) flag; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "VALUE :: ifx true then NEXT else 1 / 0; NEXT :: 42; main :: ()->int { return VALUE; }"
        ),
        42
    );
}

#[test]
fn nested_conditional_and_logical_results_have_valid_phi_predecessors() {
    assert_eq!(
        execute(
            "forever :: ()->bool { while true {} return true; } main :: ()->int { flag := ifx true || forever() then (ifx false then forever() else true) else forever(); result := ifx flag then (ifx false then 99 else 42) else 0; return result; }"
        ),
        42
    );
    assert_eq!(
        execute("main :: ()->int { return ifx false then 1 else ifx true then 42 else 2; }"),
        42
    );
}

#[test]
fn integer_widths_flow_through_storage_arguments_and_results() {
    for ty in ["s8", "s16", "s32", "s64", "u8", "u16", "u32", "u64"] {
        let source = format!(
            "global : {ty} = 19; double :: (x:{ty})->{ty} {{ return x * 2; }} main :: ()->int {{ local : {ty} = 2; global += local; return double(global); }}"
        );
        if ty == "u64" {
            // u64 cannot implicitly become int: an explicit checked cast is required.
            assert_eq!(
                execute(
                    &source.replace("return double(global)", "return cast(int) double(global)")
                ),
                42
            );
        } else {
            assert_eq!(execute(&source), 42);
        }
    }
}

#[test]
fn integer_widening_sign_and_unsigned_arithmetic_are_preserved() {
    assert_eq!(
        execute("main :: ()->int { a:u8 = 200; b:s32 = -10; c:=a+b; return c; }"),
        190
    );
    assert_eq!(
        execute("main :: ()->int { small:s8 = -42; big:s64 = small; return -big; }"),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { n:u64 = 18446744073709551615; q:=n/2; r:=n%2; if q==9223372036854775807 && r==1 && n>9223372036854775807 return 42; else return 1; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { unsigned:u8 = 128; signed:s8 = -128; return cast(s16)(unsigned>>2) - (signed>>2); }"
        ),
        64
    );
}

#[test]
fn fixed_width_arithmetic_and_constant_evaluation_wrap_identically() {
    assert_eq!(
        execute(
            "C :: cast(u8) 255 + 1; main :: ()->int { n:u8 = 255; n+=1; if C==n return 42; else return 1; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "C :: cast(u64) 18446744073709551615 * cast(u64) 18446744073709551615; main :: ()->int { n:u64 = 18446744073709551615; n*=n; if C==n && n==1 return 42; else return 1; }"
        ),
        42
    );
    assert_eq!(
        execute("main :: ()->int { n:s8 = 127; n+=1; if n == -128 return 42; else return 1; }"),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { n:s64 = -9223372036854775808; if n<0 return 42; else return 1; }"
        ),
        42
    );
}

#[test]
fn checked_casts_and_intentional_truncation_have_distinct_behavior() {
    assert_eq!(
        execute("main :: ()->int { return ifx true then 42 else cast(u8) 256; }"),
        42
    );
    assert_eq!(
        execute("main :: ()->int { if true || cast(u8) 256 return 42; else return 1; }"),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { wide:u16 = 298; narrowed:=cast,no_check(u8) wide; return narrowed; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { signed:s8 = -1; unsigned:=cast,no_check(u8) signed; if unsigned==255 return 42; else return 1; }"
        ),
        42
    );
    assert_eq!(
        execute("main :: ()->int { wide:u64 = 42; return cast(s8) wide; }"),
        42
    );
    assert_eq!(
        execute("main :: ()->int { wide:u16 = 256; return ifx true then 42 else cast(u8) wide; }"),
        42
    );
}

#[cfg(unix)]
#[test]
fn checked_integer_casts_trap_when_the_value_is_out_of_range() {
    use std::os::unix::process::ExitStatusExt;
    for source in [
        "main :: ()->int { return cast(u8) 256; }",
        "main :: ()->int { wide:u16=256; return cast(u8) wide; }",
        "main :: ()->int { n:s8 = -1; return cast(int) cast(u64) n; }",
        "main :: ()->int { n:u64 = 18446744073709551615; return cast(int) n; }",
        "main :: ()->int { n:u8 = 128; return cast(s8) n; }",
    ] {
        let status = execute_status(source);
        assert!(!status.success(), "{source}");
        assert!(
            status.signal().is_some(),
            "expected LLVM trap, got {status}: {source}"
        );
    }
}

#[test]
fn typed_ranges_and_conditional_values_keep_integer_widths() {
    assert_eq!(
        execute(
            "main :: ()->int { count:=0; for i: cast(u8)254..cast(u8)255 { count+=1; } for < j: cast(s8)-128..cast(s8)-126 { count+=1; } return count; }"
        ),
        5
    );
    assert_eq!(
        execute(
            "main :: ()->int { a:u8=42; b:u16=99; result:=ifx true then a else b; return result; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { a:u64=42; result:=ifx true then a else 0; return cast(int) result; }"
        ),
        42
    );
}

#[test]
fn conditional_integer_literals_take_their_required_storage_type() {
    assert_eq!(
        execute(
            "pick :: (flag:bool)->u8 { return ifx flag then 255 else 0; } main :: ()->int { small:u8 = ifx false then 99 else 42; return small + cast(int) pick(false); }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { large:u64 = ifx false then 18446744073709551615 else 42; return cast(int) large; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { value:=cast,no_check(u8) (ifx true then 298 else 256); return value; }"
        ),
        42
    );
}

#[test]
fn named_arguments_keep_source_evaluation_order() {
    assert_eq!(
        execute(
            "counter: int; next :: () -> int { counter += 1; return counter; } pair :: (a: int, b: int) -> int { return a * 10 + b; } main :: () -> int { return pair(b = next(), a = next()); }"
        ),
        21
    );
}
#[test]
fn parameter_defaults_and_named_required_suffix() {
    assert_eq!(
        execute(
            "DEFAULT :: 3; combine :: (a: u8 = DEFAULT, b := 4, c: int, flag := true) -> int { if flag return cast(int) a + b + c; return 0; } main :: () -> int { return combine(c = 5); }"
        ),
        12
    );
}
#[test]
fn defaults_and_arguments_validate_even_unused_procedures() {
    for source in [
        "f :: (a: u8 = 256) {} main :: () {}",
        "f :: (a: int, a: int) {} main :: () {}",
        "f :: (a: int) {} main :: () { f(a = 1, a = 2); }",
        "f :: (a: int) {} main :: () { f(1, a = 2); }",
        "f :: (a: int) {} main :: () { f(z = 1); }",
        "f :: (a: int) {} main :: () { f(); }",
        "f :: (a: int) {} main :: () { f(a = 1, 2); }",
        "f :: (a: u8) {} main :: () { f(a = 256); }",
        "f :: (a: bool) {} main :: () { f(a = 1); }",
        "v: int; f :: (a: int = v) {} main :: () {}",
    ] {
        let module = jai_syntax::parse(source).unwrap();
        assert!(jai_sema::resolve(&module).is_err(), "accepted {source}");
    }
}

#[test]
fn defaults_accept_positional_prefix_and_named_overrides() {
    assert_eq!(
        execute(
            "choose :: (a := 1, b: u8 = 2, c := 3) -> int { return a * 100 + cast(int) b * 10 + c; } main :: () -> int { if choose() != 123 return 1; if choose(2) != 223 return 2; if choose(1, c = 4) != 124 return 3; if choose(c = 5, b = 3, a = 2) != 235 return 4; return 42; }"
        ),
        42
    );
}

#[test]
fn cases_evaluate_subject_once_and_keep_arm_scopes() {
    assert_eq!(
        execute(
            "n := 0; subject :: ()->int { n += 1; return 2; } main :: ()->int { result := 0; if subject() == { case 1; x := 7; result = x; case 2; x := 20; result = x; case; result = 99; } return result + n; }"
        ),
        21
    );
}
#[test]
fn cases_through_runs_cleanup_before_next_arm() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; if 1 == { case 1; defer n += 3; n += 2; #through; case 9; n *= 4; case; n = 99; } return n; }"
        ),
        20
    );
}
#[test]
fn cases_preserve_loop_transfers_and_returns() {
    assert_eq!(
        execute(
            "pick :: (n:int)->int { if n == { case 2; return 42; case; return 9; } } main :: ()->int { n := 0; sum := 0; while n < 5 { n += 1; if n == { case 1; continue; case 3; break; case; sum += pick(n); } } return sum; }"
        ),
        42
    );
}
#[test]
fn bool_complete_cases_and_first_matching_not_equal() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; if #complete true == { case true; n = 10; case false; n = 20; } if 2 != { case 1; n += 3; case 2; n += 99; case; n += 100; } return n; }"
        ),
        13
    );
}
#[test]
fn cases_named_transfers_run_deferred_cleanup() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; for outer: 1..3 { defer n += 10; for inner: 1..3 { defer n += 1; if inner == { case 2; continue outer; case; n += 2; } } } return n; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; for outer: 1..3 { defer n += 10; for inner: 1..3 { defer n += 1; if inner == { case 2; break outer; case; n += 2; } } } return n; }"
        ),
        14
    );
}
#[test]
fn cases_narrow_unsigned_labels_and_no_match() {
    assert_eq!(
        execute(
            "main :: ()->int { x:u8 = 255; n := 0; if x == { case 1; n = 100; } if x == { case 255; n += 7; case; n += 100; } return n; }"
        ),
        7
    );
}

#[test]
fn complete_bool_cases_terminate_without_default() {
    assert_eq!(
        execute(
            "pick :: (b:bool)->int { if #complete b == { case true; return 31; case false; return 42; } } main :: ()->int { return pick(false); }"
        ),
        42
    );
    assert_eq!(
        execute(
            "pick :: (b:bool)->int { if #complete b == { case true; #through; case false; return 27; } } main :: ()->int { return pick(true); }"
        ),
        27
    );
    assert_eq!(
        execute(
            "pick :: (b:bool)->bool { if #complete b == { case true; return ifx b then false else true; case false; return ifx b then true else false; } } main :: ()->int { if pick(true) return 99; return 17; }"
        ),
        17
    );
}
#[test]
fn empty_case_lists_and_default_only_cases_preserve_subject_effects() {
    assert_eq!(
        execute(
            "n := 0; subject :: ()->int { n += 1; return 1; } main :: ()->int { if subject() == { } if subject() == { case; n += 5; } if subject() == { case 1; case; n += 100; } return n; }"
        ),
        8
    );
    assert_eq!(
        execute(
            "pick :: ()->int { if 1 == { case; return 23; } } main :: ()->int { return pick(); }"
        ),
        23
    );
}
#[test]
fn complete_bool_cases_with_falling_arms_keep_a_join() {
    assert_eq!(
        execute(
            "main :: ()->int { n := 0; if #complete false == { case true; n = ifx true then 19 else 99; case false; n = ifx false then 99 else 37; } return n; }"
        ),
        37
    );
}
