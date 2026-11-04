# UI

A native user interface library: a window, our own rendering on [WebGPU](webgpu.md), FreeType text, input routing, layout and a retained widget tree that app code declares with plain calls every frame. The [text editor](ui-editor.md) is its first user, and the library grows only as far as the editor needs. Native only.

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

- **Frame.** `frame(ui, app, build)` is `process_input`, `begin_frame`, your `build`, `end_frame` (sweep unseen nodes, layout, paint). `run_app` runs a frame only when an event, a request (`request_frame`, `wake_in`) or a resize needs one, so an idle window sleeps. A frame with queued input builds twice, so widgets above the one that reacted don't show stale state; the second build sees the input spent.
- **Keys.** A node's id is its call site, its parent and how many times that site ran under the parent this frame, so loops work without keys. Call `key(id)` before a widget in a list whose order changes. Scroll offsets, focus and per-node `state(node, T)` live in the tree by id.
- **Containers are `for` blocks.** `for row(.{gap = 8}) { ... }` is a `for_expansion` on the value `row` returns; the body is plain code in the caller's scope. `split` runs its body twice.
- **Widgets return what happened** (`button` returns clicked, `checkbox(*bool)` changed). No callbacks. The app owns its data; the library owns widget state.
- **Layout** (`layout.jai`): measure bottom up, arrange top down. Rows and columns share leftover space by `grow` and place children with `align` and `justify`; `scroll` containers clip. Sizes are points.
- **Input.** `send_mouse_*`, `send_scroll`, `send_key`, `send_text` queue events. Hit testing uses the previous frame's rectangles, so a click reaches a widget one frame after it was drawn. Focus follows clicks to the nearest `focusable` node.
- **Text** (`text.jai`): FreeType glyphs in a 1024x1024 shelf-packed atlas. `load_system_fonts` picks the platform's (override with `JAI_UI_FONT`, `JAI_UI_MONO_FONT`). A font without a face has fixed synthetic metrics, which is what `headless` uses, so layout tests need no font file. The SANS slot applies FreeType pair kerning (`pair_kerning`, cached per pixel size and pair); measuring, drawing, selection and hit testing all use it. MONO is never kerned. No shaping or bidirectional text.
- **Text editing** (`text_edit.jai`): `Text_Edit` wraps a [`Text_Buffer`](text-buffer.md) with caret, anchor and scroll; `handle_key` maps keys to it. `text_editor` is the multi-line widget (line numbers, `Editor_Options.highlight`, `marks`) and `text_field` the single-line one.
- **Rendering** (`render.jai`): a display list (`draw.jai`) of rectangles, glyph quads and clips; one instance per entry, one draw per clip. A single WGSL pipeline draws rounded rectangles as signed distances and samples the atlas for glyphs. `render_to_image` and `write_screenshot` draw offscreen. `draw_image(ui, rect, image)` stretches borrowed straight-alpha RGBA8 pixels into a rectangle; keep them alive until presentation or offscreen readback completes.
- **Native menus and IME** (with [Desktop](desktop.md)). `App_Options.menus` installs a native menu; a pick arrives as the item's shortcut key, or a `COMMAND` event carrying `action` when it has none (`send_menu_action` does the same in tests). Composition arrives as `COMPOSITION` events (`send_composition`). `Text_Edit` shows marked text over the caret without touching the buffer, so dirty state and undo change only on commit, which replaces the selection as one undo step; cancel or focus loss drops it. The focused widget sets `ui.ime_active` and `ui.ime_rect` for the candidate window.
- Roles, labels and values are kept on every node for tests and a future accessibility export.

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

- New widget: call `widget(kind, role, style, loc)` (or `begin_container`/`end_container`) with `#caller_location`, set what it needs on the node (`pointer`, `focusable`, `text`, `paint`), read `events_of(ui, node)` and return what happened. A `.LEAF` node's `paint_data` must outlive the frame (`frame_allocator(ui)`).
- Style defaults live in `Theme` (`theme.jai`).
- New draw primitive: a `Draw_Kind`, a case in `encode`, and the shader in `render.jai`.
- A focusable widget that ends layout with zero size is reported once as a warning naming its role, label and key. `#import "Extensions/UI"(DEBUG_WARNINGS = false)` turns that off.
- Text layout changes must keep width, byte offsets, hit testing and drawing in step; test them with a real font, since synthetic metrics have no kerning.
- Gotchas: a widget's own rectangle comes from the previous layout; a `for_expansion` container inside a `for` named `it` needs a different loop name; strings a node keeps must be copied with `frame_string`; the display list is in points and the renderer multiplies by `ui.text.scale`.

Options: `App_Options` (`title`, `width`, `height`, `theme`, `fonts`, `frame_limit`, `menus`), `Headless_Options`, `Theme`. Needs the wgpu-native library in `native-libs`. Tests: `jacket test --path stdlib/Extensions/UI` (synthetic fonts, no GPU). Local screenshot: `jaic examples/ui-gallery/main.jai -interp -- --screenshot out.png`.

## Image rendering checks

Images use nearest filtering and clamp-to-edge sampling. Destinations and clips are in logical points, multiplied by `ui.text.scale`; scissors floor the starting boundary and ceil the ending boundary. Uploaded images and offscreen targets use `RGBA8Unorm`, without sRGB conversion. The shader premultiplies straight-alpha source colors and blends source-over. With a transparent-black clear, readback RGB is premultiplied. With an opaque clear, alpha stays opaque. `Rendered_Image.pixels` holds packed RGBA rows from top to bottom; release it with `array_free`.

`create_offscreen_renderer(width, height, adapter_options = null)` accepts optional `WGPURequestAdapterOptions`, including `forceFallbackAdapter`. Null uses default selection, and setup failure returns null. The driven test preflights the same options: only adapter absence exits 77; device and setup failures fail the test. It awaits a validation error scope around each render/readback and captures unexpected GPU error logs for the renderer lifetime.

```sh
PYTHONPATH=tools python3 tools/bindings/run_webgpu_native.py --jaic target/debug/jaic --test ui-images --require-adapter
```

This runs the production upload, shader, blending and readback in interpreted and compiled modes. Fixtures cover orientation, RGBA order, stretching, transparent and translucent pixels, draw order across solid batches, scale, clipping and changed caller pixels across frames. Native FreeType and wgpu-native libraries are required; synthetic fonts need no system font files. Extend `tests/driven/images.jai` when changing rendering semantics; keep borrowed-buffer and argument checks in ordinary `tests/images.jai`.
