add_keymap :: (set: *Keymap_Set, section_name: string) -> *Keymap {
    initial: Keymap;
    initial.section_name = section_name;
    _, result := bucket_array_add(*set.keymaps, initial);
    return result;
}

reset_keymaps :: (set: *Keymap_Set) {
    for map: set.keymaps reset_keymap(map);
}

reset_keymap :: (keymap: *Keymap) {
    for mapping: keymap.mappings if mapping.proc_index < 0 free(mapping.command_string);
    array_reset(*keymap.mappings);
}

enable :: (set: *Keymap_Set, keymap: *Keymap, value: bool) {
    keymap.enabled = value;
}

add_proc :: (keymap: *Keymap, name: string, proc: Key_Press_Proc) {
    info: Keymap.Proc_Info;
    info.name = name;
    info.press_proc = proc;
    array_add(*keymap.procs, info);
}

add_proc :: (keymap: *Keymap, name: string, proc: Key_Hold_Proc) {
    info: Keymap.Proc_Info;
    info.name = name;
    info.toggle_proc = proc;
    array_add(*keymap.procs, info);
}

add_mapping :: (keymap: *Keymap, key: Key_Code, modifier_flags: Event.Modifier_Flags, respect_ignore_modifier_keys: bool, command: string, is_proc: bool) -> (success: bool, error_type: Add_Mapping_Error_Type) {
    if (key == .SHIFT && modifier_flags.shift_pressed) || (key == .CTRL && modifier_flags.ctrl_pressed) || (key == .ALT && modifier_flags.alt_pressed) || (key == .CMD && modifier_flags.cmd_meta_pressed) return false, .NONSENSICAL_MODIFIERS;

    mapping: Single_Key_Mapping;
    mapping.utf32 = cast(u32) key;
    mapping.modifier_flags = modifier_flags;
    mapping.respect_ignore_modifier_keys = respect_ignore_modifier_keys;

    if is_proc {
        for info, index: keymap.procs if info.name == command {
            mapping.proc_index = cast(s32) index;
            break;
        }

        if mapping.proc_index < 0 return false, .UNKNOWN_COMMAND;
    } else mapping.command_string = copy_string(command);

    for *existing: keymap.mappings {
        if existing.utf32 == mapping.utf32 && existing.modifier_flags.packed == modifier_flags.packed {
            if existing.proc_index < 0 free(existing.command_string);
            existing.* = mapping;
            return true, .NO_ERROR;
        }
    }

    array_add(*keymap.mappings, mapping);

    return true, .NO_ERROR;
}

handle_event :: (set: *Keymap_Set, event: Event) -> bool {
    for map: set.keymaps if map.enabled && keymap_handle_event(map.*, event) return true;
    return false;
}

keymap_handle_event :: (keymap: Keymap, e: Event) -> bool {
    if e.type != .KEYBOARD && e.type != .MOUSE_WHEEL return false;

    key := e.key_code;
    down := e.key_pressed != 0;

    if e.type == .MOUSE_WHEEL {
        if e.wheel_delta == 0 return false;
        key = .MOUSE_WHEEL_UP;
        if e.wheel_delta < 0 key = .MOUSE_WHEEL_DOWN;
        down = true;
    }

    flags := e.modifier_flags;

    if down {
        if key == .SHIFT flags.shift_pressed = false;
        if key == .CTRL flags.ctrl_pressed = false;
        if key == .ALT flags.alt_pressed = false;
        if key == .CMD flags.cmd_meta_pressed = false;

        for *mapping: keymap.mappings {
            if mapping.utf32 != cast(u32) key || !mapping_modifiers_match(keymap, mapping.*, flags) continue;

            if mapping.proc_index >= 0 {
                info := keymap.procs[mapping.proc_index];
                if info.press_proc info.press_proc();
                else if info.toggle_proc && !mapping.toggled_on {
                    mapping.toggled_on = true;
                    info.toggle_proc(true);
                }
            } else {
                assert(keymap.command_string_proc != null, "a command binding requires command_string_proc");
                keymap.command_string_proc(mapping.command_string);
            }

            return true;
        }

        return false;
    }

    handled := false;

    for *mapping: keymap.mappings {
        if !mapping.toggled_on continue;

        if mapping.utf32 == cast(u32) key || !mapping_modifiers_match(keymap, mapping.*, flags) {
            mapping.toggled_on = false;
            assert(mapping.proc_index >= 0);
            proc := keymap.procs[mapping.proc_index].toggle_proc;

            assert(proc != null);
            proc(false);
            handled = true;
        }
    }

    return handled;
}

keymap_copy_procs :: (source: *Keymap, dest: *Keymap) {
    if source == dest return;
    array_reset(*dest.procs);
    for info: source.procs array_add(*dest.procs, info);
}

keymap_copy :: (source: *Keymap, dest: *Keymap) {
    if source == dest return;

    reset_keymap(dest);
    keymap_copy_procs(source, dest);
    dest.section_name = source.section_name;
    dest.enabled = source.enabled;
    dest.command_string_proc = source.command_string_proc;
    dest.ignore_modifier_keys = source.ignore_modifier_keys;

    for item: source.mappings {
        copied := item;
        if copied.proc_index < 0 copied.command_string = copy_string(item.command_string);
        array_add(*dest.mappings, copied);
    }
}

#scope_module
mapping_modifiers_match :: (keymap: Keymap, mapping: Single_Key_Mapping, flags: Event.Modifier_Flags) -> bool {
    ignore := keymap.ignore_modifier_keys;
    if !mapping.respect_ignore_modifier_keys ignore = !ignore;
    return ignore || mapping.modifier_flags.packed == flags.packed;
}
