default_overall_theme: Overall_Theme;
draw_procs: Draw_Procs;
occluder_being_drawn: *Active_Widget;
occlusions_current_frame: [..] Occlusion_Record;
occlusions_next_frame: [..] Occlusion_Record;
state_table: Table(u64, *Active_Widget);

#if OS == .WINDOWS {
    pointers: [NUM_POINTERS] Windows.HCURSOR;
} else {
    pointers: [NUM_POINTERS] *NSCursor;
}

THEME_COUNT :: 7;
Theme_Proc :: #type () -> Overall_Theme;
default_theme_procs: [THEME_COUNT] Theme_Proc = .[theme_default, theme_grayscale, theme_bentonite, theme_nimbus, theme_blood, theme_freddie, theme_contrast];

set_global_rounding_radius_in_pixels :: (value: float) {
    rounding_pixels = value;
}

set_global_frame_thickness_in_pixels :: (value: float) {
    frame_pixels_global = value;
}

set_global_divider_thickness_in_pixels :: (value: float) {
    divider_pixels = value;
}

set_global_initial_divider_position_in_pixels :: (value: float) {
    divider_initial_pixels = value;
}

set_global_slider_nub_thickness_in_pixels :: (value: float) {
    nub_pixels = value;
}

set_default_theme :: (theme: Overall_Theme) {
    default_overall_theme = theme;
}

ui_init :: (default_font := "", procs: *Draw_Procs = null) {
    if procs != null draw_procs = procs.*;

    if default_font.count > 0 {
        assert(draw_procs.get_font_at_size != null, "font callback required");
        default_overall_theme.font = draw_procs.get_font_at_size(.{ default_font.count, default_font.data }, 16);
    }
}

ui_get_button_state :: (key: Key_Code) -> Key_Current_State {
    index := cast(int) key;
    assert(index >= 0 && index < button_states.count);
    return button_states[index];
}

windows_per_frame_update :: () {
    for *s: button_states s.* &= .DOWN;
    wheel_records.count = 0;
}

switch_window :: (window: Window_Type, _render_target_width: s32, _render_target_height: s32) -> (background_occluded: bool, is_active: bool) {
    current_window = window;
    render_width = _render_target_width;
    render_height = _render_target_height;
    mouse_valid = false;

    if draw_procs.get_mouse_pointer_position != null {
        x, y, success := draw_procs.get_mouse_pointer_position(window, !LEFT_HANDED);
        mouse_position = .{ cast(float) x, cast(float) y };
        mouse_valid = success;
    }

    occlusion_per_window_update(window);

    return events_occluded(), active_widgets.count > 0;
}

ui_per_frame_update :: (window: Window_Type, _render_target_width: s32, _render_target_height: s32, _current_time: float64) -> (background_occluded: bool, is_active: bool) {
    frame_number += 1;
    dt = cast(float) max(_current_time - current_time, 0.0);
    current_time = _current_time;
    active_tooltip = .{};
    click_consumed = false;
    windows_per_frame_update();
    active_widgets_per_frame_update();
    occlusion_per_frame_update();

    return switch_window(window, _render_target_width, _render_target_height);
}

get_status_flags :: (r: Rect, widget_if_querying_outside_of_render_time: *Active_Widget = null, ignore := Status_Flags_Ignore.NOTHING) -> Status_Flags {
    result: Status_Flags;
    if !mouse_valid || !is_inside(mouse_position.x, mouse_position.y, r) return result;
    if (ignore & .SCISSOR) == 0 && !is_visible_in_scissor(mouse_position.x, mouse_position.y) return result;
    if events_occluded(widget_if_querying_outside_of_render_time) return result;

    result |= .OVER | .FOCUS;

    if (ignore & .LEFT_CLICK) == 0 && !click_consumed {
        b := ui_get_button_state(.MOUSE_BUTTON_LEFT);
        if (b & .DOWN) != 0 result |= .DOWN;
        if (b & .START) != 0 result |= .PRESSED;
    }

    return result;
}

has_focus :: (rect: Rect) -> bool {
    return (get_status_flags(rect) & .FOCUS) != 0;
}

