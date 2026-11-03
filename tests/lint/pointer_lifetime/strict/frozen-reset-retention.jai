#import "Basic";

build :: () -> string {
    text := tprint("temporary");
    reset_temporary_storage();
    return text;
}

FROZEN :: #run build();
main :: () { print("% %", FROZEN, FROZEN_DEFERRED); }

build_deferred :: () -> string {
    defer reset_temporary_storage();
    return tprint("temporary with deferred reset");
}

FROZEN_DEFERRED :: #run build_deferred();
