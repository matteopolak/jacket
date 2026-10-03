// Complete native storage from pinned SDL2-compatible 2.32.70 SDL_syswm.h.
// Separate nominal types keep this ABI independent of the smaller public API.
Version :: struct {
    major: u8;
    minor: u8;
    patch: u8;
}

// Complete SDK event payload; the public UserEvent intentionally exposes only
// type/timestamp. Native event queues carry this within their full event union.
User_Event :: struct {
    type: u32;
    timestamp: u32;
    windowID: u32;
    code: s32;
    data1: *void;
    data2: *void;
}

WM_Info :: struct {
    version: Version;
    subsystem: s32;
    info: union {
        bytes: [64] u8;
        pointers: [8] *void;
    };
}

// Complete desktop ABI, including native-sized counts and C callbacks.
RWops :: struct {
    size: #type (source: *RWops) -> s64 #c_call;
    seek: #type (source: *RWops, offset: s64, whence: s32) -> s64 #c_call;
    read: #type (source: *RWops, data: *void, size: u64, count: u64) -> u64 #c_call;
    write: #type (source: *RWops, data: *void, size: u64, count: u64) -> u64 #c_call;
    close: #type (source: *RWops) -> s32 #c_call;
    type: u32;
    hidden: union {
        #if OS == .WINDOWS {
            bytes: [40] u8;
            pointers: [5] *void;
        } else {
            bytes: [24] u8;
            pointers: [3] *void;
        }
    };
}

rw_from_file :: (path: *u8, mode: *u8) -> *RWops #foreign lib "SDL_RWFromFile";
load_bmp_rw :: (source: *RWops, close_source: s32) -> *void #foreign lib "SDL_LoadBMP_RW";

get_wm_info :: (window: *void, info: *WM_Info) -> s32 #foreign lib "SDL_GetWindowWMInfo";
set_error :: (fmt: *u8, args: ..Any) -> s32 #foreign lib "SDL_SetError";

#scope_file
lib :: #library,system "SDL2";
