// Input: original-API names (add_resize_record, window_minimized, WHEEL_DELTA).
#import "Basic";
#import "Window_Type";
Input :: #import "Input"(ENABLE_NATIVE_INPUT_ADAPTERS=false);

main :: () {
    assert(Input.WHEEL_DELTA == 120);
    assert(!Input.window_minimized);

    window: Window_Type;
    Input.add_resize_record(window, 640, 480);
    Input.add_resize_record(window, 800, 600);  // coalesced per window
    Input.update_window_events();
    resizes := Input.get_window_resizes();
    found := false;
    for resizes if it.width == 800 && it.height == 600 found = true;
    assert(found);
    print("ok\n");
}
