init :: (fpm: *Free_Page_Map, cbitsFpm: s32, cbPg: s32) {
    assert(fpm != null && cbitsFpm >= 0 && cbPg == 4096, "Debug_Info: invalid free-page map initialization");
    array_free(fpm.rgw);
    fpm.* = .{};
    fpm.cbPg = cbPg;
    fpm.fBigMsf = true;
    fpm.wFill = -1;
    ensure_room(fpm, cast(u32) cbitsFpm, true);
}

ensure_room :: (using fpm: *Free_Page_Map, upnMac: UPN, fill: bool) {
    assert(fpm != null && fpm.fBigMsf && fpm.cbPg == 4096 && fpm.iwMac == fpm.rgw.count, "Debug_Info: invalid free-page map state");
    words := (cast(s64) upnMac + 63) / 64;
    old := fpm.rgw.count;

    if words <= old return;

    array_resize(*fpm.rgw, words);
    value: s64;
    if fill value = -1;
    for word: old..words - 1 fpm.rgw[word] = value;

    fpm.iwMac = cast(u32) words;
    fpm.wFill = value;
}

set_all :: (using fpm: *Free_Page_Map) {
    assert(fpm != null && fpm.iwMac == fpm.rgw.count, "Debug_Info: invalid free-page map state");
    for *word: fpm.rgw word.* = -1;
    fpm.iwRover = 0;
    fpm.wFill = -1;
}

is_free_page :: (using fpm: Free_Page_Map, page_number: UPN) -> bool {
    word, mask := msf_page_bit(fpm, page_number);
    bits := cast(u64) fpm.rgw[word];
    return (bits & mask) != 0;
}

alloc_page :: (using fpm: *Free_Page_Map, page_number: UPN) {
    assert(fpm != null, "Debug_Info: null free-page map");
    word, mask := msf_page_bit(fpm.*, page_number);
    bits := cast(u64) fpm.rgw[word];
    bits &= ~mask;
    fpm.rgw[word] = cast(s64) bits;
}

free_page :: (using fpm: *Free_Page_Map, page_number: UPN) {
    assert(fpm != null, "Debug_Info: null free-page map");
    word, mask := msf_page_bit(fpm.*, page_number);
    bits := cast(u64) fpm.rgw[word];
    bits |= mask;
    fpm.rgw[word] = cast(s64) bits;
}

is_fpm_page_number :: (using fpm: Free_Page_Map, pn: UPN) -> bool {
    assert(fpm.fBigMsf && fpm.cbPg == 4096, "Debug_Info: unsupported free-page map format");
    index := pn % cast(u32) fpm.cbPg;
    return index == 1 || index == 2;
}

#scope_file
#import "Basic";

UPN :: u32;

msf_page_bit :: (fpm: Free_Page_Map, page: UPN) -> s64, u64 {
    word := cast(s64) page / Free_Page_Map.BPW;

    assert(fpm.rgw.count >= 0 && fpm.iwMac == fpm.rgw.count && fpm.rgw.data != null && word < fpm.rgw.count,
        "Debug_Info: page outside initialized free-page map");
    one: u64 = 1;

    return word, one << (page % Free_Page_Map.BPW);
}
