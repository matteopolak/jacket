# UI (`Extensions/UI`)

> [!NOTE]
> UI is a jaic extension. It is not part of the reference standard library, no other Jai compiler has it, and it may change between jaic releases.

## What it is

A native user interface library: a window, our own rendering on [WebGPU](webgpu.md), FreeType text, input routing, layout and a retained widget tree that app code declares with plain calls every frame. The [text editor](ui-editor.md) is its first user, and the library grows only as far as the editor needs.

```jai
#import "Basic";
#import "Extensions/UI";

App :: struct { count: int; }

main :: () {
    app: App;
    run_app(.{title = "Counter", width = 320, height = 200}, *app, build);
}

build :: (app: *App) {
    for column(.{gap = 8, padding = 16}) {
        label(tprint("Clicked % times", app.count));
        if button("Click me") app.count += 1;
    }
}
```

## How it works

```
build (your code) --declares--> widget tree (retained, keyed) --layout--> rectangles
                                                                           |
platform events --> Input_Event queue --hit test on last layout--> hover/press/focus/scroll
                                                                           |
                                      display list <--paint-- tree  -->  Renderer (WebGPU) --> window or PNG
```

- **Frame.** `frame(ui, app, build)` is `process_input`, `begin_frame`, your `build`, `end_frame` (sweep unseen nodes, layout, paint). `run_app` runs a frame only when an event, a request (`request_frame`, `wake_in`) or a resize needs one, so an idle window sleeps. A frame that had queued input builds twice: input is applied while `build` runs, so widgets declared above the one that reacted would otherwise show the old state for a frame. The second build sees the input spent (no clicks, keys or focus changes), so a handler runs once. A test sends input and calls `frame` once to see the result; a click needs a frame before it, since it aims at the previous layout.
- **Keys.** A node's id is its call site, its parent and how many times that call site ran under the parent this frame, so loops work without keys. Give `key(id)` before a widget in a list whose order changes. The tree keeps scroll offsets, focus and per-node `state(node, T)` slots by id.
- **Containers are `for` blocks.** `for row(.{gap = 8}) { ... }` opens the row, runs the body once and closes the row, also on `break`. It is a `for_expansion` on the `Container` value `row` returns; the body is plain code in the caller's scope. `split` runs its body twice (`for pane: split(.HORIZONTAL, *width)`, `pane` is 0 then 1).
- **Widgets return what happened:** `button` returns clicked, `checkbox(*bool)` returns changed, `text_editor(*edit)` returns changed. No callbacks. The app owns its data; the library owns widget state.
- **Layout** (`layout.jai`): measure bottom up (text from the font metrics), arrange top down. Rows and columns share leftover space by `grow`, place children with `align` (across) and `justify` (along); `scroll` containers lay out their content in a larger space shifted by the scroll offset and clip to their rectangle. Sizes are points.
- **Input.** `send_mouse_move/down/up`, `send_scroll`, `send_key`, `send_text` queue events; `run_app` translates the `Input` module's events, tests call them directly. Hit testing uses the previous frame's rectangles, so a click reaches a widget one frame after it was drawn, as in other immediate-style libraries. Focus follows clicks to the nearest `focusable` node. Keyboard and typed text reach the focused widget as `ui.input.keys`.
- **Text** (`text.jai`): FreeType rasterizes glyphs at `size * scale` pixels into a one-channel atlas (1024x1024, shelf packed, cleared and refilled if full). The slot fonts are `SANS` and `MONO`, then fallbacks for missing glyphs; `load_system_fonts` picks the platform's (override with `JAI_UI_FONT` and `JAI_UI_MONO_FONT`). A font without a face has fixed synthetic metrics (0.6 of the size per character), which is what `headless` uses, so layout tests need no font file. There is no kerning or shaping yet; cursor movement uses `Extensions/Text_Buffer`'s grapheme and word boundaries.
- **Text editing** (`text_edit.jai`): `Text_Edit` is the model (a [`Text_Buffer`](text-buffer.md), caret, anchor, scroll); `handle_key` maps keys to it (arrows, Home/End, word and line movement, shift to extend, Tab, Enter with auto-indent, Cmd/Ctrl+A/C/X/V/Z/Y). `text_editor` is the multi-line widget (line numbers, highlighting through `Editor_Options.highlight`, `marks` for search results and diagnostics, only the visible lines are drawn) and `text_field` the single-line one.
- **Rendering** (`render.jai`): the display list (`draw.jai`) holds rectangles, glyph quads and clip changes. The renderer makes one instance per entry and one draw per clip rectangle (scissor). A single WGSL pipeline evaluates rounded rectangles and borders as signed distances and samples the atlas for glyphs. `present` draws into a window surface; `render_to_image` and `write_screenshot` draw offscreen and read the pixels back.
- **Roles, labels and values** are kept on every node for tests and a later accessibility export; v1 does not export one.

Testing a build procedure needs no window or GPU:

```jai
ui := headless(.{width = 320, height = 200});
defer free_ui(*ui);

frame(*ui, *app, build);
click(*ui, find_label(*ui, "Click me"));
frame(*ui, *app, build);
expect_equal(app.count, 1);
expect_snapshot(t, layout_dump(*ui));   // the tree with roles, text and rectangles
```

## How to change it

- **A new widget** calls `widget(kind, role, style, loc)` (or `begin_container`/`end_container`) with `#caller_location`, sets what it needs on the node (`pointer`, `focusable`, `text`, `paint`), reads `events_of(ui, node)` and returns what happened. A `.LEAF` node paints through its `paint` procedure and `paint_data`, which must outlive the frame (`frame_allocator(ui)`).
- **Style defaults** live in `Theme` (`theme.jai`); a widget reads `ui.theme` when it builds.
- **A new draw primitive** adds a `Draw_Kind`, a case in `encode` and the shader in `render.jai`.
- **Zero-size warning.** A focusable widget that ends layout with no width or height (a `text_editor` in a column nobody gave `grow`) is reported once through `context.logger` as a warning that names its role, label and `key`. The check is on unless the module is imported with `DEBUG_WARNINGS = false`.
- Gotchas: the hit test and a widget's own rectangle come from the previous layout; `for_expansion` containers inside a `for` named `it` need a different loop name; strings a node keeps must be copied with `frame_string`; the display list is in points and the renderer multiplies by `ui.text.scale`.

## Configuration

The module parameter `DEBUG_WARNINGS` (default `true`; `#import "Extensions/UI"(DEBUG_WARNINGS = false)` turns the layout warnings and the key text they need off). `App_Options` (`title`, `width`, `height`, `theme`, `fonts`, `frame_limit`), `Headless_Options` (size, `scale`, `system_fonts`, `theme`), `Theme`, the environment variables `JAI_UI_FONT` and `JAI_UI_MONO_FONT`.

## Dependencies

[`Extensions/WebGPU`](webgpu.md) (needs the wgpu-native library in `native-libs`), `Window_Creation` and `Input`, `freetype`, `Clipboard`, [`Extensions/Text_Buffer`](text-buffer.md), `Extensions/Image` (PNG screenshots), `Extensions/Environment`, `Hash_Table`, `Flat_Pool`. Native only: no browser or wasm target yet.

Tests: `stdlib/Extensions/UI/tests/` (`layout.jai`, `editing.jai`), run by `jacket test --path stdlib/Extensions/UI`; they use synthetic fonts and no GPU. Screenshots are local only: `jaic examples/ui-gallery/main.jai -interp -- --screenshot out.png`.
