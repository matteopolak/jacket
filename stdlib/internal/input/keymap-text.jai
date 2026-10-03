load_from_memory :: (set: *Keymap_Set, memory: string, short_name := "unnamed", full_path := "(no path)") -> bool {
    handler: Text_File_Handler;
    TH.start_from_memory(*handler, short_name, full_path, memory);
    defer TH.deinit(*handler);
    return load_from_handler(set, null, *handler);
}

load_from_memory :: (using keymap: *Keymap, memory: string, short_name: string, full_path: string, optional := false) -> bool {
    handler: Text_File_Handler;
    TH.start_from_memory(*handler, short_name, full_path, memory);
    defer TH.deinit(*handler);
    return load_from_handler(null, keymap, *handler);
}

load_from_file :: (set: *Keymap_Set, short_name: string, filename: string, optional := false) -> bool {
    handler: Text_File_Handler;
    TH.start_file(*handler, short_name, filename, optional);
    defer TH.deinit(*handler);
    return load_from_handler(set, null, *handler);
}

load_from_file :: (using keymap: *Keymap, short_name: string, full_path: string, optional := false) -> bool {
    handler: Text_File_Handler;
    TH.start_file(*handler, short_name, full_path, optional);
    defer TH.deinit(*handler);
    return load_from_handler(null, keymap, *handler);
}

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

map_string_to_key :: (s: string) -> bool, Key_Code {
    for item: named_keys if item.name == s return true, item.key;

    code, bytes, status := U.character_utf8_to_utf32(s.data, s.count, true);
    if status != .CONVERSION_OK || bytes != s.count || bytes == 0 return false, .UNKNOWN;
    if code >= #char "a" && code <= #char "z" code -= 32;

    return true, cast(Key_Code) code;
}

map_key_to_string :: (k: Key_Code) -> string {
    for item: named_keys if item.key == k return item.name;
    return U.character_utf32_to_utf8(cast(u32) k);
}

print_to_builder :: (builder: *String_Builder, set: *Keymap_Set) {
    append(builder, "[1]\n");

    for map: set.keymaps {
        append(builder, "[");
        append(builder, map.section_name);
        append(builder, "]\n");
        print_keymap_mappings_to_builder(builder, map);
    }
}

print_to_builder :: (builder: *String_Builder, keymap: *Keymap) {
    append(builder, "[1]\n");
    print_keymap_mappings_to_builder(builder, keymap);
}

print_keymap_mappings_to_builder :: (builder: *String_Builder, keymap: *Keymap) {
    for mapping: keymap.mappings {
        assert(!mapping.modifier_flags.cmd_meta_pressed, "reviewed keymap persistence grammar has no Meta modifier prefix");
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

save_to_file :: (set: *Keymap_Set, filename: string) {
    builder: String_Builder;

    print_to_builder(*builder, set);
    written := File.write_entire_file(filename, *builder);
    if !written {
        context.logger("keymap file write failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "keymap file write failed");
    }
}

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
