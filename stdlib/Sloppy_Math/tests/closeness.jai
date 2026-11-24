// is_approximately_zero and values_are_close use an inclusive absolute bound, and any NaN or matching infinity is not close.
#import "Basic";
M :: #import "Math";
S :: #import "Sloppy_Math";

main :: () {
    zero: float = 0;
    nan := zero / zero;
    inf: float = 1;
    inf /= zero;

    // Approximate zero, with the default bound 0.0001 and with explicit bounds.
    assert(S.is_approximately_zero(0));
    assert(S.is_approximately_zero(0.00005));
    assert(S.is_approximately_zero(-0.00005));
    assert(!S.is_approximately_zero(0.01));
    assert(S.is_approximately_zero(0.01, 0.1));
    assert(S.is_approximately_zero(0.25, 0.25)); // the bound is inclusive
    assert(S.is_approximately_zero(-0.25, 0.25));
    assert(!S.is_approximately_zero(0.5, 0.25));
    assert(!S.is_approximately_zero(nan));
    assert(!S.is_approximately_zero(inf));

    // Close values, absolute difference, inclusive bound.
    assert(S.values_are_close(1.0, 1.00005));
    assert(!S.values_are_close(1.0, 1.01));
    assert(S.values_are_close(1.0, 1.01, 0.1));
    assert(S.values_are_close(1.0, 1.5, 0.5)); // difference exactly equal to the bound
    assert(!S.values_are_close(1.0, 1.625, 0.5));
    assert(S.values_are_close(-3.0, -2.5, 0.5));
    assert(!S.values_are_close(nan, nan));
    assert(!S.values_are_close(nan, 0.0));
    assert(!S.values_are_close(inf, inf)); // the difference of two equal infinities is NaN
    assert(!S.values_are_close(inf, -inf));

    // Vector2: every component is compared with the same bound.
    assert(S.values_are_close(M.Vector2.{ 1.0, 2.0 }, M.Vector2.{ 1.0, 2.00005 }));
    assert(!S.values_are_close(M.Vector2.{ 1.0, 2.0 }, M.Vector2.{ 1.0, 3.0 }));
    assert(!S.values_are_close(M.Vector2.{ nan, 0.0 }, M.Vector2.{ nan, 0.0 }));

    // Vector3
    assert(S.values_are_close(M.Vector3.{ 1.0, 2.0, 3.0 }, M.Vector3.{ 1.0, 2.0, 3.00005 }));
    assert(!S.values_are_close(M.Vector3.{ 1.0, 2.0, 3.0 }, M.Vector3.{ 1.0, 2.0, 3.5 }));
    assert(S.values_are_close(M.Vector3.{ 1.0, 2.0, 3.0 }, M.Vector3.{ 1.5, 2.5, 3.5 }, 0.5));
    assert(!S.values_are_close(M.Vector3.{ 1.0, 2.0, 3.0 }, M.Vector3.{ 1.5, 2.5, 3.5 }, 0.4));

    print("ok\n");
}
