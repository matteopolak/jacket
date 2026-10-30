// Keyboard, mouse and gamepad folded into the one `Controls` value the simulation reads. Keys are
// read from Input's button states, so held keys and fresh presses come from the same table.

key_down :: (key: u32) -> bool {
    return (input_button_states[key] & .DOWN) != 0;
}

key_pressed :: (key: u32) -> bool {
    return (input_button_states[key] & .START) != 0;
}

pad_down :: (code: Gamepad_Code) -> bool {
    return gamepad.connected && (gamepad.buttons[cast(s64) code] & .DOWN) != 0;
}

pad_pressed :: (code: Gamepad_Code) -> bool {
    return gamepad.connected && (gamepad.buttons[cast(s64) code] & .START) != 0;
}

deadzone :: (v: Vector2) -> Vector2 {
    if length(v) < 0.2 return .{};

    return v;
}

// Call after `update_window_events`. `player` is where the ship is, for aiming at the mouse.
read_controls :: (r: *Renderer, player: Vector2, use_gamepad: bool) -> Controls {
    controls: Controls;

    if key_down(#char "A") || key_down(cast(u32) Key_Code.ARROW_LEFT) controls.move.x -= 1;
    if key_down(#char "D") || key_down(cast(u32) Key_Code.ARROW_RIGHT) controls.move.x += 1;
    if key_down(#char "W") || key_down(cast(u32) Key_Code.ARROW_UP) controls.move.y -= 1;
    if key_down(#char "S") || key_down(cast(u32) Key_Code.ARROW_DOWN) controls.move.y += 1;

    controls.fire = key_down(cast(u32) Key_Code.SPACEBAR) || key_down(cast(u32) Key_Code.MOUSE_BUTTON_LEFT);
    controls.confirm = key_pressed(cast(u32) Key_Code.ENTER) || key_pressed(cast(u32) Key_Code.SPACEBAR);
    controls.back = key_pressed(cast(u32) Key_Code.ESCAPE);
    controls.pause = key_pressed(#char "P");

    if key_down(cast(u32) Key_Code.MOUSE_BUTTON_LEFT) && r.window {
        x, y, ok := get_mouse_pointer_position(r.window, false);

        if ok {
            target := Vector2.{ cast(float) x / r.scale, cast(float) y / r.scale };

            controls.aim = normalize(target - player);
        }
    }

    if use_gamepad && gamepad.connected {
        controls.move += deadzone(Vector2.{ gamepad.left_stick.x, -gamepad.left_stick.y });

        if pad_down(.DPAD_LEFT) controls.move.x -= 1;
        if pad_down(.DPAD_RIGHT) controls.move.x += 1;
        if pad_down(.DPAD_UP) controls.move.y -= 1;
        if pad_down(.DPAD_DOWN) controls.move.y += 1;

        stick := deadzone(Vector2.{ gamepad.right_stick.x, -gamepad.right_stick.y });

        if length(stick) > 0 controls.aim = normalize(stick);

        controls.fire ||= pad_down(.A) || pad_down(.RIGHT_TRIGGER);
        controls.confirm ||= pad_pressed(.A) || pad_pressed(.START);
        controls.back ||= pad_pressed(.B) || pad_pressed(.BACK);
        controls.pause ||= pad_pressed(.START);
    }

    return controls;
}
