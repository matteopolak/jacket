// Key bindings that you can load from a text file.
//
// Build a `Keymap_Set` with `add_keymap`, register the procedures a binding may name with
// `add_proc`, load bindings with `load_from_memory` or `load_from_file`, then pass each `Event` from
// the `Input` module to `handle_event`. It returns true when a binding took it.
//
// After a `[1]` version line, a binding is a key with optional `S-`, `C-` and `A-` prefixes (shift, control, alt), then
// either a procedure name or `:` and a command string. `[name]` starts the bindings of a keymap.
//
// ```jai
// #import "Input";
//
// set: Keymap_Set;
// map := add_keymap(*set, "editor");
// set.default_keymap = map;
// add_proc(map, "save", () { /* runs on C-s */ });
// assert(load_from_memory(*set, "[1]\n[editor]\nC-s save\n"));
//
// event: Event;
// event.type = .KEYBOARD;
// event.key_code = cast(Key_Code) #char "S";
// event.key_pressed = 1;
// event.ctrl_pressed = true;
// assert(handle_event(*set, event));
// reset_keymaps(*set);
// ```

// A procedure a key press runs.
Key_Press_Proc :: #type ();
// A procedure for a key that is held: called with true when the key goes down, false when it is released.
Key_Hold_Proc :: #type (bool);

// A group of keymaps. `handle_event` tries each enabled one in turn; `default_keymap` receives
// bindings that appear before any `[section]` line.
Keymap_Set :: struct {
    keymaps: Bucket_Array(Keymap, 16, true);
    default_keymap: *Keymap;
}

// One section of bindings, plus the procedures they can name.
Keymap :: struct {
    procs: [..] Proc_Info;
    mappings: [..] Single_Key_Mapping;
    command_string_proc: (command: string);
    ignore_modifier_keys := false;
    Proc_Info :: struct {
        name: string;
        press_proc: Key_Press_Proc;
        toggle_proc: Key_Hold_Proc;
    }
    section_name: string;
    enabled := true;
}

// One binding: a key, its modifiers, and the procedure or command string it runs.
Single_Key_Mapping :: struct {
    utf32: u32;
    respect_ignore_modifier_keys := true;
    toggled_on := false;
    modifier_flags: Modifier_Flags;
    command_string: string;
    proc_index: s32 = -1;
}

// Why `add_mapping` refused a binding.
Add_Mapping_Error_Type :: enum u8 {
    NO_INFORMATION_ABOUT_THE_ERROR :: 0;
    NONSENSICAL_MODIFIERS :: 1;
    UNKNOWN_COMMAND :: 2;
    NO_ERROR :: 3;
}

#load "../internal/input/keymap-actions.jai";
#load "../internal/input/keymap-text.jai";

#scope_module
#import "Bucket_Array";
#import "Basic";
TH :: #import "Text_File_Handler";
File :: #import "File";
U :: #import "Unicode";
Gamepad :: #import "Gamepad";
Input :: #import "Input";

Modifier_Flags :: Input.Event.Modifier_Flags;
Event :: Input.Event;
Key_Code :: Input.Key_Code;
Text_File_Handler :: TH.Text_File_Handler;
