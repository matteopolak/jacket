// test: only(.MACOS) serial
#load "../module.jai";
WC :: #import "Window_Creation";

main :: () {
    begin_desktop_events();
    window := WC.create_window(320, 200, "IME extension test", 50, 100);

    assert(window != null);
    defer close_window(window);
    view := result_message(window, "contentView", *void);
    original := object_get_class(view);

    set_text_input_rect(window, true, 20, 40, 16);
    assert(object_get_class(view) != original);
    text_set_marked(view, null, native_string("かな"), .{ 2, 0 }, .{});
    assert(composition_events.count == 1 && composition_events[0].window == window);
    actual: Text_Range;
    rect := text_first_rect(view, null, .{}, *actual);
    screen := content_screen_rect(window);

    assert(rect.origin.x == screen.origin.x + 20);
    assert(rect.origin.y == screen.origin.y + 200 - 40 - 16);
    assert(rect.size.height == 16);
    detach_text_input(window);
    assert(object_get_class(view) == original);
    assert(composition_events.count == 0);
}
