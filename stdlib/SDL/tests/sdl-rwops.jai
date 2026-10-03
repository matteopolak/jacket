// Real SDK-owned native pointers; never call C through public Jai callbacks.
#import "Basic";
S :: #import "SDL";
#load "../../internal/bindings/sdl-native/module.jai";

probe :: #library "sdl_runtime_abi_probe";
native_size :: () -> u64 #foreign probe "binding_runtime_rw_size";
native_hidden :: () -> u64 #foreign probe "binding_runtime_rw_hidden_offset";
track_close :: (source: *RWops) #foreign probe "binding_runtime_track_close";
close_count :: () -> s32 #foreign probe "binding_runtime_close_count";
libc :: #library,system "c";
unlink :: (path: *u8) -> s32 #foreign libc;

main :: () {
    arguments := get_command_line_arguments();

    defer free(arguments.data);

    assert(arguments.count == 2);
    path := to_c_string(arguments[1]);

    defer free(path);
    assert(size_of(S.SDL_RWops) == 48);
    assert(size_of(RWops) == native_size());
    assert(offset_of(RWops, "hidden") == native_hidden());
    // The file factory retains the native SDK symbol and stream identity.
    assert(cast(*void) S.SDL_RWFromFile == cast(*void) rw_from_file);
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

    file := S.SDL_RWFromFile(path, "w+b");

    assert(file != null);
    native := cast(*RWops) file;

    assert(native.write(native, bytes.data, 1, cast(u64) bytes.count) == cast(u64) bytes.count);
    assert(native.size(native) == bytes.count);
    assert(native.seek(native, 0, 0) == 0);
    track_close(native);
    before := close_count();
    surface := S.SDL_LoadBMP_RW(file, 0);

    assert(surface != null && surface.w == 1 && surface.h == 1);
    S.SDL_FreeSurface(surface);
    assert(close_count() == before);
    assert(native.size(native) == bytes.count);
    assert(native.seek(native, 0, 0) == 0);
    copied: [2] u8;

    assert(native.read(native, copied.data, 1, 2) == 2);
    assert(copied[0] == 0x42 && copied[1] == 0x4d);
    assert(native.seek(native, 0, 0) == 0);
    surface = S.SDL_LoadBMP_RW(file, 1);
    assert(surface != null && close_count() == before + 1);
    S.SDL_FreeSurface(surface);

    // Failed decode closes the SDK file exactly when requested, too.
    file = S.SDL_RWFromFile(path, "w+b");
    assert(file != null);
    native = cast(*RWops) file;
    track_close(native);
    before = close_count();
    assert(S.SDL_LoadBMP_RW(file, 0) == null);
    assert(close_count() == before && native.size(native) == 0);
    assert(S.SDL_GetError() != null && S.SDL_GetError()[0] != 0);
    assert(S.SDL_LoadBMP_RW(file, 1) == null);
    assert(close_count() == before + 1);
    file = S.SDL_RWFromFile(path, "rb");
    assert(file != null);
    track_close(cast(*RWops) file);
    before = close_count();
    // Full-width public int flags must reach the native close operation.
    assert(S.SDL_LoadBMP_RW(file, 0x1_0000_0000) == null);
    assert(close_count() == before + 1);
    assert(unlink(path) == 0);
    assert(S.SDL_RWFromFile(path, "rb") == null);
    assert(S.SDL_GetError() != null && S.SDL_GetError()[0] != 0);
}