getrect_handle_event :: (event: Event) {
    if event.window != current_window return;

    if event.type == .KEYBOARD {
        index := cast(int) event.key_code;
        if index >= 0 && index < button_states.count {
            if event.key_pressed != 0 button_states[index] |= .START | .DOWN;
            else {
                button_states[index] &= ~ .DOWN;
                button_states[index] |= .END;
            }
        }
    }

    if event.type == .MOUSE_WHEEL && event.wheel_delta != 0 &&
       (event.key_code == .MOUSE_WHEEL_UP || event.key_code == .MOUSE_WHEEL_DOWN) {
        scale := abs(cast(float64) event.typical_wheel_delta);
        if scale == 0 scale = 1;

        notches := cast(float64) event.wheel_delta / scale;
        key := ifx notches > 0 then Key_Code.MOUSE_WHEEL_UP else Key_Code.MOUSE_WHEEL_DOWN;
        button_states[cast(int) key] |= .START;
        found := false;

        for *record: wheel_records if record.window == event.window {
            record.notches += notches;
            found = true;
            break;
        }

        if !found array_add(*wheel_records, .{ event.window, notches });
    }

    if event.type == .TOUCH {
        mouse_position = .{ event.touch_x, event.touch_y };
        mouse_valid = true;
        if event.touch_type == .PRESSED button_states[cast(int) Key_Code.MOUSE_BUTTON_LEFT] |= .START | .DOWN;

        if event.touch_type == .RELEASED {
            button_states[cast(int) Key_Code.MOUSE_BUTTON_LEFT] &= ~ .DOWN;
            button_states[cast(int) Key_Code.MOUSE_BUTTON_LEFT] |= .END;
        }
    }

    one: [1] Event = .[event];
    // Snapshot protects iteration against callback removal and additions.
    snapshot := array_copy(active_widgets);

    defer array_free(snapshot);

    for w: snapshot {
        still_registered := array_find(active_widgets, w);
        if !still_registered continue;
        if w.window == event.window && w.serial_events_proc != null w.serial_events_proc(w, one);
    }
}

active_widget_add :: (widget: *Active_Widget, is_grabbing := false) {
    array_add_if_unique(*active_widgets, widget);
    widget.is_on_state_stack = true;
    widget.window = current_window;
    widget.last_getrect_frame = frame_number;
    if is_grabbing array_add_if_unique(*grabbing_widgets, widget);
}

active_widget_remove :: (widget: *Active_Widget) {
    array_ordered_remove_by_value(*active_widgets, widget);
    array_ordered_remove_by_value(*grabbing_widgets, widget);
    widget.is_on_state_stack = false;
}

active_widget_deactivate_all :: (except: *Active_Widget = null) {
    snapshot := array_copy(active_widgets);

    defer array_free(snapshot);

    for w: snapshot if w != except {
        still_registered := array_find(active_widgets, w);
        if still_registered {
            active_widget_remove(w);
            if w.deactivate_proc != null w.deactivate_proc(w);
        }
    }
}

active_widgets_per_frame_update :: () {
    i := 0;
    while i < active_widgets.count {
        w := active_widgets[i];
        if w.last_getrect_frame + 1 < frame_number {
            active_widget_remove(w);
            if w.deactivate_proc != null w.deactivate_proc(w);
        } else i += 1;
    }
}

is_grabbing :: () -> bool {
    return grabbing_widgets.count > 0;
}

occlusion_per_frame_update :: () {
    swap(*occlusions_current_frame, *occlusions_next_frame);
    occlusions_next_frame.count = 0;
    occluder_being_drawn = null;
}

occlusion_per_window_update :: (window: Window_Type) {
    current_window = window;
}

occluder_start :: (widget: *Active_Widget, rect: Rect) {
    array_add(*occluder_stack, occluder_being_drawn);
    if widget != null widget.exists_on_occluder = occluder_being_drawn;
    occluder_being_drawn = widget;
    array_add(*occlusions_next_frame, .{ cast(s32) occlusions_next_frame.count, rect, widget });
}

occluder_finish :: (widget: *Active_Widget, rect := Rect.{}) {
    assert(occluder_being_drawn == widget && occluder_stack.count > 0, "unbalanced occluder stack");

    if rect.w != 0 || rect.h != 0 {
        for *o: occlusions_next_frame if o.widget == widget o.rect = rect;
    }

    occluder_being_drawn = occluder_stack[occluder_stack.count - 1];
    occluder_stack.count -= 1;
}

