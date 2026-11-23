//! A foreign procedure is classified once and called many times: every kind of argument and
//! result must still come out right on the second call and the thousandth.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

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
        r#"
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
        "#,
    );
}
