#import "Basic";
WT :: #import "Window_Type";
I :: #import "Input";
S :: #import "Simp";

main :: () {
    window: WT.Window_Type = WT.INVALID_WINDOW;

    assert(window == WT.INVALID_WINDOW);
    e: I.Event;

    assert(e.type == .UNINITIALIZED);
    assert(e.key_code == .UNKNOWN);
    assert(e.modifier_flags.packed == 0);
    assert(size_of(I.Event.Modifier_Flags) == 4);
    e.shift_pressed = true;
    assert(e.modifier_flags.packed != 0);
    assert(cast(u32) I.Key_Code.META == cast(u32) I.Key_Code.CMD);
    assert(cast(u32) I.Key_Code.GAMEPAD_1_BEGIN == cast(u32) I.Key_Code.GAMEPAD_0_END + 1);
    assert(I.NUM_BUTTON_STATES == cast(int) I.Key_Code.TOUCH + 1);
    t: S.Texture;

    assert(t.width == 1 && t.height == 1 && t.depth == 1);
    assert(t.mipmap_count == 1 && t.array_length == 1);
    assert(t.gl_handle == 0);
    #assert size_of(S.Texture_Platform_Specific) == 4;
    #assert offset_of(S.Texture.gl_handle) == 20;
    font: S.Dynamic_Font;

    assert(font.utf32_for_unknown_character == 0xfffd);
    shader: S.Shader;

    assert(shader.position_loc == -1);
    state: S.Immediate_State;

    assert(state.coordinate_system == .RIGHT_HANDED);
    print("ok\n");
}
