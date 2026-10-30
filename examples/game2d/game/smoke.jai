// The scripted session behind `./game2d --smoke`, run by tools/smoke_game.sh. It exercises the
// pieces without a person: the simulation twice with the same scripted input (the runs must agree),
// the save file round trip and its corruption checks, the zip pack against the loose files, hot
// reload through File_Watcher, one frame drawn offscreen and read back, and the audio device.
// A check that cannot run on this machine (no display, no audio device) says so and is skipped;
// every check that runs must pass.

Smoke :: struct {
    checks: s64;
    failures: s64;
    skipped: s64;
}

check :: (smoke: *Smoke, ok: bool, what: string, args: ..Any) {
    smoke.checks += 1;

    message := tprint(what, ..args);

    if ok {
        print("  ok    %\n", message);
    } else {
        smoke.failures += 1;
        print("  FAIL  %\n", message);
    }
}

skip :: (smoke: *Smoke, what: string) {
    smoke.skipped += 1;
    print("  skip  %\n", what);
}

// A fixed pattern of moves that fires all the time: enough to score and to get hit.
scripted_controls :: (tick: s64) -> Controls {
    controls: Controls;
    t := cast(float) tick * 0.02;

    controls.move = .{ sin(t * 0.7), cos(t * 0.5) * 0.6 };
    controls.fire = true;

    return controls;
}

Run_Result :: struct {
    hash: u64;
    score: s64;
    kills: s64;
    wave: s64;
    ticks: s64;
    events: s64;
}

run_headless :: (seed: s64, ticks: s64) -> Run_Result {
    world: World;

    world_init(*world, seed);

    result: Run_Result;

    for tick: 0..ticks - 1 {
        // Several lives lost ends the run early; that is part of the session.
        world_step(*world, scripted_controls(tick));
        result.events += world.events.count;
        array_reset_keeping_memory(*world.events);

        if world.state == .OVER break;
    }

    result.hash = world_hash(*world);
    result.score = world.score;
    result.kills = world.kills;
    result.wave = world.wave;
    result.ticks = world.tick;

    world_free(*world);

    return result;
}

smoke_simulation :: (smoke: *Smoke, cli: Cli) {
    print("simulation\n");

    seed := cast(s64) (ifx cli.seed != 0 then cli.seed else 4242);
    first := run_headless(seed, cli.smoke_ticks);
    second := run_headless(seed, cli.smoke_ticks);
    other := run_headless(seed + 1, cli.smoke_ticks);

    check(smoke, first.hash == second.hash, "the same seed and input give the same state (hash %)", first.hash);
    check(smoke, first.hash != other.hash, "another seed gives another state");
    check(smoke, first.score > 0 && first.kills > 0, "the scripted pilot scores: % points, % kills in % ticks (wave %)", first.score, first.kills, first.ticks, first.wave);
    check(smoke, first.events > 0, "the simulation reports events for sound and effects (%)", first.events);

    result := New_Match.{ seed = seed, score = first.score, kills = first.kills, duration_ms = first.ticks * 1000 / TICK_RATE };
    plausible_run, why := plausible(result);

    check(smoke, plausible_run, "the finished match is one the server accepts (%)", why);
}

smoke_save :: (smoke: *Smoke, directory: string) {
    print("save file\n");

    save: Save_Data;

    save.name = "smoke";
    save.token = "0123456789abcdef";
    save.account_id = 7;
    save.muted = true;
    remember_score(*save, 120);
    remember_score(*save, 340);

    path := join(directory, "/test.sav");

    check(smoke, save_to_file(path, save), "the save is written");

    loaded, ok, why := save_from_file(path);

    check(smoke, ok, "the save is read back (%)", why);
    check(smoke, loaded.high_score == 340 && loaded.games == 2 && loaded.name == "smoke" && loaded.token == save.token && loaded.muted && loaded.recent.count == 2, "its fields survive");

    bytes, _ := read_entire_file(path);

    check(smoke, bytes.count > 24 && bytes.count < 200, "it is small (% bytes)", bytes.count);

    flipped := copy_string(bytes);

    flipped[20] = flipped[20] ^ 0x40;
    _, flipped_ok, flipped_why := save_decode(flipped);

    check(smoke, !flipped_ok, "a flipped bit is refused (%)", flipped_why);

    _, short_ok, short_why := save_decode(string.{ bytes.count - 5, bytes.data });

    check(smoke, !short_ok, "a truncated file is refused (%)", short_why);

    _, junk_ok, _ := save_decode("this is not a save file at all, no.");

    check(smoke, !junk_ok, "junk is refused");

    save_free(*loaded);
}

