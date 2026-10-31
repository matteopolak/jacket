// Reads bindings from `memory` into the set. Returns false if any line was invalid; the valid
// ones still apply.
//
// The text starts with a `[1]` version line. Each line is `[S-][C-][A-]key action`, where `key` is a character or a name such as `Space`, `Enter`, `F5` or `ArrowUp`. `action` is the name of a procedure registered with
// `add_proc`, or `:` followed by a command string passed to `command_string_proc`. A `N-` prefix
// flips whether the binding respects the keymap's `ignore_modifier_keys`. `#` starts a comment, and
// `[name]` selects the keymap the following lines go to.
//
// ```jai
// set: Keymap_Set;
// map := add_keymap(*set, "game");
// set.default_keymap = map;
// add_proc(map, "jump", () { /* ... */ });
// assert(load_from_memory(*set, "[1]\nSpace jump\nS-F5 jump\n"));
// assert(map.mappings.count == 2);
// assert(!load_from_memory(*set, "[1]\nNOT_A_KEY jump"));
// reset_keymaps(*set);
// ```
load_from_memory :: (set: *Keymap_Set, memory: string, short_name := "unnamed", full_path := "(no path)") -> bool {
    handler: Text_File_Handler;
    TH.start_from_memory(*handler, short_name, full_path, memory);
    defer TH.deinit(*handler);
    return load_from_handler(set, null, *handler);
}

// Reads bindings from `memory` into one keymap, ignoring `[section]` lines. Returns false if any
// line was invalid.
load_from_memory :: (using keymap: *Keymap, memory: string, short_name: string, full_path: string, optional := false) -> bool {
    handler: Text_File_Handler;
    TH.start_from_memory(*handler, short_name, full_path, memory);
    defer TH.deinit(*handler);
    return load_from_handler(null, keymap, *handler);
}

// Reads bindings from a file into the set, in the format of `load_from_memory`. Returns false when
// the file cannot be read or has an invalid line. With `optional`, a missing file is not an error.
//
// ```jai,no_run
// set: Keymap_Set;
// map := add_keymap(*set, "game");
// set.default_keymap = map;
// add_proc(map, "jump", () { /* ... */ });
// load_from_file(*set, "keys", "keys.txt", optional = true);
// ```
load_from_file :: (set: *Keymap_Set, short_name: string, filename: string, optional := false) -> bool {
    handler: Text_File_Handler;
    TH.start_file(*handler, short_name, filename, optional);
    defer TH.deinit(*handler);
    return load_from_handler(set, null, *handler);
}

// Reads bindings from a file into one keymap, ignoring `[section]` lines.
load_from_file :: (using keymap: *Keymap, short_name: string, full_path: string, optional := false) -> bool {
    handler: Text_File_Handler;
    TH.start_file(*handler, short_name, full_path, optional);
    defer TH.deinit(*handler);
    return load_from_handler(null, keymap, *handler);
}

// Parses bindings from an open `Text_File_Handler`. The `load_from_` procedures call it.
load_from_handler :: (set: *Keymap_Set, keymap: *Keymap, handler: *Text_File_Handler) -> bool {
    if handler.failed return false;

    current := keymap;
    if set && !current current = set.default_keymap;

    success := true;

    while true {
        line, found := TH.consume_next_line(handler);
        if !found break;

        if line.count >= 2 && line[0] == #char "[" && line[line.count - 1] == #char "]" {
            current = null;

            if set {
                section: string = .{ line.count - 2, line.data + 1 };
                for map: set.keymaps if map.section_name == section {
                    current = map;
                    break;
                }
            }

            if !current {
                TH.error(handler.*, "unknown or unexpected keymap section");
                success = false;
            }

            continue;
        }

        if !current {
            TH.error(handler.*, "binding has no selected keymap");
            success = false;
            continue;
        }

        chord, action := TH.break_by_spaces(line);
        flags: Event.Modifier_Flags;
        respect := true;
        valid := true;

        while chord.count >= 2 && chord[1] == #char "-" {
            prefix := chord[0];
            if prefix == #char "S" flags.shift_pressed = true;
            else if prefix == #char "C" flags.ctrl_pressed = true;
            else if prefix == #char "A" flags.alt_pressed = true;
            else if prefix == #char "N" respect = !respect;
            else {
                valid = false;
                break;
            }

            chord = .{ chord.count - 2, chord.data + 2 };
        }

        known, key := map_string_to_key(chord);
        if !valid || !known || action.count == 0 {
            TH.error(handler.*, "invalid key binding");
            success = false;
            continue;
        }

        is_proc := action[0] != #char ":";
        if !is_proc action = .{ action.count - 1, action.data + 1 };

        added, reason := add_mapping(current, key, flags, respect, action, is_proc);
        if !added {
            TH.error(handler.*, "invalid binding action or modifiers (%)", reason);
            success = false;
        }
    }

    return success && !handler.failed;
}

