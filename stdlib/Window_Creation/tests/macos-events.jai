#import "Basic";
I :: #import "Input";
WC :: #import "Window_Creation";
#load "../../internal/window/cocoa.jai";

post_key :: (window: *A.NSWindow, down: bool, characters: string, repeat := false) {
    Create :: #type (receiver: *void, selector: O.Selector, type: u64, location: O.NSPoint,
                     modifiers: u64, timestamp: float64, window_number: s64, context: *void,
                     characters: *void, unmodified: *void, repeat: s8, code: u16) -> *void #c_call;
    create := cast(Create) native_send;
    number := result_message(window, "windowNumber", s64);
    type: u64 = 11;

    if down type = 10;

    event := create(native_class("NSEvent"), O.selector("keyEventWithType:location:modifierFlags:timestamp:windowNumber:context:characters:charactersIgnoringModifiers:isARepeat:keyCode:"),
                    type, .{}, 1 << 17, 0, number, null, native_string(characters), native_string("a"), cast(s8) repeat, 0);

    assert(event != null);
    assert(O.to_string(cast(*O.NSString) result_message(event, "characters", *void)) == characters);
    Post :: #type (receiver: *void, selector: O.Selector, event: *void, at_start: s8) #c_call;
    post := cast(Post) native_send;

    app := result_message(native_class("NSApplication"), "sharedApplication", *void);

    assert(app != null);
    post(app, O.selector("postEvent:atStart:"), event, 0);
}

main :: () {
    WC.init_mac_app();
    window := WC.create_window(320, 200, "Clean-room native event test", 40, 200);

    assert(window != null);
    I.update_window_events();
    I.set_custom_cursor_handling(true);
    assert(result_message(window, "areCursorRectsEnabled", s8) == 0);
    I.set_custom_cursor_handling(false);
    assert(result_message(window, "areCursorRectsEnabled", s8) != 0);
    post_key(window, true, "A");
    I.update_window_events();
    found_key := false;
    found_text := false;

    for event: I.events_this_frame {
        if event.type == .KEYBOARD && event.key_code == cast(I.Key_Code) #char "A" {
            assert(event.key_pressed == 1 && !event.repeat && event.shift_pressed);
            assert(event.text_input_count == 1);
            assert(event.window == window);
            found_key = true;
        }

        if event.type == .TEXT_INPUT {
            if event.utf32 != #char "A" || event.text_input_count != 0 || event.window != window {
                print("unexpected native text: value=%, count=%, target_window=%\n", event.utf32, event.text_input_count, event.window == window);
                for observed: I.events_this_frame print("frame event: type=%, key=%, value=%, down=%, repeat=%, flags=%\n", observed.type, observed.key_code, observed.utf32, observed.key_pressed, observed.repeat, observed.modifier_flags.packed);
            }

            // AppKit re-translates posted physical A+shift to layout text.
            // The independent Objective-C environment probe confirms this.
            assert(event.utf32 == #char "A" && event.text_input_count == 0);
            assert(event.window == window);
            found_text = true;
        }
    }

    assert(found_key && found_text);
    assert((I.input_button_states[#char "A"] & .START) != .NONE);
    post_key(window, true, "a", true);
    I.update_window_events();
    assert(I.input_button_states[#char "A"] == .DOWN);
    post_key(window, false, "a");
    I.update_window_events();
    assert(I.input_button_states[#char "A"] == .END);
    external: I.Event;
    external.type = .TEXT_INPUT;
    external.window = window;
    external.utf32 = 0x1f600;
    external.text_input_count = 1;
    array_add(*I.osx_externally_generated_events, external);
    I.update_window_events();
    assert(I.osx_externally_generated_events.count == 0);
    assert(I.events_this_frame.count == 1 && I.events_this_frame[0].utf32 == 0x1f600);
    // Native notifications must propagate actual resize and close lifecycle.
    pool := autorelease_pool();

    defer release(pool);
    argument_message(window, "setContentSize:", O.NSSize.{ 360, 240 });
    I.update_window_events();
    resizes := I.get_window_resizes();

    assert(resizes.count > 0);
    assert(resizes[resizes.count - 1].width == 360 && resizes[resizes.count - 1].height == 240);
    // Close with the native cursor-policy retain still active. Removal must
    // finish before release; re-enabling afterward must not touch a dead window.
    I.set_custom_cursor_handling(true);
    close_window(window);
    I.update_window_events();
    I.set_custom_cursor_handling(false);
    found_close := false;
    for event: I.events_this_frame if event.type == .QUIT && event.window == window found_close = true;

    assert(found_close);
    print("ok\n");
}
