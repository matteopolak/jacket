#import "Basic";
S :: #import "SDL";
Native :: #import "internal/bindings/sdl-native";

probe :: #library "sdl_abi_probe";
size_matches :: (which: u32, actual: u64) -> s32 #foreign probe "binding_sdl_check_size";
native_offset :: (which: u32) -> u64 #foreign probe "binding_sdl_offset";
rw_offset :: (which: u32) -> u64 #foreign probe "binding_sdl_rw_offset";
user_offset :: (which: u32) -> u64 #foreign probe "binding_sdl_user_offset";
native_alignment :: (which: u32) -> u64 #foreign probe "binding_sdl_alignment";
fill_user :: (event: *Native.User_Event, data1: *void, data2: *void) #foreign probe "binding_sdl_fill_native_user";

main :: () {
    assert(size_matches(0, size_of(S.SDL_version)) != 0);
    assert(size_matches(1, size_of(S.SDL_Rect)) != 0);
    assert(size_matches(2, size_of(S.SDL_PixelFormat)) != 0);
    assert(size_matches(3, size_of(S.SDL_Surface)) != 0);
    assert(size_matches(4, size_of(S.SDL_Event)) != 0);
    assert(offset_of(S.SDL_PixelFormat, "palette") == native_offset(0));
    assert(offset_of(S.SDL_PixelFormat, "refcount") == native_offset(1));
    assert(offset_of(S.SDL_Surface, "pixels") == native_offset(2));
    assert(offset_of(S.SDL_Surface, "clip_rect") == native_offset(3));
    assert(offset_of(S.SDL_RWops, "type") == native_offset(4));
    // Public prefixes retain the reviewed layout. C receives complete storage.
    assert(size_of(S.SDL_RWops) == 48 && align_of(S.SDL_RWops) == 8);
    assert(size_of(S.SDL_UserEvent) == 8 && align_of(S.SDL_UserEvent) == 4);
    assert(offset_of(S.SDL_UserEvent, "type") == 0 && offset_of(S.SDL_UserEvent, "timestamp") == 4);
    assert(size_matches(5, size_of(Native.RWops)) != 0);
    assert(size_matches(6, size_of(Native.User_Event)) != 0);
    assert(align_of(Native.RWops) == native_alignment(0));
    assert(align_of(Native.User_Event) == native_alignment(1));
    rw_fields := u64.[offset_of(Native.RWops, "size"), offset_of(Native.RWops, "seek"),
        offset_of(Native.RWops, "read"), offset_of(Native.RWops, "write"),
        offset_of(Native.RWops, "close"), offset_of(Native.RWops, "type"), offset_of(Native.RWops, "hidden")];
    public_fields := u64.[offset_of(S.SDL_RWops, "size"), offset_of(S.SDL_RWops, "seek"),
        offset_of(S.SDL_RWops, "read"), offset_of(S.SDL_RWops, "write"),
        offset_of(S.SDL_RWops, "close"), offset_of(S.SDL_RWops, "type")];

    for offset, index: rw_fields assert(offset == rw_offset(cast(u32) index));
    for offset, index: public_fields assert(offset == rw_fields[index]);

    user_fields := u64.[offset_of(Native.User_Event, "type"), offset_of(Native.User_Event, "timestamp"),
        offset_of(Native.User_Event, "windowID"), offset_of(Native.User_Event, "code"),
        offset_of(Native.User_Event, "data1"), offset_of(Native.User_Event, "data2")];
    for offset, index: user_fields assert(offset == user_offset(cast(u32) index));

    guarded: struct {
        before: [8] u8;
        event: Native.User_Event;
        after: [8] u8;
    };

    for *byte: guarded.before byte.* = 0x3a;
    for *byte: guarded.after byte.* = 0xc7;

    first, second: s64;

    fill_user(*guarded.event, *first, *second);
    assert(guarded.event.type == cast(u32) S.SDL_USEREVENT);
    assert(guarded.event.timestamp == 0x89abcdef && guarded.event.windowID == 19);
    assert(guarded.event.code == -42 && guarded.event.data1 == *first && guarded.event.data2 == *second);
    prefix := cast(*S.SDL_UserEvent) *guarded.event;

    assert(prefix.type == .SDL_USEREVENT && prefix.timestamp == guarded.event.timestamp);
    for byte: guarded.before assert(byte == 0x3a);
    for byte: guarded.after assert(byte == 0xc7);
}