// Turns a key name or a single character into a `Key_Code`. Returns false for a name it does not know.
//
// ```jai
// #import "Input";
//
// ok, key := map_string_to_key("F5");
// assert(ok && key == .F5);
// ok, key = map_string_to_key("a");
// assert(ok && key == cast(Key_Code) #char "A");
// ```
map_string_to_key :: (s: string) -> bool, Key_Code {
    for item: named_keys if item.name == s return true, item.key;

    code, bytes, status := U.character_utf8_to_utf32(s.data, s.count, true);
    if status != .CONVERSION_OK || bytes != s.count || bytes == 0 return false, .UNKNOWN;
    if code >= #char "a" && code <= #char "z" code -= 32;

    return true, cast(Key_Code) code;
}

// The name `map_string_to_key` accepts for a key.
//
// ```jai
// assert(map_key_to_string(.ENTER) == "Enter");
// ```
map_key_to_string :: (k: Key_Code) -> string {
    for item: named_keys if item.key == k return item.name;
    return U.character_utf32_to_utf8(cast(u32) k);
}

// Writes the bindings of every keymap in the set in the format `load_from_memory` reads.
//
// ```jai
// #import "Input";
//
// set: Keymap_Set;
// map := add_keymap(*set, "game");
// add_proc(map, "jump", () { /* ... */ });
// flags: Event.Modifier_Flags;
// add_mapping(map, .SPACEBAR, flags, true, "jump", true);
//
// builder: String_Builder;
// print_to_builder(*builder, *set);
// text := builder_to_string(*builder);
// assert(text.count > 0);
// free(text);
// reset_keymaps(*set);
// ```
print_to_builder :: (builder: *String_Builder, set: *Keymap_Set) {
    append(builder, "[1]\n");

    for map: set.keymaps {
        append(builder, "[");
        append(builder, map.section_name);
        append(builder, "]\n");
        print_keymap_mappings_to_builder(builder, map);
    }
}

// Writes the bindings of one keymap in the format `load_from_memory` reads.
print_to_builder :: (builder: *String_Builder, keymap: *Keymap) {
    append(builder, "[1]\n");
    print_keymap_mappings_to_builder(builder, keymap);
}

// Writes only the binding lines of a keymap, without its `[section]` header.
print_keymap_mappings_to_builder :: (builder: *String_Builder, keymap: *Keymap) {
    for mapping: keymap.mappings {
        assert(!mapping.modifier_flags.cmd_meta_pressed, "keymap persistence grammar has no Meta modifier prefix");
        if mapping.modifier_flags.shift_pressed append(builder, "S-");
        if mapping.modifier_flags.ctrl_pressed append(builder, "C-");
        if mapping.modifier_flags.alt_pressed append(builder, "A-");
        if !mapping.respect_ignore_modifier_keys append(builder, "N-");

        name := map_key_to_string(cast(Key_Code) mapping.utf32);

        append(builder, name);
        // Named strings are borrowed; the Unicode encoder returns owned storage.
        named := false;

        for item: named_keys if item.key == cast(Key_Code) mapping.utf32 {
            named = true;
            break;
        }

        if !named free(name);

        append(builder, " ");

        if mapping.proc_index < 0 {
            append(builder, ":");
            append(builder, mapping.command_string);
        } else append(builder, keymap.procs[mapping.proc_index].name);

        append(builder, "\n");
    }
}