smoke_assets :: (smoke: *Smoke, cli: Cli, directory: string) -> Assets {
    print("assets\n");

    assets: Assets;

    check(smoke, assets_open_directory(*assets, directory), "loose assets open (%)", directory);

    sprites, ok := asset_bytes(*assets, "sprites.png");

    check(smoke, ok && sprites.count > 100, "sprites.png loads (% bytes)", sprites.count);

    if cli.pack {
        pack: Assets;

        if check_pack(smoke, *pack, cli.pack, *assets) {
            check(smoke, true, "the pack and the loose folder hold the same files");
        }

        assets_close(*pack);
    } else {
        skip(smoke, "no --pack given: the zip pack is not checked");
    }

    return assets;
}

check_pack :: (smoke: *Smoke, pack: *Assets, path: string, loose: *Assets) -> bool {
    opened := assets_open_pack(pack, path);

    check(smoke, opened, "the pack opens (%)", path);

    if !opened return false;

    all_same := true;

    for name: SOUND_FILES {
        from_pack, packed_ok := asset_bytes(pack, name);
        from_disk, disk_ok := asset_bytes(loose, name);

        same := packed_ok && disk_ok && from_pack.count == from_disk.count && XXH64_of(cast(string) from_pack) == XXH64_of(cast(string) from_disk);

        check(smoke, same, "% from the pack matches the file (% bytes, xxh64 %)", name, from_pack.count, XXH64_of(cast(string) from_pack));

        all_same &&= same;
    }

    for name: string.["sprites.png", "font.png"] {
        from_pack, packed_ok := asset_bytes(pack, name);
        from_disk, disk_ok := asset_bytes(loose, name);

        same := packed_ok && disk_ok && XXH64_of(cast(string) from_pack) == XXH64_of(cast(string) from_disk);

        check(smoke, same, "% from the pack matches the file", name);

        all_same &&= same;
    }

    _, missing := asset_bytes(pack, "nothing.png");

    check(smoke, !missing, "a name that is not in the pack is not found");

    return all_same;
}

smoke_render :: (smoke: *Smoke, cli: Cli, assets: *Assets) -> Renderer, bool {
    print("rendering\n");

    renderer: Renderer;

    if !renderer_open_offscreen(*renderer, 2) {
        skip(smoke, "no window or OpenGL here: the offscreen frame is not drawn");

        return renderer, false;
    }

    loaded := load_texture(assets, *renderer.sheet, "sprites.png") && load_texture(assets, *renderer.font, "font.png", .NEAREST);

    check(smoke, loaded, "the sprite sheets load into textures");

    if !loaded return renderer, false;

    check(smoke, renderer.sheet.width == SHEET_COLUMNS * SHEET_CELL && renderer.sheet.height == SHEET_ROWS * SHEET_CELL, "the sprite sheet is % x %", renderer.sheet.width, renderer.sheet.height);

    world: World;
    field: Starfield;

    world_init(*world, 99);
    starfield_init(*field, 5);

    for tick: 0..239 {
        world_step(*world, scripted_controls(tick));
        array_reset_keeping_memory(*world.events);
    }

    hashes: [2] u64;
    lit_pixels: s64;

    for pass: 0..1 {
        renderer_begin(*renderer);
        draw_world(*renderer, *world, *field, 1.0);
        draw_hud(*renderer, *world, "SMOKE");
        use_font(*renderer);
        text_centered(*renderer, "GAME2D", 60, 16);

        frame := renderer_read(*renderer);

        if pass == 0 {
            lit_pixels = count_lit(*frame);

            if cli.smoke_frame check(smoke, bitmap_save(*frame, cli.smoke_frame, true), "the frame is saved as %", cli.smoke_frame);
        }

        hashes[pass] = XXH64_of(string.{ frame.stride * frame.height, frame.data.data });

        check(smoke, frame.width == VIEW_W * 2 && frame.height == VIEW_H * 2, "the frame is % x %", frame.width, frame.height);
        deinit(*frame);
    }

    check(smoke, lit_pixels > 2000, "the frame has content: % lit pixels", lit_pixels);
    check(smoke, hashes[0] == hashes[1], "drawing the same scene twice gives the same pixels (xxh64 %)", hashes[0]);

    world_free(*world);

    return renderer, true;
}

count_lit :: (frame: *Bitmap) -> s64 {
    count: s64;

    for y: 0..frame.height - 1 for x : 0..frame.width - 1 {
        p := frame.data.data + y * frame.stride + x * 4;

        // The clear colour is a very dark blue; anything brighter was drawn.
        if p[0] > 40 || p[1] > 40 || p[2] > 60 count += 1;
    }

    return count;
}

