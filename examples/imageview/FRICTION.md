# Friction

## Fixed

- UI could draw glyphs and rectangles but no raster images. `feat(ui): draw borrowed RGBA images`
  adds textured quads, straight-alpha blending, logical scaling and existing clip handling.
- Ico_File could write icons but could not decode them. `feat(image): decode PNG and truecolor ICO entries`
  adds the decoder to Extensions/Image with checked directory/payload access, size selection, PNG and 24/32-bit DIB decoding with masks.

## Open

- Thumbnail worker batches join on the UI thread. Only the visible page is decoded, but a page
  containing slow or large files can stall interaction. A persistent pool with incremental completion
  delivery would make loading responsive; it needs a careful cancellation/ownership design.
- UI uploads each drawn raster into a new GPU texture each frame. Thumbnails have a bounded CPU
  cache; a persistent GPU texture cache belongs in UI and would avoid repeated uploads.
- Animated GIF decoding exposes all frames in one allocation and offers no frame-count/memory
  limit. The viewer checks dimensions before decoding but cannot bound animation memory through
  this API. Animation storage and selected-image pixels are outside the thumbnail cache cap.
- ICO palette entries and compressed/bitfield DIB entries are rejected explicitly. PNG entries and
  uncompressed 24/32-bit entries work; unsupported entries currently do not fall back to another size.
- Ico_File's existing writer emits oversized DIB payloads and writes source rows without flipping
  to bottom-up order. The decoder follows the DIB orientation convention. Tests generate a flat
  asymmetric colour for the writer's five sizes; changing its output requires a separate compatibility
  decision and updates to its existing wire tests.
- 1:1 means one image pixel per logical point; a Retina display scales that to multiple physical
  pixels. A physical-pixel toggle would need passing backing scale into the view transform.
- This Mac session exposes no WebGPU adapter. UI headless interaction and codecs were exercised;
  texture upload, blending/readback and an actual window require a GPU-enabled session.
