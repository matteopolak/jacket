#import "Basic";
G :: #import "GetRect";
I :: #import "Input";

main :: () {
    state: G.Text_Input_State;

    G.set_input(*state, "abc😀");
    defer free(state.pre_modification_text);
    state.active = true;
    G.handle_event(*state, .{ type = .KEYBOARD, key_code = cast(I.Key_Code) 65, key_pressed = 1, ctrl_pressed = true });
    assert(state.selection.active && state.selection.start_character == 0 && state.selection.end_character == state.text.count, "Ctrl+A must use the key code, not the pressed flag");
    G.deselect(*state.selection);
    G.handle_event(*state, .{ type = .KEYBOARD, key_code = cast(I.Key_Code) 65, key_pressed = 0, ctrl_pressed = true });
    assert(!state.selection.active);
    G.handle_event(*state, .{ type = .KEYBOARD, key_code = cast(I.Key_Code) 65, key_pressed = 1, cmd_meta_pressed = true });
    assert(state.selection.active);
    G.deselect(*state.selection);
    G.handle_event(*state, .{ type = .KEYBOARD, key_code = .UNKNOWN, key_pressed = 65, ctrl_pressed = true });
    assert(!state.selection.active, "a pressed flag must never be interpreted as a character code");
    print("canonical keyboard shortcuts ok\n");
}
