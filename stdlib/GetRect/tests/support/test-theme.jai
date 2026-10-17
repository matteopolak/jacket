// Test renderer supplies an explicit baseline through the existing theme API;
// it does not pretend to expose the missing concrete Font metrics ABI.
install_test_baselines :: () {
    t := *G.default_overall_theme;

    test_button_baseline(*t.button_theme);
    test_button_baseline(*t.checkbox_theme.button_theme);
    test_dropdown_baseline(*t.dropdown_theme);
    test_button_baseline(*t.slider_theme.foreground);
    test_button_baseline(*t.slider_theme.background);
    test_button_baseline(*t.slider_theme.spinbox_theme);
    test_button_baseline(*t.slider_theme.text_input_theme.button_theme);
    test_button_baseline(*t.text_input_theme.button_theme);
    test_picker_baseline(*t.color_picker_theme);
    test_button_baseline(*t.color_animation_editor_theme.button_theme);
    test_dropdown_baseline(*t.color_animation_editor_theme.dropdown_theme);
    test_picker_baseline(*t.color_animation_editor_theme.color_picker_theme);
    test_window_baseline(*t.color_animation_editor_theme.color_picker_subwindow_theme);
    test_window_baseline(*t.color_animation_editor_theme.time_view_subwindow_theme);
    test_window_baseline(*t.subwindow_theme);
}

test_button_baseline :: (theme: *G.Button_Theme) {
    theme.text_baseline_vertical_position = 0.25;
}

test_dropdown_baseline :: (theme: *G.Dropdown_Theme) {
    test_button_baseline(*theme.theme_for_current_value);
    test_button_baseline(*theme.theme_for_current_choice);
    test_button_baseline(*theme.theme_for_each_choice);
}

test_picker_baseline :: (theme: *G.Color_Picker_Theme) {
    test_button_baseline(*theme.apply_and_revert_button_theme);
    test_button_baseline(*theme.mode_switch_button_theme);
    test_button_baseline(*theme.stash_button_theme);
    test_button_baseline(*theme.type_ins_input_theme.button_theme);
}

test_window_baseline :: (theme: *G.Subwindow_Theme) {
    test_button_baseline(*theme.title_bar);
    test_button_baseline(*theme.close_button);
}
