// GetRect_LeftHanded widgets driven frame by frame with synthetic input: the program supplies the
// drawing callbacks (which record what is drawn), the pointer position and the mouse and key
// events, so no window or graphics context is needed. Checked: buttons press and release on the
// right frames, checkboxes, radio buttons and flag boxes change their values, dropdowns open and
// pick, sliders follow the pointer and the arrow keys, text and number inputs take typed text,
// scroll regions take the wheel, split regions, subwindows, tooltips, scissors, the color widgets,
// flowing text, themes and the text-editing helpers. GetRect imports Simp, whose native libraries
// have no wasm build, so the wasm targets run only the pure geometry and color helpers.
#import "Basic";
#import "Math";

#if OS == .WASM {
    // The helpers live in files GetRect loads; read them directly, as the y-down module would.
    GETRECT_Y_UP :: false;
    Math :: #import "Math";
    #load "../../stdlib/GetRect_Common/geometry.jai";
    #load "../../stdlib/GetRect_Common/geometry-helpers.jai";

    main :: () {
        pure_helpers();
        print("ok\n");
    }
} else {
    GR :: #import "GetRect_LeftHanded";
    Input :: #import "Input";
    #import "Window_Type";
    #import "Clipboard";

    get_rect :: GR.get_rect;
    Rect :: GR.Rect;
    cut_left :: GR.cut_left;
    cut_right :: GR.cut_right;
    cut_top :: GR.cut_top;
    cut_bottom :: GR.cut_bottom;
    intersect :: GR.intersect;
    bound :: GR.bound;
    get_quad :: GR.get_quad;
    get_hash :: GR.get_hash;
    darken :: GR.darken;
    whiten :: GR.whiten;
    brighten :: GR.brighten;
    hsv_to_rgb :: GR.hsv_to_rgb;
    rgb_to_hsv :: GR.rgb_to_hsv;
    is_inside :: GR.is_inside;

    WINDOW :: cast(Window_Type) 1;
    WIDTH :: 800;
    HEIGHT :: 600;

    // What the drawing callbacks saw.
    triangles: int;
    quads: int;
    scissor_sets: int;
    scissor_clears: int;
    last_scissor: [4] s32;
    prepared: string;
    drawn_texts: [..] string;
    pointer_images: [..] GR.Pointer_Image;

    // The fake font: every character is 8 pixels wide.
    font: GR.Font;

    mouse := Vector2.{ -100, -100 };
    now: float64 = 1;

    procs :: () -> GR.Draw_Procs {
        result: GR.Draw_Procs;
        result.set_scissor = (x0: s32, y0: s32, x1: s32, y1: s32) {
            scissor_sets += 1;
            last_scissor = .[x0, y0, x1, y1];
        };
        result.clear_scissor = () {
            scissor_clears += 1;
        };
        result.set_shader_for_color = (enable_blend := false) {};
        result.set_shader_for_images = (texture: *GR.Texture) {};
        result.prepare_text = (f: *GR.Font, text: string, effects: GR.Font_Effects = 0) -> s64 {
            prepared = text;
            return 8 * text.count;
        };
        result.draw_prepared_text = (f: *GR.Font, x: s64, y: s64, color: Vector4, effects: GR.Font_Effects = 0) {
            array_add(*drawn_texts, copy_string(prepared));
        };
        result.get_mouse_pointer_position = (window: Window_Type, right_handed: bool) -> int, int, bool {
            return cast(int) mouse.x, cast(int) mouse.y, true;
        };
        result.set_pointer_image = (image: GR.Pointer_Image, widget: *GR.Active_Widget) {
            array_add(*pointer_images, image);
        };
        result.get_font_at_size = (memory: [] u8, pixel_height: int) -> *GR.Font {
            return *font;
        };
        result.immediate_triangle = (p0: Vector3, p1: Vector3, p2: Vector3, c0 := Vector4.{ 1, 1, 1, 1 }, c1 := Vector4.{ 1, 1, 1, 1 }, c2 := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{ z = 1 }) {
            triangles += 1;
        };
        result.immediate_quad = (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {
            quads += 1;
        };
        result.immediate_flush = () {};
        return result;
    }

    key :: (code: Input.Key_Code, pressed: bool) -> Input.Event {
        event: Input.Event;
        event.type = .KEYBOARD;
        event.key_code = code;
        event.key_pressed = xx pressed;
        return event;
    }
    typed :: (character: u32) -> Input.Event {
        event: Input.Event;
        event.type = .TEXT_INPUT;
        event.utf32 = character;
        return event;
    }
    wheel :: (delta: s32) -> Input.Event {
        event: Input.Event;
        event.type = .MOUSE_WHEEL;
        event.wheel_delta = delta;
        return event;
    }

    // One UI frame: this frame's events reach Input (as the pump would deliver them), then GetRect.
    begin_frame :: (at: Vector2, events: ..Input.Event) {
        mouse = at;
        Input.input_per_frame_event_and_flag_update();
        for events Input.publish_input_event(it);
        now += 1.0 / 60;
        array_reset(*drawn_texts);
        GR.ui_per_frame_update(WINDOW, WIDTH, HEIGHT, now);
    }
    press :: () -> Input.Event {
        return key(.MOUSE_BUTTON_LEFT, true);
    }
    release :: () -> Input.Event {
        return key(.MOUSE_BUTTON_LEFT, false);
    }
    center :: (r: Rect) -> Vector2 {
        return .{ r.x + r.w / 2, r.y + r.h / 2 };
    }
    was_drawn :: (text: string) -> bool {
        for drawn_texts if it == text return true;
        return false;
    }

    Fruit :: enum {
        APPLE;
        PEAR;
        PLUM;
    }
    Options :: enum_flags u8 {
        SALT;
        PEPPER;
    }

    WORDS :: string.["apple", "apricot"];
    completions :: (prefix: string, data: *void) -> [] string, int {
        return WORDS, 2;
    }

    subwindow_draws: int;
    subwindow_content: Rect;

    main :: () {
        pure_helpers();
        saved_clipboard := os_clipboard_get_text();

        font.em_width = 8;
        font.max_ascender = 10;
        font.max_descender = 3;
        font.character_height = 13;
        custom := procs();
        GR.ui_init("", *custom);
        theme := GR.theme_proc_default();
        theme.font = *font;
        theme.label_theme.font = *font;
        GR.set_default_theme(theme);

        themes_and_sizes();
        buttons_and_boxes();
        dropdowns();
        sliders();
        text_inputs();
        regions();
        color_widgets();
        flowing_text();
        text_editing_helpers();
        compatibility_helpers();
        temporary_storage_frames();

        GR.ui_deinit();
        if saved_clipboard.count os_clipboard_set_text(saved_clipboard);
        print("ok\n");
    }

    themes_and_sizes :: () {
        procs := GR.default_theme_procs;
        for procs {
            theme := it();
            assert(theme.button_theme.surface_color.w == 1 && theme.background_color.w == 1, "theme % has transparent colors", it_index);
        }
        assert(GR.theme_proc_grayscale().background_color.x == GR.theme_proc_grayscale().background_color.y, "grayscale is not gray");
        GR.theme_proc_bentonite();
        GR.theme_proc_nimbus();
        GR.theme_proc_blood_vampire();
        GR.theme_proc_freddie_freeloader();
        GR.theme_proc_high_contrast();
        custom: GR.Overall_Theme;
        GR.set_theme_from_base_colors(*custom, .{ 0.1, 0.1, 0.1, 1 }, .{ 0.5, 0.5, 0.5, 1 }, .{ 1, 0, 0, 1 }, .{ 1, 1, 1, 1 }, .{ 0, 0, 0, 1 });
        assert(custom.button_theme.surface_color_flash.x == 1 && custom.text_color.x == 1 && custom.background_color.x == 0.1);

        r := get_rect(0, 0, 200, 50);
        assert(GR.get_float_parameter(r, .RELATIVE_TO_HEIGHT, 0.5, 7) == 25);
        assert(GR.get_float_parameter(r, .RELATIVE_TO_WIDTH, 0.5, 7) == 100);
        assert(GR.get_float_parameter(r, .ABSOLUTE_FROM_THEME_FIELD, 3, 7) == 3);
        assert(GR.get_float_parameter(r, .ABSOLUTE_FROM_GLOBAL, 3, 7) == 7);
        assert(GR.get_float_parameter(r, .ABSOLUTE_FROM_GLOBAL, 3, 7, override = 9) == 9);
        // Pinned sizes survive a frame; the rest follow the render target.
        GR.set_global_rounding_radius_in_pixels(5);
        GR.set_global_frame_thickness_in_pixels(2);
        GR.set_global_divider_thickness_in_pixels(6);
        GR.set_global_initial_divider_position_in_pixels(50);
        GR.set_global_slider_nub_thickness_in_pixels(12);
        begin_frame(.{ -100, -100 });
        shape: GR.Rectangle_Shape;
        shape.roundedness_type = .ABSOLUTE_FROM_GLOBAL;
        shape.frame_thickness_type = .ABSOLUTE_FROM_GLOBAL;
        before := triangles;
        thickness, radius := GR.rounded_rectangle(get_rect(0, 0, 100, 40), shape, .{ 1, 1, 1, 1 }, .{ 0, 0, 0, 1 });
        assert(thickness == 2 && radius == 5, "rounded_rectangle used thickness % radius %", thickness, radius);
        assert(triangles > before, "rounded_rectangle drew no triangles");
        GR.draw_popups();
    }

    buttons_and_boxes :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        button_rect := get_rect(10, 10, 100, 20);
        // Hovering: no press, but the tooltip given just before the button becomes active.
        begin_frame(center(button_rect));
        GR.tooltip("tip", null, 3);
        pressed, state, released := GR.button(button_rect, "Go", loc = here);
        assert(!pressed && !released && (state.status & .OVER) != 0, "hover: pressed %, status %", pressed, state.status);
        assert(was_drawn("Go"), "the button label was not drawn: %", drawn_texts);
        tip := GR.get_active_tooltip();
        assert(tip.text == "tip" && tip.int_data == 3, "active tooltip %", tip);
        assert(GR.has_focus(button_rect) && !GR.has_focus(get_rect(500, 500, 5, 5)));
        GR.draw_popups();

        begin_frame(center(button_rect), press());
        assert(GR.ui_get_button_state(.MOUSE_BUTTON_LEFT) == .START | .DOWN);
        pressed, state, released = GR.button(button_rect, "Go", loc = here);
        assert(pressed && !released, "press frame: pressed %, released %", pressed, released);
        // A second widget under the same click does not get it too.
        other_pressed := GR.button(button_rect, "Shadow", identifier = 1);
        assert(!other_pressed, "two widgets took one click");
        GR.draw_popups();

        begin_frame(center(button_rect), release());
        pressed, state, released = GR.button(button_rect, "Go", loc = here);
        assert(!pressed && released, "release frame: pressed %, released %", pressed, released);
        GR.draw_popups();

        // A click elsewhere does nothing; a disabled press ignores the click.
        begin_frame(.{ 400, 400 }, press());
        pressed = GR.button(button_rect, "Go", loc = here);
        assert(!pressed);
        GR.draw_popups();
        begin_frame(center(button_rect), press());
        pressed = GR.button(button_rect, "Go", disable_press = true, loc = here);
        assert(!pressed, "disable_press let the click through");
        GR.draw_popups();
        begin_frame(center(button_rect), release());
        GR.draw_popups();

        // Checkbox, radio buttons and flag boxes flip on the click frame only.
        box := get_rect(10, 40, 100, 20);
        checked := false;
        begin_frame(center(box), press());
        changed := GR.checkbox(box, "Check", *checked);
        assert(changed && checked, "checkbox: changed %, value %", changed, checked);
        GR.draw_popups();
        begin_frame(center(box), release());
        changed = GR.checkbox(box, "Check", *checked);
        assert(!changed && checked);
        GR.draw_popups();

        fruit := Fruit.APPLE;
        options: Options;
        radio := get_rect(10, 70, 100, 20);
        flags := get_rect(10, 100, 100, 20);
        begin_frame(center(radio), press());
        GR.radio_button(radio, "Pear", *fruit, .PEAR);
        GR.radio_button(get_rect(10, 130, 100, 20), *fruit, .PLUM);
        GR.checkbox_flags(flags, "Pepper", *options, .PEPPER);
        GR.checkbox_flags(get_rect(10, 160, 100, 20), *options, .SALT);
        assert(fruit == .PEAR && options == 0, "radio picked %, flags %", fruit, options);
        assert(was_drawn("PLUM") && was_drawn("SALT"), "enum labels not drawn: %", drawn_texts);
        GR.draw_popups();
        begin_frame(center(flags), release());
        GR.draw_popups();
        begin_frame(center(flags), press());
        GR.checkbox_flags(flags, "Pepper", *options, .PEPPER);
        assert(options == .PEPPER, "flags after a click: %", options);
        GR.draw_popups();
        begin_frame(center(flags), release());
        GR.draw_popups();
        begin_frame(center(flags), press());
        GR.checkbox_flags(flags, "Pepper", *options, .PEPPER);
        assert(options == 0, "flags after a second click: %", options);
        GR.draw_popups();
        begin_frame(center(flags), release());
        GR.draw_popups();
    }

    dropdowns :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        choices := string.["zero", "one", "two"];
        value: s32 = 0;
        r := get_rect(200, 10, 100, 20);
        begin_frame(center(r), press());
        changed, state := GR.dropdown(r, choices, *value, loc = here);
        assert(!changed && state.open, "the click did not open the dropdown");
        assert(was_drawn("zero"), "current choice not shown: %", drawn_texts);
        GR.draw_popups();
        assert(was_drawn("two"), "the open list did not draw its choices: %", drawn_texts);
        begin_frame(center(r), release());
        GR.dropdown(r, choices, *value, loc = here);
        GR.draw_popups();
        // The third choice sits three rows below the closed dropdown.
        row := r;
        row.y += r.h * 3;
        begin_frame(center(row), press());
        GR.dropdown(r, choices, *value, loc = here);
        GR.draw_popups();
        assert(value == 2, "picked index %", value);
        begin_frame(center(row), release());
        changed, state = GR.dropdown(r, choices, *value, loc = here);
        assert(!state.open, "the list stayed open after a pick");
        GR.draw_popups();
        // Keyboard: open it, then arrow up.
        begin_frame(center(r), press());
        GR.dropdown(r, choices, *value, loc = here);
        GR.draw_popups();
        begin_frame(center(r), release(), key(.ARROW_UP, true));
        changed = GR.dropdown(r, choices, *value, loc = here);
        assert(changed && value == 1, "arrow up: changed %, value %", changed, value);
        GR.draw_popups();
        begin_frame(center(r), key(.ESCAPE, true));
        changed, state = GR.dropdown(r, choices, *value, loc = here);
        assert(!state.open, "escape left the list open");
        GR.draw_popups();
        // A click outside every widget deactivates.
        begin_frame(.{ 700, 590 }, press());
        GR.draw_popups();
        begin_frame(.{ 700, 590 }, release());
        GR.draw_popups();
    }

    sliders :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        theme := GR.default_overall_theme.slider_theme;
        theme.use_spinboxes = false;
        theme.text_editable = false;
        r := get_rect(10, 200, 200, 20);
        amount: float = 0;
        begin_frame(center(r), press());
        changed := GR.slider(r, *amount, 0.0, 100.0, 1.0, *theme, "Amount: ", "%", loc = here);
        assert(changed && amount > 40 && amount < 60, "a click in the middle gave %", amount);
        GR.draw_popups();
        begin_frame(.{ r.x + r.w + 50, center(r).y });
        GR.slider(r, *amount, 0.0, 100.0, 1.0, *theme, loc = here);
        assert(amount == 100, "dragging past the end gave %", amount);
        GR.draw_popups();
        begin_frame(.{ r.x - 50, center(r).y }, release());
        GR.slider(r, *amount, 0.0, 100.0, 1.0, *theme, loc = here);
        assert(amount == 0, "dragging past the start gave %", amount);
        GR.draw_popups();
        begin_frame(.{ r.x + r.w, center(r).y });
        GR.slider(r, *amount, 0.0, 100.0, 1.0, *theme, loc = here);
        assert(amount == 0, "the slider kept following after release: %", amount);
        GR.draw_popups();

        // Integers step with the arrow keys while the pointer is over the slider.
        count: s32 = 5;
        int_rect := get_rect(10, 230, 200, 20);
        begin_frame(center(int_rect), key(.ARROW_RIGHT, true));
        changed = GR.slider(int_rect, *count, 0, 10, 2, *theme, identifier = 2, loc = here);
        assert(changed && count == 7, "arrow right: %", count);
        assert(was_drawn("7"), "slider value not shown: %", drawn_texts);
        GR.draw_popups();
        begin_frame(center(int_rect), key(.ARROW_LEFT, true), key(.ARROW_LEFT, true));
        GR.slider(int_rect, *count, 0, 10, 2, *theme, identifier = 2, loc = here);
        assert(count == 3, "two arrows left: %", count);
        GR.draw_popups();

        // Spin boxes: the "+" button steps up.
        spin := theme;
        spin.use_spinboxes = true;
        spin_rect := get_rect(10, 260, 200, 20);
        begin_frame(.{ spin_rect.x + spin_rect.w - 3, center(spin_rect).y }, press());
        GR.slider(spin_rect, *count, 0, 10, 1, *spin, identifier = 9, loc = here);
        assert(count == 4, "the + spin box gave %", count);
        GR.draw_popups();
        begin_frame(.{ 700, 590 }, release());
        GR.draw_popups();

        // Default formatting and parsing.
        formatted := GR.slider_format_text_float_default("x=", "!", 2.5, *theme, null, .DISPLAY);
        assert(String_Starts(formatted, "x=2.5") && formatted[formatted.count - 1] == #char "!", "formatted %", formatted);
        assert(GR.slider_format_text_int_default("", "", 42, *theme, null, .EDITING) == "42");
        assert(GR.slider_format_text_int_default("n", "s", 3, *theme, null, .DISPLAY) == "n3s");
        ok, parsed := GR.slider_parse_text_float_default("1.25", *theme, null);
        assert(ok && parsed == 1.25);
        ok2, parsed2 := GR.slider_parse_text_int_default("-17", *theme, null);
        assert(ok2 && parsed2 == -17);
        bad := GR.slider_parse_text_int_default("x", *theme, null);
        assert(!bad);
        slider_state: GR.Slider_State;
        GR.slider_draw_background_default(r, *theme, *slider_state);
        GR.slider_draw_foreground_default(r, r, *theme, *slider_state);
    }

    String_Starts :: (s: string, prefix: string) -> bool {
        return s.count >= prefix.count && string.{ prefix.count, s.data } == prefix;
    }

    // A game running on temporary storage it resets every frame. Widget states (with their text
    // and command history), slider runtimes and GetRect's per-frame lists are made during frames,
    // so they must survive the reset and the reuse of that storage.
    temporary_storage_frames :: () {
        push_allocator(temp);
        here := #location();
        theme := GR.default_overall_theme.text_input_theme;
        slider_theme := GR.default_overall_theme.slider_theme;
        slider_theme.use_spinboxes = false;
        slider_theme.text_editable = false;
        scribble :: () {
            reset_temporary_storage();
            junk := NewArray(256 * 1024, u8, false);
            memset(junk.data, 0xa5, junk.count);
        }
        for widget: 0..11 {
            r := get_rect(10, 300, 200, 20);
            begin_frame(center(r), press());
            action, text, state := GR.text_input(r, "go", *theme, identifier = 100 + widget, loc = here);
            first_state := state;
            GR.draw_popups();
            scribble();
            begin_frame(center(r), release(), key(.END, true), typed(cast(u32) (#char "a" + widget)));
            action, text, state = GR.text_input(r, "go", *theme, identifier = 100 + widget, loc = here);
            assert(state == first_state, "widget %: the state was not kept", widget);
            letters := "abcdefghijkl";
            assert(text == tprint("go%", string.{ 1, letters.data + widget }), "widget %: typed \"%\"", widget, text);
            GR.draw_popups();
            scribble();
            begin_frame(center(r), key(.ENTER, true));
            action, text, state = GR.text_input(r, "go", *theme, identifier = 100 + widget, loc = here);
            assert((action & .ENTERED) != 0 && state.command_history.count == 1, "widget %: enter gave %, % history", widget, action, state.command_history.count);
            GR.draw_popups();
            scribble();
            // The history entry and the text from before the edit outlive the frames that made them.
            assert(state.command_history[0] == text, "widget %: history \"%\", text \"%\"", widget, state.command_history[0], text);
            amount: float = 0;
            slider_rect := get_rect(10, 200, 200, 20);
            begin_frame(center(slider_rect), press());
            GR.slider(slider_rect, *amount, 0.0, 100.0, 1.0, *slider_theme, identifier = 100 + widget, loc = here);
            GR.draw_popups();
            scribble();
            begin_frame(.{ slider_rect.x + slider_rect.w + 50, center(slider_rect).y }, release());
            GR.slider(slider_rect, *amount, 0.0, 100.0, 1.0, *slider_theme, identifier = 100 + widget, loc = here);
            assert(amount == 100, "widget %: the slider gave %", widget, amount);
            GR.draw_popups();
            scribble();
        }
    }

    text_inputs :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        theme := GR.default_overall_theme.text_input_theme;
        r := get_rect(10, 300, 200, 20);
        begin_frame(center(r), press());
        action, text, state := GR.text_input(r, "start", *theme, loc = here);
        assert(state.active && text == "start", "click: active %, text \"%\"", state.active, text);
        GR.draw_popups();
        // Typed characters arrive through the widget's serial events, then edit keys.
        begin_frame(center(r), release(), key(.END, true), typed(#char "!"), typed(0xe9));
        action, text, state = GR.text_input(r, "start", *theme, loc = here);
        assert(text == "start!é", "typed text \"%\"", text);
        assert((action & .TEXT_MODIFIED) != 0, "typing did not report a modification: %", action);
        GR.draw_popups();
        begin_frame(center(r), key(.BACKSPACE, true), key(.HOME, true), key(.DELETE, true));
        action, text, state = GR.text_input(r, "start", *theme, loc = here);
        assert(text == "tart!", "after backspace, home, delete: \"%\"", text);
        GR.draw_popups();
        begin_frame(center(r), key(.ENTER, true));
        action, text, state = GR.text_input(r, "start", *theme, loc = here);
        assert((action & .ENTERED) != 0 && !state.active, "enter: action %, active %", action, state.active);
        GR.draw_popups();
        // Drawing a state directly.
        begin_frame(.{ 700, 590 });
        GR.draw(state, r);
        GR.draw_popups();

        // Escape restores the text from before the edit.
        begin_frame(center(r), press());
        action, text, state = GR.text_input(r, "start", *theme, identifier = 5, loc = here);
        GR.draw_popups();
        begin_frame(center(r), release(), typed(#char "x"));
        GR.text_input(r, "start", *theme, identifier = 5, loc = here);
        GR.draw_popups();
        begin_frame(center(r), key(.ESCAPE, true));
        action, text, state = GR.text_input(r, "start", *theme, identifier = 5, loc = here);
        assert((action & .ESCAPED) != 0 && text == "start", "escape: action %, text \"%\"", action, text);
        GR.draw_popups();

        // Number input: type a value and press enter.
        number: s32 = 1;
        number_theme: GR.Number_Input_Theme;
        number_rect := get_rect(10, 330, 200, 20);
        begin_frame(center(number_rect), press());
        GR.number_input(number_rect, "1", *number, 0, 100, *number_theme, loc = here);
        GR.draw_popups();
        begin_frame(center(number_rect), release(), key(.BACKSPACE, true), typed(#char "4"), typed(#char "2"));
        GR.number_input(number_rect, "1", *number, 0, 100, *number_theme, loc = here);
        GR.draw_popups();
        begin_frame(center(number_rect), key(.ENTER, true));
        changed := GR.number_input(number_rect, "1", *number, 0, 100, *number_theme, loc = here);
        assert(changed && number == 42, "number input: changed %, value %", changed, number);
        GR.draw_popups();
    }

    regions :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        // Scissors nest by intersection and restore on pop.
        begin_frame(.{ 700, 590 });
        sets := scissor_sets;
        clears := scissor_clears;
        GR.push_scissor(get_rect(0, 0, 100, 100));
        GR.push_scissor(get_rect(50, 50, 100, 100));
        assert(last_scissor[0] == 50 && last_scissor[2] == 100, "nested scissor %", last_scissor);
        assert(GR.is_visible_in_scissor(60, 60) && !GR.is_visible_in_scissor(20, 20));
        assert(GR.is_visible_in_scissor(get_rect(90, 90, 20, 20)) && !GR.is_visible_in_scissor(get_rect(0, 0, 10, 10)));
        GR.pop_scissor();
        assert(last_scissor[0] == 0 && last_scissor[2] == 100, "outer scissor restored as %", last_scissor);
        GR.pop_scissor();
        assert(scissor_sets == sets + 3 && scissor_clears == clears + 1, "% sets, % clears", scissor_sets - sets, scissor_clears - clears);
        assert(GR.is_visible_in_scissor(20, 20));
        GR.draw_popups();

        // A scrollable region scrolls with the wheel while the pointer is over it.
        area := get_rect(300, 100, 150, 150);
        scroll: float = 0;
        for 1..2 {
            if it == 1 begin_frame(center(area));
            else begin_frame(center(area), wheel(-120));
            state, inside := GR.begin_scrollable_region(area);
            assert(inside.w > 0 && inside.w < area.w && inside.h > 0, "scroll region inside %", inside);
            GR.label(get_rect(inside.x, inside.y - scroll, 100, 20), "row", null);
            GR.end_scrollable_region(state, inside.x + inside.w, inside.y + 1000, *scroll);
            GR.draw_popups();
        }
        assert(scroll > 0, "the wheel did not scroll: %", scroll);

        // A split region: the two sides and the divider fill it.
        split := get_rect(10, 400, 300, 100);
        begin_frame(.{ 700, 590 });
        state, first, second, divider := GR.slidable_region(split);
        assert(abs(first.w + divider.w + second.w - split.w) < 0.01 || abs(first.h + divider.h + second.h - split.h) < 0.01, "split % % %", first, divider, second);
        GR.switch_to_second_rect(state);
        GR.switch_to_first_rect(state);
        GR.finish(state);
        top, middle, bottom := GR.compute_rects(split, state, 4);
        assert(top.w + middle.w + bottom.w == split.w || top.h + middle.h + bottom.h == split.h);
        GR.draw_popups();

        // A subwindow draws its contents through its callback, from the popup list.
        info: GR.Subwindow_Info;
        info.rect = get_rect(450, 300, 200, 150);
        info.title_text = "Sub";
        info.draw = (content: Rect, data: *void) {
            subwindow_draws += 1;
            subwindow_content = content;
        };
        begin_frame(.{ 700, 590 });
        sub := GR.add_subwindow(*info, loc = here);
        assert(sub != null);
        GR.draw_popups();
        assert(subwindow_draws == 1 && subwindow_content.w > 0 && subwindow_content.w <= info.rect.w, "subwindow drew % times into %", subwindow_draws, subwindow_content);
        title := GR.get_title_bar_height(*sub.theme);
        outer := GR.subwindow_rect_from_content_rect(get_rect(0, 0, 100, 100), *sub.theme);
        assert(outer.h >= 100 && outer.w >= 100, "outer rect % for a 100x100 content (title %)", outer, title);
        GR.raise(sub);
        GR.start_resizing(sub, get_rect(0, 0, 200, 100));
        assert(sub.resizing_aspect == 2, "resizing aspect %", sub.resizing_aspect);
        assert(GR.get_pointer_from_resize_control(0) == .NORMAL);
        assert(GR.rescale(get_rect(10, 10, 10, 10), WIDTH / 2, HEIGHT / 2).w == 20);
        // Dragging the title bar moves the window.
        bar := sub.title_bar_rect;
        if bar.h > 0 {
            start := info.rect;
            begin_frame(center(bar), press());
            GR.add_subwindow(*info, loc = here);
            GR.draw_popups();
            begin_frame(center(bar) + .{ 30, 20 });
            GR.add_subwindow(*info, loc = here);
            GR.draw_popups();
            begin_frame(center(bar) + .{ 30, 20 }, release());
            GR.add_subwindow(*info, loc = here);
            GR.draw_popups();
            assert(info.rect.x != start.x || info.rect.y != start.y, "dragging the title bar did not move the subwindow");
        }
        // Cursors go to the program's callback.
        GR.pointer_image_sink = (image: GR.Pointer_Image, widget: *GR.Active_Widget) {
            array_add(*pointer_images, image);
        };
        begin_frame(.{ 5, 590 });
        before := pointer_images.count;
        GR.getrect_set_pointer_image(.PRESSABLE, null);
        assert(pointer_images.count == before + 1 && pointer_images[before] == .PRESSABLE, "pointer images %", pointer_images);
        GR.init_pointers();
        GR.pointer_image_sink = null;
        GR.draw_popups();
    }

    color_widgets :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        rgb := Vector3.{ 1, 0, 0 };
        r := get_rect(500, 10, 250, 250);
        state: *GR.Color_Picker_State;
        for 1..2 {
            begin_frame(.{ 700, 590 });
            _, _, picker := GR.color_picker(r, *rgb, loc = here);
            state = picker;
            GR.draw_popups();
        }
        assert(state != null && rgb.x == 1 && rgb.y == 0, "the picker changed the color without input: %", rgb);
        picker_theme := GR.default_overall_theme.color_picker_theme;
        assert(GR.get_desired_aspect_ratio(*picker_theme) > 0);
        GR.set_original_and_current_color_rgb(state, .{ 0, 1, 0 });
        assert(state.current_rgb_color.y == 1 && state.original_rgb_color.y == 1);
        // The picker keeps hue in degrees.
        GR.set_current_color_hsv(state, .{ 240, 1, 1 });
        assert(abs(state.current_rgb_color.z - 1) < 0.01 && state.current_rgb_color.x < 0.01, "hsv blue gave %", state.current_rgb_color);
        GR.set_current_color_rgb(state, *picker_theme, .{ 1, 1, 0 });
        assert(abs(state.current_hsv_color.x - 60) < 0.1, "yellow hue %", state.current_hsv_color.x);
        argb := GR.hsv_to_argb(.{ 360, 1, 1 });
        assert(argb.x == 1 && argb.y == 0 && argb.w == 1, "hsv_to_argb red: %", argb);
        stashed := state.stashed_colors.count;
        GR.stash_color(state, .{ 0.5, 0.5, 0.5 });
        assert(state.stashed_colors.count == min(stashed + 1, GR.MAX_STASHED_COLORS), "% stashed colors after %", state.stashed_colors.count, stashed);
        assert(state.stashed_colors[state.stashed_colors.count - 1].x == 0.5, "the newest stash is last");
        mode := state.hsv_picker_in_rectangle_mode;
        GR.toggle_rectangle_picker_mode(state);
        assert(state.hsv_picker_in_rectangle_mode != mode);
        warped := state.circle_picker_in_warped_mode;
        GR.toggle_circle_picker_mode(state);
        assert(state.circle_picker_in_warped_mode != warped);
        // Every picker mode draws.
        for GR.Color_Pick_Mode.HSV_CIRCLE..GR.Color_Pick_Mode.TYPE_INS {
            state.color_pick_mode = it;
            begin_frame(.{ 700, 590 });
            GR.color_picker(r, *rgb, loc = here);
            GR.draw_popups();
        }

        // The animation editor: keyframes, interpolation and samples.
        editor_rect := get_rect(10, 520, 400, 60);
        editor: *GR.Color_Animation_Editor_State;
        begin_frame(.{ 700, 590 });
        editor = GR.color_animation_editor(editor_rect, loc = here);
        GR.draw_popups();
        frames := GR.Color_Keyframe.[.{ .{ 0, 0, 1, 1 }, 1 }, .{ .{ 1, 0, 0, 1 }, 0 }];
        GR.set_keyframes(editor, frames);
        sorted := GR.get_keyframes(editor);
        assert(sorted.count == 2 && sorted[0].t == 0 && sorted[0].color_rgb.x == 1 && sorted[1].t == 1, "sorted keyframes %", sorted);
        interp := GR.get_interpolation(editor);
        interp.geometry = .LINEAR;
        interp.color_space = .RGB;
        GR.set_interpolation(editor, interp);
        assert(GR.get_interpolation(editor).geometry == .LINEAR);
        samples: [5] Vector4;
        GR.generate_color_samples(GR.get_sorted_keyframes(editor), interp, samples);
        assert(abs(samples[0].x - 1) < 0.01 && abs(samples[4].z - 1) < 0.01, "samples run % .. %", samples[0], samples[4]);
        assert(abs(samples[2].x - 0.5) < 0.05 && abs(samples[2].z - 0.5) < 0.05, "midpoint sample %", samples[2]);
        animation_theme := GR.default_overall_theme.color_animation_editor_theme;
        assert(GR.get_desired_aspect_ratio(*animation_theme) > 0);
        assert(GR.find_index_of_t(GR.get_sorted_keyframes(editor), 0.5) == 0 && GR.find_index_of_t(GR.get_sorted_keyframes(editor), 1) == 1);
        assert(GR.find_index_of_t(GR.get_sorted_keyframes(editor), -1) == -1);
        // Recoloring the t = 1 keyframe from the others leaves it the color of the t = 0 one.
        GR.init_color_by_interpolation(editor, 0);
        recolored := editor.color_keyframes[0].color_rgb;
        assert(abs(recolored.x - 1) < 0.01 && recolored.z < 0.01, "interpolated keyframe %", recolored);
        begin_frame(.{ 700, 590 });
        before := quads + triangles;
        GR.draw_color_gradient_bar(editor_rect, *animation_theme, editor, 2);
        assert(quads + triangles > before, "the gradient bar drew nothing");
        GR.draw_popups();
        GR.open_color_picker(*animation_theme, editor, 0, editor_rect);
        GR.open_time_view(*animation_theme, editor, editor_rect);
        for 1..2 {
            begin_frame(.{ 700, 590 });
            GR.color_animation_editor(editor_rect, loc = here);
            GR.draw_popups();
        }
        GR.copy_output_to_clipboard(editor, .SOURCE_KEYFRAMES);
        copied := os_clipboard_get_text();
        #if OS != .LINUX {
            // Linux needs a clipboard helper program, which CI does not have.
            assert(copied.count > 0, "nothing was copied");
        }
        GR.copy_output_to_clipboard(editor, .RESAMPLED_ARRAY);
    }

    flowing_text :: () {
        // Each widget keeps its state across frames only when called from the same place.
        here := #location();
        pieces := GR.Unflowed_Line_Piece.[.{ font = *font, text = "aaaa bbbb cccc", color = .{ 1, 1, 1, 1 } }];
        lines := GR.Unflowed_Line.[.{ pieces = pieces }];
        text: GR.Flowable_Text;
        array_add(*text.unflowed_lines, ..lines);
        // 14 characters of 8 pixels in a 50-pixel column: several lines.
        GR.flow_text(*text, get_rect(0, 0, 50, 200), 1);
        assert(text.num_flowed_lines > 1, "% flowed lines", text.num_flowed_lines);
        for line: text.flowed_lines {
            width := 0;
            for line.pieces width += 8 * it.text.count;
            assert(width <= 50, "a flowed line is % pixels wide", width);
        }
        GR.maybe_reflow_text_if_rect_dimensions_changed(*text, get_rect(0, 0, 500, 200));
        assert(text.num_flowed_lines == 1, "after widening: % lines", text.num_flowed_lines);
        area := get_rect(10, 10, 300, 100);
        begin_frame(.{ 15, 15 }, press());
        GR.text_display(area, *text, 0, 20, loc = here);
        assert(was_drawn("aaaa bbbb cccc"), "text_display drew %", drawn_texts);
        GR.draw_popups();
        begin_frame(.{ 60, 15 });
        GR.text_display(area, *text, 0, 20, loc = here);
        GR.draw_popups();
        begin_frame(.{ 60, 15 }, release());
        GR.text_display(area, *text, 0, 20, loc = here);
        GR.draw_popups();
        found, line, character := GR.find_line_and_character(*text, WIDTH, HEIGHT, 30, 15);
        assert(found && line == 0 && character >= 1 && character <= 3, "pointer at x 30 is line %, character %", line, character);
        GR.deinit(*text);
        // The unflowed lines stay with the caller.
        assert(text.flowed_lines.count == 0 && text.num_flowed_lines == 0 && text.unflowed_lines.count == 1);
        array_reset(*text.unflowed_lines);
    }

    text_editing_helpers :: () {
        state: GR.Text_Input_State;
        GR.set_text(*state, "hello world");
        assert(GR.get_result(*state) == "hello world" && state.insert_point == 11);
        GR.select_all(*state);
        assert(GR.get_selection(*state) == "hello world");
        GR.deselect(*state.selection);
        assert(!state.selection.active);
        state.insert_point = 99;
        GR.clamp_insert_point(*state);
        assert(state.insert_point == 11);
        GR.move_cursor_left(*state);
        GR.move_cursor_left(*state, true);
        assert(state.insert_point == 9 && GR.get_selection(*state) == "l", "selection after shift-left: \"%\"", GR.get_selection(*state));
        GR.move_cursor_right(*state);
        GR.move_cursor_to_end(*state);
        GR.move_cursor_right_by_word(*state);
        assert(state.insert_point == 11);
        state.insert_point = 0;
        GR.move_cursor_right_by_word(*state, true);
        assert(GR.get_selection(*state) == "hello ", "word selection \"%\"", GR.get_selection(*state));
        GR.delete_from_cursor(*state);
        assert(state.text == "world", "delete with a selection: \"%\"", state.text);
        GR.delete_from_cursor(*state, by_word = true);
        assert(state.text == "", "delete by word: \"%\"", state.text);
        GR.append_text(*state, "abc");
        GR.append_text(*state, "def");
        assert(state.text == "abcdef");
        GR.clear_text(*state);
        assert(state.text == "" && (state.pending_result_actions & .TEXT_MODIFIED) != 0);
        GR.set_last_inert_text(*state, "inert");
        assert(state.last_inert_text == "inert");

        // History: up goes back through earlier commands, down returns to the edit.
        array_add(*state.command_history, copy_string("first"), copy_string("second"));
        state.command_history_index = state.command_history.count;
        GR.set_text(*state, "draft");
        GR.navigate_history_backward(*state);
        assert(state.text == "second", "history back: \"%\"", state.text);
        GR.navigate_history_backward(*state);
        assert(state.text == "first");
        GR.navigate_history_forward(*state);
        GR.navigate_history_forward(*state);
        assert(state.text == "draft", "history forward: \"%\"", state.text);
        GR.refresh_input_from_history(*state);

        // Auto-complete offers the matches and accepts one.
        state.auto_complete = .{ proc = completions };
        GR.set_text(*state, "ap");
        GR.update_auto_complete(*state);
        assert(state.match_array.count == 2 && state.longest_match == 2);
        GR.advance_auto_complete(*state, false);
        assert(state.match_selection == 1);
        GR.advance_auto_complete(*state, true);
        assert(GR.accept_auto_complete(*state) && state.text == "apple", "accepted \"%\"", state.text);
        GR.reset_auto_complete(*state);
        assert(!GR.accept_auto_complete(*state));

        // Multi-click selection: two clicks pick a word, three the whole line.
        GR.set_text(*state, "one two three");
        state.num_consecutive_clicks = 2;
        GR.update_multiclick_selection(*state, 5);
        assert(GR.get_selection(*state) == "two", "double click selected \"%\"", GR.get_selection(*state));
        state.num_consecutive_clicks = 3;
        GR.update_multiclick_selection(*state, 5);
        assert(GR.get_selection(*state) == "one two three");
        x0, x1 := GR.get_selection_endpoints(*font, "abcd", 0, 100, 1, 3);
        assert(x0 == 108 && x1 == 124, "selection endpoints % %", x0, x1);
        forward := GR.make_forward_selection(.{ start_line = 2, end_line = 1, start_character = 5, end_character = 1, active = true });
        assert(forward.start_line == 1 && forward.start_character == 1 && forward.end_line == 2);

        // Clipboard: copy, cut and paste through the system clipboard.
        GR.set_text(*state, "copy me");
        GR.select_all(*state);
        GR.copy_selection_to_clipboard(*state);
        #if OS != .LINUX assert(os_clipboard_get_text() == "copy me");
        GR.copy_selection_to_clipboard(*state, cut = true);
        assert(state.text == "", "cut left \"%\"", state.text);
        #if OS != .LINUX {
            GR.insert_clipboard_at_cursor(*state);
            assert(state.text == "copy me", "paste gave \"%\"", state.text);
        }
    }

    compatibility_helpers :: () {
        assert(GR.get_aspect(get_rect(0, 0, 200, 100)) == 2);
        assert(!GR.is_grabbing());
        assert(GR.get_roundedness_for_rect(get_rect(0, 0, 100, 40), .ABSOLUTE_FROM_THEME_FIELD, 50, 0) == 20, "roundedness is limited to half the short side");
        y0, y1 := GR.get_y_range(*font, 100);
        assert(y1 - y0 == 13, "text y range % .. %", y0, y1);
        assert(GR.to_vec3(.{ 1, 2 }).z == 0);
        assert(GR.move_toward(0, 10, 3) == 3 && GR.move_toward(10, 0, 3) == 7 && GR.move_toward(9, 10, 3) == 10);
        assert(GR.move_toward(10, 0, 3, 1) == 9, "the falling rate applies going down");
        value: float = 0;
        GR.move_toward(*value, 1, 0.5, 1);
        assert(value == 0.5, "move_toward by pointer gave %", value);
        assert(abs(GR.fract(2.25) - 0.25) < 0.0001 && abs(GR.fract(-0.25) - 0.75) < 0.0001, "fract % %", GR.fract(2.25), GR.fract(-0.25));
        clamped: s32;
        GR.set_and_maybe_clamp(150, *clamped, 0, 100, true, true);
        assert(clamped == 100);
        GR.set_and_maybe_clamp(-5, *clamped, 0, 100, false, true);
        assert(clamped == -5);

        theme := GR.default_overall_theme.button_theme;
        surface := theme.surface_color;
        GR.invert_button_theme(*theme);
        assert(theme.surface_color.x != surface.x || surface.x == 0.5, "invert_button_theme kept %", surface);
        scroll_theme := GR.default_overall_theme.scrollable_region_theme;
        GR.invert_scrollbar(*scroll_theme);
        assert(GR.maybe_diagonalize(.EAST, 0, 100, 5, .NORTH, .SOUTH, 2) == .EAST | .NORTH);

        begin_frame(.{ 700, 590 });
        GR.set_scissor(get_rect(1, 2, 3, 4));
        button_state: GR.Button_State;
        GR.my_draw_text(*font, 5, 5, "hey", .{ 1, 1, 1, 1 });
        assert(was_drawn("hey"));
        GR.core_draw_button_label(get_rect(0, 0, 100, 20), "lbl", *button_state, *theme);
        assert(was_drawn("lbl"));
        before := triangles;
        GR.draw_arrow(*button_state, .{ 0, 0, 0 }, .{ 10, 0, 0 }, .{ 5, 10, 0 }, *theme);
        assert(triangles > before, "draw_arrow drew nothing");
        assert(GR.label(get_rect(0, 0, 100, 20), "label", null) == 40);
        assert(GR.text_size(*font, "abc") == 24 && GR.em(2, *font) == 16);
        tx := GR.text_origin(*GR.default_overall_theme.label_theme, *font, get_rect(0, 0, 100, 20), 40);
        assert(tx == 30, "centred text starts at %", tx);
        GR.label_from_prepared_text(get_rect(0, 0, 100, 20), *font, *GR.default_overall_theme.label_theme);
        GR.draw_popups();

        state: GR.Text_Input_State;
        GR.set_text(*state, "abcdef");
        assert(GR.get_insert_point_for_cursor(*state, *font, state.text, 0, 17, 0) == 2, "insert point for x 17: %", GR.get_insert_point_for_cursor(*state, *font, state.text, 0, 17, 0));
        state.active = true;
        GR.press_enter(*state);
        assert((state.pending_result_actions & .ENTERED) != 0);
        GR.press_escape(*state);
        assert((state.pending_result_actions & .ESCAPED) != 0);
        GR.press_tab(*state);
        // A left press over the box activates it (the pointer is still at 700, 590).
        state.rect = get_rect(650, 550, 100, 50);
        assert(GR.text_input_check_for_activation(*state, key(.MOUSE_BUTTON_LEFT, true)), "a press inside did not activate");
        assert(!GR.text_input_check_for_activation(*state, key(.MOUSE_BUTTON_LEFT, false)), "a release activated");
        state.rect = get_rect(0, 0, 10, 10);
        assert(!GR.text_input_check_for_activation(*state, key(.MOUSE_BUTTON_LEFT, true)), "a press outside activated");
        // Simp's loader (stb_image, which has no Windows build) refuses bytes that are not an image.
        #if OS != .WINDOWS assert(GR.require_texture("not an image") == null);

        // A second window gets its own frame state.
        GR.switch_window(cast(Window_Type) 2, 320, 240);
        GR.switch_window(WINDOW, WIDTH, HEIGHT);
        GR.getrect_handle_event(key(.F1, true));
        GR.getrect_handle_event(key(.F1, false), WINDOW);
        GR.windows_per_frame_update();
        GR.events_per_frame_update();
        GR.active_widgets_per_frame_update();
        GR.deactivate(null);
    }
}

// Shared by every target: rectangle cutting and color conversion need no UI state.
pure_helpers :: () {
    r := get_rect(10, 20, 100, 40);
    left, rest := cut_left(r, 30, 5);
    assert(left.x == 10 && left.w == 30 && rest.x == 45 && rest.w == 65, "cut_left % %", left, rest);
    right, rest2 := cut_right(r, 30, 5);
    assert(right.x == 80 && right.w == 30 && rest2.x == 10 && rest2.w == 65, "cut_right % %", right, rest2);
    top, below := cut_top(r, 10);
    assert(top.y == 20 && top.h == 10 && below.y == 30 && below.h == 30, "y-down cut_top % %", top, below);
    overlap := intersect(r, get_rect(100, 50, 50, 50));
    assert(overlap.x == 100 && overlap.y == 50 && overlap.w == 10 && overlap.h == 10, "intersect %", overlap);
    envelope := bound(r, get_rect(0, 0, 5, 5));
    assert(envelope.x == 0 && envelope.y == 0 && envelope.w == 110 && envelope.h == 60, "bound %", envelope);
    assert(is_inside(10, 20, r) && !is_inside(110, 20, r));
    p0, p1, p2, p3 := get_quad(r);
    assert(p0.x == 10 && p2.x == 110 && p0.y != p2.y && p1.x == 110 && p3.x == 10, "get_quad % % % %", p0, p1, p2, p3);
    here := #location();
    assert(get_hash(here, 1) == get_hash(here, 1) && get_hash(here, 1) != get_hash(here, 2), "get_hash is not a function of its inputs");

    // HSV round trips for the primaries, a gray and an arbitrary color.
    colors := Vector3.[.{ 1, 0, 0 }, .{ 0, 1, 0 }, .{ 0, 0, 1 }, .{ 0.5, 0.5, 0.5 }, .{ 0.2, 0.6, 0.9 }];
    for colors {
        h, s, v := rgb_to_hsv(it.x, it.y, it.z);
        r2, g2, b2 := hsv_to_rgb(h, s, v);
        assert(abs(r2 - it.x) < 0.001 && abs(g2 - it.y) < 0.001 && abs(b2 - it.z) < 0.001, "hsv round trip of % gave % % %", it, r2, g2, b2);
    }
    h, s, v := rgb_to_hsv(0, 1, 0);
    assert(abs(h - 1.0 / 3) < 0.001 && s == 1 && v == 1, "green is hsv % % %", h, s, v);
    gray := Vector3.{ 0.5, 0.5, 0.5 };
    assert(darken(gray, 0.5).x < 0.5 && whiten(gray, 0.5).x > 0.5, "darken % whiten %", darken(gray, 0.5), whiten(gray, 0.5));
    bright := brighten(gray, 0.5);
    assert(bright.x >= 0.5 && bright.x <= 1, "brighten %", bright);
}
