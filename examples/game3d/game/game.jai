// The game itself: owns the world, the job system and the renderer, runs one simulation tick and
// the window loop (intro first, then the scene).

TICK :: 1.0 / 60.0;

Game :: struct {
    world: World;
    jobs: Job_System;
    renderer: Renderer;
    camera: Camera;
    meshes_ready: bool;

    // One slot per entity, filled by the cull job.
    instances: [] Instance;
    lods: [] u8;
}

game_init :: (game: *Game, cli: Cli) -> bool {
    if !jobs_start(*game.jobs, cli.threads) {
        print("Could not start % worker threads.\n", cli.threads);

        return false;
    }

    world_init(*game.world, cli.entities - cli.entities % 4, cast(u32) cli.seed);
    game.instances = NewArray(MAX_ENTITIES, Instance);
    game.lods = NewArray(MAX_ENTITIES, u8);

    return true;
}

game_shutdown :: (game: *Game) {
    renderer_shutdown(*game.renderer);
    jobs_stop(*game.jobs);
    array_free(game.instances);
    array_free(game.lods);
}

// One simulation tick followed by culling and packing the instances for the camera.
game_tick :: (game: *Game, simulate: bool) {
    if simulate {
        zone_begin(.UPDATE);
        world_update(*game.world, *game.jobs, TICK);
        zone_end();
    }

    prepare_frame(game);
}

// Everything between the simulation and the draw call: cull entities against the camera in
// parallel, then pack what is visible into the instance buffer layout.
prepare_frame :: (game: *Game) {
    aspect := ifx game.renderer.height > 0 then cast(float) game.renderer.width / cast(float) game.renderer.height else 16.0 / 9.0;
    job: Cull_Job;

    job.world = *game.world;
    job.meshes = *game.renderer.meshes;
    job.planes = frustum_planes(camera_view_projection(*game.camera, aspect));
    job.eye = game.camera.position;
    job.instances = game.instances.data;
    job.lods = game.lods.data;

    zone_begin(.CULL);
    run_job(*game.jobs, cull_job, *job, game.world.count, CHUNK);
    zone_end();

    zone_begin(.PACK);
    pack_instances(*game.renderer, *game.world, game.instances, game.lods);
    zone_end();
}

run_game :: (cli: Cli) -> bool {
    // Assets are looked up from the working directory: make it the game's folder.
    set_working_directory(#filepath);

    window := create_window(1280, 720, "game3d");

    if !window {
        print("No window could be opened (no display?).\n");

        return false;
    }

    game: Game;

    if !game_init(*game, cli) return false;

    defer game_shutdown(*game);

    if !renderer_init(*game.renderer, window) return false;

    name := adapter_name(*game.renderer);

    print("GPU: % (%)\n", name, game.renderer.format);
    free(name);
    profile_init();

    if !cli.no_intro play_intro(*game);

    return play_scene(*game, cli);
}

// Plays the clip until it ends or the player skips it. Returns false if the window was closed.
play_intro :: (game: *Game) -> bool {
    intro: Intro;

    if !intro_open(*intro, INTRO_PATH) {
        print("The intro clip is missing; skipping it.\n");

        return true;
    }

    defer intro_close(*intro);

    intro_prepare(*intro, *game.renderer);
    last := seconds_since_init();

    while !intro.done {
        update_window_events();

        for events_this_frame if it.type == .QUIT return false;

        if key_pressed(cast(u32) Key_Code.ESCAPE) || key_pressed(cast(u32) Key_Code.SPACEBAR) || key_pressed(cast(u32) Key_Code.ENTER) break;

        now := seconds_since_init();

        intro_advance(*intro, *game.renderer, now - last);
        last = now;
        draw_intro_frame(game, *intro);
    }

    return true;
}

draw_intro_frame :: (game: *Game, intro: *Intro) {
    r := *game.renderer;
    width, height := webgpu_window_size(r.window);

    if width != r.width || height != r.height {
        r.width, r.height = width, height;
        configure_surface(r);
    }

    if width <= 0 || height <= 0 {
        sleep_milliseconds(10);

        return;
    }

    frame: WGPUSurfaceTexture;

    wgpuSurfaceGetCurrentTexture(r.surface, *frame);

    if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
        if frame.texture wgpuTextureRelease(frame.texture);

        r.width = 0;
        sleep_milliseconds(10);

        return;
    }

    view := wgpuTextureCreateView(frame.texture, null);

    intro_draw(intro, r, view, width, height);
    present_frame(r);
    wgpuTextureViewRelease(view);
    wgpuTextureRelease(frame.texture);
}

play_scene :: (game: *Game, cli: Cli) -> bool {
    paused := false;
    last := seconds_since_init();
    last_report := last;
    accumulator := 0.0;
    last_mouse: Vector2;
    frames := 0;

    while true {
        now := seconds_since_init();
        dt := cast(float) min(now - last, 0.1);

        last = now;
        zone_begin(.FRAME);

        zone_begin(.INPUT);
        update_window_events();

        for events_this_frame if it.type == .QUIT return true;

        controls := read_controls(game.renderer.window, *last_mouse, dt);

        zone_end();

        if controls.quit return true;

        if controls.pause paused = !paused;

        if key_pressed(#char "R") profile_report();

        camera_apply(*game.camera, controls, dt);

        // The simulation runs in fixed ticks however fast frames come.
        accumulator += dt;

        while accumulator >= TICK {
            if !paused {
                zone_begin(.UPDATE);
                world_update(*game.world, *game.jobs, TICK);
                zone_end();
            }

            accumulator -= TICK;
        }

        prepare_frame(game);

        if !render_to_window(*game.renderer, *game.camera) sleep_milliseconds(10);

        zone_end();
        profile_frame_end();
        frames += 1;

        if cli.frames > 0 && frames >= cli.frames {
            profile_report();

            return true;
        }

        if now - last_report > cast(float64) cli.profile_seconds {
            last_report = now;

            print("% visible, % culled, % draws, % triangles\n", game.renderer.stats.visible, game.renderer.stats.culled, game.renderer.stats.draws, game.renderer.stats.triangles);
            profile_report();
        }
    }

    return true;
}
