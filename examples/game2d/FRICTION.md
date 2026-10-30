# Friction log

Rough edges hit while writing game2d. Fixed ones name the commit subject; open ones say so.

## Fixed in the stdlib

- Text was blurry on macOS: the GL surface had no Retina backing and fonts were rasterized at logical size. Fixed by "draw at the window's pixel scale": `get_pixel_scale`, pixel-sized viewport/scissor with logical coordinates, `get_font_at_size(.., pixel_scale)`, `texture_set_filter`.
- Holding a key beeped on macOS: key events were forwarded to `[NSApp sendEvent:]` with nobody to take them. Fixed by "stop key events from beeping on macOS"; Cmd chords still reach the menu. No NSBeep-count test: existing event tests pass, but the absence of the beep was not machine-verified (listen for it).

## Open

- API docs: `Bucket_Array` `for` gives an index, not a locator; `remove;` is the way to delete during iteration, and it is not documented where users look.
- Diagnostics: `cast(u16) x << 8` parses as the cast of the shifted value; the jailint shift-overflow lint caught it, the compiler did not warn.
- LibreSSL's default EC self-signed certificates (explicit parameters) are rejected by the mbedTLS build ("PK - Invalid key tag or value"); the smoke script generates RSA ones. Repro: `openssl ecparam`-less `openssl req -x509 -newkey ec` on macOS, then `--tls-cert`.
- `thread_deinit` requires a finished thread; the failure message does not say how to wait (`thread_is_done(*t, ms)` loop).
- Sound_Player keeps pointers into decoded sound data, so a hot-reloaded sound can never be freed; reloads leak by design.
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
