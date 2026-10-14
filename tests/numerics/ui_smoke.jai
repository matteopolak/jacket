#import "Basic";
#import "Math";
G :: #import "GetRect";

draw_count: int;

#load "ui_test_theme.jai";

main :: () {
    install_test_baselines();
    procs: G.Draw_Procs;
    procs.immediate_quad = quad;
    procs.immediate_triangle = triangle;
    procs.prepare_text = prepare;
    procs.draw_prepared_text = text;
    procs.get_mouse_pointer_position = mouse;
    procs.set_scissor = scissor;
    procs.clear_scissor = clear;
    G.ui_init(procs = *procs);
    window: type_of(G.Button_State.widget.window);

    G.ui_per_frame_update(window, 800, 600, 1);
    color := xyz(1, 0, 0);
    applied, dragging, picker := G.color_picker(.{ 0, 0, 200, 200 }, *color, identifier = 100);

    assert(picker.current_hsv_color.x == 0 && !applied && !dragging);

    for mode: 1..3 {
        picker.color_pick_mode = cast(G.Color_Pick_Mode) mode;
        applied, dragging, picker = G.color_picker(.{ 0, 0, 200, 200 }, *color, identifier = 100);
        assert(picker.current_rgb_color.x == 1);
    }

    editor := G.color_animation_editor(.{ 250, 0, 300, 150 }, identifier = 200);

    assert(editor.color_samples.count > 0);
    G.open_time_view(editor.theme, editor, .{ 300, 300, 100, 100 });
    G.draw_time_view(editor.theme, editor);
    G.draw_popups();
    theme: G.Slider_Theme;
    theme = G.default_overall_theme.slider_theme;
    value: float64 = 0.5;
    changed, entered := G.slider(.{ 0, 250, 200, 20 }, *value, 0.0, 1.0, 0.1, *theme, identifier = 300);

    assert(!changed && !entered);
    integer: s32 = 4;

    changed, entered = G.slider(.{ 0, 280, 200, 20 }, *integer, 0, 10, 1, *theme, identifier = 301);
    assert(integer == 4 && value == 0.5 && !changed && !entered);
    theme2: G.Number_Input_Theme;
    theme2.text_baseline_vertical_position = 0.25;
    G.number_input(.{ 0, 310, 200, 20 }, "", *integer, 0, 10, *theme2, identifier = 302);
    choices: [2] string = .["zero", "one"];
    selected: s32;
    dropdown_changed, dropdown := G.dropdown(.{ 0, 340, 100, 20 }, choices, *selected, identifier = 400);

    assert(!dropdown_changed && !dropdown.open && selected == 0);

    G.draw_popups();
    scroll, inside := G.begin_scrollable_region(.{ 0, 380, 200, 100 }, identifier = 500);

    assert(inside.x == scroll.inner_rect.x && inside.w == scroll.inner_rect.w);
    scroll_value: float;

    G.end_scrollable_region(scroll, 200, -200, *scroll_value);
    split, a, b, divider := G.slidable_region(.{ 250, 200, 200, 100 }, identifier = 600);

    assert(a.w + b.w + divider.w == 200 && a.h == 100 && b.h == 100);

    G.switch_to_second_rect(split);
    G.finish(split);
    input_theme: G.Text_Input_Theme;
    input_theme = G.default_overall_theme.text_input_theme;
    action, result, input := G.text_input(.{ 250, 340, 200, 20 }, "abc", *input_theme, identifier = 700, input_action = .ACTIVATE);

    assert(action == 0 && result == "abc");

    G.getrect_handle_event(.{ type = .TEXT_INPUT, utf32 = 0x1f600, window = window });
    assert(input.text == "abc😀");
    G.select_all(input);
    assert(input.selection.end_character == input.text.count && input.text == "abc😀");
    G.press_enter(input);
    assert((input.pending_result_actions & .ENTERED) != 0);
    f: G.Slider_State;
    formatted := G.slider_format_text_float_default("", "", 1.25, *theme, *f, .DISPLAY);

    assert(formatted == "1.25");
    free(formatted);
    assert(draw_count > 100);
    print("ui smoke ok\n");
}

quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {
    draw_count += 1;
}

triangle :: (p0: Vector3, p1: Vector3, p2: Vector3, c0 := Vector4.{ 1, 1, 1, 1 }, c1 := Vector4.{ 1, 1, 1, 1 }, c2 := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{ z = 1 }) {
    draw_count += 1;
}

prepare :: (font: type_of(G.Label_Theme.font), text: string, effects: type_of(G.Label_Theme.effects) = 0) -> s64 {
    return text.count * 8;
}

text :: (font: type_of(G.Label_Theme.font), x: s64, y: s64, color: Vector4, effects: type_of(G.Label_Theme.effects) = 0) {
    draw_count += 1;
}

mouse :: (window: type_of(G.Button_State.widget.window), right_handed: bool) -> (x: int, y: int, success: bool) {
    return 700, 500, true;
}

scissor :: (x0: s32, y0: s32, x1: s32, y1: s32) {}
clear :: () {}
