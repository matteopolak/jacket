// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target")
// Behaviours of widgets that the other tests do not reach: dropdown lists stay drawn while open and follow `z`,
// `append_text` never replaces a selection, repeated Tab cycles the completions of the typed text, and a keyframe
// added to a wrapping color animation gets the color the wrapped gradient has there.
#import "Basic";
#import "Math";
G :: #import "GetRect";
Colors :: #import "Color_Interpolation";

position := Vector2.{ 10, 510 };
drawn: [..] string;

#load "support/test-theme.jai";

main :: () {
    install_test_baselines();
    dropdown_lists();
    appended_text();
    tab_cycling();
    wrapped_keyframe();
    print("ui behaviours ok\n");
}

press :: (window: type_of(G.Button_State.widget.window), down: bool) {
    G.getrect_handle_event(.{ type = .KEYBOARD, key_code = .MOUSE_BUTTON_LEFT, key_pressed = ifx down then cast(u32) 1 else cast(u32) 0, window = window });
}

index_of :: (name: string) -> int {
    for drawn if it == name return it_index;

    return -1;
}

// An open list is drawn every frame, and the list with the higher `z` is drawn after (over) the other.
dropdown_lists :: () {
    procs: G.Draw_Procs;
    procs.get_mouse_pointer_position = mouse;
    procs.immediate_quad = quad;
    procs.prepare_text = prepare;
    procs.draw_prepared_text = draw_text;
    G.ui_init(procs = *procs);
    window: type_of(G.Button_State.widget.window);
    high := Source_Code_Location.{ "high", 1, 1 };
    low := Source_Code_Location.{ "low", 2, 1 };
    high_choices: [2] string = .["high0", "high1"];
    low_choices: [2] string = .["low0", "low1"];
    high_value: s32;
    low_value: s32;

    // The high list is opened first, the low one second.
    position = .{ 10, 510 };
    G.ui_per_frame_update(window, 800, 600, 1);
    press(window, true);
    G.dropdown(.{ 0, 500, 100, 20 }, high_choices, *high_value, identifier = 1, z = 200, loc = high);
    G.dropdown(.{ 0, 100, 100, 20 }, low_choices, *low_value, identifier = 2, z = 100, loc = low);
    G.draw_popups();
    press(window, false);

    position = .{ 10, 110 };
    G.ui_per_frame_update(window, 800, 600, 2);
    press(window, true);
    G.dropdown(.{ 0, 500, 100, 20 }, high_choices, *high_value, identifier = 1, z = 200, loc = high);
    _, low_state := G.dropdown(.{ 0, 100, 100, 20 }, low_choices, *low_value, identifier = 2, z = 100, loc = low);

    assert(low_state.open);
    G.draw_popups();
    press(window, false);

    position = .{ 400, 300 };
    G.ui_per_frame_update(window, 800, 600, 3);
    G.dropdown(.{ 0, 500, 100, 20 }, high_choices, *high_value, identifier = 1, z = 200, loc = high);
    G.dropdown(.{ 0, 100, 100, 20 }, low_choices, *low_value, identifier = 2, z = 100, loc = low);
    drawn.count = 0;
    G.draw_popups();

    for name: string.["high0", "high1", "low0", "low1"] assert(index_of(name) >= 0, "the open list holding % is not drawn on the third frame", name);

    assert(index_of("low0") < index_of("high0"), "the list with z 200 is drawn over the list with z 100");
}

appended_text :: () {
    state: G.Text_Input_State;

    defer free(state.pre_modification_text);
    G.set_input(*state, "hello");
    state.active = true;
    G.select_all(*state);
    G.append_text(*state, "!");

    assert(state.text == "hello!", "append_text kept the text: '%'", state.text);
    assert(state.insert_point == 6 && !state.selection.active);
}

completions :: (text: string, data: *void) -> (results: [] string, longest_match_length: int) {
    all := string.["apple", "apricot", "avocado", "banana"];
    found: [..] string;
    found.allocator = temp;

    for all if text.count <= it.count && slice(it, 0, text.count) == text array_add(*found, it);

    return found, text.count;
}

slice :: (s: string, start: int, count: int) -> string {
    return .{ count, s.data + start };
}

tab_cycling :: () {
    state: G.Text_Input_State;

    defer free(state.pre_modification_text);
    defer free(state.completion_text);
    defer free(state.completion_prefix);
    state.auto_complete.proc = completions;
    G.set_input(*state, "a");
    state.active = true;

    for expected: string.["apple", "apricot", "avocado", "apple"] {
        G.press_tab(*state);
        assert(state.text == expected, "tab completed to '%', not '%'", state.text, expected);
    }

    G.press_tab(*state, true);
    assert(state.text == "avocado", "shift+tab went to '%'", state.text);

    // Typing ends the cycle: the next Tab completes what was typed.
    G.set_input(*state, "b");
    G.press_tab(*state);
    assert(state.text == "banana", "a new prefix completed to '%'", state.text);
}

wrapped_keyframe :: () {
    state: G.Color_Animation_Editor_State;

    defer array_free(state.color_keyframes);
    frames := Colors.Color_Keyframe.[
        .{ .{ 1, 0, 0, 1 }, 0.2 },
        .{ .{ 0, 0, 1, 1 }, 0.8 },
        .{ .{ 0, 0, 0, 1 }, 0.9 },
    ];

    G.set_keyframes(*state, frames);
    G.set_interpolation(*state, .{ color_space = .RGB, geometry = .LINEAR, wrapping = true });
    G.init_color_by_interpolation(*state, 2);

    color := state.color_keyframes[2].color_rgb;

    // After the last key the gradient wraps back toward the first, so the new key is a mix, not the last key's color.
    assert(color.x > 0.3 && color.z > 0.3, "the keyframe at 0.9 got % instead of a blend of the last and first", color);

    G.set_interpolation(*state, .{ color_space = .RGB, geometry = .LINEAR, wrapping = false });
    G.init_color_by_interpolation(*state, 2);
    color = state.color_keyframes[2].color_rgb;

    assert(color.x < 0.01 && color.z > 0.99, "without wrapping the keyframe holds the last color: %", color);
}

mouse :: (window: type_of(G.Button_State.widget.window), right_handed: bool) -> (x: int, y: int, success: bool) {
    return cast(int) position.x, cast(int) position.y, true;
}

quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {}

prepare :: (font: type_of(G.Label_Theme.font), text: string, effects: type_of(G.Label_Theme.effects) = 0) -> s64 {
    array_add(*drawn, copy_string(text));

    return text.count * 8;
}

draw_text :: (font: type_of(G.Label_Theme.font), x: s64, y: s64, color: Vector4, effects: type_of(G.Label_Theme.effects) = 0) {}
