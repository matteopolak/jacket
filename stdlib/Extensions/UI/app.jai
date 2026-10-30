// The application loop: a native window, the platform's input translated into UI events, a frame
// whenever something needs one, and the GPU renderer. Idle means no frames and a sleeping thread.

// Options for `run_app`.
App_Options :: struct {
    title: string = "Jacket";
    width: int = 960; // logical size in points
    height: int = 640;
    theme: Theme_Mode;
    fonts: bool = true; // load the platform's fonts
    frame_limit: int; // stop after this many frames (0: run until the window closes); for smoke tests
}

// What `run_app` can tell the app about the window.
App_Window :: struct {
    ui: *UI_State;
    window: Window_Type;
    renderer: *Renderer;
    frames: int;
}

// Open a window and run `build` for every frame until the window closes. `app` is passed to `build`
// each frame. Returns false when the window or the GPU could not be set up.
//
//     run_app(.{title = "Counter", width = 320, height = 200}, *app, build);
run_app :: (options: App_Options, app: *$T, build: (app: *T)) -> bool {
    ui := New(UI_State);

    defer {
        free_ui(ui);
        free(ui);
    }

    return run_app_with(options, ui, app, build);
}

// Like `run_app`, for a UI the caller owns (to set the theme or hooks first, or to inspect it after).
run_app_with :: (options: App_Options, ui: *UI_State, app: *$T, build: (app: *T)) -> bool {
    window := create_window(options.width, options.height, options.title);
    if !window return false;

    init(*ui.frame_pool, 256 * 1024 * 1024);
    ui.theme = default_theme(options.theme);

    if options.fonts {
        load_system_fonts(*ui.text);
    } else {
        add_synthetic_font(*ui.text);
        add_synthetic_font(*ui.text);
    }

    clipboard_window = window;
    ui.hooks.clipboard_get = platform_clipboard_get;
    ui.hooks.clipboard_set = platform_clipboard_set;

    point_w, point_h := window_points(window);
    pixel_w, pixel_h := webgpu_window_size(window);
    scale := ifx point_w > 0 then cast(float) pixel_w / point_w else 1.0;

    set_scale(*ui.text, scale);
    ui.size = .{ cast(float) point_w, cast(float) point_h };

    renderer := create_window_renderer(window, pixel_w, pixel_h);

    if !renderer {
        print("UI: no GPU or surface is available for the window\n");
        return false;
    }

    defer destroy_renderer(renderer);

    last_x := -1.0;
    last_y := -1.0;
    frames := 0;
    must_draw := true;

    push_ui(ui);
    defer pop_ui();

    while !ui.quit {
        update_window_events();
        ui.now = seconds_since_init();

        for get_window_resizes() {
            pixel_w, pixel_h = webgpu_window_size(window);
            point_w, point_h = window_points(window);
            resize_target(renderer, pixel_w, pixel_h);
            set_scale(*ui.text, ifx point_w > 0 then cast(float) pixel_w / point_w else 1.0);
            ui.size = .{ cast(float) point_w, cast(float) point_h };
            must_draw = true;
        }

        x, y, ok := get_mouse_pointer_position(window, false);

        if ok && (cast(float) x != last_x || cast(float) y != last_y) {
            last_x = cast(float) x;
            last_y = cast(float) y;
            send_mouse_move(ui, last_x, last_y);
        }

        for events_this_frame translate_event(ui, it);

        wanted := ui.input.events.count > 0 || ui.wants_frame || must_draw || (ui.wake_time > 0 && ui.now >= ui.wake_time);

        if !wanted {
            sleep_milliseconds(4);
            continue;
        }

        must_draw = false;
        reset_temporary_storage();
        frame(ui, app, build);

        if present(renderer, ui, ui.theme.background) {
            frames += 1;
        } else {
            must_draw = true;
            sleep_milliseconds(16);
        }

        if options.frame_limit > 0 && frames >= options.frame_limit break;
    }

    return true;
}

#scope_module

clipboard_window: Window_Type;

window_points :: (window: Window_Type) -> width: s32, height: s32 {
    _, _, w, h, ok := get_dimensions(window, false);
    if !ok return 0, 0;
    return w, h;
}

translate_event :: (ui: *UI_State, event: Event) {
    mods: Modifiers;
    mods.shift = event.shift_pressed;
    mods.ctrl = event.ctrl_pressed;
    mods.alt = event.alt_pressed;
    mods.super = event.cmd_meta_pressed;

    if event.type == {
        case .QUIT;
            ui.quit = true;
        case .KEYBOARD;
            if event.key_code == .MOUSE_BUTTON_LEFT || event.key_code == .MOUSE_BUTTON_RIGHT || event.key_code == .MOUSE_BUTTON_MIDDLE {
                button := Mouse_Button.LEFT;
                if event.key_code == .MOUSE_BUTTON_RIGHT button = .RIGHT;
                if event.key_code == .MOUSE_BUTTON_MIDDLE button = .MIDDLE;

                if event.key_pressed send_mouse_down(ui, button);
                else send_mouse_up(ui, button);
            } else if event.key_pressed {
                send_key(ui, event.key_code, mods, event.repeat);
            }
        case .TEXT_INPUT;
            // Shortcuts (Cmd+C, Ctrl+V) arrive as text too; they are keys, not typing.
            if primary(mods) && !mods.alt return;

            array_add(*ui.input.events, .{ kind = .TEXT, character = event.utf32, mods = mods });
        case .MOUSE_WHEEL;
            #if OS == .MACOS {
                amount := -cast(float) event.wheel_delta;
            } else {
                amount := -cast(float) event.wheel_delta * WHEEL_NOTCH_POINTS / WHEEL_NOTCH_UNITS;
            }

            if event.key_code == .UNKNOWN send_scroll(ui, amount, 0);
            else send_scroll(ui, 0, amount);
    }
}

platform_clipboard_get :: () -> string {
    #if OS == .LINUX {
        text := os_clipboard_get_text(clipboard_window);
    } else {
        text := os_clipboard_get_text();
    }

    defer free(text.data);

    return copy_string_to_temp(text);
}

platform_clipboard_set :: (text: string) {
    #if OS == .LINUX {
        os_clipboard_set_text(clipboard_window, text);
    } else {
        os_clipboard_set_text(text);
    }
}

copy_string_to_temp :: (text: string) -> string {
    return copy_string(text,, temp);
}

// A wheel notch is 120 units on Windows and Linux; it scrolls this many points.
WHEEL_NOTCH_UNITS :: 120.0;
WHEEL_NOTCH_POINTS :: 40.0;
