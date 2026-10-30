// An `#if` that compares an enum with an inferred member must not load the imported modules to
// look for an `==` overload: the `#if` picks the parameters of a later `#import "Basic"`.
// rules: modparam.14
#import "Math";

Build :: enum u8 {
    DEBUG;
    SHIPPING;
}

BUILD :: Build.DEBUG;

#if BUILD == .SHIPPING {
    #import "Basic"()(MEMORY_DEBUGGER = false);
} else {
    #import "Basic"()(MEMORY_DEBUGGER = true);
}

main :: () {
    assert(sin(0.0) == 0);
    print("ok\n");
}
