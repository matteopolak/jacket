// operator *[] taking the struct by value serves `*x[i]`, `x[i] = v` and `x[i]`.
#import "Basic";
Array :: struct(T: Type) {
    data: *T;
    count: int;
}
operator *[] :: inline (array: Array($T), index: int) -> *T {
    return *array.data[index];
}
main :: () {
    v := u8.[1, 2, 3];
    text: Array(u8);
    text.data = v.data;
    text.count = 3;
    p := *text[1];
    p.* = 7;
    text[2] = 9;
    assert(v[1] == 7 && v[2] == 9 && text[0] == 1);
}
