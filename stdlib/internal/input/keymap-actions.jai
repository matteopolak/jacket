// Adds an empty keymap named `section_name` to the set and returns it. The name is what a `[section]`
// line in a bindings file refers to.
//
// ```jai
// set: Keymap_Set;
// map := add_keymap(*set, "editor");
// assert(map.section_name == "editor" && map.enabled);
// ```
add_keymap :: (set: *Keymap_Set, section_name: string) -> *Keymap {
    initial: Keymap;
    initial.section_name = section_name;
    _, result := bucket_array_add(*set.keymaps, initial);
    return result;
}

// Clears the bindings of every keymap in the set and frees them. Keeps the registered procedures.
reset_keymaps :: (set: *Keymap_Set) {
    for map: set.keymaps reset_keymap(map);
}

// Clears the bindings of one keymap and frees them. Keeps the registered procedures.
reset_keymap :: (keymap: *Keymap) {
    for mapping: keymap.mappings if mapping.proc_index < 0 free(mapping.command_string);
    array_reset(*keymap.mappings);
}

// Turns a keymap on or off. A disabled keymap is skipped by `handle_event`.
//
// ```jai
// set: Keymap_Set;
// map := add_keymap(*set, "game");
// enable(*set, map, false);
// assert(!map.enabled);
// ```
enable :: (set: *Keymap_Set, keymap: *Keymap, value: bool) {
    keymap.enabled = value;
}

// Registers a procedure under `name` so bindings can refer to it. Pass a plain procedure for a press,
// or one that takes a `bool` for a key that is held.
//
// ```jai
// set: Keymap_Set;
// map := add_keymap(*set, "game");
//
// add_proc(map, "jump", () {
//     /* ... */
// });
// add_proc(map, "run", (down: bool) {
//     /* ... */
// });
// assert(map.procs.count == 2);
// ```
add_proc :: (keymap: *Keymap, name: string, proc: Key_Press_Proc) {
    info: Keymap.Proc_Info;
    info.name = name;
    info.press_proc = proc;
    array_add(*keymap.procs, info);
}

// Registers a procedure under `name` so bindings can refer to it. Pass a plain procedure for a press,
// or one that takes a `bool` for a key that is held.
add_proc :: (keymap: *Keymap, name: string, proc: Key_Hold_Proc) {
    info: Keymap.Proc_Info;
    info.name = name;
    info.toggle_proc = proc;
    array_add(*keymap.procs, info);
}

// Binds `key` with the given modifiers to a registered procedure (`is_proc`) or a command string.
// Rebinding the same key and modifiers replaces the old binding. Returns whether it worked, and
// if not, why. Set `respect_ignore_modifier_keys` to false to have the binding ignore the keymap's
// `ignore_modifier_keys` setting.
//
// ```jai
// #import "Input";
//
// set: Keymap_Set;
// map := add_keymap(*set, "game");
//
// add_proc(map, "jump", () {
//     /* ... */
// });
//
// flags: Event.Modifier_Flags;
// ok, reason := add_mapping(map, .SPACEBAR, flags, true, "jump", true);
//
// assert(ok);
// ok, reason = add_mapping(map, .ENTER, flags, true, "missing", true);
// assert(!ok && reason == .UNKNOWN_COMMAND);
// reset_keymaps(*set);
// ```
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

// Runs the binding that matches `event` in the first enabled keymap that has one. Returns true when
// one did, so the caller can skip its own handling. Only keyboard and mouse wheel events match.
handle_event :: (set: *Keymap_Set, event: Event) -> bool {
    for map: set.keymaps if map.enabled && keymap_handle_event(map.*, event) return true;
    return false;
}

// Like `handle_event`, but for a single keymap.
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

// Replaces the procedures of `dest` with those of `source`, so the same bindings file can load into both.
keymap_copy_procs :: (source: *Keymap, dest: *Keymap) {
    if source == dest return;
    array_reset(*dest.procs);
    for info: source.procs array_add(*dest.procs, info);
}

// Makes `dest` a copy of `source`: its bindings, procedures and settings. Command strings are copied.
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
