# Simple_Package


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Create_Package :: struct`
- `Entry_Info :: struct`
- `init :: (using package: *Create_Package)`
- `deinit :: (using package: *Create_Package)`
- `add :: (package: *Create_Package, entry_name: string, data: [] u8)`
- `write :: (package: *Create_Package, filename: string, sort_contents := true) -> bool`
- `KNOWN_HEADER_SIZE :: 64`
- `Package_Header :: struct`
- `Load_Package :: struct`
- `KNOWN_TABLE_OF_CONTENTS_SIZE :: 64`
- `Table_Of_Contents :: struct`
- `init_from_memory :: (package: *Load_Package, data: [] u8, name_for_debugging := "") -> bool`
- `init_from_file :: (package: *Load_Package, filename: string) -> bool`
- `load_table_of_contents_only :: (filename: string, okay_if_it_does_not_exist := false) -> (success: bool, package: Load_Package)`
- `load_table_of_contents :: (package: *Load_Package, data: [] u8, base_file_data_if_file_is_loaded: *u8, name_for_debugging: string, offset_for_debugging: s64) -> bool`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Create_Package`

```jai
Create_Package :: struct {
    data: String_Builder;

    entries: [..] Entry_Info;
    pool: Pool;
}
```
### `Entry_Info`

```jai
Entry_Info :: struct {
    name: string;
    data: [] u8;
    offset_from_start_of_file: s64;
    size_of_entry: s64;
}
```
### `Load_Package`

```jai
Load_Package :: struct {
    header: Package_Header;
    data: [] u8;

    table_of_contents: Table_Of_Contents;

    entries: [] Entry_Info;

    lookup: Table(string, *Entry_Info);
}
```
### `Package_Header`

```jai
Package_Header :: struct {
    magic:     u32;
    version:   u32;
    flags:     u32;
    reserved0: u32;

    table_of_contents_offset: s64;

    reserved:  [5] u64;
}
```
### `Table_Of_Contents`

```jai
Table_Of_Contents :: struct {
    table_of_contents_magic: u32;
    reserved0:               u32;

    number_of_entries:       s64;

    reserved:                [6] u64;
}
```
## Behavioral contract

`Create_Package.add` copies each entry name into the package pool immediately and retains the supplied payload slice without copying it; payload bytes must remain valid until `write` completes. `write` sorts entries by name by default (or preserves add order when `sort_contents=false`), packs payloads after the 64-byte header, then writes the table of contents. It does not reject or deduplicate duplicate names. A loaded package stores every TOC entry in `entries`, but `lookup` returns the first matching duplicate name, matching `Hash_Table.table_add` semantics.

`Load_Package.init_from_memory` borrows its input, so the package's `data`, entry-name slices, payload views, and lookup keys depend on that buffer staying alive. `init_from_file` stores the allocated full-file buffer in `package.data`; because there is no `Load_Package` deinitializer, the caller must keep it alive for the views and release its allocation when finished. Table-of-contents-only loading allocates and stores the file tail beginning at the TOC; entry names borrow that tail, but payload slices remain unset because no full-file base pointer is supplied. The caller likewise releases that buffer.

`load_table_of_contents` expects `data` to begin at a complete TOC preamble and contain the remaining TOC bytes through its end. That byte slice may be a separate copy; it need not share storage with the payload file. Entry names borrow bytes after the preamble from `data`, while, when `base_file_data_if_file_is_loaded` is non-null, each payload view points at `base_file_data_if_file_is_loaded + chunk_offset` using the absolute file offset. Keep both backing allocations alive for the views they supply. With a null base pointer, entry sizes/offsets are populated but payload slices are not. The caller must initialize `package.header` before calling this helper; `offset_for_debugging` is used for diagnostics, not addressing. Reinitializing a populated `Load_Package` has no cleanup contract: the module does not release previous buffers, arrays, or lookup storage, so use a fresh value or perform caller-managed cleanup rather than assuming repeated initialization replaces its prior state.

## Binary format

The magic values are encoded little-endian: the header signature bytes are `simp` (`0x706d6973` as a little-endian u32), and the TOC signature bytes are `toc!` (`0x21636f74`). `write` emits version 1. The reader rejects versions greater than 1; it accepts lower values but does not guarantee a distinct older layout. The format is little-endian only.

The 64-byte header starts at file offset 0. Its `table_of_contents_offset` is a signed 64-bit absolute file offset. Entry payload bytes are packed from offset 64 in the selected entry order, and the TOC starts immediately after them. The 64-byte TOC preamble stores a u32 TOC signature and a signed 64-bit entry count; its reserved members are not interpreted. Each TOC entry is a u32 name byte length, that many name bytes, a zero byte, then an 8-byte payload size and an 8-byte absolute file offset. The writer emits the latter two values as u64; the reader interprets the same 64-bit words as signed s64. Names are byte strings and the stored length excludes the terminator. The TOC extends to end-of-file; there is no separate TOC byte-length field.

The reader checks header and TOC signatures, rejects newer versions, and checks the TOC's starting offset against a known file length when one is supplied. Its current offset check also rejects a TOC whose 64-byte preamble ends exactly at EOF, including a zero-entry package produced by the writer. It does not guarantee a comprehensive malformed-input validation guarantee for every entry length, signed value, or payload range; callers should treat malformed data as unsupported, and safe consumers should validate all bounds before forming views. `load_table_of_contents_only(..., okay_if_it_does_not_exist=true)` suppresses the missing-file diagnostic but still returns failure when the file is absent. Depends on Pool/String_Builder, File, and reflection helpers.
