// Right-handed GetRect drawing through Simp's OpenGL backend: a button, a checkbox and a dropdown
// are driven for a few frames with a program-supplied pointer and synthetic mouse events, drawn into
// a render-target texture and read back to check the button's color in each state. The UI font and
// the eight image resources come from an asset folder the test writes (the font is copied from the
// operating system, since none ships with the repository). Needs OpenGL 3.3 (Windows CI runners lack
// it) and a display (Linux CI runs Xvfb with Mesa); wasm has neither (see the skip list).
#import "Basic";
#import "Math";

GR :: #import "GetRect";
Simp :: #import "Simp";
Input :: #import "Input";
#import "Window_Creation";
#import "File";
String :: #import "String";

SIZE :: 64;
ASSET_DIR :: ".scratch/getrect-simp-gl-frame";

FONT_CANDIDATES :: string.[
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "C:/Windows/Fonts/arial.ttf",
];

// The pointer, in render-target pixels with y up.
pointer_x := -100;
pointer_y := -100;
now: float64 = 1;

read_target :: (target: *Simp.Texture) -> Simp.Bitmap {
    reader := Simp.pixel_read_begin(target, .RGBA8);
    assert(reader != null, "pixel_read_begin failed");
    for 1..1000 {
        if Simp.pixel_read_is_done(reader) break;
        sleep_milliseconds(1);
    }
    return Simp.pixel_read_end(reader);
}

// y counts up from the bottom, as GetRect's right-handed rects do; the readback is top row first.
pixel :: (bitmap: *Simp.Bitmap, x: int, y: int) -> [4] u8 {
    p := bitmap.data.data + (bitmap.height - 1 - y) * bitmap.stride + x * 4;
    return .[p[0], p[1], p[2], p[3]];
}

near :: (bitmap: *Simp.Bitmap, x: int, y: int, color: Vector4) -> bool {
    p := pixel(bitmap, x, y);
    return abs(p[0] - color.x * 255) <= 4 && abs(p[1] - color.y * 255) <= 4 && abs(p[2] - color.z * 255) <= 4;
}

left_button :: (pressed: bool) -> Input.Event {
    event: Input.Event;
    event.type = .KEYBOARD;
    event.key_code = .MOUSE_BUTTON_LEFT;
    event.key_pressed = xx pressed;
    return event;
}

begin_frame :: (window: Window_Type, target: *Simp.Texture, x: int, y: int, events: ..Input.Event) {
    pointer_x, pointer_y = x, y;
    Input.input_per_frame_event_and_flag_update();
    for events Input.publish_input_event(it);
    now += 1.0 / 60;
    Simp.set_render_target(target);
    Simp.clear_render_target(0, 0, 0, 1);
    GR.ui_per_frame_update(window, SIZE, SIZE, now);
}

// The asset folder: the real font under the expected name, and stand-ins for the images.
write_assets :: () -> bool {
    font_bytes: string;
    for FONT_CANDIDATES {
        bytes, ok := read_entire_file(it, log_errors = false);
        if ok && bytes.count > 0 {
            font_bytes = bytes;
            break;
        }
    }
    if !font_bytes.count return false;
    make_directory_if_it_does_not_exist(".scratch", recursive = true);
    make_directory_if_it_does_not_exist(ASSET_DIR, recursive = true);
    for GR.getrect_asset_filenames {
        contents := ifx it_index == 4 then font_bytes else tprint("stand-in %", it);
        assert(write_entire_file(tprint("%/%", ASSET_DIR, it), contents), "could not write %", it);
    }
    free(font_bytes.data);
    return true;
}

