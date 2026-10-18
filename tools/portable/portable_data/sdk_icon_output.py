"""Native Win32 SDK readback of the unchanged compiler's +Icon output."""
import ctypes as C
from pathlib import Path
import os
import struct
import sys

if os.name != "nt":
    raise SystemExit("Native Windows is required; this is not a skipped pass")
binary, icon_path = map(Path, sys.argv[1:3])
kernel = C.WinDLL("kernel32", use_last_error=True)
kernel.LoadLibraryExW.argtypes = [C.c_wchar_p, C.c_void_p, C.c_uint32]
kernel.LoadLibraryExW.restype = C.c_void_p
kernel.FindResourceExW.argtypes = [C.c_void_p, C.c_void_p, C.c_void_p, C.c_uint16]
kernel.FindResourceExW.restype = C.c_void_p
kernel.SizeofResource.argtypes = [C.c_void_p, C.c_void_p]
kernel.SizeofResource.restype = C.c_uint32
kernel.LoadResource.argtypes = [C.c_void_p, C.c_void_p]
kernel.LoadResource.restype = C.c_void_p
kernel.LockResource.argtypes = [C.c_void_p]
kernel.LockResource.restype = C.c_void_p
kernel.FreeLibrary.argtypes = [C.c_void_p]
kernel.FreeLibrary.restype = C.c_int32

module = kernel.LoadLibraryExW(str(binary.resolve()), None, 2)
if not module:
    raise C.WinError(C.get_last_error())

def resource(kind, identifier):
    entry = kernel.FindResourceExW(module, kind, identifier, 1033)
    if not entry:
        raise C.WinError(C.get_last_error())
    size = kernel.SizeofResource(module, entry)
    loaded = kernel.LoadResource(module, entry)
    pointer = kernel.LockResource(loaded) if loaded else None
    if not size or not pointer:
        raise C.WinError(C.get_last_error())
    return C.string_at(pointer, size)

try:
    source = icon_path.read_bytes()
    assert struct.unpack_from("<HHH", source) == (0, 1, 5)
    group = resource(14, 2)
    assert group[:6] == source[:6] and len(group) == 76
    for index in range(5):
        assert group[6 + index * 14:18 + index * 14] == source[6 + index * 16:18 + index * 16]
        image_id, = struct.unpack_from("<H", group, 18 + index * 14)
        assert image_id == 2 + index
        size, offset = struct.unpack_from("<II", source, 14 + index * 16)
        assert resource(3, image_id) == source[offset:offset + size]
finally:
    if not kernel.FreeLibrary(module):
        raise C.WinError(C.get_last_error())
print("native Win32 SDK verifies real +Icon executable resources")