events_occluded :: (widget_if_querying_outside_of_render_time: *Active_Widget = null) -> bool {
    w := widget_if_querying_outside_of_render_time;
    if w == null w = occluder_being_drawn;

    i := occlusions_current_frame.count;
    while i > 0 {
        i -= 1;
        o := occlusions_current_frame[i];
        if o.widget != null && o.widget.window != current_window continue;

        if is_inside(mouse_position.x, mouse_position.y, o.rect) {
            if o.widget == w return false;

            parent := w;
            while parent != null {
                if parent == o.widget return false;
                parent = parent.exists_on_occluder;
            }

            return true;
        }
    }

    return false;
}

find_or_create_state :: ($T: Type, hash: u64, start_using := true) -> (*T, created: bool) {
    pointer := table_find_pointer(*state_table, hash);
    created := pointer == null;
    state: *T;

    if created {
        state = New(T);
        state.widget_type = T;
        table_set(*state_table, hash, cast(*Active_Widget) state);
    } else {
        assert(pointer.*.widget_type == T, "widget identity reused with different type");
        state = cast(*T) pointer.*;
    }

    if start_using {
        state.last_getrect_frame = frame_number;
        state.cosmetic_dt = cast(float) max(current_time - state.last_update_time, 0.0);
        state.last_update_time = current_time;
        state.window = current_window;
    }

    return state, created;
}

tooltip :: (text := "", pointer_data: *void = null, int_data := 0) {
    active_tooltip = .{ text, pointer_data, int_data };
}

get_active_tooltip :: () -> Tooltip {
    return active_tooltip;
}

getrect_set_pointer_image :: (image: Pointer_Image, widget: *Active_Widget) {
    if draw_procs.set_pointer_image != null {
        draw_procs.set_pointer_image(image, widget);
        return;
    }

    if !native_pointers_initialized init_pointers();

    index := cast(int) image;

    assert(index >= 0 && index < NUM_POINTERS);
    ok, message := Native_Cursors.set(cast(*void) pointers[index]);
    if !ok report_native_failure(message);
}

init_pointers :: () {
    if native_pointers_initialized {
        Native_Cursors.destroy(native_pointer_ownership);
        native_pointers_initialized = false;
    }

    handles, ok, message := Native_Cursors.create();
    if !ok {
        report_native_failure(message);
        return;
    }

    native_pointer_ownership = handles;
    for handle, i: handles pointers[i] = cast(type_of(pointers[0])) handle;

    native_pointers_initialized = true;
}

require_texture :: (data: string, loc := #caller_location) -> *Texture {
    assert(draw_procs.texture_load_from_memory != null, "texture loader callback required");
    for item: texture_cache if item.data == data return item.texture;

    texture := New(Texture);
    if !draw_procs.texture_load_from_memory(texture, .{ data.count, data.data }, true, true) {
        // Alternate loaders own cleanup of any partially created native data.
        #if Type_Indicator == void Simp.texture_destroy(texture);
        free(texture);
        return null;
    }

    array_add(*texture_cache, .{ copy_string(data), texture });

    return texture;
}

// No embedded asset payloads are supplied; all nine resource slots are empty.
get_rect_get_global_data :: () -> string, string, string, string, string, string, string, string, string {
    return "", "", "", "", "", "", "", "", "";
}

set_theme_from_base_colors :: (theme: *Overall_Theme, main_surface_color: Vector4, border_color: Vector4, interact_color: Vector4, atop_surface_text: Vector4, atop_border_text: Vector4) {
    theme.background_color = main_surface_color;
    theme.text_color = atop_surface_text;
    theme.title_bar_color = border_color;
    theme.title_bar_color_hilit = interact_color;
    theme.button_theme.surface_color = main_surface_color;
    theme.button_theme.surface_color_over = interact_color;
    theme.button_theme.surface_color_down = border_color;
    theme.button_theme.frame_color = border_color;
    theme.button_theme.text_color = atop_surface_text;
    theme.button_theme.text_color_pressed = atop_border_text;
    theme.checkbox_theme.button_theme = theme.button_theme;
    theme.label_theme.text_color = atop_surface_text;
    theme.slider_theme.foreground = theme.button_theme;
    theme.dropdown_theme.theme_for_current_value = theme.button_theme;
}

invert_button_theme :: (button_theme: *Button_Theme) {
    button_theme.surface_color = invert_rgb(button_theme.surface_color);
    button_theme.surface_color_over = invert_rgb(button_theme.surface_color_over);
    button_theme.surface_color_down = invert_rgb(button_theme.surface_color_down);
    button_theme.frame_color = invert_rgb(button_theme.frame_color);
    button_theme.text_color = invert_rgb(button_theme.text_color);
}