main :: () {
    window := create_window(160, 120, "jaic getrect gl test");
    assert(cast(u64) window != 0, "create_window failed");
    for 1..5 {
        Input.update_window_events();
        sleep_milliseconds(5);
    }
    Simp.prepare_window(window, 0);
    Simp.set_render_target(window);

    if !write_assets() {
        // Linux images normally have DejaVu; any other OS without a font has nothing to draw text with.
        assert(OS != .MACOS && OS != .WINDOWS, "no system font found");
        print("ok\n");
        return;
    }
    GR.set_asset_root(ASSET_DIR);
    radio_full, radio_empty, check_full, check_empty, font, circle, hsl, rgb, numbers := GR.get_rect_get_global_data();
    assert(radio_full == "stand-in ui_radiobox_full.png" && numbers == "stand-in mode_numbers.png", "image resources % / %", radio_full, numbers);
    assert(font.count > 1000, "the font resource has % bytes", font.count);
    images := string.[radio_empty, check_full, check_empty, circle, hsl, rgb];
    for images assert(String.begins_with(it, "stand-in "), "image resource % is %", it_index, it);

    // The pointer comes from the test; everything else is Simp's.
    procs: GR.Draw_Procs;
    procs.get_mouse_pointer_position = (window: Window_Type, right_handed: bool) -> int, int, bool {
        return pointer_x, pointer_y, true;
    };
    GR.ui_init("", *procs);

    target := Simp.texture_create_render_target(SIZE, SIZE, .RGBA8);
    assert(target != null, "texture_create_render_target failed");

    theme := GR.default_overall_theme.button_theme;
    theme.surface_color = .{ 1, 0, 0, 1 };
    theme.surface_color_over = .{ 0, 1, 0, 1 };
    theme.surface_color_down = .{ 0, 0, 1, 1 };
    theme.surface_color_flash = .{ 0, 0, 1, 1 };
    button_rect := GR.get_rect(8, 32, 48, 24);
    here := #location();
    frame :: (x: int, y: int, events: ..Input.Event) -> pressed: bool, released: bool, image: Simp.Bitmap #expand {
        begin_frame(window, target, x, y, ..events);
        pressed, state, released := GR.button(button_rect, "", *theme, loc = here);
        GR.draw_popups();
        Simp.immediate_flush();
        return pressed, released, read_target(target);
    }

    // Idle, hovered, pressed, released: the surface color follows.
    pressed, released, image := frame(-100, -100);
    assert(!pressed && near(*image, 32, 44, theme.surface_color), "idle button center %", pixel(*image, 32, 44));
    assert(near(*image, 2, 2, .{ 0, 0, 0, 1 }), "outside the button %", pixel(*image, 2, 2));
    Simp.deinit(*image);
    // Hovering fades toward the hover color over a few frames.
    for 1..60 {
        pressed, released, image = frame(32, 44);
        assert(!pressed, "hovering pressed the button");
        if near(*image, 32, 44, theme.surface_color_over) break;
        first := pixel(*image, 32, 44);
        assert(first[1] > 0, "the hover fade has not started: %", first);
        if it < 60 Simp.deinit(*image);
    }
    assert(near(*image, 32, 44, theme.surface_color_over), "hovered button center %", pixel(*image, 32, 44));
    Simp.deinit(*image);
    pressed, released, image = frame(32, 44, left_button(true));
    assert(pressed && !released, "the press frame did not press the button");
    assert(!near(*image, 32, 44, theme.surface_color) && !near(*image, 32, 44, theme.surface_color_over), "pressed button center %", pixel(*image, 32, 44));
    Simp.deinit(*image);
    pressed, released, image = frame(32, 44, left_button(false));
    assert(!pressed && released, "the release frame did not release the button");
    Simp.deinit(*image);
    // The y-up layout: a point above the rect in pixels is not inside it.
    pressed, released, image = frame(32, 60, left_button(true));
    assert(!pressed, "a click above the button pressed it");
    Simp.deinit(*image);
    pressed, released, image = frame(32, 60, left_button(false));
    Simp.deinit(*image);

    // A checkbox flips on its click frame; a dropdown opens downward (lower y) and picks.
    checked := false;
    box := GR.get_rect(4, 4, 56, 12);
    choices := string.["a", "b"];
    value: s32 = 0;
    drop := GR.get_rect(4, 44, 56, 12);
    widgets :: (x: int, y: int, events: ..Input.Event) -> changed: bool, open: bool #expand {
        begin_frame(window, target, x, y, ..events);
        changed := GR.checkbox(box, "on", *checked, loc = here);
        dropdown_changed, state := GR.dropdown(drop, choices, *value, loc = here);
        GR.draw_popups();
        Simp.immediate_flush();
        return changed || dropdown_changed, state.open;
    }
    changed, open := widgets(10, 10, left_button(true));
    assert(changed && checked, "checkbox click: changed %, value %", changed, checked);
    widgets(10, 10, left_button(false));
    changed, open = widgets(10, 50, left_button(true));
    assert(open, "the dropdown did not open");
    widgets(10, 50, left_button(false));
    // The second choice is two rows below the closed box.
    widgets(10, 50 - 24, left_button(true));
    changed, open = widgets(10, 50 - 24, left_button(false));
    assert(value == 1 && !open, "dropdown pick: value %, open %", value, open);

    // A y-up subwindow's draw callback may move its content bottom; the top edge stays put.
    info: GR.Subwindow_Info;
    info.rect = GR.get_rect(4, 4, 56, 56);
    info.title_text = "w";
    info.draw = (state: *GR.Subwindow_State, content: GR.Rect, data: *void) {
        GR.draw_proc_adjust_y1(state, content.y + 20);
    };
    begin_frame(window, target, -100, -100);
    GR.add_subwindow(*info);
    GR.draw_popups();
    Simp.immediate_flush();
    assert(info.rect.y + info.rect.h == 60 && info.rect.h < 56, "adjusted subwindow rect %", info.rect);

    GR.ui_deinit();
    GR.release_asset_data();
    Simp.deinit(target);
    free(target);
    for GR.getrect_asset_filenames file_delete(tprint("%/%", ASSET_DIR, it));

    Simp.set_render_target(window);
    Simp.clear_render_target(0, 0, 0, 1);
    Simp.swap_buffers(window);
    print("ok\n");
}
