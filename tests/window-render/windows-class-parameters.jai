#import "Basic";
W :: #import "Windows";
Default :: #import "Window_Creation";
Single_Sample :: #import "Window_Creation"(DEFAULT_MSAA = 0);

main :: () {
    first := Default.create_window(200, 100, "Default module window");

    assert(first != null);
    defer assert(W.DestroyWindow(first) != 0);
    second := Single_Sample.create_window(240, 120, "Other module parameter window");

    assert(second != null && second != first);
    defer assert(W.DestroyWindow(second) != 0);
    _, _, width, height, ok := Single_Sample.get_dimensions(second, false);

    assert(ok && width == 240 && height == 120);
    print("ok\n");
}
