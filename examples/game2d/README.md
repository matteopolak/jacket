# game2d

A small arena shooter using Simp/OpenGL, Window_Creation, Sound_Player, packed assets,
and an optional HTTP score backend. Build `game/main.jai` with jaic, or run
`bash examples/game2d/tools/smoke_game.sh` from the repository root.

`--help` lists the flags: `--scale N` sizes the window, `--pack FILE` loads a generated
asset pack, `--server URL` enables scores, `--no-audio` disables audio, and
`--autoplay N` exercises the game loop with scripted controls. The smoke script builds
an asset generator and the game, compares regenerated audio bytes and decoded PNG pixels, runs simulation/rendering,
asset reload and optional audio/server checks, then runs autoplay and checks its save.
Use `FRAME=/absolute/frame.png` to keep the offscreen rendering.

## Desktop DPI and CI

`--smoke --expect-pixel-scale 2` requires a real OpenGL window and checks its reported
scale against 2 (tolerance 0.01). Without that flag the smoke prints `pixel_scale: N`
and can skip rendering when a display is unavailable. It also checks that the window
viewport matches its actual framebuffer before switching to the fixed-size offscreen
texture. The texture deliberately remains 640 by 360 at every desktop scale.

```sh
EXPECT_PIXEL_SCALE=2 FRAME=/absolute/frame.png bash examples/game2d/tools/smoke_game.sh
```

Window_Creation chooses per-monitor v2 awareness before creating the first Win32 window,
queries `GetDpiForWindow`, and Input applies `WM_DPICHANGED`'s suggested outer rectangle.
An embedding process that already selected DPI awareness keeps its policy. On X11,
`get_pixel_scale` reads `Xft.dpi` from the current root `RESOURCE_MANAGER` property. Without
a valid setting it estimates DPI from the window's screen width in pixels and millimetres.
Explicit DPI keeps fractional scales (120 → 1.25, 144 → 1.5, 192 → 2); physical estimates
round to the nearest integer, ties up. Values outside 48–768 DPI fall back to 1; scales
never shrink below 1. The resource parser accepts the exact `Xft.dpi` name and a decimal
value; malformed settings are ignored.

Win32/X11 window sizes and input coordinates remain physical pixels, as their native API
contract specifies. Simp therefore uses those sizes directly for viewport and scissor;
Cocoa still converts points to backing pixels. Desktop DPI is available for explicit UI
and font sizing. Input has no scale-change event: query scale each frame (Simp's
`update_window` does this). X11 resource changes are visible on the next query.

The manual **Dogfood GPU** workflow runs this smoke under Xvfb with `Xft.dpi: 192`, requiring
`pixel_scale: 2`, and on Windows's 96 DPI hosted desktop, requiring `pixel_scale: 1`.
Windows uses the pinned Mesa OpenGL DLL copied beside the generated executable via
`OPENGL_DRIVER_DIR`; Linux uses Mesa under Xvfb. Audio and backend checks may skip when
those devices/services are absent; the required scale/render checks cannot skip.
Each job uploads a PNG. Look for zero failed checks, a matching viewport, successful
autoplay/save, and a nonblank shooter frame. A Windows mixed-DPI monitor transition and
live `xrdb` changes still need manual desktop validation; the synthetic Windows rectangle
test does not simulate a monitor change.

## Changing it

Rendering and smoke assertions live in `game/render.jai` and `game/smoke.jai`; flags are
in `game/main.jai`. DPI policy and parsing are private in `stdlib/internal/window/dpi.jai`,
with native queries beside it. Tests are `Window_Creation/tests/dpi.jai`,
`Input/tests/windows-dpi.jai` and `Simp/tests/pixel-scale.jai`. The workflow also runs the
pure DPI rules and Windows suggested-rectangle test. Dependencies are the repo's native
Simp/audio/image/compression libraries, Mesa on software desktops, and an optional backend
under `server/`. See [FRICTION.md](FRICTION.md) for remaining limitations.
