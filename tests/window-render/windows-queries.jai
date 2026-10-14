#import "Basic";
W :: #import "Window_Creation";

main :: () {
    dimensions: #type (W.Window_Type, bool) -> s32, s32, s32, s32, bool = W.get_dimensions;
    pointer: #type (W.Window_Type, bool) -> int, int, bool = W.get_mouse_pointer_position;
    global_pointer: #type (bool) -> int, int, bool = W.get_mouse_pointer_position;

    assert(dimensions != null && pointer != null && global_pointer != null);
    print("ok\n");
}
