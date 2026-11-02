F :: #type (*int);

alloc :: () -> *int {
    return null;
} @Allocates
release :: (p: *int) {} @FreesArgument
release2 :: (p: *int) {} @FreesArgument
quiet :: (p: *int) {}

sink :: (p: *int) -> int {
    return p.*;
}

use_handle :: (h: int) -> int {
    return h;
}

close_handle :: (owner: int, h: int) {} @Closes(1)
lib :: #library "c";

unknown :: (p: *int) #foreign lib "lifetime_fixture_unknown";
provider :: () -> *int #foreign lib "lifetime_fixture_provider"; @ValidUntilNextCall

refresh :: () {
    provider();
}

maybe_refresh :: (flag: bool) {
    if flag provider();
}

maybe_release :: (p: *int, flag: bool) {
    if flag release(p);
}

conditional :: (flag: bool) {
    p := alloc();
    if flag release(p);
    sink(p);
}

wrapper :: (flag: bool) {
    p := alloc();
    maybe_release(p, flag);
    sink(p);
}

closed :: (flag: bool) {
    p := alloc();
    f: F;

    if flag f = release;
    else f = release2;

    f(p);
    sink(p);
}

mixed :: (flag: bool) {
    p := alloc();
    f: F;

    if flag f = release;
    else f = quiet;

    f(p);
    sink(p);
}

foreign_unknown :: () {
    p := alloc();
    unknown(p);
    sink(p);
}

handle :: () {
    owner := 1;
    h := 7;

    close_handle(owner, h);
    use_handle(owner);
    use_handle(h);
}

expired :: () {
    p := provider();
    provider();
    sink(p);
}

expired_through_wrapper :: () {
    p := provider();
    refresh();
    sink(p);
}

maybe_expired :: (flag: bool) {
    p := provider();
    maybe_refresh(flag);
    sink(p);
}
