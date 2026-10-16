// Cursor callbacks work without allocating or changing any system cursor.
#import "Basic";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

Right_Widget :: type_of(G.Button_State.widget);
Left_Widget :: type_of(L.Button_State.widget);
expected_right: *Right_Widget;
expected_left: *Left_Widget;
right_calls: int;
left_calls: int;

main :: () {
    right: Right_Widget;
    left: Left_Widget;

    expected_right = *right;
    expected_left = *left;
    G.draw_procs.set_pointer_image = right_cursor;
    L.draw_procs.set_pointer_image = left_cursor;
    G.getrect_set_pointer_image(.LOCKED, *right);
    L.getrect_set_pointer_image(.DRAGGING_FREE, *left);
    assert(right_calls == 1 && left_calls == 1);
    G.getrect_set_pointer_image(.LOCKED, *right);
    assert(right_calls == 2 && left_calls == 1);
    print("caller cursor callbacks ok\n");
}

right_cursor :: (image: G.Pointer_Image, widget: *Right_Widget) {
    assert(image == .LOCKED && widget == expected_right);
    right_calls += 1;
}

left_cursor :: (image: L.Pointer_Image, widget: *Left_Widget) {
    assert(image == .DRAGGING_FREE && widget == expected_left);
    left_calls += 1;
}
