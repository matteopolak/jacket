// Axis-aligned rectangle in pixels, used for all widget layout, hit testing and drawing.
//
// `x` and `y` are the corner with the lowest coordinates and `w` and `h` the extent along each axis. The y axis points up in GetRect
// and down in GetRect_LeftHanded.
Rect :: struct {
    x, y: float;
    w, h: float;
}

#if !LEFT_HANDED {
    // A rectangle recorded by [occluder_start] that hides the widgets below it from mouse events.
    //
    // `serial` is the position of the record among the occluders started in the frame. Widgets are checked against the records of the previous frame.
    Occlusion_Record :: struct {
        serial: s32;
        rect: Rect;
        widget: *Active_Widget;
    }
}

// Interface shared by the color types the brightness helpers accept: any struct with `x`, `y` and `z` float members, such as `Vector3` and `Vector4`.
// Used as the constraint of [darken], [whiten] and [brighten].
VBase3 :: struct {
    x, y, z: float;
}

// Mouse state of a rectangle, as returned by [get_status_flags].
Status_Flags :: enum_flags u32 {
    OVER :: 0x01; // the pointer is over the rectangle, which is visible and not occluded
    FOCUS :: 0x02; // same condition as OVER
    DOWN :: 0x04; // the left mouse button is held while over the rectangle
    PRESSED :: 0x08; // the left mouse button went down this frame while over the rectangle
}

// Hit-test checks that [get_status_flags] skips when passed in `ignore`; combine them with `|`.
Status_Flags_Ignore :: enum_flags u8 {
    NOTHING;
    LEFT_CLICK; // do not report DOWN or PRESSED
    TOOLTIP; // accepted and has no effect: a tooltip is text set with [tooltip], not a region that is hit-tested
    SCISSOR; // do not limit the hit test to the current scissor rectangle
}

// The sides on which a region keeps a margin; combine with `|`.
Margin :: enum_flags {
    Horizontal :: 0x1; // left and right sides
    Vertical :: 0x2; // top and bottom sides
}

// Horizontal placement of label text inside its rectangle. `Left` and `Right` keep `alignment_pad_ems` ems from the edge.
Text_Alignment :: enum u8 {
    Left :: 0;
    Center :: 1;
    Right :: 2;
}

// Mouse cursor shapes GetRect asks the program to show, through `Draw_Procs.set_pointer_image`.
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

// How a size in a theme is interpreted, as read by [get_roundedness_for_rect].
Size_Computation :: enum u8 {
    RELATIVE_TO_HEIGHT :: 0; // the value times the height of the rectangle
    RELATIVE_TO_WIDTH :: 1; // the value times the width of the rectangle
    ABSOLUTE_FROM_THEME_FIELD :: 2; // the value in pixels
    ABSOLUTE_FROM_GLOBAL :: 3; // the global pixel size set with the `set_global_*_in_pixels` procedures
}

// The built-in themes; the value indexes `default_theme_procs`.
Default_Themes :: enum {
    Default;
    Grayscale;
    Bentonite;
    Nimbus;
    Blood_Vampire;
    Freddie_Freeloader;
    HIGH_CONTRAST;
}

// Input style of a color picker.
Color_Pick_Mode :: enum {
    HSV_CIRCLE :: 0;
    HSV_SLIDERS :: 1;
    RGB_SLIDERS :: 2;
    TYPE_INS :: 3;
}

// Which text a slider's formatting callback is asked for: the one shown on the slider or the one shown while the user edits the value.
Slider_Format_Text_Mode :: enum u8 {
    DISPLAY :: 0; // text shown on the slider
    EDITING :: 1; // text shown while the value is being typed
}

// Scroll directions a scrollable region reacts to.
Scrollable_Region_Flags :: enum_flags u8 {
    VERTICAL :: 1;
    HORIZONTAL :: 2;
}

// Parts a subwindow draws.
Subwindow_Flags :: enum_flags u16 {
    TITLE_BAR :: 0x1; // a title bar above the content
    CLOSE_BUTTON :: 0x2; // a button in the title bar that closes the subwindow
    FRINGE :: 0x4;
    VISIBLE_RESIZE_CORNER :: 0x8; // draws the resize handle in the south-east corner
}

