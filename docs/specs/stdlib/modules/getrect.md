# GetRect

## Public declaration inventory

Program parameter: `Type_Indicator: Type = void` (second list, `#import "GetRect"()(Type_Indicator = T)`), shared by every import of GetRect and GetRect_LeftHanded in a program.

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `set_global_rounding_radius_in_pixels :: (value: float)`
- `set_global_frame_thickness_in_pixels :: (value: float)`
- `set_global_divider_thickness_in_pixels :: (value: float)`
- `set_global_initial_divider_position_in_pixels :: (value: float)`
- `set_global_slider_nub_thickness_in_pixels :: (value: float)`
- `Draw_Procs :: struct`
- `Status_Flags :: enum_flags u32`
- `Text_Alignment :: enum u8`
- `Rect :: struct`
- `KNUTH_GOLDEN_RATIO_64 :: 11400714819323198485`
- `get_hash :: (loc: Source_Code_Location, identifier: s64) -> u64`
- `combine_hashes :: (a: u64, b: u64) -> u64 #no_aoc`
- `expand :: (rect: Rect, amount: float) -> Rect`
- `expand_x :: (rect: Rect, amount: float) -> Rect`
- `expand_y :: (rect: Rect, amount: float) -> Rect`
- `shrink :: (rect: Rect, amount: float) -> Rect`
- `shrink_x :: (rect: Rect, amount: float) -> Rect`
- `shrink_y :: (rect: Rect, amount: float) -> Rect`
- `cut_left :: (rect: Rect, amount: float, margin := 0.0) -> (left: Rect, remainder: Rect)`
- `cut_right :: (rect: Rect, amount: float, margin := 0.0) -> (right: Rect, remainder: Rect)`
- `cut_bottom :: (rect: Rect, amount: float, margin := 0.0) -> (bottom: Rect, remainder: Rect)`
- `cut_top :: (rect: Rect, amount: float, margin := 0.0) -> (top: Rect, remainder: Rect)`
- `Status_Flags_Ignore :: enum_flags u8`
- `get_status_flags :: (r: Rect, widget_if_querying_outside_of_render_time: *Active_Widget = null, ignore := Status_Flags_Ignore.NOTHING) -> Status_Flags`
- `require_texture :: (data: string, loc := #caller_location) -> *Texture`
- `ui_init :: (default_font := "", procs: *Draw_Procs = null)`
- `ui_get_button_state :: (key: Key_Code) -> Key_Current_State`
- `windows_per_frame_update :: ()`
- `switch_window :: (window: Window_Type, _render_target_width: s32, _render_target_height: s32) -> (background_occluded: bool, is_active: bool)`
- `ui_per_frame_update :: (window: Window_Type, _render_target_width: s32, _render_target_height: s32, _current_time: float64) -> (background_occluded: bool, is_active: bool)`
- `is_visible_in_scissor :: (x: float, y: float) -> bool`
- `is_inside :: (x: float, y: float, r: Rect) -> bool`
- `is_visible_in_scissor :: (using rect: Rect) -> bool`
- `set_scissor :: (r: Rect)`
- `get_rect :: (x: float, y: float, w: float, h: float) -> Rect`
- `get_quad :: (using r: Rect, snap_coordinates := false) -> (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2)`
- `intersect :: (a: Rect, b: Rect) -> Rect`
- `bound :: (a: Rect, b: Rect) -> Rect`
- `push_scissor :: (_rect: Rect)`
- `pop_scissor :: ()`
- `VBase3 :: struct`
- `darken :: (color: $T/interface VBase3, t: float) -> T`
- `whiten :: (color: $T/interface VBase3, t: float) -> T`
- `brighten :: (color: $T/interface VBase3, value: float) -> T`
- `em :: (x: float, font: *Font) -> float`
- `text_size :: (font: *Font, text: string) -> (width: float, height: float)`
- `has_focus :: (rect: Rect) -> bool`
- `getrect_handle_event :: (event: Event)`
- `is_grabbing :: () -> bool`
- `get_rect_get_global_data :: () -> string, string, string, string, string, string, string, string, string`
- `active_widgets_per_frame_update :: ()`
- `active_widget_add :: (widget: *Active_Widget, is_grabbing := false)`
- `active_widget_remove :: (widget: *Active_Widget)`
- `active_widget_deactivate_all :: (except: *Active_Widget = null)`
- `occlusion_per_frame_update :: ()`
- `occlusion_per_window_update :: (window: Window_Type)`
- `occluder_start :: (widget: *Active_Widget, rect: Rect)`
- `occluder_finish :: (widget: *Active_Widget, rect := Rect.{})`
- `events_occluded :: (widget_if_querying_outside_of_render_time: *Active_Widget = null) -> bool`
- `Occlusion_Record :: struct`
- `Pointer_Image :: enum u16`
- `Windows :: #import "Windows"`
- `getrect_set_pointer_image :: (image: Pointer_Image, widget: *Active_Widget)`
- `init_pointers :: ()`
- `draw_popups :: ()`
- `find_or_create_state :: ($T: Type, hash: u64, start_using := true) -> (*T, created: bool)`
- `Size_Computation :: enum u8`
- `Rectangle_Shape :: struct`
- `Label_Theme :: struct`
- `Background_Theme :: struct`
- `Margin :: enum_flags`
- `Overall_Theme :: struct`
- `set_default_theme :: (theme: Overall_Theme)`
- `Default_Themes :: enum`
- `Theme_Proc :: #type () -> Overall_Theme`
- `default_theme_procs : [THEME_COUNT] Theme_Proc`
- `THEME_COUNT :: #run enum_highest_value(Default_Themes) + 1`
- `set_theme_from_base_colors :: (theme : * Overall_Theme, main_surface_color : Vector4, border_color : Vector4, interact_color : Vector4, atop_surface_text : Vector4, atop_border_text : Vector4)`
- `invert_button_theme :: (button_theme : *Button_Theme)`
- `invert_scrollbar :: (using scrollable_region_theme : *Scrollable_Region_Theme)`
- `Tooltip :: struct`
- `tooltip :: (text := "", pointer_data: *void = null, int_data := 0)`
- `get_active_tooltip :: () -> Tooltip`
- `update_action_durations :: (state: *Button_State, theme: *Button_Theme, begin: bool, dt: float) -> (pressed_flash_factor: float)`
- `update_production_value_button :: (r: Rect, changed: bool, state: *Button_State, status: Status_Flags, theme: *Button_Theme)`
- `update_over_state :: (state: *Button_State, theme: *Button_Theme, status: Status_Flags, dt: float)`
- `core_draw_button_label :: (r: Rect, text: string, state: *Button_State, theme: *Button_Theme)`
- `core_draw_button :: (r: Rect, text: string, state: *Button_State, status: Status_Flags, theme: *Button_Theme, texture: *Texture)`
- `button :: (r: Rect, text: string, theme: *Button_Theme = null, identifier: s64 = 0, loc := #caller_location, texture: *Texture = null, disable_over := false, disable_press := false) -> (was_just_pressed: bool, state: *Button_State, was_just_released: bool)`
- `get_colors_and_thickness_for_button :: (theme: *Button_Theme, state: *Button_State) -> (surface_color: Vector4, frame_color: Vector4, frame_thickness: float)`
- `get_roundedness_for_rect :: (r: Rect, type: Size_Computation, roundedness: float, global_value: float, limit := 0.5) -> float`
- `rounded_rectangle :: (r: Rect, shape: Rectangle_Shape, color: Vector4, frame_color: Vector4, frame_thickness_override := FLOAT32_INFINITY ) -> (frame_pixels: float, actual_roundedness: float)`
- `Button_Theme :: struct`
- `Button_State :: struct #type_info_none`
- `radio_button :: (r: Rect, text: string, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State`
- `radio_button :: (r: Rect, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State`
- `checkbox_flags :: (r: Rect, text: string, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State`
- `checkbox_flags :: (r: Rect, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State`
- `base_checkbox :: (_r: Rect, text: string, selected: bool, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State`
- `Checkbox_Theme :: struct`
- `Checkbox_State :: struct #type_info_none`
- `get_desired_aspect_ratio :: (theme: *Color_Animation_Editor_Theme) -> float`
- `color_animation_editor :: (r: Rect, theme: *Color_Animation_Editor_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (state: *Color_Animation_Editor_State)`
- `open_color_picker :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State, keyframe_index: int, bar_rect: Rect)`
- `open_time_view :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State, bar_rect: Rect)`
- `update_keyframe_color_due_to_open_color_picker :: (state: *Color_Animation_Editor_State)`
- `draw_color_picker :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State)`
- `draw_callback_color_picker :: (state: *Subwindow_State, r: Rect, data: *void)`
- `draw_time_view :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State)`
- `find_index_of_t :: (sorted: [] *Augmented_Keyframe, t: float) -> (index: int)`
- `draw_callback_time_view :: (state: *Subwindow_State, r: Rect, data: *void)`
- `draw_color_gradient_bar :: (area: Rect, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State, top_margin: float)`
- `get_sorted_keyframes :: (state: *Color_Animation_Editor_State, index_to_ignore := -1) -> [] *Augmented_Keyframe`
- `my_color_slider :: (rect: Rect, alpha: float, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State) -> (changed: bool)`
- `update_keyframes :: (rect: Rect, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State)`
- `draw_color_keyframe :: (index: int, rect: Rect, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State)`
- `copy_output_to_clipboard :: (state: *Color_Animation_Editor_State, output_type: Output_Type)`
- `init_color_by_interpolation :: (state: *Color_Animation_Editor_State, keyframe_index: int)`
- `get_interpolation :: (state: *Color_Animation_Editor_State) -> Interpolation`
- `set_interpolation :: (using state: *Color_Animation_Editor_State, interp: Interpolation)`
- `set_keyframes :: (state: *Color_Animation_Editor_State, frames: [] Color_Keyframe)`
- `get_keyframes :: (state: *Color_Animation_Editor_State) -> [] Color_Keyframe`
- `Color_Animation_Editor_Theme :: struct`
- `Color_Animation_Editor_State :: struct #type_info_none`
- `MAX_STASHED_COLORS :: 10`
- `Color_Pick_Mode :: enum`
- `get_desired_aspect_ratio :: (theme: *Color_Picker_Theme) -> float`
- `color_picker :: (r: Rect, input_and_output_color_rgb: *Vector3, theme: *Color_Picker_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (applied: bool, still_dragging: bool, state: *Color_Picker_State)`
- `set_current_color_hsv :: (state: *Color_Picker_State, hsv: Vector3, update_zoom_hue := true)`
- `set_original_and_current_color_rgb :: (state: *Color_Picker_State, rgb: Vector3)`
- `set_current_color_rgb :: (state: *Color_Picker_State, theme: *Color_Picker_Theme, rgb: Vector3, update_hsv := true, maintain_h := FLOAT32_INFINITY, update_disc_point := true)`
- `backsolve_disc_point_from_rgb :: (state: *Color_Picker_State, rgb: Vector3) -> Vector2`
- `EPSILON :: 0.0001`
- `backsolve_disc_point_from_hsv_warp :: (h: float, s: float, v: float) -> Vector2`
- `backsolve_disc_point_from_hsv :: (state: *Color_Picker_State, h: float, s: float, v: float) -> Vector2`
- `hsv_to_argb :: (hsv: Vector3) -> Vector4`
- `Color_Picker_Theme :: struct`
- `Color_Picker_State :: struct #type_info_none`
- `dropdown :: (r: Rect, choices: [] string, current_value_pointer: *s32, theme: *Dropdown_Theme = null, identifier: s64 = 0, z: s32 = 100, loc := #caller_location) -> changed: bool, Dropdown_State`
- `draw_dropdown_popup :: (_state: *void)`
- `dropdown_deactivate :: (widget: *Active_Widget)`
- `Dropdown_Theme :: struct`
- `Dropdown_State :: struct #type_info_none`
- `default_text_baseline :: (font: *Font, height: float) -> float`
- `text_origin :: (theme: *Label_Theme, font: *Font, r: Rect, text_width: float) -> (x: float, y: float)`
- `label_from_prepared_text :: (r: Rect, font: *Font, theme: *Label_Theme, do_scissor := true)`
- `label :: (r: Rect, text: string, theme: *Label_Theme, do_scissor := true) -> desired_width: float`
- `number_input :: (_r: Rect, text: string, value: *$T, min_value: T, max_value: T, theme: *Number_Input_Theme, identifier: s64 = 0, loc := #caller_location, parse_proc: (string) -> (T, bool) = null) -> bool`
- `Number_Input_Theme :: struct`
- `Number_Input_State :: struct #type_info_none`
- `begin_scrollable_region :: (r: Rect, theme: *Scrollable_Region_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (state: *Scrollable_Region_State, inside: Rect)`
- `end_scrollable_region :: (state: *Scrollable_Region_State, max_x: float, min_y: float, scroll_value: *float)`
- `Scrollable_Region_Flags :: enum_flags u8`
- `Scrollable_Region_Theme :: struct`
- `Scrollable_Region_State :: struct #type_info_none`
- `slidable_region :: (r: Rect, theme: *Slidable_Region_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (state: *Slidable_Region_State, left_or_top_rect: Rect, right_or_bottom_rect: Rect, divider_rect: Rect)`
- `switch_to_bottom_rect :: switch_to_second_rect`
- `switch_to_right_rect :: switch_to_second_rect`
- `switch_to_second_rect :: (state: *Slidable_Region_State)`
- `switch_to_top_rect :: switch_to_first_rect`
- `switch_to_left_rect :: switch_to_first_rect`
- `switch_to_first_rect :: (state: *Slidable_Region_State)`
- `finish :: (state: *Slidable_Region_State)`
- `Slidable_Region_Theme :: struct`
- `Slidable_Region_State :: struct #type_info_none`
- `slider :: (_r: Rect, value: *$T, min_value: T, max_value: T, spinbox_step: T, theme: *Slider_Theme, prefix := "", suffix := "", identifier: s64 = 0, loc := #caller_location) -> changed: bool, entered: bool`
- `set_initial_value :: (state: *Slider_State, value: $T)`
- `get_initial_value :: (state: *Slider_State, $T: Type) -> T`
- `set_and_maybe_clamp :: (new_value: $ValueType, pointer: *$T, min_value: T, max_value: T, clamp_low: bool, clamp_high: bool)`
- `to_vec3 :: (v2: Vector2) -> Vector3`
- `draw_arrow :: (state: *Button_State, p4: Vector3, p5: Vector3, p6: Vector3, theme: *Button_Theme)`
- `slider_draw_background_default :: (r: Rect, theme: *Slider_Theme, state: *Slider_State)`
- `slider_draw_foreground_default :: (r_background: Rect, r_foreground: Rect, theme: *Slider_Theme, state: *Slider_State)`
- `Slider_Format_Text_Mode :: enum u8`
- `slider_format_text_float_default :: (prefix: string, suffix: string, value: float64, theme: *Slider_Theme, state: *Slider_State, mode: Slider_Format_Text_Mode) -> string`
- `slider_format_text_int_default :: (prefix: string, suffix: string, value: s64, theme: *Slider_Theme, state: *Slider_State, mode: Slider_Format_Text_Mode) -> string`
- `slider_parse_text_float_default :: (text: string, theme: *Slider_Theme, state: *Slider_State) -> (success: bool, value: float64)`
- `slider_parse_text_int_default :: (text: string, theme: *Slider_Theme, state: *Slider_State) -> (success: bool, value: s64)`
- `Slider_Theme :: struct`
- `Slider_State :: struct #type_info_none`
- `add_subwindow :: (info: *Subwindow_Info, theme: *Subwindow_Theme = null, identifier: s64 = 0, loc := #caller_location, z: s32 = 0) -> (state: *Subwindow_State)`
- `draw_subwindow :: (state: *Subwindow_State)`
- `subwindow_rect_from_content_rect :: (content_rect: Rect, theme: *Subwindow_Theme, override_render_target_height : s32 = 0) -> Rect`
- `get_title_bar_height :: (theme: *Subwindow_Theme, override_render_target_height : s32 = 0) -> float`
- `maybe_diagonalize :: (flag: Resize_Controls, x: float, w: float, margin: float, low: Resize_Controls, high: Resize_Controls, mouse: float) -> Resize_Controls`
- `raise :: (state: *Subwindow_State)`
- `update_dragging :: (state: *Subwindow_State, full: *Rect)`
- `start_resizing :: (state: *Subwindow_State, f: Rect)`
- `adjust_resize_control :: (theme: *Subwindow_Theme, state: *Subwindow_State, f: Resize_Controls) -> Resize_Controls`
- `get_pointer_from_resize_control :: (flags: Resize_Controls) -> Pointer_Image`
- `get_aspect :: (r: Rect) -> float`
- `rescale :: (r: Rect, old_w: s32, old_h: s32) -> Rect`
- `rescale :: (v: Vector2, old_w: s32, old_h: s32) -> Vector2`
- `draw_resize_corner :: (theme: *Subwindow_Theme, state: *Subwindow_State, r: Rect)`
- `draw_proc_adjust_y1 :: (state: *Subwindow_State, new_y1: float)`
- `Subwindow_Flags :: enum_flags u16`
- `Resize_Controls :: enum_flags u16`
- `Subwindow_Theme :: struct`
- `Subwindow_Info :: struct`
- `Subwindow_State :: struct #type_info_none`
- `my_draw_text :: (font: *Font, x: s64, y: s64, text: string, color: Vector4, effects: Font_Effects = 0)`
- `clamp_insert_point :: (state: *Text_Input_State)`
- `Text_Input_Action :: enum_flags u8`
- `Text_Result_Action :: enum_flags u32`
- `set_last_inert_text :: (state: *Text_Input_State, text: string)`
- `text_input :: (_r: Rect, inert_text: string, theme: *Text_Input_Theme, identifier: s64 = 0, loc := #caller_location, input_action : Text_Input_Action = 0, auto_complete: Auto_Complete = .{}) -> (result_action: Text_Result_Action, new_string: string, state: *Text_Input_State)`
- `text_input_serial_events :: (widget: *Active_Widget, events: [] Event)`
- `handle_event :: (state: *Text_Input_State, event: Event)`
- `text_input_check_for_activation :: (widget: *Active_Widget, event: Event) -> bool`
- `refresh_input_from_history :: (using state: *Text_Input_State)`
- `set_input :: (using state: *Text_Input_State, _s: string)`
- `activate :: (using state: *Text_Input_State)`
- `start_selecting :: (using state: *Text_Input_State)`
- `get_result :: (using state: *Text_Input_State) -> string`
- `reset :: (using state: *Text_Input_State)`
- `draw_text_input_background :: (r: Rect, theme: *Text_Input_Theme, state: *Text_Input_State, active := false, changed := false)`
- `draw :: (using state: *Text_Input_State, r: Rect, theme: *Text_Input_Theme = null, identifier: s64 = 0, loc := #caller_location)`
- `sanitize :: (s: string) -> string`
- `seek_left_one_utf8_character :: (s: string, _point: int) -> int`
- `seek_left_one_word :: (s: string, point: int) -> int`
- `seek_right_one_word :: (s: string, point: int) -> int`
- `get_y_range :: (font: *Font, text_y: float) -> (y0: float, y1: float)`
- `Selection :: struct`
- `deselect :: (using selection: *Selection)`
- `selection_was_modified :: (using selection: *Selection)`
- `Auto_Complete :: struct`
- `break_into_lines :: (s: string, between_lines_string := "") -> [] string`
- `get_selection_endpoints :: (font: *Font, text: string, effect: Font_Effects, text_x: float, start_character: int, end_character: int) -> (x0: float, x1: float)`
- `get_insert_point_for_cursor :: (state: *Text_Input_State, font: *Font, text: string, current_frame_text_x: float, cursor_x: int, effect: Font_Effects, glyphs_prepped := false) -> int`
- `clear_text :: (using state: *Text_Input_State)`
- `set_text :: (state: *Text_Input_State, s: string, replace_pre_modification_text := true)`
- `append_text :: (using state: *Text_Input_State, s: string)`
- `insert_text :: (using state: *Text_Input_State, s: string) -> bool`
- `select_all :: (using state: *Text_Input_State)`
- `copy_selection_to_clipboard :: (using state: *Text_Input_State, cut := false)`
- `insert_clipboard_at_cursor :: (using state: *Text_Input_State)`
- `move_cursor_left :: (using state: *Text_Input_State, want_selection := false)`
- `move_cursor_right :: (state: *Text_Input_State, want_selection := false)`
- `move_cursor_left_by_word :: (using state: *Text_Input_State, want_selection := false)`
- `move_cursor_right_by_word :: (using state: *Text_Input_State, want_selection := false)`
- `move_cursor_to_beginning :: (state: *Text_Input_State, want_selection := false)`
- `move_cursor_to_end :: (state: *Text_Input_State, want_selection := false)`
- `navigate_history_backward :: (using state: *Text_Input_State)`
- `navigate_history_forward :: (using state: *Text_Input_State)`
- `backspace_from_cursor :: (using state: *Text_Input_State)`
- `delete_from_cursor :: (using state: *Text_Input_State)`
- `press_enter :: (using state: *Text_Input_State, shift_pressed := false)`
- `press_tab :: (using state: *Text_Input_State, shift_pressed := false)`
- `press_escape :: (using state: *Text_Input_State)`
- `text_input_deactivate :: (widget: *Active_Widget)`
- `Text_Input_Theme :: struct`
- `Text_Input_State :: struct #type_info_none`

