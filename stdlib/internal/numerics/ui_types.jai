Rect :: struct {
    x, y: float;
    w, h: float;
}

VBase3 :: struct {
    x, y, z: float;
}

Status_Flags :: enum_flags u32 {
    OVER :: 0x01;
    FOCUS :: 0x02;
    DOWN :: 0x04;
    PRESSED :: 0x08;
}

Status_Flags_Ignore :: enum_flags u8 {
    NOTHING;
    LEFT_CLICK;
    TOOLTIP;
    SCISSOR;
}

Margin :: enum_flags {
    Horizontal :: 0x1;
    Vertical :: 0x2;
}

Text_Alignment :: enum u8 {
    Left :: 0;
    Center :: 1;
    Right :: 2;
}

Pointer_Image :: enum u16 {
    NORMAL :: 0;
    PRESSABLE :: 1;
    LOCKED :: 2;
    DRAGGING_HORIZONTAL :: 3;
    DRAGGING_VERTICAL :: 4;
    DRAGGING_FREE :: 5;
    DRAGGING_NW_SE :: 6;
    DRAGGING_NE_SW :: 7;
}

Size_Computation :: enum u8 {
    RELATIVE_TO_HEIGHT :: 0;
    RELATIVE_TO_WIDTH :: 1;
    ABSOLUTE_FROM_THEME_FIELD :: 2;
    ABSOLUTE_FROM_GLOBAL :: 3;
}

Default_Themes :: enum {
    Default;
    Grayscale;
    Bentonite;
    Nimbus;
    Blood_Vampire;
    Freddie_Freeloader;
    HIGH_CONTRAST;
}

Color_Pick_Mode :: enum {
    HSV_CIRCLE :: 0;
    HSV_SLIDERS :: 1;
    RGB_SLIDERS :: 2;
    TYPE_INS :: 3;
}

Slider_Format_Text_Mode :: enum u8 {
    DISPLAY :: 0;
    EDITING :: 1;
}

Scrollable_Region_Flags :: enum_flags u8 {
    VERTICAL :: 1;
    HORIZONTAL :: 2;
}

Subwindow_Flags :: enum_flags u16 {
    TITLE_BAR :: 0x1;
    CLOSE_BUTTON :: 0x2;
    FRINGE :: 0x4;
    VISIBLE_RESIZE_CORNER :: 0x8;
}

Resize_Controls :: enum_flags u16 {
    EAST :: 0x1;
    NORTH :: 0x2;
    WEST :: 0x4;
    SOUTH :: 0x8;
    CORNER_SE :: 0x10;
    PROPORTIONAL :: 0x20;
    SNAP_EW :: 0x40;
    SNAP_NS :: 0x80;
    NS :: NORTH | SOUTH;
    EW :: EAST | WEST;
    NE :: NORTH | EAST;
    NW :: NORTH | WEST;
    SE :: SOUTH | EAST;
    SW :: SOUTH | WEST;
    ENWS :: NS | EW;
    SNAP :: SNAP_EW | SNAP_NS;
}

Text_Input_Action :: enum_flags u8 {
    ACTIVATE :: 0x1;
    DEACTIVATE :: 0x2;
}

Text_Result_Action :: enum_flags u32 {
    ENTERED :: 0x1;
    ESCAPED :: 0x2;
    TEXT_MODIFIED :: 0x4;
}

Selection :: struct {
    start_line := -1;
    start_character: int;
    end_line := -1;
    end_character: int;
    active := false;
}

Auto_Complete :: struct {
    proc: (str: string, data: *void) -> (results: [] string, longest_match_length: int);
    data: *void;
}

Tooltip :: struct {
    text: string;
    pointer_data: *void;
    int_data: s64;
}

