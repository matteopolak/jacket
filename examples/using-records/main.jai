#import "Basic";

Inner :: struct {
    value: int = 3;
}

Middle :: struct {
    using inner: Inner;
}

Outer :: struct {
    using middle: Middle;
}

main :: () {
    original: Outer;
    copy := original;

    original.value = 17;
    copy.middle.inner.value = 5;

    exit(cast(s32) (original.value + copy.value));
}