- `default_overall_theme: Overall_Theme`
- `draw_procs: Draw_Procs`
- `occluder_being_drawn: *Active_Widget`
- `occlusions_current_frame: [..] Occlusion_Record`
- `occlusions_next_frame: [..] Occlusion_Record`
- `pointers: [NUM_POINTERS] Windows.HCURSOR`
- `pointers: [NUM_POINTERS] *NSCursor`
- `state_table: Table(u64, *Active_Widget)`

## Public type members

#### Contextual type names (not GetRect exports)

GetRect's private module scope binds the types used by its exported signatures
and `Draw_Procs` callbacks. With the default `Type_Indicator = void`, those
names resolve to `Simp.Dynamic_Font`, `Simp.Font_Effects`, `Simp.Texture`,
and `Window_Creation.Window_Type`, respectively. With a non-void
`Type_Indicator`, they resolve to its `Font`, `Font_Effects`, `Texture`, and
`Window_Type` members. These aliases provide the type context for the public
GetRect API; they are not standalone GetRect exports. The default font and
texture shapes are specified by the Simp page. No fields are prescribed for
the alternate indicator's contextual types.

#### `Draw_Procs` (struct)

- `texture_load_from_memory: (texture: *Texture, memory: [] u8, srgb: bool, build_mipmaps: bool) -> bool`
- `set_scissor:   (x0: s32, y0: s32, x1: s32, y1: s32)`
- `clear_scissor: ()`
- `set_shader_for_color:  (enable_blend := false)`
- `set_shader_for_images: (texture: *Texture)`
- `prepare_text:       (font: *Font, text: string, effects: Font_Effects = 0) -> s64`
- `draw_prepared_text: (font: *Font, x: s64, y: s64, text_color: Vector4, effects: Font_Effects = 0)`
- `get_mouse_pointer_position: (window: Window_Type, right_handed: bool) -> (x: int, y: int, success: bool)`
- `set_pointer_image: (image: Pointer_Image, widget: *Active_Widget)`
- `get_font_at_size:   (memory: [] u8, pixel_height: int) -> *Font`
- `immediate_triangle:  (p0: Vector3, p1: Vector3, p2: Vector3, c0 := Vector4.{1,1,1,1}, c1 := Vector4.{1,1,1,1}, c2 := Vector4.{1,1,1,1}, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{z=1})`
- `immediate_quad:  (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{1,1,1,1}, uv0 := Vector2.{0,0}, uv1 := Vector2.{1,0}, uv2 := Vector2.{1,1}, uv3 := Vector2.{0, 1})`
- `immediate_flush: ()`