Draw_Procs :: struct {
    texture_load_from_memory: (texture: *Texture, memory: [] u8, srgb: bool, build_mipmaps: bool) -> bool;
    set_scissor: (x0: s32, y0: s32, x1: s32, y1: s32);
    clear_scissor: ();
    set_shader_for_color: (enable_blend := false);
    set_shader_for_images: (texture: *Texture);
    prepare_text: (font: *Font, text: string, effects: Font_Effects = 0) -> s64;
    draw_prepared_text: (font: *Font, x: s64, y: s64, text_color: Vector4, effects: Font_Effects = 0);
    get_mouse_pointer_position: (window: Window_Type, right_handed: bool) -> (x: int, y: int, success: bool);
    set_pointer_image: (image: Pointer_Image, widget: *Active_Widget);
    get_font_at_size: (memory: [] u8, pixel_height: int) -> *Font;
    immediate_triangle: (p0: Vector3, p1: Vector3, p2: Vector3, c0 := Vector4.{ 1, 1, 1, 1 }, c1 := Vector4.{ 1, 1, 1, 1 }, c2 := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{ z = 1 });
    immediate_quad: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 });
    immediate_flush: ();
}

Rectangle_Shape :: struct {
    roundedness_type: Size_Computation;
    roundedness := .18;
    pixels_per_edge_segment := 10.0;
    frame_thickness_type: Size_Computation;
    frame_thickness := .02;
    Rounding_Flags :: enum_flags u32 {
        NORTHEAST :: 0x1;
        NORTHWEST :: 0x2;
        SOUTHWEST :: 0x4;
        SOUTHEAST :: 0x8;
        EAST :: NORTHEAST | SOUTHEAST;
        WEST :: NORTHWEST | SOUTHWEST;
        SOUTH :: SOUTHEAST | SOUTHWEST;
        NORTH :: NORTHEAST | NORTHWEST;
        ALL :: EAST | WEST;
    };
    rounding_flags := Rounding_Flags.NORTHEAST | .NORTHWEST | .SOUTHWEST | .SOUTHEAST;
}

Label_Theme :: struct {
    font: *Font;
    effects: Font_Effects;
    text_color := Vector4.{ 1, 1, 1, 1 };
    alignment := Text_Alignment.Center;
    alignment_pad_ems := 1.0;
    text_baseline_vertical_position := FLOAT32_INFINITY;
}

Background_Theme :: struct {
    shape: Rectangle_Shape;
    color := Vector4.{ .08, .08, .08, 1.0 };
    frame_color := Vector4.{ .25, .25, .22, 1.0 };
}

Button_Theme :: struct {
    using label_theme: Label_Theme;
    rectangle_shape: Rectangle_Shape;
    surface_color := Vector4.{ .08, .08, .08, 1.0 };
    surface_color_over := Vector4.{ .15, .15, .20, 1.0 };
    surface_color_flash := Vector4.{ .60, .77, .62, 1.0 };
    surface_color_down := Vector4.{ .28, .28, .38, 1.0 };
    text_color_over := Vector4.{ .95, .95, .95, 1.0 };
    text_color_pressed := Vector4.{ .99, .99, .99, 1.0 };
    #if LEFT_HANDED text_offset_down := Vector2.{ .12, .12 };
    else text_offset_down := Vector2.{ .12, -.12 };
    frame_thickness_over := .04;
    frame_thickness_down := .05;
    frame_thickness_flash := .07;
    enable_variable_frame_thickness := false;
    frame_color := Vector4.{ .40, .40, .50, 1.0 };
    frame_color_over := Vector4.{ .55, .55, .45, 1.0 };
    frame_color_flash := Vector4.{ .99, .99, .65, 1.0 };
    frame_color_down := Vector4.{ .75, .75, .55, 1.0 };
    press_duration := 0.7;
    over_fade_in_speed := 10.0;
    over_fade_out_speed := 4.0;
    down_fade_in_speed := 12.0;
    down_fade_out_speed := 4.0;
    highlight_lighten_parameter := .15;
    highlight_darken_parameter := .6;
    alpha_scale := 1.0;
    texture_scale := 1.0;
    draw_callback: (r: Rect, text: string, theme: *Button_Theme, texture: *Texture, state: *Button_State);
    properties: enum_flags u64 {
        TOGGLE :: 0x1;
        TEXTURE_LEFT_HANDED :: 0x2;
        DO_NOT_DRAW :: 0x4;
    } = .TEXTURE_LEFT_HANDED;
}

