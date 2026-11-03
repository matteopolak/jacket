#import "Basic";

header_survives :: () {
    values := NewArray(1, string);
    values[0] = copy_string("owned");
    slot := *values[0];

    free(slot.*);
    slot.* = "replacement";
    assert(slot.count == 11);
    array_free(values);
}

data_dies :: () -> u8 {
    value := copy_string("owned");
    bytes := value.data;
    free(value);
    return bytes[0];
}

release_string :: (value: *string) {
    free(value.*);
}

wrapped_header_survives :: () {
    values := NewArray(1, string);
    values[0] = copy_string("owned");
    slot := *values[0];

    release_string(slot);
    slot.* = "replacement";
    assert(slot.count == 11);
    array_free(values);
}

Handle :: struct {
    fd: s32;
    other: int;
}

release_handle :: (value: s32) {} @FreesArgument
release_field :: (value: *s32) {
    release_handle(value.*);
}

field_header_survives :: () -> int {
    value := New(Handle);
    value.fd = 7;
    value.other = 9;
    release_field(*value.fd);
    result := value.other;

    free(value);

    return result;
}

field_value_dies :: (value: *Handle) -> s32 {
    release_field(*value.fd);
    return value.fd;
}

Entry :: struct {
    text: string;
    other: int;
}

entry_header_survives :: () -> int {
    values := NewArray(1, Entry);
    values[0].text = copy_string("owned");
    result := 0;

    for *values {
        free(it.text);
        result = it.other;
    }

    array_free(values);

    return result;
}

Thumbnail :: struct {
    pixels: [] u8;
}

Cached_Entry :: struct {
    path: string;
    thumb: Thumbnail;
}

Cache :: struct {
    entries: [..] Cached_Entry;
}

release_thumb :: (thumb: *Thumbnail) {
    free(thumb.pixels.data);
    thumb.* = .{};
}

free_cache :: (cache: *Cache) {
    for *cache.entries {
        free(it.path);
        release_thumb(*it.thumb);
    }

    array_free(cache.entries);
    cache.* = .{};
}

replace_elements :: (names: [] string) {
    for *name: names {
        normalized := copy_string("new");
        free(name.*);
        name.* = normalized;
    }
}
