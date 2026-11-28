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

// Sets the corner radius, in pixels, used by shapes whose roundedness type is `.ABSOLUTE_FROM_GLOBAL`.
//
// The radius is still limited to half of the shorter side of the rectangle being drawn.
set_global_rounding_radius_in_pixels :: (value: float) {
    rounding_pixels = value;
}

// Sets the frame thickness, in pixels, used by shapes whose frame thickness type is `.ABSOLUTE_FROM_GLOBAL`.
//
// The thickness is still limited to half of the shorter side of the rectangle being drawn.
set_global_frame_thickness_in_pixels :: (value: float) {
    frame_pixels_global = value;
}

// Sets the divider thickness, in pixels, used by [slidable_region] themes whose divider thickness type is
// `.ABSOLUTE_FROM_GLOBAL`.
set_global_divider_thickness_in_pixels :: (value: float) {
    divider_pixels = value;
}

// Sets the starting divider position, in pixels from the first edge, used by [slidable_region] themes whose
// initial position type is `.ABSOLUTE_FROM_GLOBAL`.
//
// Only read when the region's state is first created.
set_global_initial_divider_position_in_pixels :: (value: float) {
    divider_initial_pixels = value;
}

// Sets the nub width, in pixels, used by [slider] themes whose nub thickness type is `.ABSOLUTE_FROM_GLOBAL`.
set_global_slider_nub_thickness_in_pixels :: (value: float) {
    nub_pixels = value;
}

// Sets the theme that widgets use when they are passed a null theme.
//
// Widgets read their sub-theme (button, checkbox, slider and so on) from this copy at call time, so a change
// applies from the next widget call.
set_default_theme :: (theme: Overall_Theme) {
    default_overall_theme = theme;
}

// Initializes the library with a draw procedure table and a default font.
//
// - default_font: the font data to use; empty selects the bundled font.
// - procs: the drawing callbacks to copy. With null, `Simp` drawing is used when the module is imported with
//   the default `Type_Indicator`; otherwise `draw_procs` is left as it was.
//
// Call once before the first [ui_per_frame_update].
ui_init :: (default_font := "", procs: *Draw_Procs = null) {
    if procs != null draw_procs = procs.*;
    else {
        #if Type_Indicator == void draw_procs = simp_draw_procs();
    }

    default_font_data = ifx default_font.count > 0 then default_font else BUNDLED_FONT;
    array_reset(*default_fonts);
}

// Returns the state of `key` for the current frame as `Key_Current_State` flags.
//
// `.START` is set only on the frame the key went down, `.DOWN` while it is held, and `.END` on the frame it
// was released. Asserts that `key` is within the tracked range. Mouse buttons and wheel directions are keys
// too.
ui_get_button_state :: (key: Key_Code) -> Key_Current_State {
    index := cast(int) key;
    assert(index >= 0 && index < button_states.count);
    return button_states[index];
}

// Clears the per-frame key edges (`.START` and `.END`) and the recorded wheel movement, keeping `.DOWN`.
//
// Called by [ui_per_frame_update]; call it directly only when driving the frame yourself.
windows_per_frame_update :: () {
    for *s: button_states s.* &= .DOWN;
    wheel_records.count = 0;
}

// Makes `window` the target of subsequent widget calls and refreshes the mouse position for it.
//
// - _render_target_width, _render_target_height: the window's drawable size in pixels.
//
// The pointer position comes from the `get_mouse_pointer_position` callback in `draw_procs`, requested with
// the y axis flipped unless built with `LEFT_HANDED`; without that callback the mouse is treated as invalid
// and widgets see no hover. Returns `background_occluded`, which is true when the mouse is over a widget that
// blocks input (see [occluder_start]), and `is_active`, which is true while any widget is active.
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

// Starts a UI frame for `window` and returns the same pair as [switch_window].
//
// Call once per frame before any widget. Advances the frame counter, sets the frame time step from
// `_current_time` (in seconds; never negative), clears the tooltip and the click-consumed flag, and drops
// widgets that were not called during the previous frame, running their deactivate callbacks. To draw into
// more than one window in a frame, call [switch_window] for each additional window.
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

// Returns the mouse status of `r`: `.OVER` and `.FOCUS` when the pointer is inside it, plus `.DOWN` while the
// left button is held and `.PRESSED` on the frame it went down.
//
// Returns no flags when the mouse is invalid, the pointer is outside the current scissor, the pointer is
// covered by an occluder that does not own the widget, or (for `.DOWN` and `.PRESSED`) the click was already
// taken by another widget this frame.
//
// - widget_if_querying_outside_of_render_time: the widget asking, so that its own occluder does not hide it.
//   Pass null while drawing; the widget being drawn is used.
// - ignore: [Status_Flags_Ignore] bits to skip the scissor check or the left click.
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

