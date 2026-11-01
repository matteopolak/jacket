Entry :: struct {
    path: string;
    thumb: Image.Image;
    stamp: s64;
    failed: bool;
}

Cache :: struct {
    entries: [..] Entry;
    cap: s64 = 32 * 1024 * 1024;
    used: s64;
    clock: s64;
}

free_cache :: (cache: *Cache) {
    for *cache.entries {
        free(it.path);
        Image.deinit(*it.thumb);
    }

    array_free(cache.entries);
    cache.* = .{};
}

// Evict before admitting; an individual image bigger than the cap is never cached.
admit :: (cache: *Cache, index: s64, image: Image.Image) -> bool {
    bytes := image.pixels.count;
    if bytes > cache.cap return false;

    cache.used -= cache.entries[index].thumb.pixels.count;
    Image.deinit(*cache.entries[index].thumb);

    // This entry is empty when worker batches call admit.
    while cache.used + bytes > cache.cap {
        oldest: s64 = -1;
        for cache.entries if it.thumb.pixels.count > 0 && (oldest < 0 || it.stamp < cache.entries[oldest].stamp) oldest = it_index;
        if oldest < 0 break;

        cache.used -= cache.entries[oldest].thumb.pixels.count;
        Image.deinit(*cache.entries[oldest].thumb);
    }

    cache.clock += 1;
    cache.entries[index].thumb = image;
    cache.entries[index].stamp = cache.clock;
    cache.used += bytes;

    return true;
}

Thumb_Job :: struct {
    path: string;
    image: Image.Image;
    failed: bool;
}

thumb_job :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    job := cast(*Thumb_Job) work;
    picture := decode_picture(job.path);

    job.failed = picture.image.pixels.count == 0;
    if !job.failed job.image = thumbnail(picture.image);

    release_picture(*picture);
    reset_temporary_storage();

    return .CONTINUE;
}

// Only visible tiles are submitted; at most one small batch is live outside the cache cap.
load_thumbnails :: (cache: *Cache, first: s64, count: s64) {
    jobs: [..] Thumb_Job;
    indices: [..] s64;

    defer array_free(jobs);
    defer array_free(indices);
    end := min(cache.entries.count, first + count);
    for i: first..end - 1 {
        entry := *cache.entries[i];

        cache.clock += 1;
        entry.stamp = cache.clock;
        if entry.thumb.pixels.count || entry.failed continue;

        array_add(*jobs, .{ path = entry.path });
        array_add(*indices, i);
    }

    if !jobs.count return;

    group: Thread_Group;

    init(*group, cast(s32) min(3, jobs.count), thumb_job, temporary_storage_size = 1 << 20);
    group.logging = false;
    start(*group);
    for *jobs add_work(*group, it);

    wait_for_all(*group);
    shutdown(*group);

    for *jobs {
        cache.entries[indices[it_index]].failed = it.failed;
        if !admit(cache, indices[it_index], it.image) Image.deinit(*it.image);
    }
}

open_folder :: (cache: *Cache, target: string) -> bool {
    root := target;
    if !is_directory(target) root = parent_directory(target);

    paths: [..] string;
    collect :: (info: *File_Visit_Info, paths: *[..] string) {
        if info.had_error || info.is_directory return;

        name := to_lower_copy(info.short_name);

        defer free(name);
        extensions := string.[".png", ".jpg", ".jpeg", ".bmp", ".gif", ".tga", ".ico", ".qoi", ".ppm", ".pgm"];
        for extensions if ends_with(name, it) {
            array_add(paths, copy_string(info.full_name));
            break;
        }
    }

    if !visit_files(root, false, *paths, collect) {
        for paths free(it);
        array_free(paths);
        return false;
    }

    quick_sort(paths, compare_strings);
    for paths array_add(*cache.entries, .{ path = it });

    array_free(paths);

    return true;
}

parent_directory :: (path: string) -> string {
    at: s64 = -1;
    for path if it == #char "/" || it == #char "\\" at = it_index;
    if at < 0 return ".";
    if at == 0 return slice(path, 0, 1);
    if at == 2 && path[1] == #char ":" return slice(path, 0, 3);

    return slice(path, 0, at);
}
