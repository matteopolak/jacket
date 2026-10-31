//! Windows with the Microsoft toolchain: the DLL C runtime.
use super::{
    CcSpelling, DataImport, Linker, Runtime, RuntimeDlls, RuntimeLib, Toolchain, windows_gnu,
};

/// `msvcrt` is the DLL runtime's import library; `oldnames` maps POSIX names (`write`) to the
/// underscored ones, and `legacy_stdio_definitions` keeps `printf` and friends linkable as
/// functions. `vcruntime` and `ucrt` come in through `msvcrt`'s own directives, which a link
/// that drops the defaults has to name again.
const LIBS: &[RuntimeLib] = &[
    RuntimeLib {
        name: "msvcrt",
        default: Some(CcSpelling::DefaultLib),
    },
    RuntimeLib {
        name: "vcruntime",
        default: None,
    },
    RuntimeLib {
        name: "ucrt",
        default: None,
    },
    RuntimeLib {
        name: "oldnames",
        default: Some(CcSpelling::Implicit),
    },
    RuntimeLib {
        name: "legacy_stdio_definitions",
        default: Some(CcSpelling::Lib),
    },
    RuntimeLib {
        name: "legacy_stdio_wide_specifiers",
        default: None,
    },
];

/// `-fms-runtime-lib` only steers what Clang compiles; its link step always adds the static
/// runtime, which leaves a `/MD` library's `__imp_` references unresolved.
pub(super) const RUNTIME: Runtime = Runtime {
    libs: LIBS,
    exclude: &["libcmt"],
    cl_flag: Some("/MD"),
    cc_flag: Some("-fms-runtime-lib=dll"),
    dlls: Some(RuntimeDlls {
        c: windows_gnu::MSVCRT_DLL,
        stdio: windows_gnu::UCRTBASE_DLL,
    }),
};

pub(super) static WINDOWS_MSVC: Toolchain = Toolchain {
    linker: Linker::Msvc,
    runtime: RUNTIME,
    runtime_names: windows_gnu::RUNTIME_NAMES,
    names_fold_case: true,
    implicit_libraries: &[],
    renames: double_math,
    runtime_data: DataImport::DllImport {
        only: &["ucrt", "ucrtbase"],
    },
    main_stack: Some(windows_gnu::MAIN_STACK),
    foreign_modules: None,
};

/// The `double` function the runtime offers for `symbol` when `symbol` is the `long double` form
/// (`name` + `l`) of a `<math.h>` function. `long double` is `double` on Windows, and the runtime
/// exports none of these: its headers make them inline calls of the `double` functions.
fn double_math(symbol: &str) -> Option<&str> {
    const MATH: &str = "acos acosh asin asinh atan atan2 atanh cbrt ceil copysign cos cosh erf \
        erfc exp exp2 expm1 fabs fdim floor fma fmax fmin fmod frexp hypot ilogb ldexp lgamma \
        llrint llround log log10 log1p log2 logb lrint lround modf nearbyint nextafter pow \
        remainder remquo rint round scalbln scalbn sin sinh sqrt tan tanh tgamma trunc";
    let stem = symbol.strip_suffix('l')?;
    MATH.split_whitespace()
        .any(|name| name == stem)
        .then_some(stem)
}
