// Helper module for tests/stdlib/rules-scoping.jai (docs/language/scoping.md): visibility
// directives across two files and conditional loads.
#load "second.jai";

#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS || OS == .WASM {
    #load "context_part.jai";

    LOADED_PART :: true;
} else {
    LOADED_PART :: false;
}

#if false #load "never_loaded.jai";

exported_answer :: 42;

first_total :: () -> int {
    return module_only_value + second_reads_module_value() + file_only_value;
}

first_root :: () -> float {
    return sqrt(81.0);
}

#scope_module

module_only_value :: 100;

#scope_file

#import "Math";

file_only_value :: 3;
