// The public WM record is a platform prefix, while current SDL2
// reserves a 64-byte native union. Never let SDL write into the public prefix.
SDL_GetWindowWMInfo :: (window: *SDL_Window, info: *SDL_SysWMinfo) -> SDL_bool #c_call {
    if !info {
        NativeSDL.set_error("SDL_GetWindowWMInfo requires an output record");
        return SDL_FALSE;
    }

    native: NativeSDL.WM_Info;
    native.version.major = info.version.major;
    native.version.minor = info.version.minor;
    native.version.patch = info.version.patch;
    result := cast(SDL_bool) NativeSDL.get_wm_info(window, *native);
    if result == SDL_FALSE return result;

    #if OS == .WINDOWS expected :: SDL_SYSWM_WINDOWS;
    else #if OS == .LINUX expected :: SDL_SYSWM_X11;
    else expected :: SDL_SYSWM_COCOA;

    if native.subsystem != cast(s32) expected {
        NativeSDL.set_error("SDL window subsystem cannot be represented by the public SDL_SysWMinfo");
        return SDL_FALSE;
    }

    info.version.major = native.version.major;
    info.version.minor = native.version.minor;
    info.version.patch = native.version.patch;
    info.subsystem = cast(SDL_SYSWM_TYPE) native.subsystem;

    #if OS == .WINDOWS {
        info.info.win.window = native.info.pointers[0];
    } else #if OS == .LINUX {
        info.info.x11.display = cast(*Display) native.info.pointers[0];
        info.info.x11.window = cast(Window) native.info.pointers[1];
    } else {
        info.info.cocoa.window = native.info.pointers[0];
    }

    return result;
}

// SDK-owned native pointer remains unchanged. Public int is 64-bit, while C
// int is 32-bit; normalize the specified nonzero close flag before the C call.
SDL_LoadBMP_RW :: (src: *SDL_RWops, freesrc: int) -> *SDL_Surface #c_call {
    return cast(*SDL_Surface) NativeSDL.load_bmp_rw(cast(*NativeSDL.RWops) src,
                                                  ifx freesrc == 0 then 0 else 1);
}

#scope_file
NativeSDL :: #import "internal/bindings/sdl-native";
#if OS == .LINUX #import "X11";
