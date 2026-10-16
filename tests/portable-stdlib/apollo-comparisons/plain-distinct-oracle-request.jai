// Root-only official source/build probe. Its accepted/rejected status is unspecified.
Base :: struct {
    value: s64;
}

Plain :: #type Base;

operator < :: (a: Base, b: Base) -> bool {
    return a.value < b.value;
}

main :: () {
    a := Plain.{ 1 };
    b := Plain.{ 2 };
    result := a < b;
}
