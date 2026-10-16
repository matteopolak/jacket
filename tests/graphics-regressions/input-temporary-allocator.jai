// Reviewed mutable publication arrays with explicit persistent caller storage.
// Real window identities, 300 temporary-reset frames, owned filenames and payloads.
#assert OS == .MACOS, "Input fixture requires reviewed macOS osx_externally_generated_events";

#import "Basic";
#import "Window_Type";
Input :: #import "Input";

FRAMES :: 300;
WINDOWS :: 2;
windows: [WINDOWS] Window_Type;

window_for :: (index: int) -> Window_Type {
    return windows[index];
}

function_key :: (frame: int) -> Input.Key_Code {
    return cast(Input.Key_Code) (cast(int) Input.Key_Code.F1 + frame % 12);
}

key_event :: (key: Input.Key_Code, pressed: bool) -> Input.Event {
    event: Input.Event;
    event.type = .KEYBOARD;
    event.key_code = key;
    event.key_pressed = cast(u32) pressed;
    event.window = INVALID_WINDOW;

    return event;
}

scribble_over_temporary_storage :: () {
    junk := talloc_string(64 * 1024);
    memset(junk.data, 0xa5, junk.count);
}

owned_copy :: (text: string, heap: Allocator) -> string {
    push_allocator(heap);
    return copy_string(text);
}

owned_free :: (text: string, heap: Allocator) {
    push_allocator(heap);
    free(text);
}

check_frame :: (frame: int, heap: Allocator) {
    window := window_for(frame % WINDOWS);
    previous := window_for((frame + WINDOWS - 1) % WINDOWS);
    presses, releases, shifts, texts, wheels, drops: int;

    for event: Input.events_this_frame {
        if event.type == .KEYBOARD {
            if event.key_code == .SHIFT {
                assert(event.window == INVALID_WINDOW);
                assert((event.key_pressed != 0) == (frame % 2 == 0));
                shifts += 1;
            } else if event.key_pressed != 0 {
                assert(event.window == window && event.key_code == function_key(frame));
                presses += 1;
            } else {
                assert(frame > 0 && event.window == previous && event.key_code == function_key(frame - 1));
                releases += 1;
            }
        } else if event.type == .TEXT_INPUT {
            assert(event.window == window && event.utf32 == cast(u32) (#char "a" + frame % 26));
            assert(event.text_input_count == 1);
            texts += 1;
        } else if event.type == .MOUSE_WHEEL {
            assert(event.window == INVALID_WINDOW && event.wheel_delta == 120);
            wheels += 1;
        } else if event.type == .DRAG_AND_DROP_FILES {
            assert(event.window == window && event.files.count == 2);
            assert(event.files[0] == tprint("file-%.txt", frame) && event.files[1] == "second.txt");
            drops += 1;
        }
    }

    assert(presses == 1 && shifts == 1 && texts == 1 && wheels == 1);
    assert(releases == (ifx frame > 0 then 1 else 0));
    assert(drops == (ifx frame % 10 == 0 then 1 else 0));
    assert(Input.events_this_frame.allocator.proc == heap.proc && Input.events_this_frame.allocator.data == heap.data);
    assert(Input.osx_externally_generated_events.allocator.proc == heap.proc);
    current_found, other_found: bool;

    for resize: Input.get_window_resizes() {
        if resize.window == window && resize.width == 100 + frame && resize.height == 50 + frame current_found = true;
        if resize.window == previous && resize.width == 7 && resize.height == 8 other_found = true;
    }

    assert(current_found && other_found);
}

main :: () {
    defer dispose_native_windows();
    heap := context.allocator;

    defer context.allocator = heap;

    // The reviewed arrays are mutable: callers explicitly select lifetime.
    Input.events_this_frame.allocator = heap;
    Input.osx_externally_generated_events.allocator = heap;
    windows[0] = owned_native_window(160, 120, "Input allocator first");
    assert(windows[0] != INVALID_WINDOW);
    Input.update_window_events();
    windows[1] = owned_native_window(180, 140, "Input allocator second");
    assert(windows[1] != INVALID_WINDOW && windows[1] != windows[0]);
    Input.update_window_events();
    Input.input_per_frame_event_and_flag_update();
    context.allocator = temp;

    for frame: 0..FRAMES - 1 {
        window := window_for(frame % WINDOWS);
        press := key_event(function_key(frame), true);
        press.window = window;
        array_add(*Input.osx_externally_generated_events, press);
        text: Input.Event;
        text.type = .TEXT_INPUT;
        text.window = window;
        text.utf32 = cast(u32) (#char "a" + frame % 26);
        text.text_input_count = 1;
        array_add(*Input.osx_externally_generated_events, text);
        wheel: Input.Event;
        wheel.type = .MOUSE_WHEEL;
        wheel.window = INVALID_WINDOW;
        wheel.wheel_delta = 120;
        array_add(*Input.osx_externally_generated_events, wheel);
        array_add(*Input.osx_externally_generated_events, key_event(.SHIFT, frame % 2 == 0));
        first_name, second_name: string;
        owned_files: [..] string;
        owned_files.allocator = heap;

        if frame % 10 == 0 {
            // Caller owns strings; the frame list gets persistent explicit storage.
            first_name = owned_copy(tprint("file-%.txt", frame), heap);
            second_name = owned_copy("second.txt", heap);
            drop: Input.Event;
            drop.type = .DRAG_AND_DROP_FILES;
            drop.window = window;
            array_add(*owned_files, first_name);
            array_add(*owned_files, second_name);
            drop.files = owned_files;
            array_add(*Input.osx_externally_generated_events, drop);
        }

        Input.update_window_events();
        Input.add_resize_record(window, cast(s32) (100 + frame), cast(s32) (50 + frame));
        Input.add_resize_record(window_for((frame + WINDOWS - 1) % WINDOWS), 7, 8);
        check_frame(frame, heap);
        reset_temporary_storage();
        scribble_over_temporary_storage();
        check_frame(frame, heap);
        reset_temporary_storage();

        if owned_files.data {
            // Retire only this caller's borrowed payload before frame cleanup;
            // do not assume an undocumented Input ownership transfer policy.
            for *event: Input.events_this_frame {
                if event.files.data == owned_files.data event.files = .{};
            }

            array_free(owned_files);
        }

        Input.input_per_frame_event_and_flag_update();
        // No assumption that Input owns/frees the filename strings.
        if first_name.data owned_free(first_name, heap);
        if second_name.data owned_free(second_name, heap);

        release := key_event(function_key(frame), false);
        release.window = window;
        array_add(*Input.osx_externally_generated_events, release);
    }

    context.allocator = heap;
    Input.update_window_events();
    assert(has_key(window_for((FRAMES - 1) % WINDOWS), function_key(FRAMES - 1), false));
    close_native_window(windows[1]);
    close_native_window(windows[0]);
    Input.input_per_frame_event_and_flag_update();
    print("ok\n");
}

#scope_file
#load "support/input-macos-fixtures.jai";
