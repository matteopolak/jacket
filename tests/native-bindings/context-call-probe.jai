// General overload-probing regression; no library-specific names or paths.
compute :: () -> s32 {
    return 7;
}

accept :: (number: s32) -> s32 {
    return number;
}

accept :: (text: string) -> s32 {
    return cast(s32) text.count;
}

libc :: #library,system "c";
abort :: () #foreign libc;

main :: () {
    enabled := true;
    if accept(ifx enabled then compute() else 0) != 7 abort();
    enabled = false;
    if accept(ifx enabled then 0 else compute()) != 7 abort();
}
