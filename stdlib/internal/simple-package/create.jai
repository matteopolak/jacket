init :: (using package: *Create_Package) {
    package.* = .{};
    Pools.set_allocators(*pool);
    B.init_string_builder(*data);
}

deinit :: (using package: *Create_Package) {
    B.free_buffers(*data);
    B.array_free(entries);
    Pools.release(*pool);
    package.* = .{};
} @Releases

add :: (package: *Create_Package, entry_name: string, data: [] u8) {
    B.assert(entry_name.count >= 0 && entry_name.count <= 0xffffffff);
    B.assert(data.count >= 0 && (data.count == 0 || data.data != null));
    name: string;

    if entry_name.count > 0 {
        name = .{ entry_name.count, cast(*u8) Pools.get(*package.pool, entry_name.count) };
        B.assert(name.data != null, "package name allocation failed");
        memcpy(name.data, entry_name.data, name.count);
    }

    B.array_add(*package.entries, .{ name = name, data = data, size_of_entry = data.count });
}

write :: (package: *Create_Package, filename: string, sort_contents := true) -> bool {
    count := package.entries.count;
    if count < 0 || count > 0x7fffffffffffffff / size_of(*Entry_Info) return false;

    order := ([] *Entry_Info).{ count, cast(**Entry_Info) B.alloc(count * size_of(*Entry_Info)) };
    if !order.data && count != 0 return false;

    defer B.free(order.data);
    for *entry, index: package.entries order[index] = entry;
    if sort_contents && !sort_entry_pointers(order) return false;

    toc_offset: s64 = 64;
    total: s64 = 128;

    for entry: order {
        if entry.name.count < 0 || entry.name.count > 0xffffffff || entry.data.count < 0 || (entry.data.count > 0 && !entry.data.data) return false;
        if entry.data.count > 0x7fffffffffffffff - toc_offset return false;

        toc_offset += entry.data.count;
        if entry.data.count > 0x7fffffffffffffff - total return false;

        total += entry.data.count;
        if entry.name.count > 0x7fffffffffffffff - total - 21 return false;

        total += 21 + entry.name.count;
    }

    B.reset(*package.data);
    header: [64] u8;

    Wire.store_u32(header.data, 0, HEADER_MAGIC);
    Wire.store_u32(header.data, 4, 1);
    Wire.store_u64(header.data, 16, cast(u64) toc_offset);
    B.append(*package.data, header.data, header.count);
    offset: s64 = 64;

    for entry: order {
        entry.offset_from_start_of_file = offset;
        entry.size_of_entry = entry.data.count;
        B.append(*package.data, entry.data.data, entry.data.count);
        offset += entry.data.count;
    }

    toc: [64] u8;

    Wire.store_u32(toc.data, 0, TOC_MAGIC);
    Wire.store_u64(toc.data, 8, cast(u64) count);
    B.append(*package.data, toc.data, toc.count);

    for entry: order {
        length: [4] u8;
        words: [16] u8;

        Wire.store_u32(length.data, 0, cast(u32) entry.name.count);
        Wire.store_u64(words.data, 0, cast(u64) entry.size_of_entry);
        Wire.store_u64(words.data, 8, cast(u64) entry.offset_from_start_of_file);
        B.append(*package.data, length.data, length.count);
        B.append(*package.data, entry.name);
        B.append(*package.data, cast(u8) 0);
        B.append(*package.data, words.data, words.count);
    }

    if B.builder_string_length(*package.data) != total return false;

    return Files.write_entire_file(filename, *package.data, do_reset = false);
}

#scope_file
sort_entry_pointers :: (entries: [] *Entry_Info) -> bool {
    if entries.count < 2 return true;

    scratch := ([] *Entry_Info).{ entries.count, cast(**Entry_Info) B.alloc(entries.count * size_of(*Entry_Info)) };
    if !scratch.data return false;

    defer B.free(scratch.data);
    width: s64 = 1;
    while width < entries.count {
        start: s64;
        while start < entries.count {
            middle := start + B.min(width, entries.count - start);
            end := middle + B.min(width, entries.count - middle);
            left := start;
            right := middle;

            for index: start..end - 1 {
                if left < middle && (right >= end || Strings.compare(entries[left].name, entries[right].name) <= 0) {
                    scratch[index] = entries[left];
                    left += 1;
                } else {
                    scratch[index] = entries[right];
                    right += 1;
                }
            }

            start = end;
        }

        memcpy(entries.data, scratch.data, entries.count * size_of(*Entry_Info));
        if width > entries.count / 2 break;

        width *= 2;
    }

    return true;
}