Checkbox_Theme :: struct {
    using button_theme: Button_Theme;
    button_color_selected := Vector4.{ .90, .90, .90, 1.0 };
    text_color_selected := Vector4.{ .90, .60, .90, 1.0 };
    is_radio_button := false;
}

Text_Input_Theme :: struct {
    using button_theme: Button_Theme;
    text_color_autocomplete := Vector4.{ .30, .99, .30, 0.9 };
    text_color_autocomplete_failed := Vector4.{ .99, .55, .55, 0.9 };
    selection_color := Vector4.{ .15, .69, .85, 1.0 };
    cursor_color := Vector4.{ 1.0, 1.0, 1.0, 1.0 };
    text_insert_margin := 4.7;
    cursor_width_inside_text := 0.11;
    cursor_width_outside_text := .55;
    maintain_text_after_escape := false;
    deactivate_on_enter := true;
    multiclick_delay_max := 0.5;
    distance_to_break_multiclick := 0.8;
}

Number_Input_Theme :: struct {
    using text_input_theme: Text_Input_Theme;
    clamp_low := true;
    clamp_high := true;
}

Slider_Theme :: struct {
    foreground: Button_Theme;
    decimals: s32 = 2;
    background: Button_Theme;
    surface_margin := .15;
    text_editable := true;
    clamp_text_input_low := true;
    clamp_text_input_high := true;
    sliding_is_relative := false;
    sliding_becomes_relative_by_pressing_shift := true;
    relative_slide_factor := 0.7;
    relative_slide_factor_modified := 0.2;
    do_clamp_if_we_are_relative_dragging_but_started_outside_the_clamp_range := false;
    text_input_theme: Text_Input_Theme;
    use_spinboxes := true;
    spinbox_button_width := 1.0;
    surface_style: enum u8 {
        EXTEND_FROM_LEFT :: 0;
        NUB :: 1;
    };
    nub_thickness_type := Size_Computation.RELATIVE_TO_HEIGHT;
    nub_thickness := 1.2;
    spinbox_theme: Button_Theme;
    draw_background := slider_draw_background_default;
    draw_foreground := slider_draw_foreground_default;
    format_text_float := slider_format_text_float_default;
    format_text_int := slider_format_text_int_default;
    parse_text_float := slider_parse_text_float_default;
    parse_text_int := slider_parse_text_int_default;
}

Dropdown_Theme :: struct {
    theme_for_current_value: Button_Theme;
    theme_for_each_choice: Button_Theme;
    theme_for_current_choice: Button_Theme;
    dropdown_indicator_aspect_ratio := 1.0;
    arrow_flip_up_rate := 2.5;
    arrow_flip_down_rate := 3.9;
}

Scrollable_Region_Theme :: struct {
    direction := Scrollable_Region_Flags.VERTICAL;
    horizontal_margin_size := 0.005;
    vertical_margin_size := 0.005;
    margin_flags := Margin.Horizontal | .Vertical;
    scrollbar_size := 0.03;
    minimum_nib_size := 1.0;
    nib_margin := 0.10;
    scrollbar_nib_theme: Button_Theme;
    region_background: Background_Theme;
    scrollbar_background: Background_Theme;
    mouse_wheel_increment := 0.0004;
}

Slidable_Region_Theme :: struct {
    orientation: enum u8 {
        HORIZONTAL :: 0;
        VERTICAL :: 1;
    };
    divider_theme: Button_Theme;
    divider_thickness_type := Size_Computation.ABSOLUTE_FROM_GLOBAL;
    divider_thickness := .02;
    initial_divider_position_type := Size_Computation.RELATIVE_TO_HEIGHT;
    initial_divider_position := 0.5;
    first_background: Background_Theme;
    second_background: Background_Theme;
}

