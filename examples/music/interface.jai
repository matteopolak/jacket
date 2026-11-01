#import "Extensions/UI";

App :: struct {
    library: Library;
    queue: Queue;
    playback: Playback;
    wave: Waveform;
    wave_ready: bool;
    lyrics: [..] Lyric;
    selected: s64 = -1;
    split: float = 330;
    offline: bool;
    status: string;
}

select_track :: (app: *App, index: s64) {
    if index < 0 || index >= app.library.tracks.count return;

    app.selected = index;
    app.status = "";
    track := app.library.tracks[index];

    free_lyrics(*app.lyrics);
    text, ok := read_entire_file(sidecar_path(track.path));
    if ok {
        app.lyrics = parse_lyrics(text);
        free(text);
    }

    app.wave_ready = false;
    if !start_waveform(*app.wave, track.path) app.status = "Cannot start waveform worker.";
    if !app.offline open_track(*app.playback, track.path);
}

play_next :: (app: *App, automatic := false) {
    if advance(*app.queue, automatic) select_track(app, current_track(*app.queue));
    else {
        if !app.playback.paused toggle_pause(*app.playback);
        app.status = "End of queue";
    }
}

build :: (app: *App) {
    if !app.offline && pump(*app.playback) && !app.playback.paused play_next(app, true);
    if !app.wave_ready app.wave_ready = poll_waveform(*app.wave);

    // Playback uses the synchronous Sound_Player producer, so idle windows still tick.
    current_ui().wake_time = ui_time() + 0.01;

    for column(.{ gap = 10, padding = 14 }) {
        label("Music", .{ size = 26 });

        for row(.{ gap = 8, align = .CENTER }) {
            if button(ifx app.playback.paused then "Play" else "Pause", .{ variant = .PRIMARY }) {
                if !app.playback.opened {
                    if current_track(*app.queue) >= 0 select_track(app, current_track(*app.queue));
                    else if app.queue.items.count play_next(app);
                    else app.status = "Choose a recording or add one to the queue.";
                } else {
                    p := *app.playback;
                    if p.opened && p.paused && p.position >= cast(float64) p.reader.frames / p.reader.rate seek_playback(p, 0);
                    toggle_pause(p);
                }
            }

            if button("Next") play_next(app);
            if button("-10s") seek_playback(*app.playback, app.playback.position - 10);
            if button("+10s") seek_playback(*app.playback, app.playback.position + 10);
            if button("Volume -") app.playback.volume = max(0.0, app.playback.volume - 0.1);
            if button("Volume +") app.playback.volume = min(1.0, app.playback.volume + 0.1);

            label(tprint("% percent", cast(int) (app.playback.volume * 100)));
        }

        for row(.{ gap = 12 }) {
            checkbox(*app.queue.shuffle, "Shuffle");

            if button(tprint("Repeat: %", app.queue.repeat)) {
                if app.queue.repeat == .OFF app.queue.repeat = .ALL;
                else if app.queue.repeat == .ALL app.queue.repeat = .ONE;
                else app.queue.repeat = .OFF;
            }

            label(tprint("% tracks | % ms", app.library.tracks.count, cast(s64) (app.playback.position * 1000)), .{ muted = true });
        }

        if app.playback.error.count label(app.playback.error);
        if app.status.count label(app.status);
        for app.library.errors label(it, .{ muted = true });

        for pane: split(.HORIZONTAL, *app.split) {
            if pane == 0 library_pane(app);
            else detail_pane(app);
        }
    }
}

library_pane :: (app: *App) {
    for column(.{ gap = 6, padding = 6, grow = 1 }) {
        label("Library — select to play; + adds to queue", .{ size = 14 });

        for scroll(.{ grow = 1 }) {
            for track, i: app.library.tracks {
                key(i);

                for row(.{ gap = 6 }) {
                    if button("+") enqueue(*app.queue, i);

                    if list_item(tprint("% / % / %", track.artist, track.album, track.title), app.selected == i) {
                        enqueue(*app.queue, i);
                        app.queue.cursor = app.queue.items.count - 1;
                        select_track(app, i);
                    }
                }
            }
        }
    }
}

detail_pane :: (app: *App) {
    for column(.{ gap = 8, padding = 10, grow = 1 }) {
        if app.selected >= 0 {
            label(app.library.tracks[app.selected].title, .{ size = 22 });
            waveform_widget(app);
            label("Click waveform to seek", .{ muted = true });
            current := lyric_at(app.lyrics, cast(s64) (app.playback.position * 1000));

            for scroll(.{ height = 150 }) {
                for lyric, i: app.lyrics {
                    key(i);
                    if list_item(lyric.text, current == i) seek_playback(*app.playback, cast(float64) lyric.milliseconds / 1000);
                }

                if !app.lyrics.count label("No synced lyrics (.lrc)", .{ muted = true });
            }
        } else label("Choose a recording from the library");

        label("Queue", .{ size = 18 });

        for scroll(.{ grow = 1 }) {
            for track, i: app.queue.items {
                key(i);

                for row(.{ gap = 6 }) {
                    if list_item(app.library.tracks[track].title, app.queue.cursor == i) {
                        app.queue.cursor = i;
                        select_track(app, track);
                    }

                    if button("Remove") {
                        remove_queued(*app.queue, i);
                        break;
                    }
                }
            }
        }
    }
}

waveform_widget :: (app: *App) {
    node := widget(.LEAF, .BUTTON, .{ height = 100 }, #location());
    node.label = "Waveform";
    node.pointer = true;
    node.focusable = true;
    node.paint = paint_waveform;
    node.paint_data = app;
    node.paint_size = .{ 400, 100 };
    if events_of(current_ui(), node).clicked && app.selected >= 0 {
        fraction := clamp((current_ui().input.mouse_x - node.rect.x) / max(1.0, node.rect.w), 0.0, 1.0);
        track := app.library.tracks[app.selected];
        seek_playback(*app.playback, cast(float64) track.frames / track.rate * fraction);
    }
}

paint_waveform :: (ui: *UI_State, node: *Node) {
    app := cast(*App) node.paint_data;

    draw_rect(ui, node.rect, ui.theme.surface_raised, 4);
    if !app.wave_ready || !app.wave.ok return;

    r := node.rect;

    for peak, i: app.wave.peaks {
        height := max(1.0, peak * (r.h - 8));
        draw_rect(ui, .{ r.x + cast(float) i * r.w / 128, r.y + (r.h - height) / 2, max(1.0, r.w / 128 - 1), height }, ui.theme.accent);
    }

    if app.selected >= 0 {
        track := app.library.tracks[app.selected];
        fraction := cast(float) (app.playback.position * track.rate / track.frames);
        draw_rect(ui, .{ r.x + clamp(fraction, 0.0, 1.0) * r.w, r.y, 2, r.h }, rgb(255, 220, 120));
    }
}

close_app :: (app: *App) {
    finish_waveform(*app.wave);
    close_playback(*app.playback);
    free_lyrics(*app.lyrics);
    array_free(app.queue.items);
    free_library(*app.library);
}