// Writes the bindings of the set to a file, so `load_from_file` can read them back.
save_to_file :: (set: *Keymap_Set, filename: string) {
    builder: String_Builder;

    print_to_builder(*builder, set);
    written := File.write_entire_file(filename, *builder);
    if !written {
        context.logger("keymap file write failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "keymap file write failed");
    }
}

// Writes the bindings of one keymap to a file.
save_to_file :: (keymap: *Keymap, filename: string) {
    builder: String_Builder;

    print_to_builder(*builder, keymap);
    written := File.write_entire_file(filename, *builder);
    if !written {
        context.logger("keymap file write failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "keymap file write failed");
    }
}

#scope_module
Named_Key :: struct {
    name: string;
    key: Key_Code;
}

named_keys := Named_Key.[
    .{ "F1", .F1 }, .{ "F2", .F2 }, .{ "F3", .F3 }, .{ "F4", .F4 }, .{ "F5", .F5 }, .{ "F6", .F6 },
    .{ "F7", .F7 }, .{ "F8", .F8 }, .{ "F9", .F9 }, .{ "F10", .F10 }, .{ "F11", .F11 }, .{ "F12", .F12 },
    .{ "ArrowUp", .ARROW_UP }, .{ "ArrowDown", .ARROW_DOWN }, .{ "ArrowLeft", .ARROW_LEFT }, .{ "ArrowRight", .ARROW_RIGHT },
    .{ "MouseLeft", .MOUSE_BUTTON_LEFT }, .{ "MouseRight", .MOUSE_BUTTON_RIGHT }, .{ "MouseMiddle", .MOUSE_BUTTON_MIDDLE }, .{ "MouseWheelUp", .MOUSE_WHEEL_UP }, .{ "MouseWheelDown", .MOUSE_WHEEL_DOWN },
    .{ "Delete", .DELETE }, .{ "Insert", .INSERT }, .{ "End", .END }, .{ "Home", .HOME }, .{ "Backspace", .BACKSPACE }, .{ "PrintScreen", .PRINT_SCREEN }, .{ "Pause", .PAUSE }, .{ "PageUp", .PAGE_UP }, .{ "PageDown", .PAGE_DOWN }, .{ "ScrollLock", .SCROLL_LOCK }, .{ "Enter", .ENTER }, .{ "Return", .ENTER }, .{ "Tab", .TAB }, .{ "Shift", .SHIFT }, .{ "Alt", .ALT }, .{ "Control", .CTRL }, .{ "Space", .SPACEBAR }, .{ "ESC", .ESCAPE }, .{ "ENTER", .ENTER }, .{ "TAB", .TAB },
    .{ "Gamepad_ArrowUp", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.DPAD_UP) }, .{ "Gamepad_ArrowDown", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.DPAD_DOWN) }, .{ "Gamepad_ArrowLeft", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.DPAD_LEFT) }, .{ "Gamepad_ArrowRight", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.DPAD_RIGHT) },
    .{ "Gamepad_Start", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.START) }, .{ "Gamepad_Back", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.BACK) }, .{ "Gamepad_LShoulder", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_SHOULDER) }, .{ "Gamepad_RShoulder", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_SHOULDER) }, .{ "Gamepad_LTrigger", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_TRIGGER) }, .{ "Gamepad_RTrigger", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_TRIGGER) },
    .{ "Gamepad_FaceDown", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_DOWN) }, .{ "Gamepad_FaceRight", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_RIGHT) }, .{ "Gamepad_FaceLeft", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_LEFT) }, .{ "Gamepad_FaceUp", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_UP) },
    .{ "Gamepad_LStickUp", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_STICK_UP) }, .{ "Gamepad_LStickDown", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_STICK_DOWN) }, .{ "Gamepad_LStickLeft", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_STICK_LEFT) }, .{ "Gamepad_LStickRight", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_STICK_RIGHT) }, .{ "Gamepad_RStickUp", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_STICK_UP) }, .{ "Gamepad_RStickDown", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_STICK_DOWN) }, .{ "Gamepad_RStickLeft", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_STICK_LEFT) }, .{ "Gamepad_RStickRight", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_STICK_RIGHT) }, .{ "Gamepad_LStickClick", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.LEFT_STICK_CLICK) }, .{ "Gamepad_RStickClick", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.RIGHT_STICK_CLICK) },
    .{ "Gamepad_A", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_DOWN) }, .{ "Gamepad_B", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_RIGHT) }, .{ "Gamepad_X", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_LEFT) }, .{ "Gamepad_Y", cast(Key_Code) (cast(u32) Key_Code.GAMEPAD_0_BEGIN + cast(u32) Gamepad.Gamepad_Code.FACE_UP) }
];
