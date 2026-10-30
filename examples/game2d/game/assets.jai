// Where the game's files come from: a zip pack (Zip_File_Directory, deflated entries inflated with
// Extensions/Deflate and checked with their CRC) or, while developing, a loose folder that
// File_Watcher watches so a saved PNG or WAV shows up in the running game.

Sound_Id :: enum u8 {
    SHOOT;
    HIT;
    EXPLODE;
    PICKUP;
    MUSIC;
}

SOUND_FILES :: string.["shoot.wav", "hit.wav", "explode.wav", "pickup.wav", "music.ogg"];

Assets :: struct {
    directory: string; // loose files, when not loading from a pack
    pack: Zip_Directory;
    packed: bool;

    sounds: [SOUND_FILES.count] *SP.Sound_Data;
    watcher: File_Watcher(Assets);
    watching: bool;
    changed: [..] string; // names the watcher saw, handled by `assets_apply_changes`
    reloads: s64;
}

assets_open_pack :: (assets: *Assets, path: string) -> bool {
    ok, directory := load_zip_directory(path, close_file = false);

    if !ok return false;

    assets.pack = directory;
    assets.packed = true;

    return true;
}

assets_open_directory :: (assets: *Assets, path: string) -> bool {
    if !file_exists(join(path, "/sprites.png")) return false;

    assets.directory = copy_string(path);

    return true;
}

assets_close :: (assets: *Assets) {
    if assets.watching deinit(*assets.watcher);

    if assets.packed {
        file_close(*assets.pack.file);
        deinit(*assets.pack);
    }
}

// The bytes of `name`, which the caller frees.
asset_bytes :: (assets: *Assets, name: string) -> [] u8, bool {
    if assets.packed return read_pack_entry(*assets.pack, name);

    contents, ok := read_entire_file(join(assets.directory, "/", name));

    return cast([] u8) contents, ok;
}

read_pack_entry :: (pack: *Zip_Directory, name: string) -> [] u8, bool {
    for entry: pack.entries {
        if entry.filename != name continue;

        // The local header repeats the name and may carry an extra field of its own.
        header: [30] u8;

        if !file_set_position(pack.file, entry.chunk_offset) return .[], false;

        ok, got := file_read(pack.file, header.data, header.count);

        if !ok || got != header.count return .[], false;

        method := (cast(u16) header[8]) | ((cast(u16) header[9]) << 8);
        name_length := (cast(s64) header[26]) | ((cast(s64) header[27]) << 8);
        extra_length := (cast(s64) header[28]) | ((cast(s64) header[29]) << 8);

        if !file_set_position(pack.file, entry.chunk_offset + 30 + name_length + extra_length) return .[], false;

        stored: [] u8;

        stored.count = entry.compressed_len;
        stored.data = alloc(entry.compressed_len);
        ok, got = file_read(pack.file, stored.data, stored.count);

        if !ok || got != stored.count {
            free(stored.data);

            return .[], false;
        }

        if method == 0 return stored, true;

        defer free(stored.data);

        unpacked, inflated, why := inflate(stored, .RAW);

        if !inflated || unpacked.count != entry.chunk_len || crc32(unpacked) != entry.crc {
            print("pack: % is damaged (%)\n", name, why);

            return .[], false;
        }

        return unpacked, true;
    }

    return .[], false;
}

load_texture :: (assets: *Assets, texture: *Texture, name: string, filter := Texture_Filter.LINEAR) -> bool {
    bytes, ok := asset_bytes(assets, name);

    if !ok return false;

    defer free(bytes.data);

    if !texture_load_from_memory(texture, bytes) return false;

    texture_set_filter(texture, filter);

    return true;
}

load_sound :: (assets: *Assets, id: Sound_Id) -> bool {
    name := SOUND_FILES[cast(s64) id];
    bytes, ok := asset_bytes(assets, name);

    if !ok return false;

    // The sound's bytes stay in `buffer` for as long as the sound lives. A reload decodes into the same
    // `Sound_Data` so pointers to it stay good: `replace_sound_data` ends the streams that play the old
    // one and frees its bytes.
    decoded := SP.load_audio_data(name, cast(string) bytes);

    if !decoded.loaded {
        free(bytes.data);

        return false;
    }

    existing := assets.sounds[cast(s64) id];

    if existing return SP.replace_sound_data(existing, decoded);

    data := New(SP.Sound_Data);

    data.* = decoded;
    assets.sounds[cast(s64) id] = data;

    return true;
}

on_asset_change :: (watcher: *File_Watcher(Assets), change: *File_Change, assets: *Assets) {
    if change.events & .REMOVED return;

    name := path_filename(change.full_path);

    for SOUND_FILES if it == name {
        array_add(*assets.changed, copy_string(name));

        return;
    }

    if name == "sprites.png" || name == "font.png" array_add(*assets.changed, copy_string(name));
}

assets_watch :: (assets: *Assets) -> bool {
    if assets.packed || !assets.directory return false;

    if !init(*assets.watcher, on_asset_change, assets, merge_window_seconds = 0.15) return false;

    if !add_directories(*assets.watcher, assets.directory) {
        deinit(*assets.watcher);

        return false;
    }

    assets.watching = true;

    return true;
}

// Call once a frame, on the thread that owns the OpenGL context. Returns the names it reloaded.
assets_poll :: (assets: *Assets, renderer: *Renderer) -> [] string {
    reloaded: [..] string;

    reloaded.allocator = temp;

    if !assets.watching return reloaded;

    process_changes(*assets.watcher);

    for assets.changed {
        ok := false;

        if it == "sprites.png" {
            replacement: Texture;

            ok = load_texture(assets, *replacement, it);

            if ok {
                texture_destroy(*renderer.sheet);
                renderer.sheet = replacement;
            }
        } else if it == "font.png" {
            replacement: Texture;

            ok = load_texture(assets, *replacement, it, .NEAREST);

            if ok {
                texture_destroy(*renderer.font);
                renderer.font = replacement;
            }
        } else {
            ok = reload_sound_named(assets, it);
        }

        if ok {
            assets.reloads += 1;
            array_add(*reloaded, it);
        }
    }

    array_reset_keeping_memory(*assets.changed);

    return reloaded;
}

reload_sound_named :: (assets: *Assets, name: string) -> bool {
    for SOUND_FILES if it == name return load_sound(assets, cast(Sound_Id) it_index);

    return false;
}
