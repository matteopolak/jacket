S :: #import "SDL";

probe :: #library "../../.work/native/sdl_abi_probe";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

write_fixture :: (path: *u8) -> s32 #foreign probe "binding_sdl_fixture";
make_user_event :: (event: *S.SDL_Event, code: s32, data: *void) #foreign probe "binding_sdl_user_event";
user_matches :: (event: *S.SDL_Event, code: s32, data: *void) -> s32 #foreign probe "binding_sdl_user_matches";

main :: () {
    check(S.SDL_Init(.SDL_INIT_EVENTS) == 0);
    defer S.SDL_Quit();
    compile_version, runtime_version: S.SDL_version;

    S.SDL_VERSION(*compile_version);
    S.SDL_GetVersion(*runtime_version);
    check(compile_version.major == 2 && compile_version.minor == 0 && compile_version.patch == 1);
    check(runtime_version.major == 2);
    check(S.SDL_GetPlatform() != null);
    first := S.SDL_GetTicks();

    check(S.SDL_GetTicks() >= first);
    check(S.SDL_SetHint("CLEANROOM_BINDING_TEST", "native") == .SDL_TRUE);
    value := S.SDL_GetHint("CLEANROOM_BINDING_TEST");

    check(value != null && value[0] == #char "n" && value[5] == #char "e");
    pixels: [4] u32 = .[0xff0000ff, 0xff00ff00, 0xffff0000, 0xffffffff];
    surface := S.SDL_CreateRGBSurfaceFrom(pixels.data, 2, 2, 32, 8, 0xff, 0xff00, 0xff0000, 0xff000000);

    check(surface != null);
    check(surface.w == 2 && surface.h == 2 && surface.pitch == 8);
    check(surface.pixels == pixels.data && surface.format.BytesPerPixel == 4);
    check(surface.format.Rmask == 0xff && surface.format.Amask == 0xff000000);
    S.SDL_FreeSurface(surface);
    check(pixels[0] == 0xff0000ff);
    check(write_fixture(".work/native/sdl-test.bmp") == 0);
    bitmap := S.SDL_LoadBMP(".work/native/sdl-test.bmp");

    check(bitmap != null && bitmap.w == 2 && bitmap.h == 1);
    S.SDL_FreeSurface(bitmap);
    S.SDL_ClearError();
    missing := S.SDL_LoadBMP(".work/native/no-such-bitmap.bmp");

    check(missing == null);
    error := S.SDL_GetError();

    check(error != null && error[0] != 0);
    S.SDL_FlushEvents(0, 0xffff);
    input, output: S.SDL_Event;
    payload: u32 = 42;

    // The C fixture fills a complete native event in the public 56-byte union.
    // Missing Jai UserEvent members remain a separate layout failure.
    make_user_event(*input, 17, *payload);
    check(S.SDL_PushEvent(*input) == 1);
    check(S.SDL_PollEvent(*output) == 1);
    check(output.type == .SDL_USEREVENT && user_matches(*output, 17, *payload) == 1);
}