#### `Status_Flags` (enum_flags)

- `OVER     :: 0x01`
- `FOCUS    :: 0x02`
- `DOWN     :: 0x04`
- `PRESSED  :: 0x08`

#### `Text_Alignment` (enum)

- `Left   :: 0`
- `Center :: 1`
- `Right  :: 2`

#### `Rect` (struct)

- `x, y: float`
- `w, h: float`

#### `Status_Flags_Ignore` (enum_flags u8)

- `NOTHING`
- `LEFT_CLICK`
- `TOOLTIP`
- `SCISSOR`

#### `VBase3` (struct)

- `x, y, z: float`

#### `Occlusion_Record` (struct)

- `serial: s32`
- `rect:   Rect`
- `widget: *Active_Widget`

#### `Pointer_Image` (enum u16)

- `NORMAL    :: 0`
- `PRESSABLE :: 1`
- `LOCKED    :: 2`
- `DRAGGING_HORIZONTAL   :: 3`
- `DRAGGING_VERTICAL     :: 4`
- `DRAGGING_FREE         :: 5`
- `DRAGGING_NW_SE        :: 6`
- `DRAGGING_NE_SW        :: 7`

#### `Size_Computation` (enum)

- `RELATIVE_TO_HEIGHT        :: 0`
- `RELATIVE_TO_WIDTH         :: 1`
- `ABSOLUTE_FROM_THEME_FIELD :: 2`
- `ABSOLUTE_FROM_GLOBAL      :: 3`