invert_scrollbar :: (using scrollable_region_theme: *Scrollable_Region_Theme) {
    invert_button_theme(*scrollbar_nib_theme);
    scrollbar_background.color = invert_rgb(scrollbar_background.color);
    region_background.color = invert_rgb(region_background.color);
}

#scope_module
Native_Cursors :: #import "internal/numerics/platform_cursors";
#if OS == .MACOS || OS == .WINDOWS || OS == .LINUX Canonical_Clipboard :: #import "Clipboard";

native_pointer_ownership: [8] *void;
native_pointers_initialized: bool;

Wheel_Record :: struct {
    window: Window_Type;
    notches: float64;
}

wheel_records: [..] Wheel_Record;

take_wheel_notches :: () -> float64 {
    for *record: wheel_records if record.window == current_window {
        value := record.notches;
        record.notches = 0;
        return value;
    }

    value: float64;
    if (ui_get_button_state(.MOUSE_WHEEL_UP) & .START) != 0 value += 1;
    if (ui_get_button_state(.MOUSE_WHEEL_DOWN) & .START) != 0 value -= 1;
    if value != 0 array_add(*wheel_records, .{ current_window, 0 });

    return value;
}

report_native_failure :: (message: string) {
    print("GetRect native integration failure: %\n", message, to_standard_error = true);
    native_failure_trap();
}

native_failure_trap :: () #intrinsic "llvm.trap";

clipboard_write :: (text: string) {
    #if OS == .MACOS || OS == .WINDOWS || OS == .LINUX Canonical_Clipboard.os_clipboard_set_text(text);
    else report_native_failure("Canonical clipboard has no native backend for this target");
}

clipboard_read :: () -> string {
    #if OS == .MACOS || OS == .WINDOWS || OS == .LINUX return Canonical_Clipboard.os_clipboard_get_text();
    else {
        report_native_failure("Canonical clipboard has no native backend for this target");
        return ""; // Unreachable: failed integration terminates, never yields local data.
    }
}

child_id :: (loc: Source_Code_Location, parent: s64, tag: u64) -> s64 #no_aoc {
    return cast,trunc(s64) combine_hashes(get_hash(loc, parent), tag);
}

frame_number: u64;
current_time: float64;
dt: float;
current_window: Window_Type;
render_width: s32;
render_height: s32;
mouse_position: Vector2;
mouse_valid: bool;
button_states: [1024] Key_Current_State;
click_consumed: bool;
active_widgets: [..] *Active_Widget;
grabbing_widgets: [..] *Active_Widget;
occluder_stack: [..] *Active_Widget;
active_tooltip: Tooltip;
rounding_pixels: float = 6;
frame_pixels_global: float = 1;
divider_pixels: float = 4;
divider_initial_pixels: float = 100;
nub_pixels: float = 12;

Texture_Cache_Entry :: struct {
    data: string;
    texture: *Texture;
}

texture_cache: [..] Texture_Cache_Entry;

invert_rgb :: (v: Vector4) -> Vector4 {
    return .{ 1 - v.x, 1 - v.y, 1 - v.z, v.w };
}

theme_default :: () -> Overall_Theme {
    return .{};
}

theme_grayscale :: () -> Overall_Theme {
    t: Overall_Theme;
    set_theme_from_base_colors(*t, .{ .15, .15, .15, 1 }, .{ .3, .3, .3, 1 }, .{ .5, .5, .5, 1 }, .{ 1, 1, 1, 1 }, .{ 0, 0, 0, 1 });
    return t;
}

theme_bentonite :: () -> Overall_Theme {
    t := theme_default();
    t.background_color = .{ .24, .19, .16, 1 };
    return t;
}

theme_nimbus :: () -> Overall_Theme {
    t := theme_grayscale();
    invert_button_theme(*t.button_theme);
    t.background_color = .{ .9, .9, .95, 1 };
    return t;
}

theme_blood :: () -> Overall_Theme {
    t := theme_default();
    t.button_theme.surface_color_over = .{ .7, .05, .06, 1 };
    return t;
}

theme_freddie :: () -> Overall_Theme {
    t := theme_default();
    t.button_theme.surface_color_over = .{ .1, .3, .6, 1 };
    return t;
}

theme_contrast :: () -> Overall_Theme {
    t := theme_grayscale();
    t.background_color = .{ 0, 0, 0, 1 };
    t.button_theme.frame_color = .{ 1, 1, 1, 1 };
    t.button_theme.text_color = .{ 1, 1, 0, 1 };

    return t;
}