Subwindow_Theme :: struct {
    title_bar_size := 0.03;
    title_bar: Button_Theme;
    close_button: Button_Theme;
    close_button_width := 1.2;
    close_button_height := 0.8;
    close_button_right_margin := 0.4;
    close_button_x_height := 0.7;
    region_background: Background_Theme;
    horizontal_margin_size := 0.005;
    vertical_margin_size := 0.005;
    subwindow_flags := Subwindow_Flags.TITLE_BAR | .FRINGE | .CLOSE_BUTTON | .VISIBLE_RESIZE_CORNER;
    fringe_color := Vector4.{ 0, 0, 0, .2 };
    fringe_thickness := 0.004;
    resize_corner_line_thickness := 0.025;
    resize_corner_color := Vector4.{ .4, .4, .4, 1 };
    drag_intrusion_east := 0.006;
    drag_extrusion_east := 0.008;
    drag_intrusion_west := 0.006;
    drag_extrusion_west := 0.008;
    drag_intrusion_north := 0.002;
    drag_extrusion_north := 0.011;
    drag_intrusion_south := 0.006;
    drag_extrusion_south := 0.008;
    drag_corner_margin := 0.05;
    resize_controls := Resize_Controls.CORNER_SE | .EAST | .NORTH | .WEST | .SOUTH;
    resize_rect_size := 0.05;
    resize_minimum_size := Vector2.{ .08, .08 };
    resize_snap_margin := 0.02;
}

Color_Picker_Theme :: struct {
    horizontal_padding := .06;
    vertical_padding := .06;
    stashed_color_width := .10;
    margin_between_color_input_and_stashed_colors := .04;
    background: Background_Theme;
    outer_circle_diameter := .99;
    inner_circle_diameter := .73;
    inner_disc_radius := .95;
    initial_stashed_colors: [] Vector3 = .[];
    apply_and_revert_button_theme: Button_Theme;
    apply_string := "Apply";
    revert_string := "Revert";
    slider_hue_min := -50.0;
    slider_hue_max := 345.0;
    hue_range_in_zoomed_slider := 60.0;
    stash_button_theme: Button_Theme;
    mode_switch_button_theme: Button_Theme;
    type_ins_label_theme: Label_Theme;
    type_ins_input_theme: Number_Input_Theme;
    stashed_colors_shape: Rectangle_Shape;
    starting_color_pick_mode := Color_Pick_Mode.HSV_CIRCLE;
}

Color_Animation_Editor_Theme :: struct {
    horizontal_padding := .015;
    vertical_padding := .015;
    keyframe_color_inactive := Vector4.{ .35, .35, .35, 1 };
    keyframe_color_active := Vector4.{ .94, .94, .94, 1 };
    keyframe_color_picking := Vector4.{ .87, .87, .87, 1 };
    keyframe_color_backing := Vector4.{ .05, .05, .05, 1 };
    keyframe_width_relative_to_bar_height_nondragging := 0.10;
    keyframe_scale_when_dragging := 1.5;
    keyframe_scale_when_color_picking := 1.3;
    keyframe_selection_size_relative_to_visual_size := 1.4;
    keyframe_inset_ratio := 0.8;
    keyframe_backing_ratio := 0.24;
    background: Background_Theme;
    color_picker_theme: Color_Picker_Theme;
    color_picker_subwindow_theme: Subwindow_Theme;
    time_view_subwindow_theme: Subwindow_Theme;
    button_theme: Button_Theme;
    label_theme: Label_Theme;
    number_input_theme: Number_Input_Theme;
    dropdown_theme: Dropdown_Theme;
    checkbox_theme: Checkbox_Theme;
    default_interpolation := Interpolation.{ .OKLAB, .HERMITE, false };
    default_num_clipboard_samples: s32 = 16;
    default_keyframes: [] Color_Keyframe = .[.{ .{ 0.926563, 0.832695, 0.841836, 1 }, 0.505263 }, .{ .{ 0.828598, 0.029386, 0.579748, 1 }, 0.997368 }, .{ .{ 1, 0.33835, 0, 1 }, 0.003289 },];
    default_time_view_period := 3.0;
    Draw_Subwidgets_Flags :: enum_flags u8 {
        INTERPOLATION_CONTROLS :: 0x01;
        TIME_VIEW :: 0x02;
        SAMPLING_CONTROLS :: 0x04;
        COPY_TO_CLIPBOARD :: 0x08;
    };
    DRAW_ALL_SUBWIDGETS :: Draw_Subwidgets_Flags.INTERPOLATION_CONTROLS | .TIME_VIEW | .SAMPLING_CONTROLS | .COPY_TO_CLIPBOARD;
    draw_subwidgets_flags := DRAW_ALL_SUBWIDGETS;
}