#### `Default_Themes` (enum)

- `Default`
- `Grayscale`
- `Bentonite`
- `Nimbus`
- `Blood_Vampire`
- `Freddie_Freeloader`
- `HIGH_CONTRAST`

#### `Rectangle_Shape` (struct)

- `roundedness_type: Size_Computation`
- `roundedness := .18`
- `pixels_per_edge_segment  := 10.0`
- `frame_thickness_type: Size_Computation`
- `frame_thickness := .02`
- `Rounding_Flags :: enum_flags u32 { NORTHEAST :: 0x1; NORTHWEST :: 0x2; SOUTHWEST :: 0x4; SOUTHEAST :: 0x8; EAST :: NORTHEAST | SOUTHEAST; WEST :: NORTHWEST | SOUTHWEST; SOUTH :: SOUTHEAST | SOUTHWEST; NORTH :: NORTHEAST | NORTHWEST; ALL :: EAST | WEST; }`
- `rounding_flags := Rounding_Flags.NORTHEAST | .NORTHWEST | .SOUTHWEST | .SOUTHEAST`

#### `Label_Theme` (struct)

- `font:    *Font`
- `effects: Font_Effects`
- `text_color                      := Vector4.{1, 1, 1, 1}`
- `alignment                       := Text_Alignment.Center`
- `alignment_pad_ems               := 1.0`
- `text_baseline_vertical_position := FLOAT32_INFINITY`

