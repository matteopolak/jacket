#import "Basic";
K :: #import "Keymap";
G :: #import "Gamepad";

press :: () {
    calls += 1;
}

hold :: (down: bool) {
    if down calls += 1;
}

calls: int;

main :: () {
    map: K.Keymap;

    assert(map.enabled);
    info: K.Keymap.Proc_Info;
    info.press_proc = press;
    info.toggle_proc = hold;
    info.press_proc();
    info.toggle_proc(true);
    assert(calls == 2);
    binding: K.Single_Key_Mapping;

    assert(binding.proc_index == -1 && binding.respect_ignore_modifier_keys);
    pad: G.Gamepad;

    assert(pad.vendor_id == 0xffff && pad.product_id == 0xffff);
    assert(pad.left_stick_threshold == 0.5 && pad.left_trigger_threshold == 0.3);
    assert(G.Gamepad_Code.A == .FACE_DOWN && G.Gamepad_Code.Y == .FACE_UP);
    print("ok\n");
}
