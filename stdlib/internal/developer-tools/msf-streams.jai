Free_Page_Map :: struct {
    BPW :: 8 * size_of(u64);
    iwMac: u32;
    iwRover: u32;
    cbPg: s32;
    fBigMsf: bool;
    rgw: [..] s64;
    wFill: s64;
}

Msf :: struct {
    header: *Msf_Header_Big;
    parms: Msf_Parms;
    stream_table: Stream_Table;
    fpm: Free_Page_Map;
    fpm_freed: Free_Page_Map;
    siPnList: Stream_Info;
    content: string;
}

is_msf :: (file_content: string) -> bool {
    magic :: "Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0";
    if file_content.count < magic.count || !file_content.data return false;
    for byte, index: magic if file_content[index] != byte return false;
    return true;
}

is_pdb :: is_msf;

parse_msf :: (file_content: string, path := "", verbose := false) -> success: bool, Msf {
    view, error := tools_msf_decode(file_content);
    if error != .NONE {
        log_error("Debug_Info: invalid or unsupported MSF % (error %)\n", path, error);
        return false, .{};
    }

    defer tools_msf_release(*view);
    result: Msf;
    keep := false;

    defer if !keep msf_release(*result);
    result.content = file_content;
    result.header = cast(*Msf_Header_Big) file_content.data;
    result.parms = .{ cbPg = 4096, page_size_in_bytes = 4096, mask_cb_pg_mod = 4095, cbitsFpm = 32768,
        cpgFileGrowth = 1, pnMax = 0xffffffff, cpnFpm = cast(u32) ((view.block_count + 32767) / 32768),
        pnFpm0 = 1, pnFpm1 = 2, pnDataMin = 3 };
    result.stream_table.page_size_in_bytes = 4096;

    for stream: view.streams {
        item: Stream_Info;
        item.byte_count = cast(s32) stream.size;
        array_add(*result.stream_table.stream_infos, item);
        current := *result.stream_table.stream_infos[result.stream_table.stream_infos.count - 1];
        for page: stream.pages array_add(*current.stream_page_numbers, page);
    }

    result.siPnList.byte_count = result.header.data.st.byte_count;
    for page: view.directory_pages array_add(*result.siPnList.stream_page_numbers, page);

    words := (view.block_count + 63) / 64;

    result.fpm.cbPg = 4096;
    result.fpm.fBigMsf = true;
    result.fpm.wFill = -1;
    result.fpm.iwMac = cast(u32) words;
    array_resize(*result.fpm.rgw, words);
    result.fpm_freed.cbPg = 4096;
    result.fpm_freed.fBigMsf = true;
    result.fpm_freed.iwMac = cast(u32) words;
    array_resize(*result.fpm_freed.rgw, words);

    for word: 0..words - 1 {
        page := word * 64;
        bitmap_page := cast(s64) view.active_fpm + (page / 32768) * 4096;

        if bitmap_page >= view.block_count return false, .{};

        position := bitmap_page * 4096 + (page % 32768) / 8;
        bits: u64;

        for byte: 0..7 bits |= cast(u64) file_content[position + byte] << (byte * 8);

        result.fpm.rgw[word] = cast(s64) bits;
    }

    if verbose log("Debug_Info: MSF % contains % streams\n", path, result.stream_table.stream_infos.count);

    keep = true;

    return true, result;
}

is_valid_user_stream_number :: (msf: Msf, stream_index: UNSN) -> bool {
    return stream_index >= 5 && is_valid_stream_number(msf, stream_index);
}

verify_integrity :: (msf: Msf) -> bool {
    if !msf.header || cast(*u8) msf.header != msf.content.data return false;

    view, error := tools_msf_decode(msf.content);
    if error != .NONE {
        log_error("Debug_Info: MSF wire integrity check failed (%)\n", error);
        return false;
    }

    defer tools_msf_release(*view);
    if msf.stream_table.page_size_in_bytes != 4096 || msf.stream_table.stream_infos.count != view.streams.count ||
        (view.streams.count && !msf.stream_table.stream_infos.data) return false;

    for stream, index: view.streams {
        stored := msf.stream_table.stream_infos[index];
        if stored.byte_count != stream.size || stored.stream_page_numbers.count != stream.pages.count ||
            (stream.pages.count && !stored.stream_page_numbers.data) return false;
        for page, page_index: stream.pages if page != stored.stream_page_numbers[page_index] return false;
    }

    if msf.siPnList.byte_count != msf.header.data.st.byte_count || msf.siPnList.stream_page_numbers.count != view.directory_pages.count ||
        (view.directory_pages.count && !msf.siPnList.stream_page_numbers.data) return false;
    for page, index: view.directory_pages if msf.siPnList.stream_page_numbers[index] != page return false;

    words := (view.block_count + 63) / 64;
    if msf.fpm.cbPg != 4096 || !msf.fpm.fBigMsf || msf.fpm.iwMac != words || msf.fpm.rgw.count != words || !msf.fpm.rgw.data return false;

    for page: 0..view.block_count - 1 {
        bitmap_page := cast(s64) view.active_fpm + (page / 32768) * 4096;
        byte := msf.content[bitmap_page * 4096 + (page % 32768) / 8];
        expected := (byte & (cast(u8) 1 << (page % 8))) != 0;
        if is_free_page(msf.fpm, cast(u32) page) != expected return false;
    }

    return true;
}