#### `Background_Theme` (struct)

- `shape: Rectangle_Shape`
- `color       := Vector4.{.08, .08, .08, 1.0}`
- `frame_color := Vector4.{.25, .25, .22, 1.0}`

#### `Margin` (enum_flags)

- `Horizontal :: 0x1`
- `Vertical   :: 0x2`

#### `Overall_Theme` (struct)

- `button_theme:            Button_Theme`
- `checkbox_theme:          Checkbox_Theme`
- `color_picker_theme:      Color_Picker_Theme`
- `dropdown_theme:          Dropdown_Theme`
- `label_theme:             Label_Theme`
- `scrollable_region_theme: Scrollable_Region_Theme`
- `slidable_region_theme:   Slidable_Region_Theme`
- `slider_theme:            Slider_Theme`
- `subwindow_theme:         Subwindow_Theme`
- `text_input_theme:        Text_Input_Theme`
- `color_animation_editor_theme:  Color_Animation_Editor_Theme`
- `font: *Font`
- `background_color        := Vector4.{.10, .10, .10, 1.0}`
- `background_color_bright := Vector4.{.15, .15, .15, 1.0}`
- `text_color              := Vector4.{.80, .80, .80, 1.0}`
- `title_bar_color         := Vector4.{.80, .30, .30, 1.0}`
- `title_bar_color_hilit   := Vector4.{.95, .50, .50, 1.0}`
- `border_width       := .004`
- `outer_margin_size  := .007`
- `outer_margin_flags := Margin.Horizontal | .Vertical`
- `scrollable_section_margin_width := .007`
- `scrollable_section_margin_flags := Margin.Horizontal | .Vertical`
- `alpha_scale := 1.0`

#### `Tooltip` (struct)

- `text: string`
- `pointer_data: *void`
- `int_data:     s64`

#### `Button_Theme` (struct)

- `using label_theme: Label_Theme;`

- `rectangle_shape:   Rectangle_Shape`
- `surface_color            := Vector4.{.08, .08, .08, 1.0}`
- `surface_color_over       := Vector4.{.15, .15, .20, 1.0}`
- `surface_color_flash      := Vector4.{.60, .77, .62, 1.0}`
- `surface_color_down       := Vector4.{.28, .28, .38, 1.0}`
- `text_color_over          := Vector4.{.95, .95, .95, 1.0}`
- `text_color_pressed       := Vector4.{.99, .99, .99, 1.0}`
- `text_offset_down         := Vector2.{.12, -.12}`
- `frame_thickness_over     := .04`
- `frame_thickness_down     := .05`
- `frame_thickness_flash    := .07`
- `enable_variable_frame_thickness := false`
- `frame_color              := Vector4.{.40, .40, .50, 1.0}`
- `frame_color_over         := Vector4.{.55, .55, .45, 1.0}`
- `frame_color_flash        := Vector4.{.99, .99, .65, 1.0}`
- `frame_color_down         := Vector4.{.75, .75, .55, 1.0}`
- `press_duration       := 0.7`
- `over_fade_in_speed   := 10.0`
- `over_fade_out_speed  := 4.0`
- `down_fade_in_speed   := 12.0`
- `down_fade_out_speed  := 4.0`
- `highlight_lighten_parameter := .15`
- `highlight_darken_parameter  := .6`
- `alpha_scale := 1.0`
- `texture_scale := 1.0`
- `draw_callback: (r: Rect, text: string, theme: *Button_Theme, texture: *Texture, state: *Button_State)`
- `properties: enum_flags u64 { TOGGLE :: 0x1; TEXTURE_LEFT_HANDED :: 0x2; DO_NOT_DRAW :: 0x4; } = .TEXTURE_LEFT_HANDED`

#### `Button_State` (struct)

- `#as using widget: Active_Widget`
- `hash:    u64`
- `status: Status_Flags`
- `rect:   Rect`
- `frame_pixels: float`
- `over_duration := 0.0`
- `over_effect_t := 0.0`
- `down_duration := 0.0`
- `down_effect_t := 0.0`
- `action_duration   := -1.0`
- `action_duration_2 := -1.0`
- `over_factor    := 0.0`
- `pressed_factor := 0.0`
- `pressed := false`
- `user_data_float: float`
- `user_data_s64:   s64`

#### `Checkbox_Theme` (struct)

- `using button_theme: Button_Theme;`

- `button_color_selected   := Vector4.{.90, .90, .90, 1.0}`
- `text_color_selected     := Vector4.{.90, .60, .90, 1.0}`
- `is_radio_button := false`

#### `Checkbox_State` (struct)

- `#as using widget: Active_Widget`
- `base: Button_State`
- `selected_t := 0.0`

#### `Color_Animation_Editor_Theme` (struct)

- `horizontal_padding  := .015`
- `vertical_padding    := .015`
- `keyframe_color_inactive := Vector4.{.35, .35, .35, 1}`
- `keyframe_color_active   := Vector4.{.94, .94, .94, 1}`
- `keyframe_color_picking  := Vector4.{.87, .87, .87, 1}`
- `keyframe_color_backing  := Vector4.{.05, .05, .05, 1}`
- `keyframe_width_relative_to_bar_height_nondragging := 0.10`
- `keyframe_scale_when_dragging      := 1.5`
- `keyframe_scale_when_color_picking := 1.3`
- `keyframe_selection_size_relative_to_visual_size := 1.4`
- `keyframe_inset_ratio   := 0.8`
- `keyframe_backing_ratio := 0.24`
- `background: Background_Theme`
- `color_picker_theme:           Color_Picker_Theme`
- `color_picker_subwindow_theme: Subwindow_Theme`
- `time_view_subwindow_theme:    Subwindow_Theme`
- `button_theme:                 Button_Theme`
- `label_theme:                  Label_Theme`
- `number_input_theme:           Number_Input_Theme`
- `dropdown_theme:               Dropdown_Theme`
- `checkbox_theme:               Checkbox_Theme`
- `default_interpolation := Interpolation.{.OKLAB, .HERMITE, false}`
- `default_num_clipboard_samples : s32 = 16`
- `default_keyframes: []Color_Keyframe = .[.{.{0.926563, 0.832695, 0.841836, 1}, 0.505263}, .{.{0.828598, 0.029386, 0.579748, 1}, 0.997368}, .{.{1, 0.33835, 0, 1}, 0.003289}, ]`
- `default_time_view_period := 3.0`
- `Draw_Subwidgets_Flags :: enum_flags u8 { INTERPOLATION_CONTROLS :: 0x01; TIME_VIEW :: 0x02; SAMPLING_CONTROLS :: 0x04; COPY_TO_CLIPBOARD :: 0x08; }`
- `DRAW_ALL_SUBWIDGETS :: Draw_Subwidgets_Flags.INTERPOLATION_CONTROLS | .TIME_VIEW | .SAMPLING_CONTROLS | .COPY_TO_CLIPBOARD`
- `draw_subwidgets_flags := DRAW_ALL_SUBWIDGETS`

