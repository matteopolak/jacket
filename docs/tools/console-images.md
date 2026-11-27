# Console images

`docs/images/console/*.svg` are terminal windows showing real output, colours included, for the changelog and release notes. `tools/dev/console_svg.py` generates them; don't edit them by hand.

## How it works

Each image is a `Shot` in the script. It takes its output from one of two places:

- **A guide capture.** `(guide, block, index)` reads `docs/guides/<guide>.console.json`, which is the coloured output `jacket doc` recorded for that guide's console block.
- **A command.** `command` and `files` write the files to a temporary folder and run the built tool there with `FORCE_COLOR=1` and `JAIC_DIAGNOSTICS=unicode`.

The script turns SGR escapes (bold, dim and the 16 colours) into SVG `<tspan>`s inside a drawn window. Spaces become non-breaking spaces so that every renderer keeps the alignment. The changelog shows them two to a row in `SHOTS` order. Both images of a row are as wide as its longest line, so they scale alike side by side.

## How to change it

Add or edit a `Shot` in `SHOTS`, then rerun:

```bash
CARGO_TARGET_DIR=... JAIC_STDLIB=$PWD/stdlib cargo build -p jaic-cli -p jailint
CARGO_TARGET_DIR=... JAIC_STDLIB=$PWD/stdlib PYTHONPATH=tools python3 tools/dev/console_svg.py
```

Passing shot names renders only those shots. The colours, font and window are constants at the top of the script.

A guide capture changes only when `jacket doc` reruns that block, so regenerate the images after updating the guides.

## Dependencies

- Python 3.
- The debug `jaic` and `jailint` builds in `CARGO_TARGET_DIR`. The script stops with the build command if one is missing.
- `JAIC_STDLIB`.