// Edges and behaviours that resize a subwindow when dragged; combine with `|`.
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

// Requests passed to [text_input] to activate or deactivate the field from the program.
Text_Input_Action :: enum_flags u8 {
    ACTIVATE :: 0x1; // start editing, as if the field had been clicked
    DEACTIVATE :: 0x2; // stop editing
}

// What happened to a text input this frame, returned by [text_input]; combine with `|`.
Text_Result_Action :: enum_flags u32 {
    ENTERED :: 0x1; // Enter was pressed
    ESCAPED :: 0x2; // Escape was pressed; the text reverts to the last saved input
    TEXT_MODIFIED :: 0x4; // the text changed
}

// A selected range of text in a [Text_Input_State]; `active` says whether a range is selected.
Selection :: struct {
    start_line := -1; // -1 when there is no selection
    start_character: int;
    end_line := -1;
    end_character: int;
    active := false;
}

// Tab-completion callback for [text_input].
//
// Tab calls `proc` with the current text and `data`. It returns the candidate completions and `longest_match_length`, which is
// stored in [Text_Input_State.longest_match]. Repeated Tab presses cycle through the candidates of the text as it was typed
// before the first Tab, and Shift+Tab goes backwards; typing or setting the text starts a new cycle.
Auto_Complete :: struct {
    proc: (str: string, data: *void) -> (results: [] string, longest_match_length: int);
    data: *void;
}

// Text and application data to show as a tooltip, set with [tooltip] and read with [get_active_tooltip].
Tooltip :: struct {
    text: string;
    pointer_data: *void;
    int_data: s64;
}

// Callbacks through which GetRect draws, measures text and reads the mouse.
//
// [ui_init] uses the callbacks of the default renderer when it is given none and the default types are in use.
Draw_Procs :: struct {
    texture_load_from_memory: (texture: *Texture, memory: [] u8, srgb: bool, build_mipmaps: bool) -> bool;
    set_scissor: (x0: s32, y0: s32, x1: s32, y1: s32); // takes the corners of the rectangle, not a size
    clear_scissor: ();
    set_shader_for_color: (enable_blend := false);
    set_shader_for_images: (texture: *Texture);
    prepare_text: (font: *Font, text: string, effects: Font_Effects = 0) -> s64;
    draw_prepared_text: (font: *Font, x: s64, y: s64, text_color: Vector4, effects: Font_Effects = 0);
    get_mouse_pointer_position: (window: Window_Type, right_handed: bool) -> (x: int, y: int, success: bool); // returns the position in pixels; `right_handed` is true unless `LEFT_HANDED`; `success` is false when unavailable
    set_pointer_image: (image: Pointer_Image, widget: *Active_Widget);
    get_font_at_size: (memory: [] u8, pixel_height: int) -> *Font;
    immediate_triangle: (p0: Vector3, p1: Vector3, p2: Vector3, c0 := Vector4.{ 1, 1, 1, 1 }, c1 := Vector4.{ 1, 1, 1, 1 }, c2 := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{ z = 1 });
    immediate_quad: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 });
    immediate_flush: ();
}

// How a rectangle is drawn: corner rounding and frame thickness, as used by [rounded_rectangle].
Rectangle_Shape :: struct {
    roundedness_type: Size_Computation; // how `roundedness` is read
    roundedness := .18; // corner radius, read according to `roundedness_type`
    pixels_per_edge_segment := 10.0; // target arc length in pixels of each segment that approximates a rounded corner
    frame_thickness_type: Size_Computation; // how `frame_thickness` is read
    frame_thickness := .02; // read according to `frame_thickness_type`
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
    rounding_flags := Rounding_Flags.NORTHEAST | .NORTHWEST | .SOUTHWEST | .SOUTHEAST; // the corners that are rounded
}