#### `Color_Animation_Editor_State` (struct)

- `#as using widget: Active_Widget`
- `ring_point     := Vector2.{1, 0}`
- `disc_point     := Vector2.{1, 0}`
- `dragging_in_strip:       bool`
- `dragging_keyframe_index: int`
- `color_picker_keyframe_index: int`
- `color_picker_subwindow_info: Subwindow_Info`
- `color_picker_subwindow_info.open = false`
- `time_view_subwindow_info: Subwindow_Info`
- `time_view_subwindow_info.open = false`
- `color_picker_editing_color_applied: Vector3`
- `color_picker_subwindow_rect_initialized := false`
- `theme: *Color_Animation_Editor_Theme`
- `last_color_picker_state: *Color_Picker_State`
- `color_samples: [..] Vector4`
- `num_clipboard_samples: s32`
- `interpolation_color_space_s32: s32`
- `interpolation_geometry_s32:    s32`
- `interpolation_wrapping :=      false`
- `output_type_s32: s32 = xx Output_Type.RESAMPLED_ARRAY`
- `time_view_period: float`
- `time_view_playing := true`
- `time_view_current_time: float64`
- `color_keyframes: [..] Augmented_Keyframe`

#### `Color_Pick_Mode` (enum)

- `HSV_CIRCLE  :: 0`
- `HSV_SLIDERS :: 1`
- `RGB_SLIDERS :: 2`
- `TYPE_INS    :: 3`

#### `Color_Picker_Theme` (struct)

- `horizontal_padding  := .06`
- `vertical_padding    := .06`
- `stashed_color_width := .10`
- `margin_between_color_input_and_stashed_colors := .04`
- `background: Background_Theme`
- `outer_circle_diameter := .99`
- `inner_circle_diameter := .73`
- `inner_disc_radius     := .95`
- `initial_stashed_colors: [] Vector3 = Vector3.[`
- `apply_and_revert_button_theme: Button_Theme`
- `apply_string  := "Apply"`
- `revert_string := "Revert"`
- `slider_hue_min := -50.0`
- `slider_hue_max := 345.0`
- `hue_range_in_zoomed_slider := 60.0`
- `stash_button_theme:       Button_Theme`
- `mode_switch_button_theme: Button_Theme`
- `type_ins_label_theme: Label_Theme`
- `type_ins_input_theme: Number_Input_Theme`
- `stashed_colors_shape: Rectangle_Shape`
- `starting_color_pick_mode := Color_Pick_Mode.HSV_CIRCLE`

#### `Color_Picker_State` (struct)

- `#as using widget: Active_Widget`
- `ring_point     := Vector2.{1, 0}`
- `disc_point     := Vector2.{1, 0}`
- `dragging_in_ring:    bool`
- `dragging_in_disc:    bool`
- `dragging_in_h: bool`
- `dragging_in_h_zoom: bool`
- `dragging_in_s: bool`
- `dragging_in_l: bool`
- `dragging_in_r: bool`
- `dragging_in_g: bool`
- `dragging_in_b: bool`
- `dragging_in_hsv_rectangle: bool`
- `dragging_in_ring_t: float`
- `dragging_in_disc_t: float`
- `dragging_in_h_t: float`
- `dragging_in_h_zoom_t: float`
- `dragging_in_s_t: float`
- `dragging_in_l_t: float`
- `dragging_in_r_t: float`
- `dragging_in_g_t: float`
- `dragging_in_b_t: float`
- `dragging_in_hsv_rectangle_t: float`
- `toggled_s: bool`
- `toggled_v: bool`
- `toggled_s_t: float`
- `toggled_v_t: float`
- `hsv_picker_in_rectangle_mode := true`
- `circle_picker_in_warped_mode := false`
- `zoom_hue_center := 0.0`
- `color_pick_mode := Color_Pick_Mode.HSV_CIRCLE`
- `current_hsv_color: Vector3`
- `current_rgb_color: Vector3`
- `original_rgb_color: Vector3`
- `stashed_colors: [..] Vector3`

#### `Dropdown_Theme` (struct)

- `theme_for_current_value:  Button_Theme`
- `theme_for_each_choice:    Button_Theme`
- `theme_for_current_choice: Button_Theme`
- `dropdown_indicator_aspect_ratio := 1.0`
- `arrow_flip_up_rate   := 2.5`
- `arrow_flip_down_rate := 3.9`

#### `Dropdown_State` (struct)

- `#as using widget: Active_Widget`
- `open := false`
- `open_t := 0.0`
- `status : Status_Flags`
- `hash:      u64`
- `base_rect: Rect`
- `choices:   [] string`
- `theme:     Dropdown_Theme = ---`
- `loc:       Source_Code_Location`
- `z:         s32`
- `current_value_pointer: *s32`
- `current_value_state:   *Button_State`

#### `Number_Input_Theme` (struct)

- `using text_input_theme: Text_Input_Theme;`

- `clamp_low  := true`
- `clamp_high := true`

#### `Number_Input_State` (struct)

- `#as using widget: Active_Widget`
- `text_input_state: Text_Input_State`
- `inputting_text := false`

#### `Scrollable_Region_Flags` (enum_flags)

- `VERTICAL :: 1`
- `HORIZONTAL :: 2`

#### `Scrollable_Region_Theme` (struct)

- `direction := Scrollable_Region_Flags.VERTICAL`
- `horizontal_margin_size := 0.005`
- `vertical_margin_size   := 0.005`
- `margin_flags := Margin.Horizontal | .Vertical`
- `scrollbar_size := 0.03`
- `minimum_nib_size := 1.0`
- `nib_margin := 0.10`
- `scrollbar_nib_theme: Button_Theme`
- `region_background: Background_Theme`
- `scrollbar_background: Background_Theme`
- `mouse_wheel_increment := 0.0004`

#### `Scrollable_Region_State` (struct)

- `#as using widget: Active_Widget`
- `nib_state:  Button_State`
- `outer_rect: Rect`
- `inner_rect: Rect`
- `bar_rect:   Rect`
- `dragging    := false`
- `teleporting := false`
- `theme: Scrollable_Region_Theme`

