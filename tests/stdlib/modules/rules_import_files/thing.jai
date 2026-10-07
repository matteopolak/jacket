// Imported with `#import,file` by tests/stdlib/rules-modules-and-imports.jai; its own
// `#import,file` resolves next to this file.
Helper :: #import,file "helper.jai";

thing_value :: () -> int {
    return Helper.helper_value * 2;
}
