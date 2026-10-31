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
- **Text** (`text.jai`): FreeType glyphs in a 1024x1024 shelf-packed atlas. `load_system_fonts` picks the platform's (override with `JAI_UI_FONT`, `JAI_UI_MONO_FONT`). A font without a face has fixed synthetic metrics, which is what `headless` uses, so layout tests need no font file. No kerning or shaping yet.
- **Text editing** (`text_edit.jai`): `Text_Edit` wraps a [`Text_Buffer`](text-buffer.md) with caret, anchor and scroll; `handle_key` maps keys to it. `text_editor` is the multi-line widget (line numbers, `Editor_Options.highlight`, `marks`) and `text_field` the single-line one.
- **Rendering** (`render.jai`): a display list (`draw.jai`) of rectangles, glyph quads and clips; one instance per entry, one draw per clip. A single WGSL pipeline draws rounded rectangles as signed distances and samples the atlas for glyphs. `render_to_image` and `write_screenshot` draw offscreen.
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
- Gotchas: a widget's own rectangle comes from the previous layout; a `for_expansion` container inside a `for` named `it` needs a different loop name; strings a node keeps must be copied with `frame_string`; the display list is in points and the renderer multiplies by `ui.text.scale`.

Options: `App_Options` (`title`, `width`, `height`, `theme`, `fonts`, `frame_limit`), `Headless_Options`, `Theme`. Needs the wgpu-native library in `native-libs`. Tests: `jacket test --path stdlib/Extensions/UI` (synthetic fonts, no GPU). Local screenshot: `jaic examples/ui-gallery/main.jai -interp -- --screenshot out.png`.