#### `Slidable_Region_Theme` (struct)

- `orientation: enum u8 { HORIZONTAL :: 0; VERTICAL :: 1; }`
- `divider_theme: Button_Theme`
- `divider_thickness_type := Size_Computation.ABSOLUTE_FROM_GLOBAL`
- `divider_thickness      := .02`
- `initial_divider_position_type := Size_Computation.RELATIVE_TO_HEIGHT`
- `initial_divider_position      := 0.5`
- `first_background:  Background_Theme`
- `second_background: Background_Theme`

#### `Slidable_Region_State` (struct)

- `#as using widget: Active_Widget`
- `divider_state:  Button_State`
- `left_or_top_rect:     Rect`
- `right_or_bottom_rect: Rect`
- `divider_rect:         Rect`
- `theme:                *Slidable_Region_Theme`
- `divider_t:              float`
- `divider_over_factor:    float`
- `divider_pressed_factor: float`
- `dragging    := false`

#### `Slider_Format_Text_Mode` (enum)

- `DISPLAY :: 0`
- `EDITING :: 1`

#### `Slider_Theme` (struct)

- `foreground: Button_Theme`
- `decimals: s32 = 2`
- `background: Button_Theme`
- `surface_margin   := .15`
- `text_editable    := true`
- `clamp_text_input_low  := true`
- `clamp_text_input_high := true`
- `sliding_is_relative   := false`
- `sliding_becomes_relative_by_pressing_shift := true`
- `relative_slide_factor          := 0.7`
- `relative_slide_factor_modified := 0.2`
- `do_clamp_if_we_are_relative_dragging_but_started_outside_the_clamp_range := false`
- `text_input_theme: Text_Input_Theme`
- `use_spinboxes        := true`
- `spinbox_button_width := 1.0`
- `surface_style: enum u8 { EXTEND_FROM_LEFT :: 0; NUB :: 1; }`
- `nub_thickness_type := Size_Computation.RELATIVE_TO_HEIGHT`
- `nub_thickness      := 1.2`
- `spinbox_theme:      Button_Theme`
- `draw_background := slider_draw_background_default`
- `draw_foreground := slider_draw_foreground_default`
- `format_text_float := slider_format_text_float_default`
- `format_text_int   := slider_format_text_int_default`
- `parse_text_float := slider_parse_text_float_default`
- `parse_text_int   := slider_parse_text_int_default`

#### `Slider_State` (struct)

- `#as using widget: Active_Widget`
- `background: Button_State`
- `foreground: Button_State`
- Anonymous union containing `initial_value_float32: float32`, `initial_value_float64: float64`, and `initial_value_int: int`; only the member matching the slider's value type is active.
- `sliding_t: float`
- `slowing_t: float`
- `accumulated_delta_for_int_slider := 0.0`
- `sliding             := false`
- `sliding_is_relative := false`
- `inputting_text      := false`
- `value_type: *Type_Info`
- `current_slide_setting: float`
- `surface_margin_this_frame_in_pixels: float`
- `changed_this_frame: bool`

#### `Subwindow_Flags` (enum_flags)

- `TITLE_BAR    ::          0x1`
- `CLOSE_BUTTON ::          0x2`
- `FRINGE       ::          0x4`
- `VISIBLE_RESIZE_CORNER :: 0x8`

#### `Resize_Controls` (enum_flags)

- `EAST         :: 0x1`
- `NORTH        :: 0x2`
- `WEST         :: 0x4`
- `SOUTH        :: 0x8`
- `CORNER_SE    :: 0x10`
- `PROPORTIONAL :: 0x20`
- `SNAP_EW      :: 0x40`
- `SNAP_NS      :: 0x80`
- `NS   :: NORTH | SOUTH`
- `EW   :: EAST  | WEST`
- `NE   :: NORTH | EAST`
- `NW   :: NORTH | WEST`
- `SE   :: SOUTH | EAST`
- `SW   :: SOUTH | WEST`
- `ENWS :: NS | EW`
- `SNAP :: SNAP_EW | SNAP_NS`

#### `Subwindow_Theme` (struct)

- `title_bar_size := 0.03`
- `title_bar: Button_Theme`
- `close_button: Button_Theme`
- `close_button_width        := 1.2`
- `close_button_height       := 0.8`
- `close_button_right_margin := 0.4`
- `close_button_x_height     := 0.7`
- `region_background: Background_Theme`
- `horizontal_margin_size := 0.005`
- `vertical_margin_size   := 0.005`
- `subwindow_flags := Subwindow_Flags.TITLE_BAR | .FRINGE | .CLOSE_BUTTON | .VISIBLE_RESIZE_CORNER`
- `fringe_color := Vector4.{0, 0, 0, .2}`
- `fringe_thickness := 0.004`
- `resize_corner_line_thickness := 0.025`
- `resize_corner_color          := Vector4.{.4, .4, .4, 1}`
- `drag_intrusion_east  := 0.006`
- `drag_extrusion_east  := 0.008`
- `drag_intrusion_west  := 0.006`
- `drag_extrusion_west  := 0.008`
- `drag_intrusion_north := 0.002`
- `drag_extrusion_north := 0.011`
- `drag_intrusion_south := 0.006`
- `drag_extrusion_south := 0.008`
- `drag_corner_margin   := 0.05`
- `resize_controls := Resize_Controls.CORNER_SE | .EAST | .NORTH | .WEST | .SOUTH`
- `resize_rect_size := 0.05`
- `resize_minimum_size := Vector2.{.08, .08}`
- `resize_snap_margin := 0.02`

#### `Subwindow_Info` (struct)

- `rect: Rect`
- `title_text: string`
- `fixed_aspect_ratio := -1.0`
- `snap_to_size: Vector2`
- `user_data: *void`
- `draw: (state: *Subwindow_State, content_rect: Rect, user_data: *void)`
- `open := true`
- `wants_a_raise := false`

#### `Subwindow_State` (struct)

- `#as using widget: Active_Widget`
- `base: Button_State`
- `title_bar_rect: Rect`
- `content_rect:   Rect`
- `info: *Subwindow_Info`
- `wants_draw    := false`
- `wants_a_raise := false`
- `hash:           u64`
- `render_target_width:  s32`
- `render_target_height: s32`
- `dragging: enum u8 { TITLE :: 0x1; RESIZE :: 0x2; }`
- `dragging_start:  Vector2`
- `dragging_last:   Vector2`
- `dragging_offset: Vector2`
- `dragging_resize_control: Resize_Controls`
- `resizing_aspect: float`
- `theme: Subwindow_Theme`
- `draw_proc_adjusted_y := false`
- `draw_proc_new_y1     := 0.0`

