Key_Press_Proc :: #type ();
Key_Hold_Proc :: #type (bool);

Keymap_Set :: struct {
    keymaps: Bucket_Array(Keymap, 16, true);
    default_keymap: *Keymap;
}

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

Single_Key_Mapping :: struct {
    utf32: u32;
    respect_ignore_modifier_keys := true;
    toggled_on := false;
    modifier_flags: Modifier_Flags;
    command_string: string;
    proc_index: s32 = -1;
}

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
