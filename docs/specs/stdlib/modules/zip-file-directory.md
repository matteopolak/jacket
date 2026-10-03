# Zip_File_Directory


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Zip_Entry :: struct`
- `Zip_Directory :: struct`
- `deinit :: (dir: *Zip_Directory)`
- `load_zip_directory :: (zip_filename: string, close_file := true) -> (success: bool, dir: Zip_Directory)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Zip_Directory`

```jai
Zip_Directory :: struct {
    name := "";
    entries: [..] Zip_Entry;

    memory_block: string;
    file: File;
}
```
### `Zip_Entry`

```jai
Zip_Entry :: struct {
    filename: string;
    position_in_file: s64;

    version_made_by: u16;
    version_needed_to_extract: u16;

    bit_flag:        u16;
    crc:             u32;
    compressed_len:  u32;
    chunk_len:       u32;
    name_len:        u16;
    extra_field_len: u16;
    file_comment_len:    u16;
    internal_attributes: u16;
    external_attributes: u32;
    chunk_offset:        u32;
}
```
## Behavioral contract

`load_zip_directory` reads the classic ZIP central directory into `Zip_Entry` records; it does not decompress entries. It does not support ZIP64, and asserts on unexpected signatures or data-descriptor flag bit 3. Failures generally return `success=false` without a detailed reason. Each entry's filename is a view into `Zip_Directory.memory_block`; keep the directory alive while using names. `deinit` frees its owned name/entry array/central-directory buffer but deliberately does not close the file handle. With `close_file=false`, caller must close `dir.file`. Depends on file APIs and assumes little-endian data.