#### `Text_Input_Action` (enum_flags)

- `ACTIVATE   :: 0x1`
- `DEACTIVATE :: 0x2`

#### `Text_Result_Action` (enum_flags)

- `ENTERED       :: 0x1`
- `ESCAPED       :: 0x2`
- `TEXT_MODIFIED :: 0x4`

#### `Selection` (struct)

- `start_line       := -1`
- `start_character  : int`
- `end_line         := -1`
- `end_character    : int`
- `active := false`

#### `Auto_Complete` (struct)

- `proc: (str: string, data: *void) -> (results: [] string, longest_match_length: int)`
- `data: *void`

#### `Text_Input_Theme` (struct)

- `using button_theme: Button_Theme;`

- `text_color_autocomplete        := Vector4.{.30, .99, .30, 0.9}`
- `text_color_autocomplete_failed := Vector4.{.99, .55, .55, 0.9}`
- `selection_color         := Vector4.{.15, .69, .85, 1.0}`
- `cursor_color            := Vector4.{1.0, 1.0, 1.0, 1.0}`
- `text_insert_margin         := 4.7`
- `cursor_width_inside_text   := 0.11`
- `cursor_width_outside_text  := .55`
- `maintain_text_after_escape := false`
- `deactivate_on_enter        := true`
- `multiclick_delay_max         := 0.5`
- `distance_to_break_multiclick := 0.8`

#### `Text_Input_State` (struct)

- `#as using widget: Active_Widget`
- `base: Button_State`
- `rect: Rect`
- `camera_initted  := false`
- `camera_x        := 0.0`
- `active_t        := 0.0`
- `active         := false`
- `disabled       := false`
- `shrunken_deadzone_margin := -1.0`
- `last_displayed_text_x    := FLOAT32_INFINITY`
- `last_keypress_time: float64`
- `last_inert_text := ""`
- `last_click_time       : float64`
- `current_click_time    : float64 = -1`
- `num_consecutive_clicks : s32`
- `last_click_position: Vector2`
- `click_position:      Vector2`
- `insert_point := 0`
- `cursor_tracking_mouse := false`
- `did_initial_mouse_click_so_check_for_deadzone_change := false`
- `pending_result_actions: Text_Result_Action`
- `last_saved_input: string`
- `pre_modification_text: string`
- `command_history: [..] string`
- `command_history_index := 0`
- `selection: Selection`
- `buffered_events: [..] Event`
- `text: string`
- `MAX_BUFFER_SIZE :: 8000`
- `input_buffer: [MAX_BUFFER_SIZE] u8`
- `auto_complete : Auto_Complete`
- `tab_pressed := false`
- `completion_change_time: float64`
- `match_array     : [] string`
- `match_length    : int`
- `match_selection : int`
- `longest_match   : int = -1`
- `do_active_widget_deactivate_all := false`

## Reachable field-type shapes

These declarations are not independent module exports: the first is module-scoped and the latter two are file-scoped in the originating declaration context. Their nominal relationships and fields are nevertheless required to describe public procedure signatures and public aggregate fields that refer to them.

#### `Active_Widget` (contextual struct)

```jai
Active_Widget :: struct {
    widget_type: Type;
    deactivate\         _proc: (w: *Active_Widget);
    serial_events\      _proc: (w: *Active_Widget, events: [] Event);
    check_for_activation_proc: (w: *Active_Widget, event: Event) -> bool;
    last_getrect_frame: u64;
    is_on_state_stack := false;
    last_update_time: float64;
    cosmetic_dt: float32;
    window: Window_Type;
    exists_on_occluder: *Active_Widget;
}
```

#### `Augmented_Keyframe` (contextual struct)

```jai
Augmented_Keyframe :: struct {
    using base: Color_Keyframe;
    active: bool;
    active_t: float;
    dragging_t: float;
    color_picking_t: float;
    pos_x: float;
}
```

#### `Output_Type` (contextual enum)

```jai
Output_Type :: enum {
    RESAMPLED_ARRAY :: 0;
    SOURCE_KEYFRAMES :: 1;
}
```

`NUM_POINTERS` is module-scoped and evaluates to `8`, the number of `Pointer_Image` values. The platform cursor array therefore has eight entries.

## Behavioral contract and constraints

GetRect provides immediate-mode UI layout, drawing, widget, input, and popup helpers. Applications submit widget/layout operations during frames and provide platform drawing/input integration. UI state is stored per widget identity and persists between frames; active widgets receive serial input events and a deactivation callback. Frame update and input/event collection must run before drawing/querying widgets for that frame. Window/event integration and draw-procedure registration are caller supplied; the module does not own the native window.

`ui_per_frame_update` advances the module frame, clears per-frame tooltip/click state, updates elapsed time, deactivates an active widget that was not drawn in the preceding frame, and refreshes occlusion/event state for the specified window. Its returned pair reports whether the window is occluded by a background occluder and whether a widget is active. A widget state pointer must remain valid while it is registered as active or present on the state stack; deactivation and removal are the lifecycle hooks for releasing per-widget state. The module does not define thread safety.

The `cut_left`, `cut_right`, `cut_top`, and `cut_bottom` helpers do not clamp sizes. A requested `amount` larger than the available axis can produce a negative remainder dimension; negative amounts and margins are likewise applied arithmetically. Each result consumes `amount` plus half the margin from the remainder and gives the returned side rectangle `amount` minus half the margin, leaving a full `margin` gap. Callers that need nonnegative rectangles must validate or clamp inputs themselves.

`combine_hashes(a, b)` returns `a * 0x9e3779b97f4a7c15 + b` with `u64` wraparound. `get_hash` is deterministic for the same source location and identifier, but the complete location-hash construction is not a portable serialization contract.

`is_inside(x, y, rect)` includes all four edges (`x >= rect.x`, `x <= rect.x + rect.w`, and the corresponding inclusive y bounds). A rectangle with a negative width or height contains no point on that axis. Rounded-rectangle geometry rounds generated pixel coordinates with `floor(value + 0.5)`; exact half ties therefore round toward positive infinity, including for negative coordinates. The API does not globally snap every `Rect` operation to pixels.

The color picker and interpolation helpers represent HSV hue in degrees. Hue values are converted to/from the angular picker position in degrees; callers providing HSV values should not pass a normalized turn value without scaling it to degrees.

The color-animation editor’s keyframe data is the source representation; its sampled output is a separate representation. Sampling is available without drawing the editor. UI assets and theme-dependent artwork are caller-provided through the draw and theme interfaces; the contract does not promise embedded default image payloads. Text rendering and several platform integrations are marked incomplete, so unsupported facilities must not be assumed available. The coordinate and winding convention differs in the left-handed module variant.
