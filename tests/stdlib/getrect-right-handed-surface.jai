GR :: #import "GetRect";

// Compile-only right-handed caller surface. Resource-backed UI is not executed here.
right_handed_surface :: (window: GR.Window_Type, theme: *GR.Overall_Theme,
                       choices: [] string, selected: *s32, amount: *float) {
    rectangle := GR.get_rect(x = 10, y = 20, w = 100, h = 60);
    low: float = rectangle.y;
    top, remainder := GR.cut_top(rect = rectangle, amount = 15, margin = 2);
    GR.ui_init(default_font = "", procs = null);
    background_occluded, is_active := GR.ui_per_frame_update(
        window = window, _render_target_width = 640, _render_target_height = 480,
        _current_time = cast(float64) 1);
    GR.label(r = rectangle, text = "Mesh", theme = *theme.label_theme);
    changed, dropdown_state := GR.dropdown(
        r = rectangle, choices = choices, current_value_pointer = selected,
        theme = *theme.dropdown_theme, identifier = 1);
    toggled, checkbox_state := GR.base_checkbox(
        _r = rectangle, text = "Show Skeleton", selected = true,
        theme = *theme.checkbox_theme, identifier = 2);
    pressed, button_state, released := GR.button(
        r = rectangle, text = "Reset", theme = *theme.button_theme,
        identifier = 3, disable_over = false, disable_press = false);
    GR.slider(_r = rectangle, value = amount, min_value = cast(float) 0,
        max_value = cast(float) 120, spinbox_step = cast(float) 1,
        theme = *theme.slider_theme, prefix = "Animation Speed: ", suffix = "FPS");
    action, new_text, edit_state := GR.text_input(
        _r = rectangle, inert_text = "Name", theme = *theme.text_input_theme, identifier = 4);
    width, height := GR.text_size(font = theme.font, text = "Title");
    padding := GR.em(x = 1, font = theme.font);
    scroll, inside := GR.begin_scrollable_region(
        r = rectangle, theme = *theme.scrollable_region_theme, identifier = 5);
    GR.end_scrollable_region(state = scroll, max_x = inside.x + inside.w,
        min_y = inside.y - 40, scroll_value = amount);
    GR.draw_popups();
}

historical_asset_surface :: () {
    radio_full, radio_empty, check_full, check_empty, font,
    circle, hsl, rgb, numbers := GR.get_rect_get_global_data();
}
