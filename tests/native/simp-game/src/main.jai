// The game that tests/native/simp-game/first.jai builds: a window, a texture loaded from a PNG
// (stb_image, from the prebuilt native libraries), GL viewport calls and Input events, drawn for
// a few frames.
#import "Basic";
#import "Window_Creation";
#import "System";
#import "String";
#import "Math";

GL :: #import "GL";
Input :: #import "Input";
Simp :: #import "Simp";

#load "player.jai";

WORLD_WIDTH :: 320;
WORLD_HEIGHT :: 180;
FRAMES :: 30;

window: Window_Type;
player: Player;

main :: () {
    window = create_window(640, 360, "Game");
    width, height := Simp.get_render_dimensions(window);
    Simp.set_render_target(window, .RIGHT_HANDED);

    // The texture comes from a PNG written next to the executable (stb_image_write).
    texture_path := join(path_strip_filename(get_path_of_running_executable()), "player.png");
    write_player_png(texture_path);
    init(*player, texture_path);

    time := seconds_since_init();
    for 1..FRAMES {
        Input.update_window_events();
        for Input.get_window_resizes() {
            Simp.update_window(it.window);
            if it.window == window {
                width = it.width;
                height = it.height;
            }
        }
        for Input.events_this_frame {
            if it.type == .QUIT return;
        }

        now := seconds_since_init();
        frame(*player, now - time);
        time = now;

        Simp.clear_render_target(0, 0, 0, 1);
        GL.glViewport(0, 0, cast(u32) width, cast(u32) height);
        Simp.set_shader_for_color();
        Simp.immediate_set_2d_projection(WORLD_WIDTH, WORLD_HEIGHT);
        Simp.immediate_quad(0, 0, cast(float) WORLD_WIDTH, cast(float) WORLD_HEIGHT, .{ 0.2, 0.3, 0.3, 1 });
        draw(*player);
        Simp.swap_buffers(window);
        reset_temporary_storage();
    }
    print("game: drew % frames\n", FRAMES);
}

write_player_png :: (path: string) {
    bitmap: Simp.Bitmap;
    Simp.bitmap_alloc(*bitmap, 16, 16, .RGBA8);
    defer Simp.deinit(*bitmap);
    for i: 0..bitmap.data.count / 4 - 1 {
        bitmap.data[i * 4 + 0] = 230;
        bitmap.data[i * 4 + 1] = 120;
        bitmap.data[i * 4 + 2] = 40;
        bitmap.data[i * 4 + 3] = 255;
    }
    if !Simp.bitmap_save(*bitmap, path, save_alpha = true) {
        print("game: could not write %\n", path);
        exit(1);
    }
}
