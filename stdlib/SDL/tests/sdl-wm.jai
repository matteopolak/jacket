S :: #import "SDL";
#import "Basic";
#load "../../internal/bindings/sdl-native/module.jai";

probe :: #library "sdl_runtime_abi_probe";
native_size :: () -> u64 #foreign probe "binding_runtime_wm_size";
native_offset :: () -> u64 #foreign probe "binding_runtime_wm_offset";
native_union_size :: () -> u64 #foreign probe "binding_runtime_wm_union_size";

check :: (value: bool, loc := #caller_location) {
    assert(value, "SDL window-manager ABI/runtime check", loc = loc);
}

main :: () {
    // Independent native header assertions for the complete private storage.
    native: WM_Info;

    check(native_size() == size_of(WM_Info));
    check(native_offset() == (cast(u64) *native.info) - (cast(u64) *native));
    check(native_union_size() == size_of(type_of(native.info)));

    packet: struct {
        before: u64 = 0x89abcdef01234567;
        info: S.SDL_SysWMinfo;
        after: [16] u64;
    }

    for *packet.after it.* = 0xfedcba9876543210;

    S.SDL_VERSION(*packet.info.version);
    check(S.SDL_GetWindowWMInfo(null, *packet.info) == S.SDL_FALSE);
    check(packet.before == 0x89abcdef01234567);
    for packet.after check(it == 0xfedcba9876543210);

    check(S.SDL_GetError() != null && S.SDL_GetError()[0] != 0);

    // Real hidden native window; no substitute SDK entry points are linked.
    initialized := S.SDL_Init(S.SDL_INIT_VIDEO);
    if initialized != 0 print("SDL_Init error: %\n", to_string(S.SDL_GetError()));

    check(initialized == 0);
    defer S.SDL_Quit();
    window := S.SDL_CreateWindow("binding ABI test", 0, 0, 64, 64, S.SDL_WINDOW_HIDDEN);

    check(window != null);
    defer S.SDL_DestroyWindow(window);
    wm_result := S.SDL_GetWindowWMInfo(window, *packet.info);
    if wm_result != S.SDL_TRUE {
        version: S.SDL_version;

        S.SDL_GetVersion(*version);
        print("SDL WM evidence: result=% subsystem=% driver=% version=%.%.% revision=% public_size=% native_size=%\n",
              wm_result, packet.info.subsystem, to_string(S.SDL_GetCurrentVideoDriver()),
              version.major, version.minor, version.patch, to_string(S.SDL_GetRevision()),
              size_of(S.SDL_SysWMinfo), native_size());
    }

    check(wm_result == S.SDL_TRUE);
    #if OS == .MACOS check(packet.info.subsystem == S.SDL_SYSWM_COCOA && packet.info.info.cocoa.window != null);
    else #if OS == .WINDOWS check(packet.info.subsystem == S.SDL_SYSWM_WINDOWS && packet.info.info.win.window != null);
    else check(packet.info.subsystem == S.SDL_SYSWM_X11 && packet.info.info.x11.display != null && packet.info.info.x11.window != 0);
    check(packet.before == 0x89abcdef01234567);
    for packet.after check(it == 0xfedcba9876543210);
}
