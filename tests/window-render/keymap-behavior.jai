#import "Basic";
K :: #import "Keymap";
I :: #import "Input";
Bucket :: #import "Bucket_Array";

presses: int;
holds: int;
command_seen: string;
errors: int;

capture_error :: (text: string, #discard data: *void, info: Log_Info) {
    assert(text.count > 0 && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

press :: () {
    presses += 1;
}

hold :: (down: bool) {
    if down holds += 1;
    else holds -= 1;
}

command :: (text: string) {
    command_seen = text;
}

main :: () {
    set: K.Keymap_Set;
    map := K.add_keymap(*set, "editor");

    set.default_keymap = map;
    K.add_proc(map, "press", press);
    K.add_proc(map, "hold", hold);
    map.command_string_proc = command;
    assert(K.load_from_memory(*set, "[1]\nC-a press # comment\nS-b hold\nc :hello world\n"));
    event: I.Event;
    event.type = .KEYBOARD;
    event.key_code = cast(I.Key_Code) #char "A";
    event.key_pressed = 1;
    event.ctrl_pressed = true;
    assert(K.handle_event(*set, event) && presses == 1);
    assert(K.handle_event(*set, event) && presses == 2);
    event.key_code = cast(I.Key_Code) #char "B";
    event.ctrl_pressed = false;
    event.shift_pressed = true;
    assert(K.handle_event(*set, event) && holds == 1);
    assert(K.handle_event(*set, event) && holds == 1);
    event.key_pressed = 0;
    event.key_code = .SHIFT;
    event.shift_pressed = false;
    assert(K.handle_event(*set, event) && holds == 0);
    event.key_code = cast(I.Key_Code) #char "C";
    event.key_pressed = 1;
    assert(K.handle_event(*set, event));
    assert(command_seen == "hello world");
    builder: String_Builder;

    K.print_to_builder(*builder, *set);
    serialized := builder_to_string(*builder);

    defer free(serialized);
    copied: K.Keymap;

    K.keymap_copy_procs(map, *copied);
    other: K.Keymap_Set;
    second := K.add_keymap(*other, "editor");

    K.keymap_copy_procs(*copied, second);
    assert(K.load_from_memory(*other, serialized));
    assert(second.mappings.count == map.mappings.count);
    assert(second.mappings[2].command_string == "hello world");
    assert(second.mappings[2].command_string.data != map.mappings[2].command_string.data);
    path := sprint("%../../target/window-keymap-test.txt", #filepath);

    defer free(path);
    K.save_to_file(*set, path);
    K.reset_keymaps(*other);
    assert(K.load_from_file(*other, "saved keymap", path));
    assert(second.mappings.count == map.mappings.count && second.mappings[2].command_string == "hello world");

    policy: K.Keymap;

    K.add_proc(*policy, "press", press);
    assert(K.load_from_memory(*policy, "[1]\nN-x press\ny press\nN-x press\n", "policy", "memory"));
    assert(policy.mappings.count == 2);
    event = .{};
    event.type = .KEYBOARD;
    event.key_pressed = 1;
    event.key_code = cast(I.Key_Code) #char "X";
    event.ctrl_pressed = true;
    before := presses;

    assert(K.keymap_handle_event(policy, event) && presses == before + 1);
    policy.ignore_modifier_keys = true;
    assert(!K.keymap_handle_event(policy, event));
    event.key_code = cast(I.Key_Code) #char "Y";
    assert(K.keymap_handle_event(policy, event) && presses == before + 2);
    good, key := K.map_string_to_key("é");

    assert(good && cast(u32) key == 0xe9);
    good, key = K.map_string_to_key("ab");
    assert(!good && key == .UNKNOWN);
    succeeded, reason := K.add_mapping(*policy, .SHIFT, .{ shift_pressed = true }, true, "press", true);

    assert(!succeeded && reason == .NONSENSICAL_MODIFIERS);
    succeeded, reason = K.add_mapping(*policy, .F1, .{}, true, "missing", true);
    assert(!succeeded && reason == .UNKNOWN_COMMAND);
    scoped := context;
    scoped.logger = capture_error;

    {
        push_context, defer_pop scoped;
        assert(!K.load_from_memory(*policy, "[1]\nq missing\nz press\n[bad]\na press\n", "invalid", "memory"));
    }

    assert(errors == 3 && policy.mappings.count == 3);
    K.reset_keymap(*policy);
    array_free(policy.mappings);
    array_free(policy.procs);
    K.reset_keymaps(*set);
    assert(map.mappings.count == 0 && map.procs.count == 2 && set.keymaps.count == 1);
    K.reset_keymaps(*other);
    array_free(map.procs);
    array_free(second.procs);
    array_free(copied.procs);
    Bucket.bucket_array_reset(*set.keymaps);
    Bucket.bucket_array_reset(*other.keymaps);
    print("ok\n");
}
