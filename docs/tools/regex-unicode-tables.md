# Regex Unicode tables

The Unicode data behind `stdlib/Extensions/Regex`: properties, scripts, Perl classes and case folding. `tools/generators/gen_regex_unicode.py` turns the Unicode Character Database into the committed `stdlib/Extensions/Regex/unicode_data.jai`; `unicode.jai` decodes it.

```sh
PYTHONPATH=tools python3 tools/generators/gen_regex_unicode.py          # download (cached), verify, write
PYTHONPATH=tools python3 tools/generators/gen_regex_unicode.py --check  # exit 1 if the committed file differs
```

## How it works

- The generated file is plain strings: range sets stored once as varints, plus a sorted name index (`<kind><name>=<offset>`).
- `unicode_property` binary-searches the index and decodes only the set it needs. The fold table is decoded on first use.

## How to change it

- Move Unicode versions by changing `UCD_VERSION`, `UCD_URL` and `UCD_SHA256` together. The zip is cached under `$XDG_CACHE_HOME/jaic/`.
- A new property family needs a reader in `build_sets` and a kind letter in `unicode.jai` (`unicode_property`, `find_name`).
- Encoding changes touch `varint`/`encode_*` and `read_number`/`load_fold_table` together.
- Test with `jaic stdlib/Extensions/Regex/tests/unicode.jai -interp`.