is_valid_stream_number :: (msf: Msf, stream_index: UNSN) -> bool {
    return msf.stream_table.stream_infos.count >= 0 && stream_index < msf.stream_table.stream_infos.count && msf.stream_table.stream_infos.data != null;
}

stream_exists :: (msf: Msf, stream_index: UNSN) -> bool {
    return is_valid_stream_number(msf, stream_index) && msf.stream_table.stream_infos[stream_index].byte_count >= 0;
}

get_stream_size :: (msf: Msf, stream_index: UNSN) -> s64 {
    if !stream_exists(msf, stream_index) return -1;
    return msf.stream_table.stream_infos[stream_index].byte_count;
}

is_valid_page_number :: (msf: Msf, pn: UPN) -> bool {
    if !msf.header return false;
    return pn < msf.header.data.page_count;
}

get_stream_data :: (msf: Msf, stream_index: UNSN) -> [] u8, success: bool {
    if !stream_exists(msf, stream_index) {
        log_error("Debug_Info: absent or invalid MSF stream %\n", stream_index);
        return .{}, false;
    }

    if !msf.header || msf.content.count < size_of(Msf_Header_Big) || !msf.content.data || cast(*u8) msf.header != msf.content.data {
        log_error("Debug_Info: MSF header must refer to the retained input\n");
        return .{}, false;
    }

    page_size := cast(s64) msf.header.data.page_size;
    if page_size != 4096 || msf.stream_table.page_size_in_bytes != page_size || msf.header.data.page_count * page_size != msf.content.count || !is_msf(msf.content) {
        log_error("Debug_Info: invalid or unsupported MSF page layout\n");
        return .{}, false;
    }

    stream := msf.stream_table.stream_infos[stream_index];
    size := cast(s64) stream.byte_count;
    required_pages := (size + page_size - 1) / page_size;

    if stream.stream_page_numbers.count != required_pages || (required_pages > 0 && !stream.stream_page_numbers.data) {
        log_error("Debug_Info: MSF stream size and page list disagree\n");
        return .{}, false;
    }

    for page: stream.stream_page_numbers {
        if !is_valid_page_number(msf, page) || page == 0 || page % page_size == 1 || page % page_size == 2 {
            log_error("Debug_Info: invalid or reserved MSF data page\n");
            return .{}, false;
        }
    }

    if size == 0 return .{}, true;

    output := cast(*u8) alloc(size);
    if !output {
        log_error("Debug_Info: MSF stream allocation failed\n");
        return .{}, false;
    }

    copied: s64;

    for page: stream.stream_page_numbers {
        count := min(page_size, size - copied);
        memcpy(output + copied, msf.content.data + cast(s64) page * page_size, count);
        copied += count;
    }

    return .{ size, output }, true;
}

#scope_file
#import "Basic";

#scope_module
msf_release :: (msf: *Msf) {
    for stream: msf.stream_table.stream_infos array_free(stream.stream_page_numbers);

    array_free(msf.stream_table.stream_infos);
    array_free(msf.siPnList.stream_page_numbers);
    array_free(msf.fpm.rgw);
    array_free(msf.fpm_freed.rgw);
    msf.* = .{};
}

#scope_file
UPN :: u32;
UNSN :: u32;

Msf_Header_Big :: union {
    pg: Pg;
    data: struct {
        magic: [30] u8;
        page_size: s32;
        valid_fpm_page_no: UPN;
        page_count: UPN;
        st: Stream_Info_Persist;
        mpspnpnSt: [2] u32;
    }
}

Pg :: struct {
    rgb: [4096] u8;
}

Stream_Info_Persist :: struct {
    byte_count: s32;
    stream_page_numbers: s32;
}

Msf_Parms :: struct {
    cbPg: s32;
    page_size_in_bytes: u32;
    mask_cb_pg_mod: u32;
    cbitsFpm: s32;
    cpgFileGrowth: s32;
    pnMax: UPN;
    cpnFpm: UPN;
    pnFpm0: UPN;
    pnFpm1: UPN;
    pnDataMin: UPN;
}

Stream_Table :: struct {
    page_size_in_bytes: u32;
    stream_infos: [..] Stream_Info;
}

Stream_Info :: struct {
    byte_count: s32 = -1;
    stream_page_numbers: [..] UPN;
}
