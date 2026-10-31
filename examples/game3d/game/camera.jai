// A free-flying camera and the keys and mouse that move it. Input comes from the standard
// `Input` module's button table, the same way game2d reads it.

Camera :: struct {
    position: Vector3 = .{ 0, 9, 52 };
    yaw: float; // 0 looks along -z; positive turns right
    pitch: float = -0.12;
    fov: float = 1.0; // vertical, radians
    near: float = 0.1;
    far: float = 300;
}

// What the player asked for this frame.
Controls :: struct {
    move: Vector3; // x right, y up, z forward
    turn: Vector2; // radians to add to yaw and pitch
    fast: bool;
    quit: bool;
    skip: bool; // skip the intro
    pause: bool;
}

camera_forward :: (camera: *Camera) -> Vector3 {
    return .{ sin(camera.yaw) * cos(camera.pitch), sin(camera.pitch), -cos(camera.yaw) * cos(camera.pitch) };
}

camera_right :: (camera: *Camera) -> Vector3 {
    return .{ cos(camera.yaw), 0, sin(camera.yaw) };
}

camera_view :: (camera: *Camera) -> Matrix4 {
    eye := camera.position;

    return make_look_at_matrix(eye, eye + camera_forward(camera), .{ 0, 1, 0 }, x_is_forward = false);
}

// View then projection, with clip depth in 0..1 as WebGPU wants.
camera_view_projection :: (camera: *Camera, aspect: float) -> Matrix4 {
    projection := make_projection_matrix(camera.fov, aspect, camera.near, camera.far, depth_range_01 = true);

    return projection * camera_view(camera);
}

camera_apply :: (camera: *Camera, controls: Controls, dt: float) {
    camera.yaw += controls.turn.x;
    camera.pitch = clamp(camera.pitch + controls.turn.y, -1.4, 1.4);

    speed := ifx controls.fast then 36.0 else 14.0;
    forward := camera_forward(camera);
    right := camera_right(camera);
    movement := forward * controls.move.z + right * controls.move.x + Vector3.{ 0, controls.move.y, 0 };

    camera.position += movement * (speed * dt);
    camera.position.y = max(camera.position.y, 0.5);
}

key_down :: (key: u32) -> bool {
    return (input_button_states[key] & .DOWN) != 0;
}

key_pressed :: (key: u32) -> bool {
    return (input_button_states[key] & .START) != 0;
}

// Reads the keyboard and mouse. Call after `update_window_events`. Holding the right mouse
// button turns the camera with the mouse; the arrow keys always turn it.
read_controls :: (window: Window_Type, last_mouse: *Vector2, dt: float) -> Controls {
    controls: Controls;

    if key_down(#char "W") controls.move.z += 1;
    if key_down(#char "S") controls.move.z -= 1;
    if key_down(#char "D") controls.move.x += 1;
    if key_down(#char "A") controls.move.x -= 1;
    if key_down(#char "E") controls.move.y += 1;
    if key_down(#char "Q") controls.move.y -= 1;

    turn_speed := 1.8 * dt;

    if key_down(cast(u32) Key_Code.ARROW_LEFT) controls.turn.x -= turn_speed;
    if key_down(cast(u32) Key_Code.ARROW_RIGHT) controls.turn.x += turn_speed;
    if key_down(cast(u32) Key_Code.ARROW_UP) controls.turn.y += turn_speed;
    if key_down(cast(u32) Key_Code.ARROW_DOWN) controls.turn.y -= turn_speed;

    x, y, ok := get_mouse_pointer_position(window, false);

    if ok {
        mouse := Vector2.{ cast(float) x, cast(float) y };

        if key_down(cast(u32) Key_Code.MOUSE_BUTTON_RIGHT) {
            controls.turn.x += (mouse.x - last_mouse.x) * 0.004;
            controls.turn.y -= (mouse.y - last_mouse.y) * 0.004;
        }

        last_mouse.* = mouse;
    }

    controls.fast = key_down(cast(u32) Key_Code.SHIFT);
    controls.quit = key_pressed(cast(u32) Key_Code.ESCAPE);
    controls.skip = key_pressed(cast(u32) Key_Code.SPACEBAR) || key_pressed(cast(u32) Key_Code.ENTER) || key_pressed(cast(u32) Key_Code.MOUSE_BUTTON_LEFT);
    controls.pause = key_pressed(#char "P");

    return controls;
}

// A scripted flight for the smoke test and the attract mode: a slow orbit around the arena
// centre, looking at it. `t` is in seconds.
scripted_camera :: (t: float) -> Camera {
    camera: Camera;
    angle := t * 0.35;
    radius := 46.0;

    camera.position = .{ sin(angle) * radius, 12 + sin(t * 0.5) * 3, cos(angle) * radius };

    to_center := Vector3.{ 0, 6, 0 } - camera.position;

    camera.yaw = atan2(to_center.x, -to_center.z);
    camera.pitch = atan2(to_center.y, sqrt(to_center.x * to_center.x + to_center.z * to_center.z));

    return camera;
}
