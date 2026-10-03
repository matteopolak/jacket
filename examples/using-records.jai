Inner :: struct { value: int = 3; }
Middle :: struct { using inner: Inner; }
Outer :: struct { using middle: Middle; }

main :: () -> int {
    original: Outer;
    copy := original;
    original.value = 17;
    copy.middle.inner.value = 5;
    return original.value + copy.value;
}