Overall_Theme :: struct {
    button_theme: Button_Theme;
    checkbox_theme: Checkbox_Theme;
    color_picker_theme: Color_Picker_Theme;
    dropdown_theme: Dropdown_Theme;
    label_theme: Label_Theme;
    scrollable_region_theme: Scrollable_Region_Theme;
    slidable_region_theme: Slidable_Region_Theme;
    slider_theme: Slider_Theme;
    subwindow_theme: Subwindow_Theme;
    text_input_theme: Text_Input_Theme;
    color_animation_editor_theme: Color_Animation_Editor_Theme;
    font: *Font;
    background_color := Vector4.{ .10, .10, .10, 1.0 };
    background_color_bright := Vector4.{ .15, .15, .15, 1.0 };
    text_color := Vector4.{ .80, .80, .80, 1.0 };
    title_bar_color := Vector4.{ .80, .30, .30, 1.0 };
    title_bar_color_hilit := Vector4.{ .95, .50, .50, 1.0 };
    border_width := .004;
    outer_margin_size := .007;
    outer_margin_flags := Margin.Horizontal | .Vertical;
    scrollable_section_margin_width := .007;
    scrollable_section_margin_flags := Margin.Horizontal | .Vertical;
    alpha_scale := 1.0;
}

Button_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    hash: u64;
    status: Status_Flags;
    rect: Rect;
    frame_pixels: float;
    over_duration := 0.0;
    over_effect_t := 0.0;
    down_duration := 0.0;
    down_effect_t := 0.0;
    action_duration := -1.0;
    action_duration_2 := -1.0;
    over_factor := 0.0;
    pressed_factor := 0.0;
    pressed := false;
    user_data_float: float;
    user_data_s64: s64;
}

Checkbox_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    base: Button_State;
    selected_t := 0.0;
}

Text_Input_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    base: Button_State;
    rect: Rect;
    camera_initted := false;
    camera_x := 0.0;
    active_t := 0.0;
    active := false;
    disabled := false;
    shrunken_deadzone_margin := -1.0;
    last_displayed_text_x := FLOAT32_INFINITY;
    last_keypress_time: float64;
    last_inert_text := "";
    last_click_time: float64;
    current_click_time: float64 = -1;
    num_consecutive_clicks: s32;
    last_click_position: Vector2;
    click_position: Vector2;
    insert_point := 0;
    cursor_tracking_mouse := false;
    did_initial_mouse_click_so_check_for_deadzone_change := false;
    pending_result_actions: Text_Result_Action;
    last_saved_input: string;
    pre_modification_text: string;
    command_history: [..] string;
    command_history_index := 0;
    selection: Selection;
    buffered_events: [..] Event;
    text: string;
    MAX_BUFFER_SIZE :: 8000;
    input_buffer: [MAX_BUFFER_SIZE] u8;
    auto_complete: Auto_Complete;
    tab_pressed := false;
    completion_change_time: float64;
    match_array: [] string;
    match_length: int;
    match_selection: int;
    longest_match: int = -1;
    do_active_widget_deactivate_all := false;
}

Number_Input_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    text_input_state: Text_Input_State;
    inputting_text := false;
}

Slider_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    background: Button_State;
    foreground: Button_State;
    union {
        initial_value_float32: float32;
        initial_value_float64: float64;
        initial_value_int: int;
    }
    sliding_t: float;
    slowing_t: float;
    accumulated_delta_for_int_slider := 0.0;
    sliding := false;
    sliding_is_relative := false;
    inputting_text := false;
    value_type: *Type_Info;
    current_slide_setting: float;
    surface_margin_this_frame_in_pixels: float;
    changed_this_frame: bool;
}

Dropdown_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    open := false;
    open_t := 0.0;
    status: Status_Flags;
    hash: u64;
    base_rect: Rect;
    choices: [] string;
    theme: Dropdown_Theme = ---;
    loc: Source_Code_Location;
    z: s32;
    #if LEFT_HANDED original_value: s32 = -1;
    current_value_pointer: *s32;
    current_value_state: *Button_State;
}

