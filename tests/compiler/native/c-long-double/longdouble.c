// C `long double` across the C ABI (see crates/jaic-cli/tests/native.rs, c_long_double).
// On x86-64 System V it is x87 extended (passed in memory, returned in st0); on arm64 Linux
// binary128 (q registers); on Apple arm64 and MSVC plain double.

#include <stdarg.h>

#include "longdouble.h"

long double ld_third(void) { return 1.0L / 3; }
long double ld_add(long double a, long double b) { return a + b; }
long double ld_mix(int i, double d, long double x, long double y, int j) { return x * i + y * j + d; }
double ld_to_double(long double x) { return (double) x; }

// 1 when `x` holds bits that a double cannot (always 0 where long double is double).
int ld_beyond_double(long double x) { return x != (long double) (double) x; }

struct LdBox box_scale(struct LdBox b, long double k) {
    struct LdBox r = { b.v * k };
    return r;
}

struct LdTagged tagged_bump(struct LdTagged t) {
    struct LdTagged r = { (char) (t.tag + 1), t.v * 2 };
    return r;
}

struct LdPair pair_swap(struct LdPair p) {
    struct LdPair r = { p.b, p.a };
    return r;
}

// More floating-point arguments than there are registers for them.
long double ld_spill(double d0, double d1, double d2, double d3, double d4, double d5, double d6,
                     double d7, double d8, long double x, int k, long double y) {
    return d0 + d1 + d2 + d3 + d4 + d5 + d6 + d7 + d8 + x * k + y;
}

long double ld_apply(long double (*f)(long double, long double), long double a, long double b) {
    return f(a, b);
}

struct LdBox box_apply(struct LdBox (*f)(struct LdBox, int), struct LdBox b) {
    return f(b, 3);
}

// Variadic long doubles (not declared in longdouble.h: the bindings generator test reads that).
long double ld_vsum(int n, ...) {
    va_list ap;
    va_start(ap, n);
    long double sum = 0;
    for (int k = 0; k < n; k++) sum += va_arg(ap, long double);
    va_end(ap);
    return sum;
}

long double ld_vmix(int n, ...) {
    va_list ap;
    va_start(ap, n);
    long double sum = 0;
    for (int k = 0; k < n; k++) {
        long double x = va_arg(ap, long double);
        int i = va_arg(ap, int);
        double d = va_arg(ap, double);
        sum += x * i + d;
    }
    va_end(ap);
    return sum;
}