// Appearance of a text label, used by [label].
Label_Theme :: struct {
    font: *Font;
    effects: Font_Effects;
    text_color := Vector4.{ 1, 1, 1, 1 };
    alignment := Text_Alignment.Center;
    alignment_pad_ems := 1.0; // distance from the edge for left and right alignment, in ems of the font
    text_baseline_vertical_position := FLOAT32_INFINITY; // baseline position as a fraction of the rectangle height; infinity uses the font default
}

// Fill and frame of a rectangle drawn behind a widget.
Background_Theme :: struct {
    shape: Rectangle_Shape;
    color := Vector4.{ .08, .08, .08, 1.0 };
    frame_color := Vector4.{ .25, .25, .22, 1.0 };
}

// Appearance and behaviour of a [button]. Other widget themes embed it.
Button_Theme :: struct {
    using label_theme: Label_Theme;
    rectangle_shape: Rectangle_Shape;
    surface_color := Vector4.{ .08, .08, .08, 1.0 };
    surface_color_over := Vector4.{ .15, .15, .20, 1.0 };
    surface_color_flash := Vector4.{ .60, .77, .62, 1.0 }; // color shown briefly after a press, fading over `press_duration`
    surface_color_down := Vector4.{ .28, .28, .38, 1.0 };
    text_color_over := Vector4.{ .95, .95, .95, 1.0 };
    text_color_pressed := Vector4.{ .99, .99, .99, 1.0 };
    #if LEFT_HANDED text_offset_down := Vector2.{ .12, .12 };
    else text_offset_down := Vector2.{ .12, -.12 };
    frame_thickness_over := .04; // frame thickness, as a fraction of the rectangle height, when the pointer is over; used with `enable_variable_frame_thickness`
    frame_thickness_down := .05; // same, while pressed
    frame_thickness_flash := .07;
    enable_variable_frame_thickness := false; // blends the frame thickness between the shape's and the over and down values
    frame_color := Vector4.{ .40, .40, .50, 1.0 };
    frame_color_over := Vector4.{ .55, .55, .45, 1.0 };
    frame_color_flash := Vector4.{ .99, .99, .65, 1.0 };
    frame_color_down := Vector4.{ .75, .75, .55, 1.0 };
    press_duration := 0.7; // seconds the flash colors take to fade after a press
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
        TOGGLE :: 0x1; // a press flips the pressed state instead of holding it only while the button is down
        TEXTURE_LEFT_HANDED :: 0x2;
        DO_NOT_DRAW :: 0x4; // the button only reacts and draws nothing
    } = .TEXTURE_LEFT_HANDED;
}

// Appearance of a [checkbox_flags] or [radio_button].
Checkbox_Theme :: struct {
    using button_theme: Button_Theme;
    button_color_selected := Vector4.{ .90, .90, .90, 1.0 };
    text_color_selected := Vector4.{ .90, .60, .90, 1.0 };
    is_radio_button := false;
}

// Appearance and editing behaviour of a [text_input].
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

// Appearance of a [number_input].
Number_Input_Theme :: struct {
    using text_input_theme: Text_Input_Theme;
    clamp_low := true;
    clamp_high := true;
}