Scrollable_Region_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    nib_state: Button_State;
    outer_rect: Rect;
    inner_rect: Rect;
    bar_rect: Rect;
    dragging := false;
    teleporting := false;
    theme: Scrollable_Region_Theme;
}

Slidable_Region_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    divider_state: Button_State;
    left_or_top_rect: Rect;
    right_or_bottom_rect: Rect;
    divider_rect: Rect;
    theme: *Slidable_Region_Theme;
    divider_t: float;
    divider_over_factor: float;
    divider_pressed_factor: float;
    dragging := false;
}

Subwindow_Info :: struct {
    rect: Rect;
    title_text: string;
    fixed_aspect_ratio := -1.0;
    snap_to_size: Vector2;
    user_data: *void;
    draw: (state: *Subwindow_State, content_rect: Rect, user_data: *void);
    open := true;
    wants_a_raise := false;
}

Subwindow_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    base: Button_State;
    title_bar_rect: Rect;
    content_rect: Rect;
    info: *Subwindow_Info;
    wants_draw := false;
    wants_a_raise := false;
    hash: u64;
    render_target_width: s32;
    render_target_height: s32;
    dragging: enum u8 {
        TITLE :: 0x1;
        RESIZE :: 0x2;
    };
    dragging_start: Vector2;
    dragging_last: Vector2;
    dragging_offset: Vector2;
    dragging_resize_control: Resize_Controls;
    resizing_aspect: float;
    theme: Subwindow_Theme;
    #if !LEFT_HANDED {
        draw_proc_adjusted_y := false;
        draw_proc_new_y1 := 0.0;
    }
}

Color_Picker_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    ring_point := Vector2.{ 1, 0 };
    disc_point := Vector2.{ 1, 0 };
    dragging_in_ring: bool;
    dragging_in_disc: bool;
    dragging_in_h: bool;
    dragging_in_h_zoom: bool;
    dragging_in_s: bool;
    dragging_in_l: bool;
    dragging_in_r: bool;
    dragging_in_g: bool;
    dragging_in_b: bool;
    dragging_in_hsv_rectangle: bool;
    dragging_in_ring_t: float;
    dragging_in_disc_t: float;
    dragging_in_h_t: float;
    dragging_in_h_zoom_t: float;
    dragging_in_s_t: float;
    dragging_in_l_t: float;
    dragging_in_r_t: float;
    dragging_in_g_t: float;
    dragging_in_b_t: float;
    dragging_in_hsv_rectangle_t: float;
    toggled_s: bool;
    toggled_v: bool;
    toggled_s_t: float;
    toggled_v_t: float;
    hsv_picker_in_rectangle_mode := true;
    circle_picker_in_warped_mode := false;
    zoom_hue_center := 0.0;
    color_pick_mode := Color_Pick_Mode.HSV_CIRCLE;
    current_hsv_color: Vector3;
    current_rgb_color: Vector3;
    original_rgb_color: Vector3;
    stashed_colors: [..] Vector3;
}

Color_Animation_Editor_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    ring_point := Vector2.{ 1, 0 };
    disc_point := Vector2.{ 1, 0 };
    dragging_in_strip: bool;
    dragging_keyframe_index: int;
    color_picker_keyframe_index: int;
    color_picker_subwindow_info: Subwindow_Info;
    #if !LEFT_HANDED color_picker_subwindow_info.open = false;
    time_view_subwindow_info: Subwindow_Info;
    #if !LEFT_HANDED time_view_subwindow_info.open = false;
    color_picker_editing_color_applied: Vector3;
    color_picker_subwindow_rect_initialized := false;
    theme: *Color_Animation_Editor_Theme;
    last_color_picker_state: *Color_Picker_State;
    color_samples: [..] Vector4;
    num_clipboard_samples: s32;
    interpolation_color_space_s32: s32;
    interpolation_geometry_s32: s32;
    interpolation_wrapping := false;
    output_type_s32: s32 = xx Output_Type.RESAMPLED_ARRAY;
    time_view_period: float;
    time_view_playing := true;
    time_view_current_time: float64;
    color_keyframes: [..] Augmented_Keyframe;
}
