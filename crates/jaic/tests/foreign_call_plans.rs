//! A foreign procedure is classified once and called many times: every kind of argument and
//! result must still come out right on the second call and the thousandth.
use std::rc::Rc;

use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};

fn check(source: &str) {
    let mut options = Options::host();

    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));

    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.src.sources));
    }
}

#[test]
fn repeated_scalar_calls_use_each_calls_own_arguments() {
    check(
        r#"
        labs :: (x: s64) -> s64 #foreign;
        abs :: (x: s32) -> s32 #foreign;
        sqrt :: (x: float64) -> float64 #foreign;
        sqrtf :: (x: float32) -> float32 #foreign;
        pow :: (x: float64, y: float64) -> float64 #foreign;
        strlen :: (s: *u8) -> u64 #foreign;
        memcmp :: (a: *void, b: *void, n: u64) -> s32 #foreign;

        run :: () -> s64 {
            sum: s64;
            for 1..1000 {
                sum += labs(-it);
                sum += abs(cast(s32) -it);
            }
            if sqrt(16.0) != 4.0 return -1;
            if sqrtf(9.0) != 3.0 return -2;
            if pow(2.0, 10.0) != 1024.0 return -3;
            text := "hello";
            if strlen(text.data) != 5 return -4;
            a := "abc"; b := "abd";
            if memcmp(a.data, b.data, 2) != 0 return -5;
            if memcmp(a.data, b.data, 3) >= 0 return -6;
            return sum;
        }
        #assert run() == 1001000;
        main :: () {}
        "#,
    );
}

#[test]
fn aggregate_results_come_back_through_the_plan() {
    check(
        r"
        Div_T :: struct { quot: s32; rem: s32; }
        Ldiv_T :: struct { quot: s64; rem: s64; }
        div :: (a: s32, b: s32) -> Div_T #foreign;
        ldiv :: (a: s64, b: s64) -> Ldiv_T #foreign;

        run :: () -> s64 {
            total: s64;
            for 1..100 {
                d := div(cast(s32) (it * 7), 5);
                l := ldiv(it * 1000, 7);
                total += d.quot + d.rem + l.quot + l.rem;
            }
            return total;
        }
        #assert run() == 728920;
        main :: () {}
        ",
    );
}

#[test]
fn mixed_integer_and_float_registers_keep_their_own_order() {
    check(
        r#"
        fma :: (x: float64, y: float64, z: float64) -> float64 #foreign;
        fmaf :: (x: float32, y: float32, z: float32) -> float32 #foreign;
        ldexp :: (x: float64, e: s32) -> float64 #foreign;
        copysignf :: (x: float32, y: float32) -> float32 #foreign;
        strtol :: (s: *u8, end: **u8, base: s32) -> s64 #foreign;
        abs :: (x: s32) -> s32 #foreign;
        toupper :: (c: s32) -> s32 #foreign;
        isdigit :: (c: s32) -> s32 #foreign;

        run :: () -> s64 {
            for 1..200 {
                if fma(2.0, 3.0, cast(float64) it) != 6.0 + cast(float64) it return -1;
                if fmaf(2.0, 4.0, 1.0) != 9.0 return -2;
                if ldexp(1.5, 4) != 24.0 return -3;
                if copysignf(3.0, -1.0) != -3.0 return -4;
            }
            text := "ff";
            if strtol(text.data, null, 16) != 255 return -5;
            if abs(-7) != 7 return -6;
            if toupper(97) != 65 return -7;
            if isdigit(55) == 0 return -8;
            return 0;
        }
        #assert run() == 0;
        main :: () {}
        "#,
    );
}

#[test]
fn callbacks_handed_to_c_keep_working_across_many_calls() {
    check(
        r"
        qsort :: (base: *void, count: u64, size: u64, compare: (*void, *void) -> s32 #c_call) #foreign;
        abs :: (x: s32) -> s32 #foreign;

        by_value :: (a: *void, b: *void) -> s32 #c_call {
            x := (cast(*s32) a).*;
            y := (cast(*s32) b).*;
            if x < y return -1;
            if x > y return 1;
            return 0;
        }

        run :: () -> s64 {
            for 1..50 {
                values := s32.[5, 3, 9, 1, 7, 2];
                qsort(values.data, 6, 4, by_value);
                if values[0] != 1 || values[5] != 9 || values[2] != 3 return -1;
                if abs(cast(s32) -it) != cast(s32) it return -2;
            }
            return 0;
        }
        #assert run() == 0;
        main :: () {}
        ",
    );
}
