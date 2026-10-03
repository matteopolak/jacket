init_gamepad :: () {
    if initialized return;

    owns_event_queue := SDL.SDL_WasInit(SDL.SDL_INIT_EVENTS) == 0;

    sdl_set_main_ready();
    result := SDL.SDL_InitSubSystem(SDL.SDL_INIT_GAMECONTROLLER);

    Basic.assert(result == 0, "SDL2 game-controller initialization failed");

    if owns_event_queue {
        // No pre-existing caller event queue exists. Polling supplies the data;
        // avoid accumulating duplicate raw SDK events that this module never drains.
        SDL.SDL_GameControllerEventState(0);
        SDL.SDL_JoystickEventState(0);
        Basic.assert(SDL.SDL_GameControllerEventState(-1) == 0 && SDL.SDL_JoystickEventState(-1) == 0, "SDL2 polling event configuration failed");
    }

    gamepad.buttons = .{ Input.GAMEPAD_BUTTON_COUNT, button_storage.data };
    initialized = true;
}

update_gamepad :: () {
    init_gamepad();
    SDL.SDL_PumpEvents();
    SDL.SDL_GameControllerUpdate();
    for *state: button_storage state.* &= .DOWN;

    if controller && SDL.SDL_GameControllerGetAttached(controller) == .SDL_FALSE {
        SDL.SDL_GameControllerClose(controller);
        controller = null;
    }

    if !controller {
        count := SDL.SDL_NumJoysticks();

        Basic.assert(count >= 0, "SDL2 joystick enumeration failed");

        for index: 0..count - 1 {
            if SDL.SDL_IsGameController(cast(s32) index) == .SDL_FALSE continue;
            controller = SDL.SDL_GameControllerOpen(cast(s32) index);
            Basic.assert(controller != null, "SDL2 game-controller open failed");
            break;
        }
    }

    gamepad.connected = controller != null;

    if !controller {
        gamepad.left_stick = .{};
        gamepad.right_stick = .{};
        gamepad.left_trigger = 0;
        gamepad.right_trigger = 0;
        gamepad.vendor_id = 0xffff;
        gamepad.product_id = 0xffff;
        for index: 0..Input.GAMEPAD_BUTTON_COUNT - 1 update_button(index, false);

        return;
    }

    gamepad.vendor_id = SDL.SDL_GameControllerGetVendor(controller);
    gamepad.product_id = SDL.SDL_GameControllerGetProduct(controller);
    gamepad.left_stick = .{ axis(0), -axis(1) };
    gamepad.right_stick = .{ axis(2), -axis(3) };
    gamepad.left_trigger = Basic.max(0.0, axis(4));
    gamepad.right_trigger = Basic.max(0.0, axis(5));

    for native_button, code: native_button_codes {
        if native_button < 0 continue;
        down := SDL.SDL_GameControllerGetButton(controller, cast(SDL.SDL_GameControllerButton) native_button) != 0;
        update_button(code, down);
    }

    update_button(cast(int) Gamepad_Code.LEFT_TRIGGER, gamepad.left_trigger >= gamepad.left_trigger_threshold);
    update_button(cast(int) Gamepad_Code.RIGHT_TRIGGER, gamepad.right_trigger >= gamepad.right_trigger_threshold);
    update_button(cast(int) Gamepad_Code.LEFT_STICK_UP, gamepad.left_stick.y >= gamepad.left_stick_threshold);
    update_button(cast(int) Gamepad_Code.LEFT_STICK_DOWN, gamepad.left_stick.y <= -gamepad.left_stick_threshold);
    update_button(cast(int) Gamepad_Code.LEFT_STICK_LEFT, gamepad.left_stick.x <= -gamepad.left_stick_threshold);
    update_button(cast(int) Gamepad_Code.LEFT_STICK_RIGHT, gamepad.left_stick.x >= gamepad.left_stick_threshold);
    update_button(cast(int) Gamepad_Code.RIGHT_STICK_UP, gamepad.right_stick.y >= gamepad.right_stick_threshold);
    update_button(cast(int) Gamepad_Code.RIGHT_STICK_DOWN, gamepad.right_stick.y <= -gamepad.right_stick_threshold);
    update_button(cast(int) Gamepad_Code.RIGHT_STICK_LEFT, gamepad.right_stick.x <= -gamepad.right_stick_threshold);
    update_button(cast(int) Gamepad_Code.RIGHT_STICK_RIGHT, gamepad.right_stick.x >= gamepad.right_stick_threshold);
}

#scope_module
sdk_lifetime :: #library,system "SDL2";
sdl_set_main_ready :: () #foreign sdk_lifetime "SDL_SetMainReady";
initialized := false;
controller: *SDL.SDL_GameController;
button_storage: [Input.GAMEPAD_BUTTON_COUNT] Key_Current_State;
// Indexed by our public Gamepad_Code; values are SDL2's documented button enum.
native_button_codes := s32.[11, 12, 13, 14, 6, 4, 9, 10, -1, -1, 0, 1, 2, 3, -1, -1, -1, -1, -1, -1, -1, -1, 7, 8];

axis :: (index: s32) -> float {
    value := SDL.SDL_GameControllerGetAxis(controller, cast(SDL.SDL_GameControllerAxis) index);
    if value < 0 return cast(float) value / 32768;
    return cast(float) value / 32767;
}

update_button :: (index: int, down: bool) {
    old_down := (button_storage[index] & .DOWN) != 0;
    state := Key_Current_State.NONE;

    if down state |= .DOWN;
    if down && !old_down state |= .START;
    if !down && old_down state |= .END;

    button_storage[index] = state;
    code := cast(Input.Key_Code) (cast(int) Input.Key_Code.GAMEPAD_0_BEGIN + index);
    Input.input_button_states[cast(int) code] = state;

    if (state & (.START | .END)) == 0 return;
    if !gamepad.generate_events return;
    if index >= cast(int) Gamepad_Code.LEFT_STICK_UP && index <= cast(int) Gamepad_Code.LEFT_STICK_RIGHT && !gamepad.generate_left_stick_events return;

    event: Input.Event;
    event.type = .KEYBOARD;
    event.key_code = code;
    event.key_pressed = cast(u32) down;
    Basic.array_add(*Input.events_this_frame, event);
}
