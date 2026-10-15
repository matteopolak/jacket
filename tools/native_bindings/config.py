"""Native library spelling and private dependency scope for public facades.

These are packaging choices. Public ABI declarations come from the specs.
"""
IMPORTS = {
    'd3d11': ['Windows', 'dxgi', 'd3d_compiler'], 'd3d12': ['Windows', 'dxgi', 'd3d_compiler'],
    'd3d_compiler': ['Windows'], 'dxc_compiler': ['Windows'],
    'dxgi': ['Windows'], 'Linux': ['POSIX'], 'Metal': ['Objective_C'],
    'Objective_C': [], 'macOS': ['POSIX', 'Objective_C'], 'nvidia_aftermath': ['Windows', 'd3d12'],
    'X11': ['POSIX'], 'Android': ['POSIX'],
    'Android/AAudio': [], 'Android/EGL': ['Android'],
    'Vulkan': [],
}

# These facade entry points need independently authored native storage adapters.
# This is an explicit reviewed safety exception to direct foreign forwarding;
# signatures are still matched in full, including argument names and defaults.
NATIVE_ABI_ADAPTERS = {
    ('macOS', 'FSEventStreamCreate'): 'FSEventStreamCreate',
    ('SDL', 'SDL_GetWindowWMInfo'): 'SDL_GetWindowWMInfo',
    ('SDL', 'SDL_LoadBMP_RW'): 'SDL_LoadBMP_RW',
}
NATIVE_SHAPE_DELEGATES = {'POSIX':'android-posix-shapes.json'}
# Explicit private C SDK build profile, not reviewed public Jai option defaults.
RPMALLOC_SDK_BUILD_OPTIONS = {
    'RPMALLOC_FIRST_CLASS_HEAPS': 1, 'RPMALLOC_CONFIGURABLE': 1,
    'ENABLE_STATISTICS': 1, 'ENABLE_ASSERTS': 1,
    'ENABLE_VALIDATE_ARGS': 1, 'ENABLE_OVERRIDE': 0,
}
LIBRARIES = {
    'gdi': 'gdi32', 'gl': 'opengl32',
    'glx': 'GLX', 'libX11': 'X11',
    'xxh_vendor': 'xxhash070', 'lib_stb_vorbis': 'stb_vorbis',
    'liblz4': 'lz4', 'meshopt_lib': 'meshoptimizer', 'libcurl': 'curl',
    'libfreetype': 'freetype', 'freetype': 'freetype', 'libvulkan': 'vulkan',
    'libnvtt': 'nvtt', 'libSystem': 'System', 'libobjc': 'objc',
    'libc': 'c', 'libm': 'm', 'libdl': 'dl', 'libpthread': 'pthread',
    'liblog': 'log', 'libandroid': 'android', 'libaaudio': 'aaudio',
    'libEGL': 'EGL', 'libegl': 'EGL', 'lib_glue':'android_native_app_glue',
    'libmediandk': 'mediandk', 'libcamera2ndk': 'camera2ndk',
}
PARAMETERS = {
    'd3d11': '#module_parameters(INCLUDE_DEBUG_BINDINGS := false)();',
    'd3d12': '#module_parameters(INCLUDE_DEBUG_BINDINGS := false)();',
    'dxc_compiler': '#module_parameters(NINTENDO_SUPPORT := false)();',
    'GL': '#module_parameters(DUMP_GL_ERRORS := false, ENABLE_ALL_EXTENSIONS := false, DEFAULT_MSAA: s32 = 4)();',
    'telemetry3': '#module_parameters(ENABLED := false)();',
}
CXX_RUNTIME = '''#if OS == .MACOS { #library,system,link_always "c++"; }
else #if OS == .LINUX { #library,system,link_always "stdc++"; }'''

