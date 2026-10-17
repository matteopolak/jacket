#load "generic.jai";

Marker :: struct {
    value: int;
}

operator == :: (a: Marker, b: Marker) -> bool {
    return a.value == b.value;
}
