// Private staged MSF 7 wire reader. This is not the reviewed Msf facade.
#scope_module
Tools_Msf_Error :: enum {
    NONE;
    HEADER;
    DIRECTORY;
    PAGE;
    FREE_PAGE;
    STREAM;
    UNSUPPORTED_STREAM;
}

Tools_Msf_Stream :: struct {
    size: s64;
    pages: [..] u32;
}

Tools_Msf_View :: struct {
    content: string; // Borrowed; the caller must retain it unchanged.
    block_size: s64;
    block_count: s64;
    active_fpm: u32;
    streams: [..] Tools_Msf_Stream;
    directory_pages: [..] u32;
}

tools_msf_release :: (view: *Tools_Msf_View) {
    for stream: view.streams array_free(stream.pages);
    array_free(view.streams);
    array_free(view.directory_pages);
    view.* = .{};
}

tools_msf_decode :: (content: string) -> Tools_Msf_View, Tools_Msf_Error {
    empty: Tools_Msf_View;
    if !content.data || content.count < 56 || !tools_msf_magic(content) return empty, .HEADER;

    block_size := cast(s64) tools_msf_u32(content, 32);
    // Other documented sizes have layout variants that are not implemented.
    if block_size != 4096 return empty, .HEADER;

    active_fpm := tools_msf_u32(content, 36);
    block_count := cast(s64) tools_msf_u32(content, 40);
    directory_size := cast(s64) tools_msf_u32(content, 44);
    map_page := tools_msf_u32(content, 52);

    if (active_fpm != 1 && active_fpm != 2) || block_count < 4 || block_count * block_size != content.count return empty, .HEADER;
    if directory_size < 4 || directory_size > content.count || directory_size % 4 != 0 return empty, .DIRECTORY;

    directory_page_count := (directory_size + block_size - 1) / block_size;
    if directory_page_count > block_size / 4 return empty, .DIRECTORY;

    view: Tools_Msf_View = .{ content = content, block_size = block_size, block_count = block_count, active_fpm = active_fpm };
    keep := false;

    defer if !keep tools_msf_release(*view);
    owned: [..] bool;

    array_resize(*owned, block_count);
    defer array_free(owned);
    owned[0] = true;

    for page: 1..block_count - 1 {
        if page % block_size == 1 || page % block_size == 2 owned[page] = true;
    }

    if !tools_msf_claim(view, owned, map_page) return empty, .PAGE;

    directory: [..] u8;

    array_resize(*directory, directory_size);
    defer array_free(directory);
    copied: s64;

    for index: 0..directory_page_count - 1 {
        page := tools_msf_u32(content, cast(s64) map_page * block_size + index * 4);
        if !tools_msf_claim(view, owned, page) return empty, .PAGE;

        array_add(*view.directory_pages, page);

        count := min(block_size, directory_size - copied);
        for byte: 0..count - 1 directory[copied + byte] = content[cast(s64) page * block_size + byte];

        copied += count;
    }

    dir: string = .{ directory.count, directory.data };
    stream_count := cast(s64) tools_msf_u32(dir, 0);

    if stream_count > (directory_size - 4) / 4 return empty, .DIRECTORY;

    position := 4 + stream_count * 4;

    for index: 0..stream_count - 1 {
        size := tools_msf_u32(dir, 4 + index * 4);
        if size == 0xffffffff {
            array_add(*view.streams, Tools_Msf_Stream.{ size = -1 });
            continue;
        }

        // The canonical public descriptor stores a signed 32-bit byte count.
        if size > 0x7fffffff return empty, .UNSUPPORTED_STREAM;

        pages := (cast(s64) size + block_size - 1) / block_size;
        if pages > (directory_size - position) / 4 return empty, .DIRECTORY;

        stream: Tools_Msf_Stream = .{ size = cast(s64) size };

        array_add(*view.streams, stream);

        for page_index: 0..pages - 1 {
            page := tools_msf_u32(dir, position + page_index * 4);
            if !tools_msf_claim(view, owned, page) return empty, .PAGE;
            array_add(*view.streams[index].pages, page);
        }

        position += pages * 4;
    }

    if position != directory_size return empty, .DIRECTORY;

    // Check only reachable/reserved pages. Allocated but unreachable pages can
    // belong to an earlier incremental transaction; they are not rejected.
    for used, page: owned {
        if !used continue;

        bitmap_page := cast(s64) active_fpm + (page / (block_size * 8)) * block_size;
        if bitmap_page >= block_count return empty, .FREE_PAGE;

        bit_index := page % (block_size * 8);
        byte := content[bitmap_page * block_size + bit_index / 8];

        if byte & (cast(u8) 1 << (bit_index % 8)) return empty, .FREE_PAGE;
    }

    keep = true;

    return view, .NONE;
}

tools_msf_copy_stream :: (view: Tools_Msf_View, index: s64) -> [] u8, Tools_Msf_Error {
    if index < 0 || index >= view.streams.count return .{}, .STREAM;

    stream := view.streams[index];
    if stream.size < 0 return .{}, .STREAM;
    if stream.size == 0 return .{}, .NONE;

    output: [..] u8;

    array_resize(*output, stream.size);
    copied: s64;

    for page: stream.pages {
        if page >= view.block_count {
            array_free(output);
            return .{}, .PAGE;
        }

        count := min(view.block_size, stream.size - copied);
        for byte: 0..count - 1 output[copied + byte] = view.content[cast(s64) page * view.block_size + byte];

        copied += count;
    }

    if copied != stream.size {
        array_free(output);
        return .{}, .STREAM;
    }

    return .{ output.count, output.data }, .NONE;
}

#scope_file
#import "Basic";

tools_msf_magic :: (content: string) -> bool {
    prefix :: "Microsoft C/C++ MSF 7.00\r\n";
    for byte, index: prefix if content[index] != byte return false;

    tail := u8.[0x1a, 0x44, 0x53, 0, 0, 0];
    for byte, index: tail if content[prefix.count + index] != byte return false;

    return true;
}

tools_msf_u32 :: (content: string, offset: s64) -> u32 {
    value: u32;
    for index: 0..3 value |= (cast(u32) content[offset + index]) << (index * 8);
    return value;
}

tools_msf_claim :: (view: Tools_Msf_View, owned: [] bool, page: u32) -> bool {
    if page >= view.block_count || owned[page] return false;
    owned[page] = true;
    return true;
}
