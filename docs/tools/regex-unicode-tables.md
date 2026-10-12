# Regex Unicode tables

## What it is

The Unicode data behind `stdlib/Extensions/Regex`: property and script ranges, the Perl classes and simple case-fold orbits. `tools/gen_regex_unicode.py` turns the Unicode Character Database (UCD 17.0.0) into `stdlib/Extensions/Regex/unicode_data.jai` (about 80 KB, committed). `unicode.jai` decodes it.

## How it works

The generated file holds three things, all plain strings so that parsing it stays cheap (no array literals):

- `UNICODE_RANGE_DATA`: every distinct range set, back to back. A set is a count, then per range the gap since the previous range (`lo - (previous_hi + 2)`, so touching ranges never occur) and `hi - lo`. Numbers are varints in 6-bit digits: five payload bits low group first, bit 5 means more digits follow, written with the base64url alphabet. Lines break every 110 characters and the decoder skips `\n`. Sets with equal contents are stored once. The simple case-fold orbits come last (`UNICODE_FOLD_OFFSET`): per orbit the size minus 2, the first member as a delta from the previous orbit's first, then member gaps minus 1.
- `UNICODE_NAME_INDEX`: one line per name, `<kind><name>=<offset>`, sorted by bytes. The name is loosely normalized (lower case, no spaces, `_` or `-`; UAX44-LM3). Kinds: `g` general category (two-letter values and the groups `L`, `LC`, `M`, `N`, `P`, `S`, `Z`, `C`), `s` Script, `x` Script_Extensions, `b` binary property, `m` `Any`/`ASCII`/`Assigned`, `P` property names accepted before `=` (value `g`, `s` or `x` names the table to look the value up in).
- `UNICODE_PERL_*_OFFSET`: offsets of `\d` (Nd), `\s` (White_Space) and `\w` (Alphabetic, M, Nd, Pc, Join_Control).

`unicode_property` binary-searches the index by bytes, reads the offset and decodes only that set. A bare name is tried as a general category, a script, a binary property, then `Any`/`ASCII`/`Assigned`; a leading `is` is retried stripped. `Name=Value` supports `gc`, `sc`, `scx` and binary properties (`yes/y/true/t`, `no/n/false/f` for the complement). The fold table is decoded on first use into a sorted array so `unicode_case_fold_ranges` can walk it against the sorted input ranges in one pass.

Sources: `UnicodeData.txt` (categories), `Scripts.txt`, `ScriptExtensions.txt`, `PropList.txt`, `DerivedCoreProperties.txt`, `emoji/emoji-data.txt`, `DerivedNormalizationProps.txt` and `extracted/DerivedBinaryProperties.txt` (binary properties; only those with an alias in `PropertyAliases.txt`), `CaseFolding.txt` (statuses `C` and `S`, closed into equivalence classes, so `k`, `K` and U+212A are one orbit and `σ`, `Σ`, `ς` are one), and the two alias files for names. `Unknown` is the script of everything not in `Scripts.txt`.

## How to change it

Run the generator and commit the result:

```
python3 tools/gen_regex_unicode.py              # download (cached), verify, write
python3 tools/gen_regex_unicode.py --ucd DIR    # an extracted UCD directory
python3 tools/gen_regex_unicode.py --zip FILE   # a local UCD.zip
python3 tools/gen_regex_unicode.py --check      # exit 1 if the committed file differs
```

To move to a new Unicode version change `UCD_VERSION`, `UCD_URL` and `UCD_SHA256` at the top of the script together. The output is deterministic. The generator round-trips every set through a Python decoder and checks that the categories partition the code space; when Python's `unicodedata` is the same Unicode version it also compares every code point's category. Adding a property family means a new reader in `build_sets` and a new kind letter handled in `unicode.jai` (`unicode_property`, `find_name`). If the encoding changes, change `varint`/`encode_*` and `read_number`/`load_fold_table` together. Test: `jaic run stdlib/Extensions/Regex/tests/unicode.jai`.

## Configuration

`--ucd DIR`, `--zip FILE`, `--check`, `--output FILE`. Without `--ucd`/`--zip` the zip is cached at `$XDG_CACHE_HOME/jaic/ucd-17.0.0.zip` (default `~/.cache/jaic/`) and its SHA-256 is verified against the pinned value before use.

## Dependencies

Python 3 standard library only; network access for the first download from `unicode.org`. At run time `unicode.jai` imports `Basic` and `Sort`.
