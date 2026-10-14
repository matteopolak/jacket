#import "Basic";
S :: #import "Soa";

Point :: struct {
    x: s32;
    y: float64;
    label: string;
    MARKER :: 17;
}

With_Overlay :: struct {
    first: s32;
    #overlay(first) alias: s32;
    tail: u16;
}

change_first :: (array: *S.Soa(Point, 4)) {
    for *array {
        it.x = 88;
        return;
    }
}

main :: () {
    points: S.Soa(Point, 4);

    #assert type_of(points.x) == [4] s32;
    #assert type_of(points.y) == [4] float64;
    #assert type_of(points.label) == [4] string;
    for i: 0..3 points[i] = Point.{ cast(s32) i, cast(float64) i + 0.5, "point" };

    item := points[2];

    assert(item.x == 2 && item.y == 2.5 && item.label == "point");
    count: s64;

    for points {
        assert(it.x == it_index && it.y == cast(float64) it_index + 0.5);
        count += 1;
    }

    assert(count == 4);

    for *points {
        it.x += 10;
        if it_index == 1 continue;
        it.y += 1;
        if it_index == 2 break;
    }

    assert(points[0].x == 10 && points[0].y == 1.5);
    assert(points[1].x == 11 && points[1].y == 1.5);
    assert(points[2].x == 12 && points[2].y == 3.5);
    assert(points[3].x == 3 && points[3].y == 3.5);
    change_first(*points);
    assert(points[0].x == 88);
    assert(points[1].x == 11 && points[2].x == 12 && points[3].x == 3);
    overlay: S.Soa(With_Overlay, 2);

    assert(type_info(type_of(overlay)).members.count == 2);
    overlay[0] = With_Overlay.{ first = 31, tail = 17 };
    read := overlay[0];

    assert(read.alias == 31 && read.tail == 17);
    empty: S.Soa(Point, 0);
    for empty assert(false, "zero-extent Soa must not iterate");

    S.example();
    print("ok\n");
}
