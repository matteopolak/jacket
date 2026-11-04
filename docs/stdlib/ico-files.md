# ICO file creation

`Ico_File` creates ICO bytes from BGR/BGRA pixels or an image filename. Each output contains 256, 128, 48, 32 and 16 pixel entries. Free the returned `string` with `Basic.free`. On error, it logs and returns an empty string.

## Decoder boundary and ownership

`internal/resource-wire/ico-load.jai` owns file reading and decoder selection. Native targets retain stb decoding, including native-only formats such as uncompressed TGA. Wasm uses `Extensions/Image.decode_image`: PNG, JPEG, QOI, BMP and PNM. There is no fallback decoder across this target boundary. RGB/RGBA decoded formats are accepted; grayscale and grayscale-alpha outputs are rejected. Palette expansion and JPEG color conversion follow each decoder's output format, so source encoding alone does not determine channel acceptance.

Both paths convert top-down straight RGBA to bottom-up BGRA before the existing encoder runs. File bytes, decoded pixels and conversion buffers remain owned until their consumer completes and are then released. The caller owns only the resulting ICO. Resizing starts from the original pixels for each entry, averages sRGB colors in linear light weighted by alpha, and stores straight BGRA. Colors whose stored alpha is zero become black, including faint colors that quantize to zero while shrinking and transparent pixels at the source size. The bottom-up AND mask marks exactly the resized pixels whose alpha is zero, with each mask row padded to four bytes. Windows needs that mask for fully transparent icons because an all-zero alpha plane invokes legacy icon handling. The ICO wire layout reserves two pixel planes and two mask planes per entry; preserve that layout when changing the encoder.

## Limits and configuration

Both loaders reject encoded files larger than `0x7fffffff` bytes. Wasm additionally uses `Image.DEFAULT_MAX_PIXELS` (`1 << 26`) before allocating decoded pixels. Native retains its decoder's dimension checks and the bridge's integer-overflow checks. There are no environment options that change decoder routing. `JAIC_STDLIB` selects this checkout's stdlib; native execution requires stb in `JAIC_NATIVE_LIBS`.

## Run the filename regression

```sh
PYTHONPATH=tools python3 tools/portable/portable_data/wire_consumer.py /tmp/ico-test fixtures
jaic -interp stdlib/Ico_File/tests/filename.jai -- /tmp/ico-test /tmp/ico-test
jaic -os wasm stdlib/Ico_File/tests/filename.jai -exe /tmp/ico-test/filename.wasm
node --no-warnings tools/wasi_run.mjs /tmp/ico-test/filename.wasm /tmp/ico-test /tmp/ico-test
```

The two arguments are the input fixture folder and writable output folder. The WASI run requires Node 24 or newer and a filesystem host. It does not provide browser access to native paths. The portable-data suite requires the filename, ICO wire and resource-directory WASI cases as well as native interpreted, O0 and O2 execution. Its fixture generator constructs independent PNG/BMP/TGA data; assertions inspect directory offsets and same-size BGRA orientation/alpha without decoding the resulting ICO through our own decoder.

## How to change it

Keep reusable decoders in `Extensions/Image`; keep filename acquisition and format bridging private in `ico-load.jai`. Extend the target's fixture matrix before changing accepted formats. `ico-encode.jai` owns entry layout and `ico-resize.jai` owns filtering; `Image.resize` has different alpha and color semantics and is not a substitute. Changes to conversion or ownership need native and WASI filename regressions plus the Image malformed-input and ICO/resize tests. On Windows, the portable-data suite also consumes clear, mixed-alpha, opaque and faint output through `CreateIconFromResourceEx`, reads the AND mask with `GetIconInfo`, and checks actual `DrawIconEx` compositing for all five sizes. Dependencies are `Basic`, `File`, the encoder/resizer, native stb or portable `Extensions/Image`.