// Returns true if the pointer is over `rect` and not hidden behind an occluder or outside the scissor.
has_focus :: (rect: Rect) -> bool {
    return (get_status_flags(rect) & .FOCUS) != 0;
}

// Feeds one input event to the library.
//
// Call it for every input event. Events for a window other than the current one are ignored. Records key and
// mouse button transitions and wheel notches, treats touches as the left mouse button at the touch position,
// and forwards the event to the active widgets of that window.
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

// Registers `widget` as active, so that it receives events and is kept alive across frames.
//
// - is_grabbing: also records the widget as grabbing the pointer, which makes [is_grabbing] return true until
//   the widget is removed.
//
// Adding a widget twice has no effect. The widget is dropped automatically if it is not called for a frame.
active_widget_add :: (widget: *Active_Widget, is_grabbing := false) {
    array_add_if_unique(*active_widgets, widget);
    widget.is_on_state_stack = true;
    widget.window = current_window;
    widget.last_getrect_frame = frame_number;
    if is_grabbing array_add_if_unique(*grabbing_widgets, widget);
}

// Unregisters `widget` from the active and grabbing sets without running its deactivate callback.
active_widget_remove :: (widget: *Active_Widget) {
    array_ordered_remove_by_value(*active_widgets, widget);
    array_ordered_remove_by_value(*grabbing_widgets, widget);
    widget.is_on_state_stack = false;
}

// Deactivates every active widget except `except`, running each widget's deactivate callback.
//
// Use it to close popups and end drags, for example when the user clicks elsewhere.
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

// Removes active widgets that were not called during the previous frame, running their deactivate callbacks.
//
// Called by [ui_per_frame_update].
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

// Returns true while any active widget has grabbed the pointer, for example a slider being dragged.
is_grabbing :: () -> bool {
    return grabbing_widgets.count > 0;
}

// Promotes the occluders recorded during the last frame to the current set used for hit testing.
//
// Occluders are therefore one frame late: a widget is hidden by what was drawn over it in the previous frame.
// Called by [ui_per_frame_update].
occlusion_per_frame_update :: () {
    swap(*occlusions_current_frame, *occlusions_next_frame);
    occlusions_next_frame.count = 0;
    occluder_being_drawn = null;
}

// Sets the current window for occlusion testing.
//
// Called by [switch_window].
occlusion_per_window_update :: (window: Window_Type) {
    current_window = window;
}

// Begins an input-blocking rectangle owned by `widget`.
//
// Until the matching [occluder_finish], widgets drawn inside belong to `widget` and can still be hovered;
// everything else under `rect` is hidden from the pointer on the next frame. Calls nest.
occluder_start :: (widget: *Active_Widget, rect: Rect) {
    array_add(*occluder_stack, occluder_being_drawn);
    if widget != null widget.exists_on_occluder = occluder_being_drawn;
    occluder_being_drawn = widget;
    array_add(*occlusions_next_frame, .{ cast(s32) occlusions_next_frame.count, rect, widget });
}

// Ends the occluder begun by [occluder_start] for `widget`.
//
// - rect: when it has a nonzero width or height, replaces the rectangle recorded for `widget`, for occluders
//   whose size is only known after their contents are laid out.
//
// Asserts if the calls are not balanced.
occluder_finish :: (widget: *Active_Widget, rect := Rect.{}) {
    assert(occluder_being_drawn == widget && occluder_stack.count > 0, "unbalanced occluder stack");

    if rect.w != 0 || rect.h != 0 {
        for *o: occlusions_next_frame if o.widget == widget o.rect = rect;
    }

    occluder_being_drawn = occluder_stack[occluder_stack.count - 1];
    occluder_stack.count -= 1;
}

// Returns true if the pointer is over a rectangle recorded as an occluder that the widget does not belong to.
//
// Occluders are tested from the most recently recorded one backwards, and the first one under the pointer
// decides. A widget is not occluded by its own occluder or by an occluder it was drawn inside.
//
// - widget_if_querying_outside_of_render_time: the widget asking; null uses the occluder currently being
//   drawn.
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

// Returns the persistent state of type `T` stored under `hash`, creating it default-initialized the first
// time.
//
// `created` is true on the call that created it. States are identified only by `hash`, which widgets build
// with [get_hash] from the call site and an identifier. Asserts if `hash` was earlier used with a different
// type.
//
// - start_using: when true, marks the state as used this frame and records the frame time step and window.
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

