#import "Basic";
G :: #import "GetRect";

main :: () {
    state: G.Text_Input_State;
    bytes := alloc_string(state.MAX_BUFFER_SIZE);

    defer free(bytes);
    for *byte: bytes byte.* = 120;

    G.set_input(*state, bytes);
    G.append_text(*state, "a");
    print("capacity assertion missed\n");
}
