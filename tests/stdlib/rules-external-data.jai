// rules: extdata.1 extdata.2 extdata.3 extdata.4
// `#elsewhere` variables: storage provided by a foreign library (or, for `__runtime_info`, by
// the compiler), named after the variable unless a symbol string follows the library.
#import "Basic";
#import "Compiler";

#if OS == .MACOS {
    c_runtime :: #system_library "libSystem";
    // extdata.2: no symbol string, so the variable's own name is the symbol.
    __stdoutp: *void #elsewhere c_runtime;
    // extdata.3: a string after the library names a different symbol.
    renamed_out: *void #elsewhere c_runtime "__stdoutp";

    default_named :: () -> *void {
        return __stdoutp;
    }
} else #if OS == .WINDOWS {
    // The C runtime DLL exports few variables; `_HUGE` is the double infinity. A DLL's variables
    // are reached through its `__imp_` pointers, which the build has to follow.
    c_runtime :: #system_library "libc";
    _HUGE: float64 #elsewhere c_runtime;
    renamed_huge: float64 #elsewhere c_runtime "_HUGE";
} else #if OS != .WASM {
    // The browser engine has no C library whose variables it could reach.
    c_runtime :: #system_library "libc";
    stdout: *void #elsewhere c_runtime;
    renamed_out: *void #elsewhere c_runtime "stdout";

    default_named :: () -> *void {
        return stdout;
    }
}

// extdata.4: with no library, `__runtime_info` is the compiler's own data.
__runtime_info: Runtime_Info #elsewhere;

main :: () {
    #if OS == .WINDOWS {
        // extdata.1: the variable has no initializer; its value is the library's.
        assert(_HUGE > 1e308, "default-named foreign variable is not infinity");
        assert(renamed_huge == _HUGE, "renamed foreign variable differs");
        assert(*renamed_huge == *_HUGE, "both names must reach the same symbol");
    } else #if OS != .WASM {
        // extdata.1: the variable has no initializer; its value is the library's.
        assert(default_named() != null, "default-named foreign variable is null");
        assert(renamed_out != null, "renamed foreign variable is null");
        assert(renamed_out == default_named(), "both names must reach the same symbol");
    }

    info := __runtime_info;
    assert(info.type_table.count > 0, "runtime info has no type table");
    print("ok\n");
}
