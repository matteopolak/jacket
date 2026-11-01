# Friction log

Rough edges hit while writing game2d. Fixed ones name the commit subject; open ones say so.

## Fixed in the stdlib

- Text was blurry on macOS: the GL surface had no Retina backing and fonts were rasterized at logical size. Fixed by "draw at the window's pixel scale": `get_pixel_scale`, pixel-sized viewport/scissor with logical coordinates, `get_font_at_size(.., pixel_scale)`, `texture_set_filter`.
- Holding a key beeped on macOS: key events were forwarded to `[NSApp sendEvent:]` with nobody to take them. Fixed by "stop key events from beeping on macOS"; Cmd chords still reach the menu. No NSBeep-count test: existing event tests pass, but the absence of the beep was not machine-verified (listen for it).

- Windows and Linux always reported desktop scale 1. Fixed by "fix(window): query desktop DPI and handle Windows DPI changes": per-monitor v2, window DPI queries, the suggested resize rectangle, Xft.dpi parsing and physical-screen fallback. Tests cover fractional scale/rounding and parser failures, plus the Windows rectangle message.
- Simp multiplied Win32/X11's already physical dimensions by desktop scale. Fixed by "fix(simp): keep native pixel dimensions at desktop DPI": only Cocoa converts points to pixels, including viewport and scissor; the pixel-scale regression distinguishes the native size contracts.

## Fixed later

- PNG byte comparisons rejected freshly generated art with identical decoded scanlines after encoder changes. Fixed by "feat(crossos): run dogfood graphics on software CI adapters": audio still compares exact bytes; PNGs compare dimensions and RGBA pixels in the actual smoke session.

- Fixed by "feat(sound-player): add release_sound_data and replace_sound_data": a hot-reloaded sound used to leak because streams point into its data; `replace_sound_data` stops those streams, frees the old bytes and keeps the `Sound_Data` pointer, and the game uses it (and restarts its music).
- Fixed by "docs(bucket-array): explain the loop index and remove": the module page and `for_expansion` say the second `for` name is an index, not a locator, and show `remove value;` in running examples.
- Fixed by "fix(thread): say how to wait in the thread_deinit failure": `thread_deinit` on a running thread now tells the caller to wait with `thread_is_done(*thread, -1)` first.
- Fixed by "fix(tls): read EC certificates and keys that spell out their curve": LibreSSL's default EC self-signed certificates and PKCS#8 keys write the curve parameters in full, which the mbedTLS build rejected ("PK - Invalid key tag or value"); they now load, as do keys with a leading `EC PARAMETERS` block. The smoke script may use EC certificates again.

## Open

- Input has no scale-change event. Scale is queried each frame; the existing resize records cover Windows suggested-rectangle resizing. Live mixed-DPI Windows monitor moves and X11 resource changes still need desktop validation.

- Diagnostics: `cast(u16) x << 8` parses as the cast of the shifted value; the jailint shift-overflow lint caught it, the compiler did not warn.
- Compile time: the game (about all of Simp, Sound_Player, Zip, Deflate) takes noticeably longer than the server; no incremental build.
- IDE: no cross-module rename across `#import,dir` project boundaries was tried; not evaluated.

## Coverage

| Area | Used | Not used |
|---|---|---|
| Window/render | Simp, Window_Creation, Input, Gamepad | SDL3, WebGPU (by design) |
| Audio | Sound_Player, stb_vorbis, Wav_File | |
| Files | File_Watcher, Zip_File_Directory, lz4, xxHash, Deflate | |
| Data | PCG, Bucket_Array, Pool | |
| Server | Http, Postgres, Tls, Io, Json, Args | HTTP/2 |
