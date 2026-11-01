# Imageview

A small folder image browser built on `Extensions/UI`. It shows pages of 24 thumbnails,
loads and scales missing visible tiles on up to three `Thread_Group` workers, and retains them
in a byte-capped LRU cache. The selected full image is separate from that cache.

```sh
jaic examples/imageview/main.jai -exe "$T/imageview"
"$T/imageview" pictures/
"$T/imageview" pictures/photo.jpg --cache-mb 16
(cd examples/imageview && jacket test -j 2)
sh examples/imageview/tools/smoke.sh
```

Set `JAIC_STDLIB` to the repository's `stdlib`, put `jaic` and `jacket` in `PATH`, and set
`JAIC_NATIVE_LIBS` if the compiler cannot locate native libraries. Set `T` to the desired
build/smoke output directory (this task uses `/Volumes/LodestoneScratch/jai-targets/codex-imageview`).
The viewer needs the platform window, WebGPU, FreeType, and stb image/resize/write libraries.
`--help` describes the CLI; `--cache-mb` accepts 1–512 MiB, default 32.

## Using it

Enter a file or folder in the top field and click **Open**. File paths open that file immediately;
folder paths show the thumbnail grid. The directory is scanned without recursion. Grid pages
have six columns and four rows. Click an image to open it. **Grid** returns to its page;
**Page back/forward** pages the grid. Bad files show an empty tile and an error when opened.

**Fit** fits both dimensions, **1:1** uses one image pixel per logical UI point, and the wheel
zooms about the cursor. Drag the image to pan. **Rotate** turns it clockwise, keeping the change
in memory. **Previous/Next** navigate in filename order. After clicking the canvas, arrow keys
navigate, `F` fits, `1` selects 1:1, `R` rotates, and Escape returns to the grid.
The details line shows dimensions, detected format, source bits per channel, decoded layout,
file size, and frame count. Decoding reduces samples to RGBA8. A checkerboard shows alpha.
GIFs animate using their frame delays (minimum 10 ms); **Pause GIF** toggles playback. Rotation
freezes an animated GIF at its current frame.

Enter an output path ending in `.png`, `.jpg`, or `.jpeg` in the second field and click
**Export as**. Export saves the current rotated frame; PNG retains alpha, JPEG uses quality 95
and drops alpha. Existing output files are overwritten. The source is only changed if chosen
as the export destination.

## How it works and how to change it

`main.jai` owns flags and the app lifetime; `app.jai` owns UI state, input, animation and painting.
`viewer/images.jai` dispatches codecs, scales thumbnails and exports. PNG/JPEG/BMP/QOI/PNM use
`Extensions/Image`; GIF/TGA use `stb_image`; icons use `Extensions/Image.decode_ico`. File signatures
are used where available; TGA is inferred from the suffix. Each worker owns its decode result,
and the UI thread admits completed thumbnails only after joining the batch. No worker modifies
UI state. `viewer/cache.jai` owns cache accounting, eviction, sorted listing and job scheduling;
`viewer/zoom.jai` holds cursor anchoring and fit math.

To add a format, update detection, folder suffixes and fixture tests together. Keep ownership
explicit: `Picture` and `Cache` own their buffers, `draw_image` borrows until the render finishes.
The cache cap covers retained thumbnail pixels; transient worker decodes, the selected image,
GIF frame storage, paths and GPU uploads are additional memory. Image decoders reject more than
`Extensions/Image.DEFAULT_MAX_PIXELS` pixels per image. Native GIF animation may consume much more
memory than one frame; use small trusted animations.

`fixtures.jai` generates gradients, checker patterns, alpha, a two-frame GIF, an ICO with five
sizes, and malformed input. `tests/` covers formats, dimensions, alpha, cache replacement/eviction,
worker admission, zoom anchoring/clamps and exports. `--smoke DIR` writes those fixtures and
outputs to DIR, drives the real UI headlessly, and, when a GPU is present, renders grid and zoom
PNGs and checks known red pixels. Run it against a fresh output directory. The shell script uses
a disposable directory and additionally checks help, invalid flags and missing-path exit codes.
Without an adapter it prints a rendering skip and still runs the interaction/codec checks.