PRIVATE = {
    'd3d12': 'Binding_Math :: #import "internal/bindings/graphics-math";',
    'nvtt': '#if OS == .WINDOWS { #import "Windows"; }\n'+CXX_RUNTIME,
    'd3d11': '__MIDL___MIDL_itf_d3d11_0000_0034_0001 :: struct { __bitfield: u32; }',
    'X11': 'wchar_t :: s32;',
    'SDL': '#if OS == .LINUX { #import "X11"; }',
    'ImGui': '#import "Math";\nNative :: #import "internal/bindings/native";\nVarargs :: #import "internal/bindings/varargs";\n'+CXX_RUNTIME,
    'macOS': '''Boolean :: u8;
UTF32Char :: u32;
UTF16Char :: u16;
StringPtr :: *u8;
ConstStringPtr :: *u8;
ConstStr255Param :: *u8;
LangCode :: s16;
RegionCode :: s16;
ScriptCode :: s16;
Block :: *Objective_C_Block_Basic;''',
    'Vulkan': 'Binding_Arrays :: #import "internal/bindings/arrays";\nbinding_abort :: () #foreign libc "abort";\n#if OS == .WINDOWS { #import "Windows"; }',
    'meshoptimizer': CXX_RUNTIME,
    'xxHash': '''binding_XXH_versionNumber :: () -> u32 #foreign xxh_vendor "XXH_versionNumber";
binding_XXH32 :: (input: *void, len: u64, seed: u32) -> u32 #foreign xxh_vendor "XXH32";
binding_XXH64 :: (input: *void, len: u64, seed: u64) -> u64 #foreign xxh_vendor "XXH64";
binding_XXH32_reset :: (state: *XXH32_state_t, seed: u32) -> s32 #foreign xxh_vendor "XXH32_reset";
binding_XXH64_reset :: (state: *XXH64_state_t, seed: u64) -> s32 #foreign xxh_vendor "XXH64_reset";
binding_XXH32_update :: (state: *XXH32_state_t, input: *void, len: u64) -> s32 #foreign xxh_vendor "XXH32_update";
binding_XXH64_update :: (state: *XXH64_state_t, input: *void, len: u64) -> s32 #foreign xxh_vendor "XXH64_update";
binding_XXH32_digest :: (state: *XXH32_state_t) -> u32 #foreign xxh_vendor "XXH32_digest";
binding_XXH64_digest :: (state: *XXH64_state_t) -> u64 #foreign xxh_vendor "XXH64_digest";
binding_abort :: () #foreign libc "abort";''',
    'stb_image': 'FILE :: struct {};',
    'stb_vorbis': 'FILE :: struct {};',
    'Curl': '''FILE :: struct {};
time_t :: s64;
socklen_t :: u32;
SOCKET :: u64;
fd_set :: struct {
    #if OS == .WINDOWS { fd_count: u32; fd_array: [64]u64; }
    else #if OS == .MACOS || OS == .IOS { fds_bits: [32]s32; }
    else { fds_bits: [16]s64; }
}
sockaddr :: struct {
    #if OS == .MACOS || OS == .IOS { sa_len: u8; sa_family: u8; }
    else { sa_family: u16; }
    sa_data: [14]u8;
}
Native :: #import "internal/bindings/native";
Varargs :: #import "internal/bindings/varargs";''',
    'dxc_compiler': 'IMalloc :: struct {};\nIStream :: struct {};\nBSTR :: *u16;',
    'freetype': 'ptrdiff_t :: s64;',
    'Objective_C': '''ptrdiff_t :: s64;
Native :: #import "internal/bindings/native";
binding_objc_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_objc_selector :: (name: *u8) -> *void #foreign libobjc "sel_registerName";
binding_objc_send :: () #foreign libobjc "objc_msgSend";
binding_abort :: () #foreign libc "abort";
binding_strlen :: (text: *u8) -> u64 #foreign libc "strlen";
#if CPU == .X64 { binding_objc_send_stret :: () #foreign libobjc "objc_msgSend_stret"; }
#library,system,link_always "Foundation";
#library,system,link_always "QuartzCore";''',
    'Metal': '''binding_objc_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_objc_selector :: (name: *u8) -> *void #foreign libobjc "sel_registerName";
binding_objc_send :: () #foreign libobjc "objc_msgSend";
#if CPU == .X64 { binding_objc_send_stret :: () #foreign libobjc "objc_msgSend_stret"; }
#library,system,link_always "Metal";
uint64_t :: u64;
uint32_t :: u32;
int32_t :: s32;
int64_t :: s64;
uint16_t :: u16;
uint8_t :: u8;
CGColorSpace :: struct {};
CGColorSpaceRef :: *CGColorSpace;
CFTimeInterval :: float64;
__IOSurface :: struct {};
IOSurfaceRef :: *__IOSurface;
binding_abort :: () #foreign libc "abort";''',
    'GL': '''#if OS == .WINDOWS { #import "Windows"; }
#if OS == .LINUX { #import "X11"; }
#if OS == .MACOS { #import "Objective_C"; #import "macOS"; }
''',
    'POSIX': '''U128 :: [2]u64;
#if OS == .MACOS || OS == .IOS {
    binding_errno_address :: () -> *s32 #foreign libc "__error";
} else #if OS == .ANDROID {
    binding_errno_address :: () -> *s32 #foreign libc "__errno";
} else {
    binding_errno_address :: () -> *s32 #foreign libc "__errno_location";
}''',
}
TARGETS = {
    'GL': 'OS == .WINDOWS || OS == .LINUX || OS == .MACOS',
    'SDL': 'OS == .WINDOWS || OS == .LINUX || OS == .MACOS',
    'Windows': 'OS == .WINDOWS', 'd3d11': 'OS == .WINDOWS',
    'd3d12': 'OS == .WINDOWS', 'd3d_compiler': 'OS == .WINDOWS',
    'dxc_compiler': 'OS == .WINDOWS', 'dxgi': 'OS == .WINDOWS',
    'Linux': 'OS == .LINUX', 'X11': 'OS == .LINUX',
    'POSIX': 'OS == .LINUX || OS == .MACOS || OS == .ANDROID || OS == .IOS',
    'Vulkan': 'OS == .WINDOWS || OS == .LINUX',
    'macOS': 'OS == .MACOS', 'Metal': 'OS == .MACOS || OS == .IOS',
    'Objective_C': 'OS == .MACOS || OS == .IOS',
    'Android': 'OS == .ANDROID', 'Android/AAudio': 'OS == .ANDROID',
    'Android/EGL': 'OS == .ANDROID', 'nvidia_aftermath': 'OS == .WINDOWS',
    'nvtt': 'OS == .WINDOWS || OS == .LINUX || OS == .MACOS',
}