// Appearance and behaviour of a [slider].
Slider_Theme :: struct {
    foreground: Button_Theme;
    decimals: s32 = 2; // digits after the decimal point shown by the default float formatting
    background: Button_Theme;
    surface_margin := .15;
    text_editable := true; // right-clicking the slider edits its value as text
    clamp_text_input_low := true; // clamp typed values to the slider's minimum
    clamp_text_input_high := true; // clamp typed values to the slider's maximum
    sliding_is_relative := false;
    sliding_becomes_relative_by_pressing_shift := true;
    relative_slide_factor := 0.7;
    relative_slide_factor_modified := 0.2;
    do_clamp_if_we_are_relative_dragging_but_started_outside_the_clamp_range := false;
    text_input_theme: Text_Input_Theme;
    use_spinboxes := true; // adds plus and minus buttons at the right end
    spinbox_button_width := 1.0; // width of each spinbox button, as a multiple of the slider height
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

// Appearance of a [dropdown].
Dropdown_Theme :: struct {
    theme_for_current_value: Button_Theme;
    theme_for_each_choice: Button_Theme;
    theme_for_current_choice: Button_Theme;
    dropdown_indicator_aspect_ratio := 1.0;
    arrow_flip_up_rate := 2.5;
    arrow_flip_down_rate := 3.9;
}

// Appearance of a scrollable region begun with [begin_scrollable_region].
Scrollable_Region_Theme :: struct {
    direction := Scrollable_Region_Flags.VERTICAL;
    horizontal_margin_size := 0.005; // as a fraction of the region's width; used when `margin_flags` has `Horizontal`
    vertical_margin_size := 0.005; // as a fraction of the region's height; used when `margin_flags` has `Vertical`
    margin_flags := Margin.Horizontal | .Vertical;
    scrollbar_size := 0.03; // thickness as a fraction of the region's width for a vertical bar, or of its height for a horizontal one
    minimum_nib_size := 1.0; // in pixels
    nib_margin := 0.10; // gap on each side of the nib, as a fraction of the bar's thickness
    scrollbar_nib_theme: Button_Theme;
    region_background: Background_Theme;
    scrollbar_background: Background_Theme;
    mouse_wheel_increment := 0.0004; // scroll per wheel notch, as a fraction of the render target height (width for a horizontal region)
}

// Appearance and initial layout of a [slidable_region].
Slidable_Region_Theme :: struct {
    orientation: enum u8 {
        HORIZONTAL :: 0;
        VERTICAL :: 1;
    };
    divider_theme: Button_Theme;
    divider_thickness_type := Size_Computation.ABSOLUTE_FROM_GLOBAL;
    divider_thickness := .02; // read according to `divider_thickness_type`
    initial_divider_position_type := Size_Computation.RELATIVE_TO_HEIGHT;
    initial_divider_position := 0.5; // read according to `initial_divider_position_type`; applied when the region is first created
    first_background: Background_Theme;
    second_background: Background_Theme;
}

// Appearance of a subwindow added with [add_subwindow].
Subwindow_Theme :: struct {
    title_bar_size := 0.03; // height as a fraction of the render target height
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
    resize_rect_size := 0.05; // side of the resize corner as a fraction of the smaller side of the subwindow
    resize_minimum_size := Vector2.{ .08, .08 }; // smallest size, as fractions of the render target width and height
    resize_snap_margin := 0.02;
}

// Appearance of a [color_picker].
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

// Appearance of a [color_animation_editor] and the pickers it opens.
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

// The complete set of themes and colors a program uses by default, set with [set_default_theme].
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

// State of one [button], kept between frames. Other widgets embed it for their buttons.
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

// State of one checkbox or radio button, returned by [checkbox_flags] and [radio_button].
Checkbox_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    base: Button_State;
    selected_t := 0.0;
}

// State of one [text_input], kept between frames.
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
    completion_prefix: string; // the text as typed before the first Tab of a cycle; owned by the state
    completion_text: string; // what the last Tab put in the text, to tell a cycle from new typing; owned by the state
    do_active_widget_deactivate_all := false;
}

// State of one [number_input], kept between frames.
Number_Input_State :: struct #type_info_none {
    #as using widget: Active_Widget;
    text_input_state: Text_Input_State;
    inputting_text := false;
}

// State of one [slider], kept between frames.
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

// State of one [dropdown], returned by it.
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

// State of one scrollable region, returned by [begin_scrollable_region].
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

// State of one [slidable_region], kept between frames.
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

// Description of a subwindow, owned by the caller and passed to [add_subwindow]. The subwindow updates `rect` as the user moves or resizes it.
Subwindow_Info :: struct {
    rect: Rect;
    title_text: string; // text shown in the title bar
    fixed_aspect_ratio := -1.0; // width divided by height to keep while resizing; 0 or less allows any shape
    snap_to_size: Vector2;
    user_data: *void;
    draw: (state: *Subwindow_State, content_rect: Rect, user_data: *void); // called each frame with the content area while the subwindow is open
    open := true; // the subwindow is shown only while this is true; the close button clears it
    wants_a_raise := false;
}

// State of one subwindow, returned by [add_subwindow].
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

// State of one [color_picker], kept between frames.
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

// State of one [color_animation_editor], kept between frames.
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
