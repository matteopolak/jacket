#import "Basic";
G :: #import "GetRect";

main :: () {
    state: G.Text_Input_State;
    bytes := alloc_string(state.MAX_BUFFER_SIZE);

    defer free(bytes);
    defer free(state.pre_modification_text);
    for *byte: bytes byte.* = 120;

    G.set_input(*state, bytes);
    state.active = true;
    assert(!G.insert_text(*state, "a"));
    G.handle_event(*state, .{ type = .TEXT_INPUT, utf32 = 65 });
    G.press_enter(*state, true);
    assert(state.text == bytes && state.insert_point == bytes.count);
    assert(state.pending_result_actions == 0 && state.active);
    print("text capacity rejection ok\n");
}
