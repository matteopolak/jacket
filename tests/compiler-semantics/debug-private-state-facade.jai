#import "Basic";
D :: #import "Debug";
N :: #import "Debug"(USE_GRAPHICS = false);

#assert !#exists(D.Shared);
#assert !#exists(N.Shared);
#assert !#exists(D.State);
#assert type_info(D.Crash_Handler) == type_info(N.Crash_Handler);
#assert type_info(D.Exit_Code) == type_info(N.Exit_Code);

main :: () {
    assert(*D.crash_handler == *N.crash_handler);
    assert(*D.ignored_asserts == *N.ignored_asserts);
    print("ok\n");
}
