#import "Basic";
W :: #import "Windows";
WC :: #import "Window_Creation";
I :: #import "Input";

user32 :: #library,system "user32";
post_message :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s32 #foreign user32 "PostMessageW";
native_character :: (code: u32, kind: u32) -> u32 #foreign user32 "MapVirtualKeyW";
native_virtual_key :: (character: u16) -> s16 #foreign user32 "VkKeyScanW";

main :: () {
    window := WC.create_window(200, 100, "Native keyboard layout input");

    assert(window != null);
    defer assert(W.DestroyWindow(window) != 0);
    I.update_window_events();
    mapped_count: int;

    for vk: u32.[W.VK_OEM_1, W.VK_OEM_2, W.VK_OEM_PLUS, W.VK_OEM_COMMA, W.VK_OEM_MINUS, W.VK_OEM_PERIOD, W.VK_ADD, W.VK_SUBTRACT, W.VK_MULTIPLY, W.VK_DIVIDE] {
        // Compare the production event path with the active native keyboard layout,
        // without loading/activating another layout or changing user preferences.
        expected := native_character(vk, 2) & 0xffff;
        if expected >= #char "a" && expected <= #char "z" expected -= 32;
        if expected < 32 || expected > 126 expected = 0;

        assert(cast(u32) I.get_key_code(vk) == expected);
        if expected == 0 continue;

        mapped_count += 1;
        inverse := native_virtual_key(cast(u16) expected);
        if inverse == -1 assert(I.get_vk(cast(I.Key_Code) expected) == 0);
        else assert(I.get_vk(cast(I.Key_Code) expected) == cast(u64) (cast(s32) inverse & 0xff));

        assert(post_message(window, W.WM_KEYDOWN, vk, 0) != 0);
        I.update_window_events();
        found := false;

        for event: I.events_this_frame if event.type == .KEYBOARD {
            assert(event.window == window && cast(u32) event.key_code == expected && event.key_pressed != 0);
            found = true;
        }

        assert(found);
        assert((I.input_button_states[expected] & .DOWN) != .NONE);
        assert(post_message(window, W.WM_KEYUP, vk, 0) != 0);
        I.update_window_events();
        assert((I.input_button_states[expected] & .DOWN) == .NONE);
    }

    assert(mapped_count > 0);
    assert(I.get_key_code(0x12345678000000bf) == .UNKNOWN);
    assert(I.get_vk(.UNKNOWN) == 0);
    print("ok\n");
}
