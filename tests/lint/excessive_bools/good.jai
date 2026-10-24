#import "Basic";

Open_Flags :: enum_flags {
    READ;
    WRITE;
    CREATE;
    TRUNCATE;
}

open :: (path: string, flags: Open_Flags) -> int {
    return path.count;
}

// At the limit of three.
three :: (a: bool, b: bool, c: bool, n: int) -> int {
    return n;
}

// Parameters that are not `bool`.
counts :: (a := 1, b := 2, c := 3, d := 4, e := 5, f: [] bool) -> int {
    return a + b + c + d + e + f.count;
}

// A fixed C signature.
callback :: (a: bool, b: bool, c: bool, d: bool) -> bool #c_call {
    return a;
}

main :: () {
    flags: Open_Flags = .READ | .CREATE;
    print("% % %\n", open("a", flags), three(true, false, true, 1), counts(f = .[true, false]));
}
