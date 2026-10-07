// Long_Double: C's `long double` as a Jai type. A jaic extension, not official Jai: other Jai
// compilers have no such module, and it may change between jaic releases.
// See docs/stdlib/extensions.md.

// The C compiler's `long double` on the target. Where C makes it the same as `double`
// (Apple arm64, Windows with MSVC) this is float64 itself. Elsewhere it is a 16-byte float in
// the C format: x87 extended precision on x86-64 Linux, macOS and MinGW, IEEE binary128 on
// arm64 Linux and wasm32. Arithmetic, comparisons and casts run at that full precision;
// print shows the value converted to float64.
Long_Double :: #jaic_type long_double;

// Whether Long_Double is a type of its own, wider than float64.
LONG_DOUBLE_IS_WIDE :: Long_Double != float64;
