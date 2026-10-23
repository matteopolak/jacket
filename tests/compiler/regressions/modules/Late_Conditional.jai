#import "Basic";

// Chosen by `LIMIT`, which the program text added before this file defines through an `#insert`.
#if LIMIT < 8 Narrow :: u8;
else #if LIMIT < 16 Narrow :: u16;
else Narrow :: u64;

generate_limit :: () -> string {
    return "LIMIT :: 5;";
}

generate_wide :: () -> string {
    return tprint("Wide :: %;", Narrow);
}
