#assert OS == .WINDOWS || OS == .MACOS, "Native UI cursor lifecycle requires the Windows/macOS provider; caller callbacks remain supported";

#import "Basic";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

Widget :: type_of(G.Button_State.widget);
callback_count: int;
expected_widget: *Widget;

main :: () {
    assert(type_of(G.pointers[0]) == type_of(L.pointers[0]), "cursor pointer types must share identity across variants");
    G.init_pointers();
    for pointer: G.pointers assert(pointer != null);

    // Reinitialization exercises release of the first owned macOS cursor set.
    G.init_pointers();
    for pointer: G.pointers assert(pointer != null);

    L.init_pointers();
    L.init_pointers();
    for pointer: L.pointers assert(pointer != null);

    widget: Widget;

    expected_widget = *widget;
    G.draw_procs.set_pointer_image = caller_cursor;
    G.getrect_set_pointer_image(.LOCKED, *widget);
    assert(callback_count == 1);
    print("native UI cursor lifecycle ok\n");
}

caller_cursor :: (image: G.Pointer_Image, widget: *Widget) {
    assert(image == .LOCKED && widget == expected_widget);
    callback_count += 1;
    // Deliberately no system set: this test never changes the visible cursor.
}
