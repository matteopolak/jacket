F :: #type (*[..] int);

grow :: (xs: *[..] int) {} @GrowsArray
other_grow :: (xs: *[..] int) {} @GrowsArray
quiet :: (xs: *[..] int) {}

sink :: (p: *int) -> int {
    return p.*;
}

lib :: #library "c";
foreign_touch :: (xs: *[..] int) #foreign lib "lifetime_fixture_unavailable";
foreign_read :: (xs: *[..] int) #foreign lib "lifetime_fixture_read"; @NoLifetimeEffects

closed :: (flag: bool) {
    xs: [..] int;
    f: F;

    if flag f = grow;
    else f = other_grow;

    p := *xs[0];

    f(*xs);
    sink(p);
}

mixed :: (flag: bool) {
    xs: [..] int;
    f: F;

    if flag f = grow;
    else f = quiet;

    p := *xs[0];

    f(*xs);
    sink(p);
}

conditional :: (flag: bool) {
    xs: [..] int;
    p := *xs[0];
    if flag grow(*xs);
    sink(p);
}

foreign_unknown :: () {
    xs: [..] int;
    p := *xs[0];
    foreign_touch(*xs);
    sink(p);
}

parameter_unknown :: (f: F) {
    xs: [..] int;
    p := *xs[0];
    f(*xs);
    sink(p);
}

factory :: () -> F {
    return grow;
}

from_factory :: () {
    xs: [..] int;
    f := factory();
    p := *xs[0];

    f(*xs);
    sink(p);
}

identity :: (f: F) -> F {
    return f;
}

from_identity :: () {
    xs: [..] int;
    f := identity(grow);
    p := *xs[0];

    f(*xs);
    sink(p);
}

global_function: F = grow;

global_unknown :: () {
    xs: [..] int;
    p := *xs[0];
    global_function(*xs);
    sink(p);
}

foreign_contract :: () {
    xs: [..] int;
    p := *xs[0];
    foreign_read(*xs);
    sink(p);
}

compile_time_target :: () -> F {
    return grow;
}

COMPILED :: #run compile_time_target();

persisted_procedure :: () {
    xs: [..] int;
    p := *xs[0];
    COMPILED(*xs);
    sink(p);
}
