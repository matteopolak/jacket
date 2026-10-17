#import "Basic";
#import "Math";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

quad_count: int;
deactivated: int;

main :: () {
    assert(G.is_inside(10, 20, .{ 0, 0, 10, 20 }));
    assert(!G.is_inside(0, 0, .{ 0, 0, -1, 20 }));
    left, remaining := G.cut_left(.{ 0, 0, 100, 50 }, 20, 4);

    assert(left.w == 18 && remaining.x == 22 && remaining.w == 78);
    top, remtop := G.cut_top(.{ 0, 0, 100, 50 }, 10, 4);
    ltop, lrem := L.cut_top(.{ 0, 0, 100, 50 }, 10, 4);

    assert(top.y == 42 && ltop.y == 0 && remtop.h == 38 && lrem.y == 12);
    assert(G.combine_hashes(0xffff_ffff_ffff_ffff, 1) == 0x61c8864680b583ec);
    assert(type_info(G.Rect).name == "Rect");
    procs: G.Draw_Procs;
    procs.immediate_quad = quad;
    procs.get_mouse_pointer_position = mouse;
    G.ui_init(procs = *procs);
    window: type_of(G.Button_State.widget.window);

    G.ui_per_frame_update(window, 100, 100, 1);
    pressed, b, released := G.button(.{ 0, 0, 30, 30 }, "", identifier = 1);

    assert(!pressed && !released && quad_count > 0);
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = 1, window = window });
    pressed, b, released = G.button(.{ 0, 0, 30, 30 }, "", identifier = 1);
    assert(pressed && b.pressed);
    b.deactivate_proc = deactivate;
    G.active_widget_add(*b.widget);
    G.ui_per_frame_update(window, 100, 100, 2);
    G.ui_per_frame_update(window, 100, 100, 3);
    assert(deactivated == 1);
    print("ui contracts ok\n");
}

quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {
    quad_count += 1;
}

mouse :: (window: type_of(G.Button_State.widget.window), right_handed: bool) -> (x: int, y: int, success: bool) {
    return 10, 10, true;
}

deactivate :: (w: *type_of(G.Button_State.widget)) {
    deactivated += 1;
}