// Sets the tooltip for the current frame.
//
// The text and data are kept until the next [ui_per_frame_update] and read back with [get_active_tooltip].
// `text` is not copied, so it must outlive the frame.
tooltip :: (text := "", pointer_data: *void = null, int_data := 0) {
    active_tooltip = .{ text, pointer_data, int_data };
}

// Returns the tooltip set by [tooltip] this frame, or an empty one if none was set.
get_active_tooltip :: () -> Tooltip {
    return active_tooltip;
}

// Changes the mouse cursor to `image` on behalf of `widget`.
//
// Uses the `set_pointer_image` callback in `draw_procs` when there is one; otherwise sets the platform's
// native cursor, creating the native cursors on first use. If the native cursor can't be created or set, prints
// the reason to standard error and traps.
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

// Creates the native cursors used by [getrect_set_pointer_image], replacing any created before.
//
// Called automatically on first use. If the cursors can't be created, prints the reason to standard error and
// traps.
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

// Returns a texture loaded from the image bytes `data`, loading it on the first request and caching it by
// content.
//
// Returns null if loading fails. The texture stays in the library's cache and is shared by later requests for
// the same bytes. Asserts if `draw_procs.texture_load_from_memory` is not set.
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

// Fills the main color fields of `theme` from five base colors.
//
// - main_surface_color: backgrounds and button surfaces.
// - border_color: title bars, button frames and the pressed surface.
// - interact_color: hovered surfaces and highlighted title bars.
// - atop_surface_text: text drawn on a surface.
// - atop_border_text: text of a pressed button.
//
// Sets the background, title bar, label, button, checkbox, slider foreground and dropdown current-value
// colors, overwriting what was there. Other fields of `theme` are left unchanged.
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

// Replaces the surface, frame and text colors of `button_theme` with their RGB inverses.
//
// Alpha and the hover and pressed text colors are not changed.
invert_button_theme :: (button_theme: *Button_Theme) {
    button_theme.surface_color = invert_rgb(button_theme.surface_color);
    button_theme.surface_color_over = invert_rgb(button_theme.surface_color_over);
    button_theme.surface_color_down = invert_rgb(button_theme.surface_color_down);
    button_theme.frame_color = invert_rgb(button_theme.frame_color);
    button_theme.text_color = invert_rgb(button_theme.text_color);
}

// Inverts the colors of a scrollable region theme: the scrollbar nib's button colors, the scrollbar
// background and the region background.
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

#if Type_Indicator == void {
    // The drawing callbacks used when the program registers none: the renderer behind the
    // default font and texture types. The native pointer image is chosen by the library itself.
    simp_draw_procs :: () -> Draw_Procs {
        procs: Draw_Procs;

        procs.texture_load_from_memory = (texture: *Texture, memory: [] u8, srgb: bool, build_mipmaps: bool) -> bool {
            return Simp.texture_load_from_memory(texture, memory, srgb, build_mipmaps);
        };
        procs.set_scissor = (x0: s32, y0: s32, x1: s32, y1: s32) {
            Simp.set_scissor(x0, y0, x1, y1);
        };
        procs.clear_scissor = () {
            Simp.clear_scissor();
        };
        procs.set_shader_for_color = (enable_blend := false) {
            Simp.set_shader_for_color(enable_blend);
        };
        procs.set_shader_for_images = (texture: *Texture) {
            Simp.set_shader_for_images(texture);
        };
        procs.prepare_text = (font: *Font, text: string, effects: Font_Effects = 0) -> s64 {
            return Simp.prepare_text(font, text, effects);
        };
        procs.draw_prepared_text = (font: *Font, x: s64, y: s64, text_color: Vector4, effects: Font_Effects = 0) {
            Simp.draw_prepared_text(font, x, y, text_color, effects);
        };
        procs.get_mouse_pointer_position = (window: Window_Type, right_handed: bool) -> (x: int, y: int, success: bool) {
            return Windows_Context.get_mouse_pointer_position(window, right_handed);
        };
        procs.get_font_at_size = (memory: [] u8, pixel_height: int) -> *Font {
            return Simp.get_font_at_size(memory, pixel_height);
        };
        procs.immediate_triangle = (p0: Vector3, p1: Vector3, p2: Vector3, c0 := Vector4.{ 1, 1, 1, 1 }, c1 := Vector4.{ 1, 1, 1, 1 }, c2 := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{ z = 1 }) {
            Simp.immediate_triangle(p0, p1, p2, c0, c1, c2, uv0, uv1, uv2, normal);
        };
        procs.immediate_quad = (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {
            Simp.immediate_quad(p0, p1, p2, p3, color, uv0, uv1, uv2, uv3);
        };
        procs.immediate_flush = () {
            Simp.immediate_flush();
        };

        return procs;
    }
}
