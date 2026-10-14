#import "Basic";
#import "Extensions/SDL3";
compare :: (a: *void, b: *void) -> s32 #c_call {
    return <<cast(*s32) a - <<cast(*s32) b;
}
filter :: (userdata: *void, event: *SDL_Event) -> bool #c_call {
    if event.type == cast(u32) SDL_EVENT_USER {
        counter := cast(*s32) userdata;
        counter.* += 1;
        event.user.code = 87;
    }
    return true;
}
thread_main :: (userdata: *void) -> s32 #c_call {
    value := cast(*s32) userdata;
    value.* = 42;
    return 17;
}
main :: () {
    assert(SDL_Init(SDL_INIT_EVENTS));
    defer SDL_Quit();
    values: [4] s32 = .[4, 1, 3, 2];
    SDL_qsort(values.data, 4, 4, compare);
    for 0..3 assert(values[it] == it + 1);
    calls: s32;
    SDL_SetEventFilter(filter, *calls);
    event: SDL_Event;
    event.type = cast(u32) SDL_EVENT_USER;
    assert(SDL_PushEvent(*event));
    assert(calls == 1);
    assert(SDL_PollEvent(*event));
    assert(event.user.code == 87);
    SDL_SetEventFilter(null, null);
    value: s32;
    thread := SDL_CreateThread(thread_main, "jai-sdl3".data, *value);
    assert(thread != null);
    status: s32;
    SDL_WaitThread(thread, *status);
    assert(value == 42 && status == 17);
    interface: SDL_StorageInterface;
    SDL_INIT_INTERFACE(*interface);
    assert(interface.version == size_of(SDL_StorageInterface));
    assert(interface.remove == null);
    assert(SDLK_A == 97);
    assert(SDL_WINDOW_MOUSE_CAPTURE == 16384);
    assert(SDL_STANDARD_GRAVITY > 9.8);
    assert(SDL_PROP_GAMEPAD_CAP_RUMBLE_BOOLEAN == SDL_PROP_JOYSTICK_CAP_RUMBLE_BOOLEAN);
    assert(SDL_VERSIONNUM(3, 4, 18) == SDL_VERSION);
    assert(SDL_VERSIONNUM_MINOR(SDL_VERSION) == 4);
    assert(SDL_AUDIO_BYTESIZE(cast(u16) SDL_AUDIO_F32) == 4);
    assert(SDL_BYTESPERPIXEL(cast(u32) SDL_PIXELFORMAT_RGBA32) == 4);
    assert(SDL_MS_TO_NS(12) == 12000000);
    assert(SDL_WINDOWPOS_ISCENTERED(SDL_WINDOWPOS_CENTERED_DISPLAY(2)));
    refs: SDL_AtomicInt;
    refs.value = 1;
    assert(SDL_AtomicIncRef(*refs) == 1);
    assert(!SDL_AtomicDecRef(*refs));
    assert(SDL_AtomicDecRef(*refs));
    ucs2 := SDL_iconv_utf8_ucs2("hello".data);
    assert(ucs2 != null);
    // The public macro requests "UCS-2", which does not promise native byte order.
    // Verify the exact documented expansion without imposing a native-endian assumption.
    expected := SDL_iconv_string("UCS-2".data, "UTF-8".data, "hello".data, 6);
    assert(expected != null);
    assert(SDL_memcmp(ucs2, expected, 12) == 0);
    SDL_free(expected);
    SDL_free(ucs2);
    assert(SDL_Swap32(0x12345678) == 0x78563412);
    rect: SDL_Rect = .{1, 2, 3, 4};
    point: SDL_Point = .{2, 3};
    assert(SDL_PointInRect(*point, *rect));
    print("SDL3 callbacks and macros passed\n");
}
