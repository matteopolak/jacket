# Ico_File

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `create_ico_file_from_bitmap_filename :: (bitmap_filename: string) -> string`
- `get_bytes_for_image :: (comp: s32, size: s32) -> (image_bytes: s32, mask_bytes: s32, total_bytes: s32)`
- `get_total_bytes_for_size :: (comp: s32, size: s32) -> s32`
- `create_ico_file :: (x: s32, y: s32, comp: s32, image_data: *u8) -> string`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`get_bytes_for_image(comp, size)` treats `size` as the side length of a square image and `comp` as bytes per pixel. It returns `image_bytes = 2 * size * size * comp + size_of(BITMAPINFOHEADER)`, `mask_bytes = (2 * size * size) / 8` using integer division, and `total_bytes = image_bytes + 2 * mask_bytes`. `get_total_bytes_for_size` returns that third value. These are signed 32-bit calculations with no input validation; meaningful inputs are positive sizes and a supported channel count, and overflow behavior is not specified. For example, with the 40-byte Windows `BITMAPINFOHEADER`, `comp=4` and `size=16` produce `(image_bytes=2088, mask_bytes=64, total_bytes=2216)`.

`create_ico_file` emits five square frames, in directory order 256, 128, 48, 32, 16 pixels. Every frame is resized independently from the supplied source image. The ICO directory is a six-byte header followed by five 16-byte entries; each entry records width/height (256 is encoded as zero), zero palette/reserved bytes, one plane, `comp * 8` bits per pixel, the helper's total byte count, and the frame's absolute offset in the file.

Each frame begins with a 40-byte `BITMAPINFOHEADER`: width is the frame side, height is twice that side, planes is 1, bit depth is `comp * 8`, image size is zero, and other header fields are zero. The resized color pixels begin immediately after that header. The file buffer is zero-filled first; the remaining bytes counted for the bitmap/masks stay zero. This produces the helper-defined frame payload length `40 + 2*size*size*comp + 2*floor(size*size/4)`, including padding beyond the resized color pixels. For example, `comp=4`, `size=16` gives a 2216-byte frame. This byte layout is observable even though it differs from the conventional compact DIB size.

`create_ico_file` reads tightly packed `x * comp` byte rows from `image_data`, performs no channel swap or row flip, and returns caller-released storage. The generated DIB has positive height and uses Windows BGR/BGRA channel order; callers supplying direct pixels must provide those channel bytes and bottom-up rows for upright display. Four-channel resizing treats channel 3 as alpha. Meaningful direct input is three or four channels with positive dimensions and a readable tightly packed buffer; the module does not validate these preconditions. Output-allocation failure returns an empty string.

`create_ico_file_from_bitmap_filename` loads through stb_image while requesting four output channels. For three- or four-channel input it changes RGB(A) from stb's top-down order into bottom-up BGR(A), then calls the frame encoder with four channels. It returns a caller-released allocation; load or output-allocation failure returns an empty string, with file-load failure logged. One- and two-channel source images are not safely handled by the supplied conversion and are outside the supported input contract. The implementation assumes little-endian byte order and Windows resource structures, so portability is not guaranteed. It depends on stb_image, stb_image_resize, Basic, Windows_Resources, Windows, and String.
