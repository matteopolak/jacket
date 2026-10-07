// A game loop that runs with `context.allocator = temp` and resets temporary storage after every
// frame. Input's queues outlive a frame, so they must not take that allocator: here every frame
// queues key, text and wheel events, resizes, moves and a file drop for a growing set of windows,
// then the frame's temporary storage is reset and overwritten before the events are read again.
// Window handles are plain tokens and no native adapter runs, so every target runs all of it.
#import "Basic";
#import "Window_Type";
Input :: #import "Input"(ENABLE_NATIVE_INPUT_ADAPTERS = false);

FRAMES :: 300;
WINDOWS :: 40;

window_for :: (index: int) -> Window_Type {
    return cast(Window_Type) (1000 + index);
}

function_key :: (frame: int) -> Input.Key_Code {
    return cast(Input.Key_Code) (cast(int) Input.Key_Code.F1 + frame % 12);
}

key_event :: (key: Input.Key_Code, pressed: bool) -> Input.Event {
    event: Input.Event;
    event.type = .KEYBOARD;
    event.key_code = key;
    event.key_pressed = xx pressed;
    return event;
}

// Reuses the temporary storage the frame just gave back, so anything still pointing into it
// reads garbage.
scribble_over_temporary_storage :: () {
    junk := NewArray(64 * 1024, u8, false);
    memset(junk.data, 0xa5, junk.count);
}

check_frame :: (frame: int) {
    window := window_for(frame % WINDOWS);
    // From the second frame on, the previous frame's key release comes first.
    base := ifx frame > 0 then 1 else 0;
    expected_events := base + 4;
    if frame % 10 == 0 expected_events += 1;
    assert(Input.events_this_frame.count == expected_events, "frame %: % events", frame, Input.events_this_frame.count);
    if base {
        release := Input.routed_events_this_frame[0];
        assert(release.window == window_for((frame - 1) % WINDOWS), "frame %: release routed to %", frame, release.window);
        assert(release.event.key_pressed == 0 && release.event.key_code == function_key(frame - 1), "frame %: release %", frame, release.event);
    }
    press := Input.events_this_frame[base];
    assert(press.type == .KEYBOARD && press.key_pressed != 0 && press.key_code == function_key(frame), "frame %: press %", frame, press);
    text := Input.events_this_frame[base + 1];
    assert(text.type == .TEXT_INPUT && text.utf32 == cast(u32) (#char "a" + frame % 26), "frame %: text %", frame, text.utf32);
    assert(Input.routed_events_this_frame.count == base + 2, "frame %: % routed", frame, Input.routed_events_this_frame.count);
    assert(Input.routed_events_this_frame[base].window == window && Input.routed_events_this_frame[base + 1].window == window);
    assert(Input.unrouted_events_this_frame.count == expected_events - base - 2);
    assert(Input.mouse_delta_z == 120, "frame %: wheel %", frame, Input.mouse_delta_z);
    if frame % 10 == 0 {
        drop := Input.events_this_frame[expected_events - 1];
        assert(drop.type == .DRAG_AND_DROP_FILES && drop.files.count == 2, "frame %: drop %", frame, drop.type);
        assert(drop.files[0] == tprint("file-%.txt", frame) && drop.files[1] == "second.txt", "frame %: dropped \"%\"", frame, drop.files[0]);
    }
    resizes := Input.get_window_resizes();
    assert(resizes.count == 2, "frame %: % resizes", frame, resizes.count);
    for resizes {
        if it.window == window assert(it.width == 100 + frame && it.height == 50 + frame, "frame %: resized to % %", frame, it.width, it.height);
        else assert(it.window == window_for(WINDOWS) && it.width == 7 && it.height == 8);
    }
    assert(Input.window_has_input_focus(window), "frame %: focus", frame);
    assert(Input.window_key_is_down(window, function_key(frame)), "frame %: key held", frame);
}

main :: () {
    heap := context.allocator;
    context.allocator = temp;
    for frame: 0..FRAMES - 1 {
        window := window_for(frame % WINDOWS);
        previous := window_for((frame + WINDOWS - 1) % WINDOWS);
        if frame > 0 Input.set_input_focus(previous, false);
        Input.set_input_focus(window, true);
        Input.queue_input_event(window, key_event(function_key(frame), true));
        text: Input.Event;
        text.type = .TEXT_INPUT;
        text.utf32 = cast(u32) (#char "a" + frame % 26);
        text.text_input_count = 1;
        Input.queue_input_event(window, text);
        wheel: Input.Event;
        wheel.type = .MOUSE_WHEEL;
        wheel.wheel_delta = 120;
        Input.queue_input_event(wheel);
        Input.queue_input_event(key_event(.SHIFT, frame % 2 == 0));
        // The dropped names and their list pass to Input, which frees them at the next update.
        if frame % 10 == 0 {
            drop: Input.Event;
            drop.type = .DRAG_AND_DROP_FILES;
            drop.files.allocator = heap;
            array_add(*drop.files, copy_string(tprint("file-%.txt", frame),, heap));
            array_add(*drop.files, copy_string("second.txt",, heap));
            Input.queue_input_event(drop);
        }
        Input.queue_window_resize(window, cast(s32) (90 + frame), 1);
        Input.queue_window_resize(window_for(WINDOWS), 7, 8);
        Input.queue_window_resize(window, cast(s32) (100 + frame), cast(s32) (50 + frame));
        Input.queue_window_move(window, cast(s32) frame, 2);
        Input.queue_mouse_delta(1, 1);

        Input.update_window_events();
        check_frame(frame);
        moves := Input.get_window_moves();
        assert(moves.count == 1 && moves[0].window == window && moves[0].x == frame, "frame %: moves", frame);

        // The game's end of frame. What update_window_events returned stays valid until the next one.
        reset_temporary_storage();
        scribble_over_temporary_storage();
        check_frame(frame);
        reset_temporary_storage();
        Input.queue_input_event(window, key_event(function_key(frame), false));
    }
    Input.update_window_events();
    assert(Input.events_this_frame.count == 1 && Input.events_this_frame[0].key_pressed == 0);
    print("ok\n");
}
