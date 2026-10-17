#import "Basic";

Point :: struct {
    x: s32;
    y: float64;
    samples: [3] u16;
    next: *Point;
    callback: (value: s32) -> s64 #c_call;
}

point_sum :: (point: *Point) -> s64 #foreign native;
native :: #foreign_library "../../../../target/developer_tools_point";

main :: () {
    point: Point;
    point.x = 13;
    point.y = 4;
    point.samples[1] = 5;
    assert(point_sum(*point) == 29);
    assert(point.samples[1] == 12);
    print("ok\n");
}
