f :: () -> int #expand { return 1; }
main :: () { x := inline f(); y := no_inline f(); }
