// Original safety reduction: real native window, private resize storage first
// used under temp, no event pump/resize notification priming before allocation.
#import "Basic";
Input :: #import "Input";

main :: () {
    window := W.create_window(160, 120, "Input resize allocator reduction");

    assert(window != null);
    heap := context.allocator;

    context.allocator = temp;
    Input.add_resize_record(window, 123, 456);
    records := Input.get_window_resizes();

    assert(records.count == 1 && records[0].window == window);
    reset_temporary_storage();
    junk := talloc_string(64 * 1024);

    memset(junk.data, 0xa5, junk.count);
    assert(records[0].window == window && records[0].width == 123 && records[0].height == 456);
    context.allocator = heap;
    reset_temporary_storage();
    native_message(window, "close");
    native_message(window, "release");
}

#scope_file
#load "support/macos-fixtures.jai";
