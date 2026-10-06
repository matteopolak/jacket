// Deliberately broken helper for tests/corpus/negative/rule-scope-3.jai: this file reads a
// #scope_file name of its sibling file, which must not compile.
#load "other.jai";

leak :: () -> int {
    return other_private;
}
