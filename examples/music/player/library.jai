Track :: struct {
    path: string;
    artist: string;
    album: string;
    title: string;
    frames: s64;
    rate: s32;
}

Library :: struct {
    tracks: [..] Track;
    errors: [..] string;
}

scan_library :: (folder: string) -> Library {
    library: Library;
    if !visit_files(folder, true, *library, collect_track, follow_directory_symlinks = false) {
        array_add(*library.errors, copy_string(tprint("Cannot scan %", folder)));
    }

    quick_sort(library.tracks, track_order);

    return library;
}

free_library :: (library: *Library) {
    for library.tracks {
        free(it.path);
        free(it.artist);
        free(it.album);
        free(it.title);
    }

    for library.errors free(it);

    array_free(library.tracks);
    array_free(library.errors);
}

track_order :: (a: Track, b: Track) -> s64 {
    // Untagged tracks sort by filename; tags form artist/album/title groups.
    first := ifx a.artist.count then a.artist else a.title;
    second := ifx b.artist.count then b.artist else b.title;
    result := compare_strings(first, second);

    if result != 0 return result;

    result = compare_strings(a.album, b.album);
    if result != 0 return result;

    result = compare_strings(a.title, b.title);
    if result != 0 return result;

    return compare_strings(a.path, b.path);
}

#scope_file
collect_track :: (info: *File_Visit_Info, library: *Library) {
    if info.had_error {
        array_add(*library.errors, copy_string(tprint("Cannot read %", info.full_name)));
        return;
    }

    if info.is_directory return;

    name := to_lower_copy(info.short_name);

    defer free(name);
    if !ends_with(name, ".wav") && !ends_with(name, ".ogg") return;

    reader: Audio.Reader;
    if !Audio.open(*reader, info.full_name) {
        array_add(*library.errors, copy_string(tprint("Unsupported or damaged: %", info.full_name)));
        return;
    }

    defer Audio.close(*reader);
    array_add(*library.tracks, .{
        path = copy_string(info.full_name), artist = copy_string(reader.artist),
        album = copy_string(reader.album),
        title = copy_string(ifx reader.title.count then reader.title else info.short_name),
        frames = reader.frames, rate = reader.rate,
    });
}
