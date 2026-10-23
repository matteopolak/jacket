// A constant `#ifx` of untyped numbers takes the unsigned type it is assigned to (#35).
#import "Basic";

main :: () {
    DISPLAY_SYNC :: true;
    sync_interval: u32 = #ifx DISPLAY_SYNC 1 else 0;

    assert(sync_interval == 1);
    small: u8 = #ifx !DISPLAY_SYNC then 1 else 200;

    assert(small == 200);
}
