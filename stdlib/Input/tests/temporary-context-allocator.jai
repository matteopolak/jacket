// A program that sets context.allocator = temp and resets temporary storage every frame, without
// choosing an allocator for the Input arrays: the frame arrays must not grow in temporary memory,
// or the next frame's cleanup frees file lists read from reused memory.
#assert OS == .MACOS, "Input fixture requires reviewed macOS osx_externally_generated_events";

#import "Basic";
Input :: #import "Input";

FRAMES :: 200;
EVENTS_PER_FRAME :: 40;

main :: () {
    defer dispose_native_windows();
    heap := context.allocator;

    // Events posted by the caller are its own storage; only the frame arrays are under test.
    Input.osx_externally_generated_events.allocator = heap;
    window := owned_native_window(160, 120, "Input temporary context");

    assert(window != null);
    Input.update_window_events();
    context.allocator = temp;

    for frame: 0..FRAMES - 1 {
        // Growing past the previous capacity each frame forces a reallocation after every reset.
        for 1..EVENTS_PER_FRAME + frame % 7 {
            event: Input.Event;
            event.type = .TEXT_INPUT;
            event.window = window;
            event.utf32 = #char "d";
            event.text_input_count = 1;
            array_add(*Input.osx_externally_generated_events, event);
        }

        Input.update_window_events();
        Input.add_resize_record(window, cast(s32) (100 + frame), 50);
        texts := 0;
        for Input.events_this_frame if it.type == .TEXT_INPUT texts += 1;

        assert(texts == EVENTS_PER_FRAME + frame % 7, "frame % saw % text events", frame, texts);
        assert(Input.events_this_frame.allocator.proc != temp.proc, "the frame events grew in temporary storage");
        assert(Input.get_window_resizes().count > 0);

        reset_temporary_storage();
        junk := talloc_string(256 * 1024);

        memset(junk.data, 0xa5, junk.count);
    }

    context.allocator = heap;
    close_native_window(window);
    Input.input_per_frame_event_and_flag_update();
    print("ok\n");
}

#scope_file
#load "support/macos-fixtures.jai";
