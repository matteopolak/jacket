#import "Basic";
#import "Math";
G :: #import "GetRect";

position := Vector2.{ 10, 110 };

#load "support/test-theme.jai";

main :: () {
    install_test_baselines();
    procs: G.Draw_Procs;
    procs.get_mouse_pointer_position = mouse;
    procs.immediate_quad = quad;
    procs.prepare_text = prepare;
    procs.draw_prepared_text = draw_text;
    G.ui_init(procs = *procs);
    window: type_of(G.Button_State.widget.window);
    loc := Source_Code_Location.{ "interaction", 1, 1 };
    choices: [2] string = .["zero", "one"];
    selected: s32;

    G.ui_per_frame_update(window, 800, 600, 1);
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = 1, window = window });
    changed, state := G.dropdown(.{ 0, 100, 100, 20 }, choices, *selected, loc = loc);

    assert(state.open);
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = 0, window = window });
    position = .{ 10, 70 };
    G.ui_per_frame_update(window, 800, 600, 2);
    changed, state = G.dropdown(.{ 0, 100, 100, 20 }, choices, *selected, loc = loc);
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = 1, window = window });
    G.draw_popups();
    assert(selected == 1);
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = 0, window = window });
    G.ui_per_frame_update(window, 800, 600, 3);
    changed, state = G.dropdown(.{ 0, 100, 100, 20 }, choices, *selected, loc = loc);
    assert(changed);
    position = .{ 190, 210 };
    G.ui_per_frame_update(window, 800, 600, 4);
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = 1, window = window });
    theme: G.Slider_Theme;
    theme = G.default_overall_theme.slider_theme;
    value: s32 = 4;
    slider_changed, entered := G.slider(.{ 0, 200, 200, 20 }, *value, 0, 10, 2, *theme, identifier = 2, loc = loc);

    assert(slider_changed && !entered && value == 6);
    print("ui interaction ok\n");
}

mouse :: (window: type_of(G.Button_State.widget.window), right_handed: bool) -> (x: int, y: int, success: bool) {
    return cast(int) position.x, cast(int) position.y, true;
}

quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {}

prepare :: (font: type_of(G.Label_Theme.font), text: string, effects: type_of(G.Label_Theme.effects) = 0) -> s64 {
    return text.count * 8;
}

draw_text :: (font: type_of(G.Label_Theme.font), x: s64, y: s64, color: Vector4, effects: type_of(G.Label_Theme.effects) = 0) {}
