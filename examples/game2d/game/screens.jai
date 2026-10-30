// The game proper: screens (title, play, pause, game over, leaderboard) and the fixed-step loop.
// Simulation steps run at TICK_RATE however fast frames are drawn; leftover time carries over.

Screen :: enum u8 {
    TITLE;
    PLAYING;
    PAUSED;
    OVER;
    BOARD;
}

Game :: struct {
    cli: Cli;
    renderer: Renderer;
    assets: Assets;
    audio: Audio;
    online: Online;
    world: World;
    field: Starfield;
    save: Save_Data;
    save_path: string;

    screen: Screen;
    quit: bool;
    time: float;
    submitted: bool;
    new_record: bool;
    toast: string;
    toast_until: float;
    seed_counter: s64;
}

default_save_path :: () -> string {
    home, ok := get_home_directory();

    if !ok return "game2d.sav";

    return join(home, "/.game2d.sav");
}

game_init :: (game: *Game, cli: Cli) -> bool {
    game.cli = cli;

    set_working_directory(#filepath);

    if cli.pack {
        if !assets_open_pack(*game.assets, cli.pack) {
            print("cannot open the pack %\n", cli.pack);

            return false;
        }
    } else {
        directory := cli.assets;

        if !directory directory = join(#filepath, "assets");

        if !assets_open_directory(*game.assets, directory) {
            print("no assets in %\n", directory);

            return false;
        }
    }

    if !renderer_open_window(*game.renderer, cli.scale) {
        print("could not open a window (is there a display and OpenGL?)\n");

        return false;
    }

    if !load_texture(*game.assets, *game.renderer.sheet, "sprites.png") || !load_texture(*game.assets, *game.renderer.font, "font.png", .NEAREST) {
        print("could not load the sprite sheets\n");

        return false;
    }

    game.save_path = ifx cli.save then cli.save else default_save_path();

    loaded, ok, why := save_from_file(game.save_path);

    if ok game.save = loaded;
    else if why != "no save yet" print("save: % (starting fresh)\n", why);

    if !game.save.name game.save.name = copy_string(cli.name);

    if !cli.no_audio {
        audio_init(*game.audio, *game.assets);
        audio_set_muted(*game.audio, game.save.muted);
        audio_start_music(*game.audio, *game.assets);
    }

    if !cli.no_hot_reload assets_watch(*game.assets);

    roots := "";

    if cli.roots {
        pem, read := read_entire_file(cli.roots);

        if read roots = pem;
    }

    if online_start(*game.online, cli.server, roots, game.save.account_id, game.save.token) && !game.save.token {
        online_register(*game.online, game.save.name);
    }

    online_refresh_board(*game.online);

    starfield_init(*game.field, 1234);
    game.seed_counter = ifx cli.seed != 0 then cli.seed else to_milliseconds(current_time_monotonic());
    game.screen = .TITLE;

    return true;
}

game_shutdown :: (game: *Game) {
    // The worker may have signed us up: keep what it learned.
    snapshot := online_snapshot(*game.online);

    if snapshot.account_id != 0 && game.online.token {
        game.save.account_id = snapshot.account_id;
        free(game.save.token);
        game.save.token = copy_string(game.online.token);
    }

    game.save.muted = game.audio.muted;
    save_to_file(game.save_path, game.save);

    online_stop(*game.online);
    audio_shutdown(*game.audio);
    assets_close(*game.assets);
}

start_match :: (game: *Game) {
    game.seed_counter += 1;
    world_init(*game.world, game.seed_counter);
    game.screen = .PLAYING;
    game.submitted = false;
    game.new_record = false;
}

finish_match :: (game: *Game) {
    world := *game.world;

    game.new_record = world.score > game.save.high_score;
    remember_score(*game.save, world.score);
    save_to_file(game.save_path, game.save);

    if !game.submitted {
        online_submit(*game.online, match_result(world));
        game.submitted = true;
    }

    game.screen = .OVER;
}

notify :: (game: *Game, message: string) {
    free(game.toast);
    game.toast = copy_string(message);
    game.toast_until = game.time + 2.5;
}

game_run :: (game: *Game) {
    accumulator := 0.0;
    last := seconds_since_init();
    frames := 0;

    while !game.quit {
        now := seconds_since_init();
        frame_time := cast(float) (now - last);

        last = now;
        accumulator += min(frame_time, 0.25);
        game.time += frame_time;

        update_window_events();

        for events_this_frame {
            if it.type == .QUIT game.quit = true;
        }

        update_gamepad();

        reloaded := assets_poll(*game.assets, *game.renderer);

        for reloaded notify(game, tprint("reloaded %", it));

        controls := read_controls(*game.renderer, game.world.player.pos, true);

        if game.cli.autoplay > 0 {
            // A scripted player for demos and the smoke test: start, play, die half way, retry.
            controls = scripted_controls(game.world.tick);

            if game.screen == .TITLE || game.screen == .OVER controls.confirm = frames % 30 == 0;

            if frames == game.cli.autoplay / 2 && game.screen == .PLAYING game.world.lives = 0;

            if frames >= game.cli.autoplay game.quit = true;
        }

        if key_pressed(#char "M") audio_set_muted(*game.audio, !game.audio.muted);

        handle_screen_input(game, controls);

        while accumulator >= TICK_SECONDS {
            accumulator -= TICK_SECONDS;

            if game.screen == .PLAYING {
                world_step(*game.world, controls);
                audio_events(*game.audio, *game.assets, *game.world, game.time);
                array_reset_keeping_memory(*game.world.events);

                if game.world.state == .OVER finish_match(game);
            }

            // A press is only one tick's worth of input: later ticks of this frame see none.
            controls.confirm = false;
            controls.pause = false;
            controls.back = false;
        }

        audio_update(*game.audio, frame_time);
        draw_screen(game);

        reset_temporary_storage();
        frames += 1;
    }
}

handle_screen_input :: (game: *Game, controls: Controls) {
    if #complete game.screen == {
        case .TITLE;
            if controls.confirm start_match(game);

            if key_pressed(#char "L") {
                online_refresh_board(*game.online);
                game.screen = .BOARD;
            }

            if controls.back game.quit = true;
        case .PLAYING;
            if controls.pause game.screen = .PAUSED;

            if controls.back game.screen = .TITLE;
        case .PAUSED;
            if controls.pause || controls.confirm game.screen = .PLAYING;

            if controls.back game.screen = .TITLE;
        case .OVER;
            if controls.confirm start_match(game);

            if controls.back game.screen = .TITLE;

            if key_pressed(#char "L") {
                online_refresh_board(*game.online);
                game.screen = .BOARD;
            }
        case .BOARD;
            if controls.back || controls.confirm game.screen = .TITLE;
    }
}

draw_screen :: (game: *Game) {
    renderer := *game.renderer;

    renderer_begin(renderer);

    snapshot := online_snapshot(*game.online);
    show_world := game.screen == .PLAYING || game.screen == .PAUSED || game.screen == .OVER;

    if show_world {
        draw_world(renderer, *game.world, *game.field, game.time);
        draw_hud(renderer, *game.world, snapshot.status);
    } else {
        draw_stars(renderer, *game.field, game.time);
    }

    use_font(renderer);

    if #complete game.screen == {
        case .TITLE;
            text_centered(renderer, "GAME2D", 40, 24, .{ 0.5, 0.9, 1, 1 });
            text_centered(renderer, "PRESS ENTER TO PLAY", 90, 8);
            text_centered(renderer, "L LEADERBOARD   M MUTE   ESC QUIT", 108, 6, .{ 0.7, 0.7, 0.8, 1 });
            text_centered(renderer, tprint("BEST %   GAMES %", game.save.high_score, game.save.games), 130, 8, .{ 1, 0.9, 0.5, 1 });
            text_centered(renderer, snapshot.status, 160, 6, .{ 0.6, 0.6, 0.7, 1 });
        case .PLAYING;
        case .PAUSED;
            text_centered(renderer, "PAUSED", 80, 16);
        case .OVER;
            text_centered(renderer, "GAME OVER", 50, 16, .{ 1, 0.4, 0.3, 1 });
            text_centered(renderer, tprint("SCORE %", game.world.score), 80, 8);
            text_centered(renderer, tprint("KILLS %", game.world.kills), 92, 8);

            if game.new_record text_centered(renderer, "NEW RECORD", 106, 8, .{ 1, 0.9, 0.3, 1 });

            if snapshot.last_rank > 0 text_centered(renderer, tprint("ONLINE RANK %", snapshot.last_rank), 118, 8, .{ 0.5, 1, 0.6, 1 });

            text_centered(renderer, "ENTER RETRY   L LEADERBOARD   ESC TITLE", 150, 6, .{ 0.7, 0.7, 0.8, 1 });
        case .BOARD;
            draw_board(renderer, snapshot);
    }

    if game.toast && game.time < game.toast_until text(renderer, game.toast, 6, VIEW_H - 12, 6, .{ 1, 1, 0.6, 1 });

    renderer_end(renderer);
}

draw_board :: (renderer: *Renderer, snapshot: Online_Snapshot) {
    text_centered(renderer, "LEADERBOARD", 12, 12, .{ 1, 0.9, 0.5, 1 });

    if snapshot.board.count == 0 {
        text_centered(renderer, snapshot.status, 80, 8, .{ 0.7, 0.7, 0.8, 1 });
    }

    for entry: snapshot.board {
        y := 40 + cast(float) it_index * 14;

        text(renderer, tprint("%.", entry.rank), 70, y, 8);
        text(renderer, entry.player, 96, y, 8, .{ 0.7, 0.9, 1, 1 });
        text(renderer, tprint("%", entry.best_score), 230, y, 8, .{ 1, 0.9, 0.5, 1 });
    }

    text_centered(renderer, "ENTER BACK", 164, 6, .{ 0.7, 0.7, 0.8, 1 });
}
