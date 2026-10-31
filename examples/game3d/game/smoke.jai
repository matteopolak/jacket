// The scripted session behind `--smoke`: no window, no input. It checks the mesh pipeline, runs the
// job system against a single-threaded run of the same world, decodes the intro clip, then renders
// frames offscreen and looks at what came back. Without a usable GPU the GPU part is skipped
// (or fails with --require-gpu).

SMOKE_WIDTH :: 640; // a multiple of 64, so rows are a multiple of 256 bytes
SMOKE_HEIGHT :: 360;
INTRO_FRAMES :: 50;

Smoke :: struct {
    checks: s64;
    failures: s64;
    log_text: string;
}

check :: (smoke: *Smoke, ok: bool, message: string, args: ..Any) {
    smoke.checks += 1;

    if ok return;

    smoke.failures += 1;
    print("FAIL: %\n", tprint(message, ..args));
}

run_smoke :: (arguments: Cli) -> bool {
    cli := arguments;

    // The assets are found from the game's folder, so a relative output path is fixed first.
    if cli.smoke_frame.count && !is_absolute_path(cli.smoke_frame) cli.smoke_frame = join(get_working_directory(), "/", cli.smoke_frame);

    set_working_directory(#filepath);

    smoke: Smoke;

    smoke_meshes(*smoke);
    smoke_jobs(*smoke, cli);
    smoke_intro_decode(*smoke);

    gpu_ok := smoke_gpu(*smoke, cli);

    if !gpu_ok && cli.require_gpu check(*smoke, false, "no usable GPU, and --require-gpu was given");

    print("smoke: % checks, % failed\n", smoke.checks, smoke.failures);

    return smoke.failures == 0;
}

// Mesh optimisation must keep the data intact and must help: fewer vertices than the soup, a
// better cache ratio, fewer triangles in the simplified level, and a lossless compressed form.
smoke_meshes :: (smoke: *Smoke) {
    set: Mesh_Set;

    build_meshes(*set);

    defer free_meshes(*set);

    for kind: 0..MESH_KINDS - 1 {
        stats := set.stats[kind];

        check(smoke, stats.vertices > 0 && stats.vertices < stats.soup_vertices, "mesh %: % vertices from % in the soup", kind, stats.vertices, stats.soup_vertices);
        check(smoke, stats.acmr_after <= stats.acmr_before + 0.001, "mesh %: cache ratio %, was %", kind, stats.acmr_after, stats.acmr_before);
        check(smoke, stats.roundtrip_ok, "mesh %: the compressed form decodes to different data", kind);
        check(smoke, stats.lod_indices > 0 && stats.lod_indices <= stats.indices, "mesh %: simplified to % of % indices", kind, stats.lod_indices, stats.indices);
    }

    sphere := set.stats[Mesh_Kind.SPHERE];

    check(smoke, sphere.lod_indices * 2 < sphere.indices, "the sphere's coarse level is %, the full one %", sphere.lod_indices, sphere.indices);
    print("meshes: sphere % -> % vertices, cache ratio % -> %, coarse level % of % indices, % bytes compressed\n", sphere.soup_vertices, sphere.vertices, sphere.acmr_before, sphere.acmr_after, sphere.lod_indices, sphere.indices, sphere.encoded_bytes);
}

Sum_Job :: struct {
    total: s64;
}

add_up :: (data: *void, first: s64, count: s64) {
    job := cast(*Sum_Job) data;
    sum := 0;

    for first..first + count - 1 sum += it;

    atomic_add(*job.total, sum);
}

// The same ticks on several threads and on one must give the same world, and a job must visit
// each item exactly once however many times it is run back to back.
smoke_jobs :: (smoke: *Smoke, cli: Cli) {
    jobs: Job_System;

    if !jobs_start(*jobs, max(cli.threads, 2)) {
        check(smoke, false, "the worker threads did not start");

        return;
    }

    defer jobs_stop(*jobs);

    for round: 1..300 {
        sum: Sum_Job;

        run_job(*jobs, add_up, *sum, 1000, 7);

        if sum.total != 499500 {
            check(smoke, false, "round %: the job summed to % instead of 499500", round, sum.total);

            break;
        }
    }

    check(smoke, true, "");

    serial: Job_System;
    parallel_world, serial_world: World;
    count := cli.entities - cli.entities % 4;

    world_init(*parallel_world, count, cast(u32) cli.seed);
    world_init(*serial_world, count, cast(u32) cli.seed);
    check(smoke, world_checksum(*parallel_world) == world_checksum(*serial_world), "the same seed gave different worlds");

    start := world_checksum(*parallel_world);

    for 1..cli.smoke_ticks {
        world_update(*parallel_world, *jobs, TICK);
        world_update(*serial_world, *serial, TICK);
    }

    parallel := world_checksum(*parallel_world);

    check(smoke, parallel != start, "the world did not change");
    check(smoke, parallel == world_checksum(*serial_world), "threads and a single thread disagree: % and %", parallel, world_checksum(*serial_world));

    // Everything stays inside the arena.
    e := *parallel_world.entities;
    inside := true;

    for i: 0..count - 1 {
        if abs(e.x[i]) > ARENA_HALF + 0.01 || abs(e.z[i]) > ARENA_HALF + 0.01 || e.y[i] < ARENA_LOW - 0.01 || e.y[i] > ARENA_HIGH + 0.01 inside = false;
    }

    check(smoke, inside, "an entity left the arena");
    print("jobs: % ticks of % entities on % threads match one thread (checksum %)\n", cli.smoke_ticks, count, jobs.workers.count, parallel);
}

// The clip decodes to the frames it should and the picture changes over time.
smoke_intro_decode :: (smoke: *Smoke) {
    intro: Intro;

    if !intro_open(*intro, INTRO_PATH) {
        check(smoke, false, "%s did not open", INTRO_PATH);

        return;
    }

    defer intro_close(*intro);

    check(smoke, intro.width == 160 && intro.height == 96, "clip size % x %", intro.width, intro.height);

    first_hash, last_hash: u64;
    frames := 0;

    while intro_next_frame(*intro) {
        frames += 1;

        if frames == 1 first_hash = bytes_hash(intro.rgba);

        last_hash = bytes_hash(intro.rgba);
    }

    check(smoke, frames == INTRO_FRAMES, "the clip has % frames, expected %", frames, INTRO_FRAMES);
    check(smoke, first_hash != last_hash, "the first and last frames look the same");
    print("intro: % frames of % x %\n", frames, intro.width, intro.height);
}

// Renders offscreen. Returns false when there is no GPU.
smoke_gpu :: (smoke: *Smoke, cli: Cli) -> bool {
    game: Game;

    if !game_init(*game, cli) return false;

    defer game_shutdown(*game);

    no_window: Window_Type; // zero: nothing to draw into, offscreen only

    if !renderer_init(*game.renderer, no_window) {
        print("smoke: no usable GPU, skipping the rendering checks\n");

        return false;
    }

    r := *game.renderer;
    name := adapter_name(r);

    print("GPU: %\n", name);
    free(name);

    profile_init();

    frames := 30;
    pixels: [] u8;

    for frame: 0..frames - 1 {
        game.camera = scripted_camera(cast(float) frame * TICK * 2);
        r.width, r.height = SMOKE_WIDTH, SMOKE_HEIGHT;

        zone_begin(.FRAME);
        game_tick(*game, true);

        zone_begin(.DRAW);
        array_free(pixels);
        pixels = render_offscreen(r, *game.camera, SMOKE_WIDTH, SMOKE_HEIGHT);
        zone_end();

        zone_end();
        profile_frame_end();
    }

    defer array_free(pixels);

    check(smoke, pixels.count == SMOKE_WIDTH * SMOKE_HEIGHT * 4, "the frame has % bytes", pixels.count);
    check(smoke, r.stats.visible > 0 && r.stats.draws > 0, "nothing was drawn: % visible, % draws", r.stats.visible, r.stats.draws);

    covered, colors := frame_coverage(pixels, r.clear);

    check(smoke, covered > SMOKE_WIDTH * SMOKE_HEIGHT / 5, "only % pixels differ from the background", covered);
    check(smoke, colors > 24, "the frame uses only % distinct colours", colors);

    // The same state drawn again gives the same picture.
    checksum := bytes_hash(pixels);
    again := render_offscreen(r, *game.camera, SMOKE_WIDTH, SMOKE_HEIGHT);

    defer array_free(again);
    check(smoke, bytes_hash(again) == checksum, "drawing the same frame twice gave different pictures");
    print("render: % x % frame, % visible (% far, % near), % draws, % triangles, % background-free pixels, checksum %\n", SMOKE_WIDTH, SMOKE_HEIGHT, r.stats.visible, r.stats.per_lod[1], r.stats.per_lod[0], r.stats.draws, r.stats.triangles, covered, checksum);

    if cli.smoke_frame.count {
        image := Image.{ width = SMOKE_WIDTH, height = SMOKE_HEIGHT, format = .RGBA8, pixels = pixels };
        png := encode_png(image);

        defer array_free(png);

        check(smoke, write_entire_file(cli.smoke_frame, png.data, png.count), "could not write %", cli.smoke_frame);
    }

    smoke_profile(smoke);
    smoke_intro_render(smoke, r);

    return true;
}

// The profile must show every phase of the frames that ran.
smoke_profile :: (smoke: *Smoke) {
    captured: string;

    push_context {
        context.logger = capture_log;
        context.logger_data = *captured;
        profile_report();
    }

    defer free(captured);

    for name: zone_names {
        if name == "input" || name == "present" continue; // not part of the offscreen frames

        check(smoke, contains(captured, name), "the profile has no '%' zone", name);
    }

    check(smoke, profile_zone_calls(.UPDATE) > 0, "the update zone never ran");
    print("%\n", captured);
}

capture_log :: (text: string, data: *void, info: Log_Info) {
    captured := cast(*string) data;

    free(captured.*);
    captured.* = copy_string(text);
}

// The clip drawn letterboxed: the middle of the frame has colour, the bars at the top and bottom
// are black.
smoke_intro_render :: (smoke: *Smoke, r: *Renderer) {
    intro: Intro;

    if !intro_open(*intro, INTRO_PATH) return;

    defer intro_close(*intro);

    intro_prepare(*intro, r);
    intro_advance(*intro, r, 0.5);

    intro_draw(*intro, r, r.offscreen_view, SMOKE_WIDTH, SMOKE_HEIGHT);

    pixels := read_texture(r, r.offscreen, SMOKE_WIDTH, SMOKE_HEIGHT);

    defer array_free(pixels);

    middle := pixel_at(pixels, SMOKE_WIDTH / 2, SMOKE_HEIGHT / 2);
    corner := pixel_at(pixels, 2, 2);

    check(smoke, middle[0] + middle[1] + middle[2] > 30, "the intro frame is black in the middle");
    check(smoke, corner[0] + corner[1] + corner[2] == 0, "the intro's letterbox is not black");
    print("intro: drawn, centre pixel (%, %, %)\n", middle[0], middle[1], middle[2]);
}

#scope_file

pixel_at :: (pixels: [] u8, x: s64, y: s64) -> [3] s32 {
    base := (y * SMOKE_WIDTH + x) * 4;

    return .[pixels[base], pixels[base + 1], pixels[base + 2]];
}

bytes_hash :: (bytes: [] u8) -> u64 {
    hash: u64 = 14695981039346656037;

    for bytes hash = (hash ^ it) * 1099511628211;

    return hash;
}

// How many pixels are not (nearly) the background colour, and how many distinct colours there are
// when each channel is cut to 4 bits.
frame_coverage :: (pixels: [] u8, clear: [4] float) -> s64, s64 {
    background: [3] s32;

    for 0..2 background[it] = cast(s32) (clear[it] * 255 + 0.5);

    seen: [4096] bool;
    covered, colors := 0, 0;

    for pixel: 0..pixels.count / 4 - 1 {
        r, g, b := cast(s32) pixels[pixel * 4], cast(s32) pixels[pixel * 4 + 1], cast(s32) pixels[pixel * 4 + 2];

        if abs(r - background[0]) > 6 || abs(g - background[1]) > 6 || abs(b - background[2]) > 6 covered += 1;

        key := ((r >> 4) << 8) | ((g >> 4) << 4) | (b >> 4);

        if !seen[key] {
            seen[key] = true;
            colors += 1;
        }
    }

    return covered, colors;
}
