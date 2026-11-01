#import "Math";
#import "Input";
#import "Window_Creation";
#import "Simp";
SP :: #import "Sound_Player";

key_down :: (key: u32) -> bool {
    return (input_button_states[key] & .DOWN) != 0;
}

key_pressed :: (key: u32) -> bool {
    return (input_button_states[key] & .START) != 0;
}

rgb :: (hex: string) -> Vector4, bool {
    if hex.count != 6 return .{}, false;

    value: u32;

    for hex {
        digit := cast(u32) it;
        if digit >= 48 && digit <= 57 digit -= 48;
        else if digit >= 65 && digit <= 70 digit -= 55;
        else if digit >= 97 && digit <= 102 digit -= 87;
        else return .{}, false;

        value = value * 16 + digit;
    }

    return .{ cast(float) ((value >> 16) & 255) / 255, cast(float) ((value >> 8) & 255) / 255, cast(float) (value & 255) / 255, 1 }, true;
}

run_window :: (m: *Machine, rom: string, cli: Cli) -> s32 {
    foreground, fg_ok := rgb(cli.foreground);
    background, bg_ok := rgb(cli.background);

    if !fg_ok || !bg_ok {
        print("colours must have six hex digits\n", to_standard_error = true);
        return 2;
    }

    window := create_window(128 * cli.scale, 64 * cli.scale, "CHIP-8 | P pause N step R reset");
    if !window {
        print("no display/window\n", to_standard_error = true);
        return 1;
    }

    prepare_window(window, 0);
    set_render_target(window, .LEFT_HANDED);
    samples: [4410] s16;
    for *samples it.* = ifx it_index % 100 < 50 then cast(s16) 3500 else -3500;

    sound := SP.Sound_Data.{ loaded = true, type = .LINEAR_SAMPLE_ARRAY, nchannels = 1, samples = samples.data, nsamples_times_nchannels = samples.count };
    config: SP.Sound_Player_Config;
    config.update_from_a_thread = false;
    audio := !cli.no_audio && SP.sound_player_init(config);

    defer {
        if audio SP.sound_player_shutdown();
    }

    if !cli.no_audio && !audio print("audio unavailable; continuing silently\n", to_standard_error = true);

    stream: *SP.Sound_Stream;

    if audio {
        stream = SP.make_stream(*sound);
        if stream {
            SP.set_repeating(stream, true);
            SP.start_playing(stream);
        }
    }

    keys :: "X123QWEASDZC4RFV";
    paused := false;
    frames := 0;
    accumulator: float64;
    last := seconds_since_init();

    while true {
        now := seconds_since_init();
        elapsed := min(now - last, 0.25);

        last = now;
        update_window_events();
        for events_this_frame if it.type == .QUIT return 0;
        if key_pressed(cast(u32) Key_Code.ESCAPE) return 0;
        for keys set_key(m, it_index, key_down(it));
        if key_pressed(#char "P") paused = !paused;

        if key_pressed(#char "R") {
            reset(m, rom, m.quirks, cli.seed);
            accumulator = 0;
        }

        if paused {
            accumulator = 0;

            if key_pressed(#char "N") {
                m.draw_blocked = false;
                step(m, 1);
            }
        } else {
            advance_time(m, cast(u64) (elapsed * 1000000000));
            accumulator += elapsed;
            while accumulator >= 1.0 / 60.0 {
                step(m, cli.cycles_per_frame);
                accumulator -= 1.0 / 60.0;
            }
        }

        if stream stream.user_volume_scale = ifx !paused && m.sound > 0 then 1.0 else 0.0;
        if audio SP.update(cast(float) elapsed);

        update_window(window);
        clear_render_target(background.x, background.y, background.z, 1);
        set_shader_for_color();
        _, _, w, h, sized := get_dimensions(window, false);
        if !sized return 1;

        scale := max(1, min(w / width(m), h / height(m)));
        ox := (w - width(m) * scale) / 2;
        oy := (h - height(m) * scale) / 2;

        for y: 0..height(m) - 1 for x : 0..width(m) - 1 {
            if m.pixels[y * 128 + x] == 0 continue;
            immediate_quad(cast(float) (ox + x * scale), cast(float) (oy + y * scale), cast(float) (ox + (x + 1) * scale), cast(float) (oy + (y + 1) * scale), foreground);
        }

        immediate_flush();
        swap_buffers(window);
        frames += 1;
        if cli.window_frames > 0 && frames >= cli.window_frames break;
        if m.halted break;

        sleep_milliseconds(1);
        reset_temporary_storage();
    }

    if m.error.count > 0 {
        print("%\n", m.error, to_standard_error = true);
        return 1;
    }

    return 0;
}
