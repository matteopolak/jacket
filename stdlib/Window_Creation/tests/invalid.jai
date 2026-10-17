#import "Basic";
W :: #import "Window_Creation";

main :: () {
    x, y, width, height, ok := W.get_dimensions(0, false);

    assert(!ok && x == 0 && y == 0 && width == 0 && height == 0);
    pointer_x, pointer_y, pointer_ok := W.get_mouse_pointer_position(0, false);

    assert(!pointer_ok && pointer_x == 0 && pointer_y == 0);
    saved: W.Saved_Window_Info;
    success, fullscreen_width, fullscreen_height := W.toggle_fullscreen(0, true, *saved);

    assert(!success && fullscreen_width == 0 && fullscreen_height == 0);
    print("ok\n");
}
