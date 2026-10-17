// Through a raw pointer, `p[i] = v` writes memory unless an operator []= takes that struct.
#import "Basic";

Array :: struct(T: Type) {
    data: *T;
    count: int;
}

operator []= :: (a: *Array($T), i: int, v: T) {
    a.data[i] = v;
}

Cursor :: struct {
    pos: int;
}

main :: () {
    cs := Cursor.[.{ 1 }, .{ 2 }, .{ 3 }];
    a: Array(Cursor);
    a.data = cs.data;
    a.count = 3;
    a.data[0] = a.data[2];
    a[1] = Cursor.{ 9 };
    assert(cs[0].pos == 3 && cs[1].pos == 9);
}
