// A constant document can be decoded by `#run` and baked into the program.

#import "Basic";
#import "Extensions/Json";

Point :: struct {
    name: string;
    tags: [] string;
    n: int;
}

make_point :: () -> Point {
    point := from_json(Point, "{\"name\":\"hi\",\"tags\":[\"a\",\"b\"],\"n\":7}");

    return point;
}

baked :: #run make_point();

main :: () {
    assert(baked.name == "hi");
    assert(baked.n == 7);
    assert(baked.tags.count == 2 && baked.tags[1] == "b");
    print("ok\n");
}
