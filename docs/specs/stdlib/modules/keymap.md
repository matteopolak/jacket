# Keymap

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Keymap_Set :: struct`
- `add_keymap :: (set: *Keymap_Set, section_name: string) -> *Keymap`
- `reset_keymaps :: (set: *Keymap_Set)`
- `enable :: (set: *Keymap_Set, keymap: *Keymap, value: bool)`
- `handle_event :: (set: *Keymap_Set, event: Event) -> bool`
- `load_from_memory :: (set: *Keymap_Set, memory: string, short_name := "unnamed", full_path := "(no path)") -> bool`
- `load_from_file :: (set: *Keymap_Set, short_name: string, filename: string, optional := false) -> bool`
- `Key_Press_Proc :: #type ()`
- `Key_Hold_Proc :: #type (bool)`
- `Keymap :: struct`
- `Single_Key_Mapping :: struct`
- `reset_keymap :: (keymap: *Keymap)`
- `load_from_file :: (using keymap: *Keymap, short_name: string, full_path: string, optional := false) -> bool`
- `load_from_memory :: (using keymap: *Keymap, memory: string, short_name: string, full_path: string, optional := false) -> bool`
- `load_from_handler :: (set: *Keymap_Set, keymap: *Keymap, handler: *Text_File_Handler) -> bool`
- `add_proc :: (keymap: *Keymap, name: string, proc: Key_Press_Proc)`
- `add_proc :: (keymap: *Keymap, name: string, proc: Key_Hold_Proc)`
- `Add_Mapping_Error_Type :: enum u8`
- `add_mapping :: ( keymap: *Keymap, key: Key_Code, modifier_flags: Event.Modifier_Flags, respect_ignore_modifier_keys: bool, command: string, is_proc: bool ) -> (success: bool, error_type: Add_Mapping_Error_Type)`
- `keymap_handle_event :: (keymap: Keymap, e: Event) -> bool`
- `map_string_to_key :: (s: string) -> bool, Key_Code`
- `map_key_to_string :: (k: Key_Code) -> string`
- `keymap_copy :: (source: *Keymap, dest: *Keymap)`
- `keymap_copy_procs :: (source: *Keymap, dest: *Keymap)`
- `save_to_file :: (set: *Keymap_Set, filename: string)`
- `print_to_builder :: (builder: *String_Builder, set: *Keymap_Set)`
- `save_to_file :: (keymap: *Keymap, filename: string)`
- `print_to_builder :: (builder: *String_Builder, keymap: *Keymap)`
- `print_keymap_mappings_to_builder :: (builder: *String_Builder, keymap: *Keymap)`

## Public type members

#### `Keymap_Set` (struct)

- `keymaps: Bucket_Array(Keymap, 16, true)`
- `default_keymap: *Keymap`

#### `Keymap` (struct)

- `procs:    [..] Proc_Info`
- `mappings: [..] Single_Key_Mapping`
- `command_string_proc: (command: string)`
- `ignore_modifier_keys := false`
- `Proc_Info :: struct { name: string; press_\proc: Key_Press_Proc; toggle_proc: Key_Hold_Proc; }`
- `section_name: string`
- `enabled := true`

#### `Single_Key_Mapping` (struct)

- `utf32: u32`
- `respect_ignore_modifier_keys := true`
- `toggled_on := false`
- `modifier_flags:  Modifier_Flags`
- `command_string: string`
- `proc_index: s32 = -1`

#### `Add_Mapping_Error_Type` (enum)

- `NO_INFORMATION_ABOUT_THE_ERROR :: 0`
- `NONSENSICAL_MODIFIERS :: 1`
- `UNKNOWN_COMMAND :: 2`
- `NO_ERROR :: 3`

## Behavioral contract and constraints

The keymap module loads and saves key bindings from text files and maps key/modifier combinations to actions. Saving strips comments and may reorder entries; round-tripping is semantic rather than text-preserving. File parse/write errors follow the returned status/file API. Key names and modifiers depend on the platform's input conventions.

### Keymap text format and resolution

