#import "Basic";

Tone :: enum u8 {
    LOW;
    MID;
    HIGH;
}

Grid_Point :: struct {
    column: s64;
    row: s64;
}

main :: () {
    points: [5] s64;

    for i: 0..4 {
        points[i] = cast(s64) i;
    }

    total := 0;

    for point: points {
        total += point;
    }

    p := Grid_Point.{ column = 8, row = 3 };
    tone := cast(Tone) 2;
    narrowed := cast(u8) p.column;

    assert(total == 10, "inclusive range fills all five slots");
    assert(p.column == 8 && p.row == 3, "named aggregate fields");
    assert(tone == .HIGH && narrowed == 8, "explicit enum and integer conversions");
    print("ok\n");
}
