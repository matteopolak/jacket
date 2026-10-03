# Rewrite


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Single_Rewrite :: struct`
- `File_To_Rewrite :: struct`
- `My_Plugin :: struct`
- `init :: (p: *Metaprogram_Plugin, options: [] string) -> bool`
- `gather :: (p: *Metaprogram_Plugin, message: *Message)`
- `finish :: (p: *Metaprogram_Plugin)`
- `rewrite_one_file :: (my: *My_Plugin, file: *File_To_Rewrite)`
- `make_new_file_data :: (my: *My_Plugin, file: *File_To_Rewrite, sorted: [] *Single_Rewrite) -> string`
- `compare_single_rewrites :: (a: *Single_Rewrite, b: *Single_Rewrite) -> int`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `File_To_Rewrite`

```jai
File_To_Rewrite :: struct {
    filename: string;

    single_rewrites: [..] Single_Rewrite;

    full_data: string;

    lines: [] string;
}
```
### `My_Plugin`

```jai
My_Plugin :: struct {
    #as using base: Metaprogram_Plugin;

    files_to_rewrite: Table(*Message_File, File_To_Rewrite);

    write_mode := false;
}
```
### `Single_Rewrite`

```jai
Single_Rewrite :: struct {
    l0, c0, l1, c1: s32;

    new_text: string;
}
```
## Contracts and constraints

This plugin scans compiler typechecked code for deprecated dereferences and schedules source rewrites. It sorts edits, reports them, and defaults to read-only operation; writing is an explicit plugin option and modifies source files. Overlapping or stale source locations can prevent a safe rewrite, and file-read failures are logged. Treat write mode as a source mutation workflow and review its planned edits before enabling it.
