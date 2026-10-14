#import "Basic";

rename_members :: (names: [] string) {
    for * names {
        if it.* == "x" it.* = "x_axis";
        if it.* == "hidden" it.* = "";
    }
}

Coordinate :: struct {
    x: s64;
    y: s64;
    hidden: s64;
}

main :: () {
    point := Coordinate.{ x = 1, y = 2, hidden = 3 };
    {
        using,map(rename_members) point;
        x_axis = 9;
        y = 11;
    }

    assert(point.x == 9 && point.y == 11 && point.hidden == 3,
           "mapped names retain lvalue access and omitted fields remain addressable by qualification");
    print("ok\n");
}
