// Public GetRect text editing / Keymap persistence; no window or GUI required.
// Arguments: enabled|disabled, text|left|set|map, worker-private output directory.
#import "Basic";
R :: #import "GetRect";
L :: #import "GetRect_LeftHanded";
K :: #import "Keymap";
F :: #import "File";
Bucket :: #import "Bucket_Array";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 4 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(10);
    } else if args[1] == "disabled" {
        if calls != 0 exit(11);
    } else exit(12);

    if args[2] == "text" {
        right: R.Text_Input_State;

        R.set_text(*right, "a");
        defer free(right.pre_modification_text);
        R.append_text(*right, "b");
        if right.text != "ab" exit(20);

        full := alloc_string(right.MAX_BUFFER_SIZE);

        defer free(full);
        memset(full.data, #char "z", full.count);
        R.set_text(*right, full);
        saved := context;
        scoped := context;
        scoped.logger = capture;
        scoped.assertion_failed = recover;
        context = scoped;
        errors = 0;
        R.append_text(*right, "!");
        context = saved;
        if errors == 0 || right.text != full exit(27);

    } else if args[2] == "left" {
        left: L.Text_Input_State;

        L.set_text(*left, "c");
        defer free(left.pre_modification_text);
        L.append_text(*left, "d");
        if left.text != "cd" exit(21);
    } else {
        set: K.Keymap_Set;
        map := K.add_keymap(*set, "editor");

        set.default_keymap = map;

        defer {
            K.reset_keymaps(*set);
            Bucket.bucket_array_reset(*set.keymaps);
        }

        if !K.load_from_memory(*set, "[1]\na :hello\n") exit(22);

        path := sprint("%/%.txt", args[3], args[2]);

        defer free(path);
        if args[2] == "set" K.save_to_file(*set, path);
        else if args[2] == "map" K.save_to_file(map, path);
        else exit(13);

        data, loaded := F.read_entire_file(path, log_errors = false);
        if !loaded exit(23);

        defer free(data);
        restored: K.Keymap_Set;
        restored_map := K.add_keymap(*restored, "editor");

        restored.default_keymap = restored_map;

        defer {
            K.reset_keymaps(*restored);
            Bucket.bucket_array_reset(*restored.keymaps);
        }

        if !K.load_from_memory(*restored, data) exit(24);
        if restored_map.mappings.count != 1 exit(25);

        mapping := restored_map.mappings[0];
        if mapping.command_string != "hello" || mapping.proc_index >= 0 exit(26);

        invalid := sprint("%/child", path);

        defer free(invalid);
        saved := context;
        scoped := context;
        scoped.logger = capture;
        scoped.assertion_failed = recover;
        context = scoped;
        errors = 0;
        if args[2] == "set" K.save_to_file(*set, invalid);
        else K.save_to_file(map, invalid);

        context = saved;
        if errors == 0 exit(28);
    }

    write_string("ok\n");
}
