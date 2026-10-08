A :: cast(Input.Key_Code) #char "A";
D :: cast(Input.Key_Code) #char "D";

Player :: struct {
    using position: Vector2;
    texture: Simp.Texture;
}

init :: (using p: *Player, texture_path: string) {
    if !Simp.texture_load_from_file(*texture, texture_path) {
        print("game: could not load %\n", texture_path);
        exit(1);
    }
    GL.glBindTexture(GL.GL_TEXTURE_2D, texture.gl_handle);
    GL.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MIN_FILTER, GL.GL_NEAREST);
    GL.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MAG_FILTER, GL.GL_NEAREST);
}

key_down :: (key: Input.Key_Code) -> bool {
    return (Input.input_button_states[key] & .DOWN) != 0;
}

frame :: (using p: *Player, dt: float64) {
    dx := cast(int) key_down(D) - cast(int) key_down(A);
    x += dx * 80 * cast(float) dt;
}

draw :: (using p: *Player) {
    Simp.set_shader_for_images(*texture);
    Simp.immediate_quad(x, y, x + 16, y + 16, .{ 1, 1, 1, 1 });
}
