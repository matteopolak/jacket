"""Check real Win32 icon masks and DrawIconEx pixels; native Windows is required."""

import ctypes as C
import os
from pathlib import Path
import struct
import sys

if os.name != "nt":
    raise SystemExit("Native Windows is required; this is not a skipped pass")

U32, I32, HANDLE = C.c_uint32, C.c_int32, C.c_void_p


class BitmapInfo(C.Structure):
    _fields_ = [
        ("size", U32),
        ("width", I32),
        ("height", I32),
        ("planes", C.c_uint16),
        ("bits", C.c_uint16),
        ("compression", U32),
        ("image_size", U32),
        ("xppm", I32),
        ("yppm", I32),
        ("used", U32),
        ("important", U32),
    ]


class IconInfo(C.Structure):
    _fields_ = [
        ("icon", I32),
        ("x", U32),
        ("y", U32),
        ("mask", HANDLE),
        ("color", HANDLE),
    ]


def bind(library, name, result, *arguments):
    """Declare pointer-width-safe Win32 signatures; e.g. bind(gdi, 'DeleteDC', I32, HANDLE)."""
    procedure = getattr(library, name)
    procedure.restype, procedure.argtypes = result, list(arguments)
    return procedure


user, gdi = (
    C.WinDLL("user32", use_last_error=True),
    C.WinDLL("gdi32", use_last_error=True),
)
create_icon = bind(
    user, "CreateIconFromResourceEx", HANDLE, HANDLE, U32, I32, U32, I32, I32, U32
)
get_icon = bind(user, "GetIconInfo", I32, HANDLE, C.POINTER(IconInfo))
destroy_icon = bind(user, "DestroyIcon", I32, HANDLE)
draw = bind(
    user, "DrawIconEx", I32, HANDLE, I32, I32, HANDLE, I32, I32, U32, HANDLE, U32
)
create_dc = bind(gdi, "CreateCompatibleDC", HANDLE, HANDLE)
delete_dc = bind(gdi, "DeleteDC", I32, HANDLE)
create_dib = bind(
    gdi,
    "CreateDIBSection",
    HANDLE,
    HANDLE,
    C.POINTER(BitmapInfo),
    U32,
    C.POINTER(HANDLE),
    HANDLE,
    U32,
)
select = bind(gdi, "SelectObject", HANDLE, HANDLE, HANDLE)
delete_object = bind(gdi, "DeleteObject", I32, HANDLE)
get_bits = bind(
    gdi, "GetDIBits", I32, HANDLE, HANDLE, U32, U32, HANDLE, C.POINTER(BitmapInfo), U32
)
flush = bind(gdi, "GdiFlush", I32)


def require(value):
    """Fail on an unsuccessful native call, preserving its Win32 error."""
    if not value:
        raise C.WinError(C.get_last_error())
    return value


def check_frame(payload, size, label):
    """Verify one DIB frame's mask and compositing over a known opaque background."""
    source = C.create_string_buffer(payload)
    icon, dc = None, None
    info, destination, previous = IconInfo(), None, None
    try:
        icon = require(create_icon(source, len(payload), 1, 0x30000, size, size, 0))
        dc = require(create_dc(None))
        require(get_icon(icon, C.byref(info)))
        mask_stride = ((size + 31) // 32) * 4
        actual_mask = C.create_string_buffer(mask_stride * size)
        # GetDIBits requires a color table for 1-bit DIBs.
        mask_info = C.create_string_buffer(C.sizeof(BitmapInfo) + 8)
        header = BitmapInfo.from_buffer(mask_info)
        header.size, header.width, header.height = 40, size, size
        header.planes, header.bits = 1, 1
        copied = require(
            get_bits(
                dc,
                info.mask,
                0,
                size,
                actual_mask,
                C.cast(mask_info, C.POINTER(BitmapInfo)),
                0,
            )
        )
        assert copied == size, (label, "complete GetDIBits mask rows", copied, size)
        expected_mask = payload[
            40 + size * size * 4 : 40 + size * size * 4 + mask_stride * size
        ]
        for row in range(size):
            for column in range(mask_stride * 8):
                bit = 128 >> (column % 8)
                offset = row * mask_stride + column // 8
                if column < size:
                    alpha = payload[40 + (row * size + column) * 4 + 3]
                    assert bool(expected_mask[offset] & bit) == (alpha == 0), (
                        label,
                        "file mask follows alpha",
                        row,
                        column,
                    )
                    assert (actual_mask.raw[offset] & bit) == (
                        expected_mask[offset] & bit
                    ), (label, "GetIconInfo AND mask", row, column)
                else:
                    # File padding is specified; GDI's in-memory padding is not pixel data.
                    assert expected_mask[offset] & bit == 0, (
                        label,
                        "file mask row padding",
                    )
        bitmap = BitmapInfo(40, size, -size, 1, 32)
        pointer = HANDLE()
        destination = require(
            create_dib(dc, C.byref(bitmap), 0, C.byref(pointer), None, 0)
        )
        require(pointer.value)
        previous = require(select(dc, destination))
        background = (47, 83, 131)
        C.memmove(pointer, bytes((*background, 255)) * (size * size), size * size * 4)
        require(draw(dc, 0, 0, icon, size, size, 0, None, 3))
        require(flush())
        rendered = C.string_at(pointer, size * size * 4)
        for y in range(size):
            for x in range(size):
                pixel = payload[40 + ((size - y - 1) * size + x) * 4 :][:4]
                alpha = pixel[3]
                for channel in range(3):
                    expected = round(
                        (pixel[channel] * alpha + background[channel] * (255 - alpha))
                        / 255
                    )
                    observed = rendered[(y * size + x) * 4 + channel]
                    assert abs(observed - expected) <= 1, (
                        label,
                        x,
                        y,
                        channel,
                        pixel.hex(),
                        observed,
                        expected,
                    )
    finally:
        if previous:
            select(dc, previous)
        if destination:
            delete_object(destination)
        if info.color:
            delete_object(info.color)
        if info.mask:
            delete_object(info.mask)
        if dc:
            delete_dc(dc)
        if icon:
            destroy_icon(icon)


root = Path(sys.argv[1])
frames = 0
for name in ("clear", "mixed", "opaque", "faint"):
    source = (root / f"gdi-{name}.ico").read_bytes()
    assert struct.unpack_from("<HHH", source) == (0, 1, 5)
    for index, size in enumerate((256, 128, 48, 32, 16)):
        length, offset = struct.unpack_from("<II", source, 14 + 16 * index)
        check_frame(source[offset : offset + length], size, f"{name}/{size}")
        frames += 1
print(f"native Win32 GetIconInfo/DrawIconEx: {frames} icon frames passed")
