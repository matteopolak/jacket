#import "Basic";
#import "Extensions/SDL3";

main :: () {
    assert(SDL_GetVersion() == SDL_VERSION);
    assert(SDL_Init(SDL_INIT_EVENTS));
    defer SDL_Quit();
    assert((SDL_WasInit(0) & SDL_INIT_EVENTS) != 0);
    event: SDL_Event;
    event.type = cast(u32) SDL_EVENT_USER;
    event.user.code = 123;
    assert(SDL_PushEvent(*event));
    received: SDL_Event;

    assert(SDL_PollEvent(*received));
    assert(received.type == cast(u32) SDL_EVENT_USER);
    assert(received.user.code == 123);
    io_data: [8] u8;
    io := SDL_IOFromMem(io_data.data, 8);

    assert(io != null);
    assert(SDL_WriteU32LE(io, 0x12345678));
    assert(SDL_SeekIO(io, 0, SDL_IO_SEEK_SET) == 0);
    value: u32;

    assert(SDL_ReadU32LE(io, *value));
    assert(value == 0x12345678);
    assert(SDL_CloseIO(io));
    surface := SDL_CreateSurface(4, 3, SDL_PIXELFORMAT_RGBA32);

    assert(surface != null);
    assert(surface.w == 4 && surface.h == 3);
    assert(SDL_FillSurfaceRect(surface, null, 0));
    SDL_DestroySurface(surface);
    guid := SDL_StringToGUID("00112233445566778899aabbccddeeff".data);
    guid_text: [33] u8;

    SDL_GUIDToString(guid, guid_text.data, 33);
    assert(SDL_strcmp(guid_text.data, "00112233445566778899aabbccddeeff".data) == 0);
    assert(!SDL_SetError("value=%d".data, cast(s32) 23));
    assert(SDL_strcmp(SDL_GetError(), "value=23".data) == 0);
    assert(SDL_ClearError());
    print("SDL3 headless passed\n");
}
