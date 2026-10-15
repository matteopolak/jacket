// Real SDK-owned native pointers; never call C through public Jai callbacks.
S :: #import "SDL";
#load "../../stdlib/internal/bindings/sdl-native/module.jai";

probe :: #library "../../.work/native/sdl_runtime_abi_probe";
native_size :: () -> u64 #foreign probe "binding_runtime_rw_size";
native_hidden :: () -> u64 #foreign probe "binding_runtime_rw_hidden_offset";
track_close :: (source: *RWops) #foreign probe "binding_runtime_track_close";
close_count :: () -> s32 #foreign probe "binding_runtime_close_count";
libc :: #library,system "c";
abort :: () #foreign libc;
unlink :: (path: *u8) -> s32 #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(size_of(S.SDL_RWops) == 48);
    check(size_of(RWops) == native_size());
    check(offset_of(RWops, "hidden") == native_hidden());
    // Both public entry points are the actual SDK symbols, preserving identity.
    check(cast(*void) S.SDL_RWFromFile == cast(*void) rw_from_file);
    check(cast(*void) S.SDL_LoadBMP_RW == cast(*void) load_bmp_rw);
    bytes: [58] u8;
    // 1x1 bottom-up BGR24 BMP, with native row padding.
    bytes[0] = 0x42;
    bytes[1] = 0x4d;
    bytes[2] = 58;
    bytes[10] = 54;
    bytes[14] = 40;
    bytes[18] = 1;
    bytes[22] = 1;
    bytes[26] = 1;
    bytes[28] = 24;
    bytes[34] = 4;
    bytes[54] = 0x19;
    bytes[55] = 0x37;
    bytes[56] = 0x59;

    path := ".work/native-rwops-test.bmp";
    file := S.SDL_RWFromFile(path.data, "w+b");

    check(file != null);
    native := cast(*RWops) file;

    check(native.write(native, bytes.data, 1, cast(u64) bytes.count) == cast(u64) bytes.count);
    check(native.size(native) == bytes.count);
    check(native.seek(native, 0, 0) == 0);
    track_close(native);
    before := close_count();
    surface := S.SDL_LoadBMP_RW(file, 0);

    check(surface != null && surface.w == 1 && surface.h == 1);
    S.SDL_FreeSurface(surface);
    check(close_count() == before);
    check(native.size(native) == bytes.count);
    check(native.seek(native, 0, 0) == 0);
    copied: [2] u8;

    check(native.read(native, copied.data, 1, 2) == 2);
    check(copied[0] == 0x42 && copied[1] == 0x4d);
    check(native.seek(native, 0, 0) == 0);
    surface = S.SDL_LoadBMP_RW(file, 1);
    check(surface != null && close_count() == before + 1);
    S.SDL_FreeSurface(surface);

    // Failed decode closes the SDK file exactly when requested, too.
    file = S.SDL_RWFromFile(path.data, "w+b");
    check(file != null);
    native = cast(*RWops) file;
    track_close(native);
    before = close_count();
    check(S.SDL_LoadBMP_RW(file, 0) == null);
    check(close_count() == before && native.size(native) == 0);
    check(S.SDL_GetError() != null && S.SDL_GetError()[0] != 0);
    check(S.SDL_LoadBMP_RW(file, 1) == null);
    check(close_count() == before + 1);
    check(unlink(path.data) == 0);
    check(S.SDL_RWFromFile(path.data, "rb") == null);
    check(S.SDL_GetError() != null && S.SDL_GetError()[0] != 0);
}
