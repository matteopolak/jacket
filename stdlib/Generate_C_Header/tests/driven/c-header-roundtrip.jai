Env_Output :: #import "Extensions/Environment";
Compiler_Output :: #import "Compiler";
#import "Basic";

Point :: struct {
    x: s32;
    y: float64;
    samples: [3] u16;
    next: *Point;
    callback: (value: s32) -> s64 #c_call;
}

point_sum :: (point: *Point) -> s64 #foreign native;
#run Compiler_Output.compiler_add_library_search_directory(Env_Output.output_directory());
native :: #library "developer_tools_point";

main :: () {
    point: Point;
    point.x = 13;
    point.y = 4;
    point.samples[1] = 5;
    assert(point_sum(*point) == 29);
    assert(point.samples[1] == 12);
    print("ok\n");
}
