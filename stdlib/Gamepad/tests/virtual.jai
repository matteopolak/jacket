#import "Basic";
G :: #import "Gamepad";
I :: #import "Input";
SDL :: #import "SDL";

main :: () {
    G.init_gamepad();
    assert(SDL.SDL_GameControllerEventState(-1) == 0 && SDL.SDL_JoystickEventState(-1) == 0, "owned polling backend left an unused native event queue enabled");
    index := attach_virtual(1, 6, 15, 0);

    assert(index >= 0, "SDL could not attach its virtual test device");
    device := SDL.SDL_JoystickOpen(index);

    assert(device != null);
    assert(set_axis(device, 4, -32768) == 0 && set_axis(device, 5, -32768) == 0);
    G.update_gamepad();
    assert(G.gamepad.connected);
    assert(G.gamepad.buttons.count == I.GAMEPAD_BUTTON_COUNT);
    a_code := cast(int) I.Key_Code.GAMEPAD_0_BEGIN + cast(int) G.Gamepad_Code.A;

    assert(set_button(device, 0, 1) == 0);
    G.update_gamepad();
    assert(I.events_this_frame.count == 0);
    assert(I.input_button_states[a_code] == (.DOWN | .START));
    assert(set_button(device, 0, 0) == 0);
    G.update_gamepad();
    I.input_per_frame_event_and_flag_update();
    G.gamepad.generate_events = true;
    assert(set_button(device, 0, 1) == 0);
    G.update_gamepad();
    assert(G.gamepad.buttons[cast(int) G.Gamepad_Code.A] == (.DOWN | .START));
    assert(I.events_this_frame.count == 1);
    assert(I.events_this_frame[0].key_code == cast(I.Key_Code) (cast(u32) I.Key_Code.GAMEPAD_0_BEGIN + cast(u32) G.Gamepad_Code.A));
    I.input_per_frame_event_and_flag_update();
    G.update_gamepad();
    assert(G.gamepad.buttons[cast(int) G.Gamepad_Code.A] == .DOWN);
    assert(I.events_this_frame.count == 0);
    assert(set_button(device, 0, 0) == 0);
    G.update_gamepad();
    assert(G.gamepad.buttons[cast(int) G.Gamepad_Code.A] == .END);
    assert(set_axis(device, 0, -32768) == 0);
    assert(set_axis(device, 1, 32767) == 0);
    G.update_gamepad();
    assert(G.gamepad.left_stick.x == -1 && G.gamepad.left_stick.y == -1);
    SDL.SDL_JoystickClose(device);
    assert(detach_virtual(index) == 0);
    G.update_gamepad();
    assert(!G.gamepad.connected && G.gamepad.left_stick.x == 0);
    print("ok\n");
}

#scope_file
library :: #library,system "SDL2";
attach_virtual :: (type: s32, axes: s32, buttons: s32, hats: s32) -> s32 #foreign library "SDL_JoystickAttachVirtual";
detach_virtual :: (index: s32) -> s32 #foreign library "SDL_JoystickDetachVirtual";
set_button :: (device: *SDL.SDL_Joystick, button: s32, value: u8) -> s32 #foreign library "SDL_JoystickSetVirtualButton";
set_axis :: (device: *SDL.SDL_Joystick, axis: s32, value: s16) -> s32 #foreign library "SDL_JoystickSetVirtualAxis";
