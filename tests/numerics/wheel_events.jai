#import "Basic";
#import "Math";
G :: #import "GetRect";

main :: () {
    p: G.Draw_Procs;
    p.immediate_quad = quad;
    p.get_mouse_pointer_position = mouse;
    G.ui_init(procs = *p);
    window: type_of(G.Button_State.widget.window);

    G.ui_per_frame_update(window, 100, 100, 1);
    theme: G.Scrollable_Region_Theme;
    theme.mouse_wheel_increment = 0.01;
    value: float = 50;

    G.getrect_handle_event(.{ type = .MOUSE_WHEEL, window = window, key_code = .MOUSE_WHEEL_DOWN, wheel_delta = -240, typical_wheel_delta = 120 });
    scroll(*theme, *value);
    assert(value == 52, "signed wheel magnitude must reach the region");
    assert((G.ui_get_button_state(.MOUSE_WHEEL_DOWN) & .START) != 0);
    scroll(*theme, *value);
    assert(value == 52, "wheel input must be consumed once");
    G.ui_per_frame_update(window, 100, 100, 2);
    scroll(*theme, *value);
    assert(value == 52, "wheel deltas must expire at the next frame");
    G.getrect_handle_event(.{ type = .MOUSE_WHEEL, window = window, key_code = .MOUSE_WHEEL_UP, wheel_delta = 60, typical_wheel_delta = 120 });
    scroll(*theme, *value);
    assert(value == 51.5, "fractional wheel notches must be preserved");
    G.ui_per_frame_update(window, 100, 100, 3);
    G.getrect_handle_event(.{ type = .MOUSE_WHEEL, window = window, key_code = .MOUSE_WHEEL_DOWN, wheel_delta = -120, typical_wheel_delta = 120 });
    G.getrect_handle_event(.{ type = .MOUSE_WHEEL, window = window, key_code = .MOUSE_WHEEL_DOWN, wheel_delta = -240, typical_wheel_delta = 120 });
    scroll(*theme, *value);
    assert(value == 54.5, "multiple native wheel records must accumulate");
    G.ui_per_frame_update(window, 100, 100, 4);
    G.getrect_handle_event(.{ type = .KEYBOARD, window = window, key_code = .MOUSE_WHEEL_UP, key_pressed = 1 });
    scroll(*theme, *value);
    assert(value == 53.5, "keyboard wheel pulses remain supported");
    G.ui_per_frame_update(window, 100, 100, 5);
    G.getrect_handle_event(.{ type = .MOUSE_WHEEL, window = window, key_code = .MOUSE_WHEEL_UP, wheel_delta = 120, typical_wheel_delta = 120 });
    G.getrect_handle_event(.{ type = .MOUSE_WHEEL, window = window, key_code = .MOUSE_WHEEL_DOWN, wheel_delta = -120, typical_wheel_delta = 120 });
    scroll(*theme, *value);
    assert(value == 53.5, "opposite wheel events must cancel without pulse fallback");
    print("canonical wheel events ok\n");
}

scroll :: (theme: *G.Scrollable_Region_Theme, value: *float) {
    state, inside := G.begin_scrollable_region(.{ 0, 0, 100, 100 }, theme, identifier = 1);
    assert(inside.h > 0);
    G.end_scrollable_region(state, 100, -200, value);
}

quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {}

mouse :: (window: type_of(G.Button_State.widget.window), right_handed: bool) -> (x: int, y: int, success: bool) {
    return 50, 50, true;
}
