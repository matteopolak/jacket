// A compile-time read through an address no memory has faults; the error names the address
// the program used.
wild :: () -> int {
    p := cast(*int) cast(u32) 0x8000_0000;
    return p.*;
}

X :: #run wild();

main :: () {}