smoke_hot_reload :: (smoke: *Smoke, renderer: *Renderer, directory: string, scratch: string) {
    print("hot reload\n");

    copy_dir := join(scratch, "/assets");

    make_directory_if_it_does_not_exist(copy_dir);

    for string.["sprites.png", "font.png"] {
        bytes, _ := read_entire_file(join(directory, "/", it));

        write_entire_file(join(copy_dir, "/", it), bytes);
    }

    assets: Assets;

    check(smoke, assets_open_directory(*assets, copy_dir), "a copy of the assets opens");
    check(smoke, assets_watch(*assets), "the watcher starts on it");

    // A different sprite sheet (the font sheet in its place) saved over sprites.png.
    replacement, _ := read_entire_file(join(copy_dir, "/font.png"));

    write_entire_file(join(copy_dir, "/sprites.png"), replacement);

    reloaded: [] string;
    deadline := seconds_since_init() + 5;

    while seconds_since_init() < deadline && assets.reloads == 0 {
        reloaded = assets_poll(*assets, renderer);
        sleep_milliseconds(20);
    }

    check(smoke, assets.reloads >= 1, "saving sprites.png reloads it (% reloads)", assets.reloads);
    check(smoke, renderer.sheet.width == GLYPH_COLUMNS * GLYPH_W && renderer.sheet.height == GLYPH_ROWS * GLYPH_H, "the running texture is now the new image (% x %)", renderer.sheet.width, renderer.sheet.height);

    assets_close(*assets);
}

smoke_audio :: (smoke: *Smoke, assets: *Assets) {
    print("audio\n");

    audio: Audio;

    config: SP.Sound_Player_Config;

    config.update_from_a_thread = false;
    audio.ready = SP.sound_player_init(config);

    if !audio.ready {
        skip(smoke, "no audio output device: sounds are not played");
    } else {
        ok := true;

        for Sound_Id.SHOOT..Sound_Id.MUSIC ok = ok && load_sound(assets, it);

        check(smoke, ok, "wav and ogg files decode through Sound_Player and stb_vorbis");

        SP.set_master_volume(0);
        audio_play(*audio, assets, .SHOOT);
        audio_play(*audio, assets, .EXPLODE);

        for 1..5 audio_update(*audio, 0.016);

        check(smoke, audio.played == 2, "two sounds were started silently");
        SP.sound_player_shutdown();
    }
}

smoke_online :: (smoke: *Smoke, cli: Cli) {
    print("online\n");

    if !cli.server {
        skip(smoke, "no --server given: the backend is not called");

        return;
    }

    roots := "";

    if cli.roots roots, _ = read_entire_file(cli.roots);

    api: Api;

    failure := api_init(*api, cli.server, roots);

    check(smoke, !failure, "the API client is set up (%)", failure);

    health, health_failure := api_health(*api);

    check(smoke, !health_failure && health.ok && health.version == API_VERSION, "the server answers health (%)", health_failure);

    name := tprint("smoke%", to_milliseconds(current_time_monotonic()) % 1_000_000);
    account, account_failure := api_create_account(*api, name);

    check(smoke, !account_failure && account.id > 0, "an account is created (%)", account_failure);

    run := run_headless(4242, cli.smoke_ticks);
    match := New_Match.{ seed = 4242, score = run.score, kills = run.kills, duration_ms = run.ticks * 1000 / TICK_RATE };
    result, submit_failure := api_submit_match(*api, match);

    check(smoke, !submit_failure && result.score == run.score, "the scripted match is accepted (%)", submit_failure);

    cheat := match;

    cheat.score = 9_000_000;
    _, cheat_failure := api_submit_match(*api, cheat);

    check(smoke, cheat_failure != "", "an impossible score is refused (%)", cheat_failure);

    board, board_failure := api_leaderboard(*api, MAX_LEADERBOARD);

    found := false;

    for board if it.player == name found = true;

    check(smoke, !board_failure && (found || board.count == MAX_LEADERBOARD), "the new player is on the leaderboard, or outscored by a full page of others (% entries)", board.count);
}

run_smoke :: (cli: Cli) -> bool {
    smoke: Smoke;

    set_working_directory(#filepath);

    directory := cli.assets;

    if !directory directory = join(#filepath, "assets");

    scratch := tprint("%/game2d-smoke-%", ifx has_environment_variable("TMPDIR") then environment_variable("TMPDIR") else "/tmp", to_milliseconds(current_time_monotonic()));

    make_directory_if_it_does_not_exist(scratch);

    smoke_simulation(*smoke, cli);
    smoke_save(*smoke, scratch);

    assets := smoke_assets(*smoke, cli, directory);

    renderer, rendered := smoke_render(*smoke, cli, *assets);

    if rendered smoke_hot_reload(*smoke, *renderer, directory, scratch);
    else skip(*smoke, "hot reload needs the renderer");

    smoke_audio(*smoke, *assets);
    smoke_online(*smoke, cli);

    print("\n% checks, % failed, % skipped\n", smoke.checks, smoke.failures, smoke.skipped);

    return smoke.failures == 0 && smoke.checks > 0;
}
