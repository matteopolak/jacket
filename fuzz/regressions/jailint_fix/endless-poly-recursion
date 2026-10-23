// Found by fuzzing: every call instantiates a new `f`, forever.
f :: (x: $T) {
    y: *T;
    f(y);
}

main :: () {
    f(1);
}