Each binding line has a key chord, at least one space, and an action. A chord consists of zero or more two-character modifier prefixes followed by one key name. `S-`, `C-`, and `A-` require Shift, Control, and Alt; `N-` reverses the keymap's `ignore_modifier_keys` policy for that binding. Prefixes are case-sensitive. Key names are matched case-sensitively against the module's named-key table; otherwise one UTF-8 character is accepted and lowercase ASCII letters are normalized to uppercase key codes. Multi-character names must use a recognized spelling. Recognized names include `F1`–`F12`; arrow keys `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`; mouse names `MouseLeft`, `MouseRight`, `MouseMiddle`, `MouseWheelUp`, `MouseWheelDown`; gamepad names `Gamepad_ArrowUp`, `Gamepad_ArrowDown`, `Gamepad_ArrowLeft`, `Gamepad_ArrowRight`, `Gamepad_LStickUp`, `Gamepad_LStickDown`, `Gamepad_LStickLeft`, `Gamepad_LStickRight`, `Gamepad_RStickUp`, `Gamepad_RStickDown`, `Gamepad_RStickLeft`, `Gamepad_RStickRight`, `Gamepad_LStickClick`, `Gamepad_RStickClick`, `Gamepad_Start`, `Gamepad_Back`, `Gamepad_LShoulder`, `Gamepad_RShoulder`, `Gamepad_LTrigger`, `Gamepad_RTrigger`, `Gamepad_FaceDown`, `Gamepad_FaceRight`, `Gamepad_FaceLeft`, `Gamepad_FaceUp`, and aliases `Gamepad_A`, `Gamepad_B`, `Gamepad_X`, `Gamepad_Y`; editing keys `Delete`, `Insert`, `End`, `Home`, `Backspace`, `PrintScreen`, `Pause`, `PageUp`, `PageDown`, `ScrollLock`, `Enter`, `Return`, `Tab`, `Shift`, `Alt`, `Control`, `Space`; and `ESC`, `ENTER`, `TAB` aliases.

The shared text-file reader accepts `#` comments, strips trailing comments, skips blank lines, and requires a bracketed numeric version header before bindings. A malformed binding sets the load result to false; parsing continues, so earlier valid mappings may already have been applied. Loading is not transactional.

An action without a leading colon names a previously registered procedure. `add_proc` registers a press callback `()`, or a hold callback `(is_down: bool)`, by name. An action beginning with `:` stores the remainder as a command string, passed to `command_string_proc` when invoked. Duplicate key-plus-modifier bindings replace the earlier binding in place. Register procedures before loading mappings that refer to them. A `Keymap_Set` section header is `[section name]`; it selects an existing keymap whose `section_name` matches exactly. Before the first section header, bindings use `default_keymap`; without one, those bindings are reported and skipped. Loading a single `Keymap` rejects section headers. The file's initial version header is handled by the text-file reader.

For a pressed keyboard event, a modifier key's own modifier bit is cleared before matching so it can be bound alone. A binding matches when its key matches and either its modifier policy says to ignore modifiers or the event modifier bits exactly match the binding. With `ignore_modifier_keys == false`, the default `respect_ignore_modifier_keys == true` requires exact modifiers; with it true, that default ignores modifiers. `N-` flips this per-binding behavior. The first matching binding runs and stops dispatch. A press callback runs on every matching press; a hold callback receives `true` only on the transition into held state. On key release, an active hold callback receives `false` when the mapped key is released or when a modifier change makes the chord cease matching. Mouse-wheel events with nonzero `wheel_delta` are treated as pressed `MouseWheelUp` or `MouseWheelDown` based on the sign. Other event types are not handled.

`handle_event(set, event)` checks enabled keymaps in insertion order and stops at the first keymap that handles the event. New keymaps default to enabled. `reset_keymaps` clears each keymap's bindings but retains the keymap entries; `reset_keymap` frees its owned command strings and clears its mappings. `keymap_copy` copies procedures and mappings, while `keymap_copy_procs` copies only the registered procedure table. Procedure names and section names are borrowed strings; command strings loaded into mappings are owned copies.

The saver emits version header `[1]`, section headers for a set, and normalized key spellings/prefixes. It does not preserve comments, original spelling, or ordering guarantees. `N-` is emitted when the per-binding policy reverses the keymap default. Command mappings are written with `:`; procedure mappings are written by registered name.
