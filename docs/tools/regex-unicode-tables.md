# Regex Unicode tables

## What it is

The Unicode data behind `stdlib/Extensions/Regex` (properties, scripts, Perl classes, simple case-fold orbits). `tools/gen_regex_unicode.py` turns the Unicode Character Database (UCD 17.0.0) into the committed `stdlib/Extensions/Regex/unicode_data.jai`; `unicode.jai` decodes it.

## How it works

The generated file is plain strings (cheap to parse): range sets stored once as varints (gaps and lengths), a sorted name index `<kind><name>=<offset>` (kinds: general category, Script, Script_Extensions, binary property, `Any`/`ASCII`/`Assigned`, property names) with loosely normalized names, and offsets for `\d`, `\s`, `\w`. `unicode_property` binary-searches the index and decodes only that set; the fold table is decoded on first use.

## How to change it

```sh
python3 tools/gen_regex_unicode.py            # download (cached), verify, write
python3 tools/gen_regex_unicode.py --check    # exit 1 if the committed file differs
```

Also `--ucd DIR`, `--zip FILE`, `--output FILE`; the zip is cached under `$XDG_CACHE_HOME/jaic/` and verified against the pinned SHA-256. To move Unicode versions change `UCD_VERSION`, `UCD_URL`, `UCD_SHA256` together. A new property family needs a reader in `build_sets` and a kind letter in `unicode.jai` (`unicode_property`, `find_name`); encoding changes touch `varint`/`encode_*` and `read_number`/`load_fold_table` together. Test: `jaic stdlib/Extensions/Regex/tests/unicode.jai -interp`. Dependencies: Python 3 and network for the first download.
