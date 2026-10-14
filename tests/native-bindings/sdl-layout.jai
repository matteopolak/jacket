S :: #import "SDL";

probe :: #library "sdl_abi_probe";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

size_matches :: (which: u32, actual: u64) -> s32 #foreign probe "binding_sdl_check_size";
native_offset :: (which: u32) -> u64 #foreign probe "binding_sdl_offset";

main :: () {
    valid := size_matches(0, size_of(S.SDL_version)) != 0;
    if size_matches(1, size_of(S.SDL_Rect)) == 0 valid = false;
    if size_matches(2, size_of(S.SDL_PixelFormat)) == 0 valid = false;
    if size_matches(3, size_of(S.SDL_Surface)) == 0 valid = false;
    if size_matches(4, size_of(S.SDL_Event)) == 0 valid = false;

    format: S.SDL_PixelFormat;
    surface: S.SDL_Surface;
    rw: S.SDL_RWops;

    check((cast(u64) *format.palette) - (cast(u64) *format) == native_offset(0));
    check((cast(u64) *format.refcount) - (cast(u64) *format) == native_offset(1));
    check((cast(u64) *surface.pixels) - (cast(u64) *surface) == native_offset(2));
    check((cast(u64) *surface.clip_rect) - (cast(u64) *surface) == native_offset(3));
    check((cast(u64) *rw.type) - (cast(u64) *rw) == native_offset(4));
    // Check every native record, then fail on any discrepancy.
    if size_matches(5, size_of(S.SDL_RWops)) == 0 valid = false;
    if size_matches(6, size_of(S.SDL_UserEvent)) == 0 valid = false;

    check(valid);
}
