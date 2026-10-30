// rules: struct.23
// `Struct.field` reads the field's default value where a value is expected, through nested
// fields too, and still names the field's type where a type is wanted.
#import "Basic";

Color :: struct {
    r: float = 0.25;
    g: float = 0.5;
}

Inner :: struct {
    color: Color;
    size := 7;
    Entry :: struct {
        key: int;
    }
}

Outer :: struct {
    inner: Inner;
    count := 42;
    label := "hi";
    zero: int;
}

main :: () {
    count: int = Outer.count;

    assert(count == 42);
    size: s64 = Outer.inner.size;

    assert(size == 7);
    zero: int = Outer.zero;

    assert(zero == 0);
    label: string = Outer.label;

    assert(label == "hi");

    green: float = Outer.inner.color.g;

    assert(green == 0.5);
    color: Color;
    color = Outer.inner.color;
    assert(color.r == 0.25);

    entry: Outer.inner.Entry;

    assert(entry.key == 0);
}
