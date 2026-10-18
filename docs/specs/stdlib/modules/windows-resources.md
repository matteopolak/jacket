# Windows_Resources


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `set_icon_by_filename :: (exe_filename: string, icon_filename: string, resource_id : s32 = RESOURCE_ID_FIRST_ICON, delete_existing_resources := false) -> bool`
- [OS == .WINDOWS] `set_icon_by_data :: (exe_filename: string, icon_data: string, resource_id : s32 = RESOURCE_ID_FIRST_ICON, delete_existing_resources := false) -> bool`
- [OS == .WINDOWS] `Manifest_Options :: struct`
- [OS == .WINDOWS] `generate_manifest :: (options: Manifest_Options) -> string`
- [OS == .WINDOWS] `add_manifest_to_executable :: (exe_filename: string, options: Manifest_Options, resource_id: s32 = RESOURCE_ID_MANIFEST) -> bool`
- [OS == .WINDOWS] `ICONDIRENTRY :: struct`
- [OS == .WINDOWS] `ICONDIR :: struct`
- [OS == .WINDOWS] `find_windows_kit_root :: (version: string) -> string`
- [OS == .WINDOWS] `find_windows_kit_root :: () -> (root: string, windows_version: s32)`
- [OS == .WINDOWS] `find_visual_studio_in_a_ridiculous_garbage_way :: () -> (library_path := "", linker_path := "")`
- [OS == .WINDOWS] `disable_runtime_console :: (w: Workspace = -1)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `ICONDIR`

```jai
ICONDIR :: struct {
    Reserved1: WORD;
    ResourceType: WORD;
    ImageCount: WORD;

}
```
### `ICONDIRENTRY`

```jai
ICONDIRENTRY :: struct {
    Width: u8;
    Height: u8;
    Colors: u8;
    Reserved2: u8;
    Planes: WORD;
    BitsPerPixel: WORD;
    dwBytesInResource: DWORD;
    dwImageOffset: DWORD;
}
```
### `Manifest_Options`

```jai
Manifest_Options :: struct {
    dpi_aware             := true;
    require_administrator := false;
    use_segment_heap      := true;
}
```
## Contracts and constraints

This module contains Windows resource/build helpers, including COM/Win32 declarations and routines used to locate tooling or adjust a workspace’s console behavior. Tool discovery is environment-dependent and may fail when Visual Studio/SDK components are absent; callers should treat returned paths as optional. The APIs depend on Windows system libraries and are not cross-platform abstractions.

The icon group ID defaults to numeric `2`, and the manifest ID defaults to numeric `1`. `set_icon_by_filename` reads and releases the ICO file buffer, then delegates to `set_icon_by_data`. The data form expects a valid ICO directory and frame table. It creates one `RT_ICON` resource per frame at IDs `resource_id + frame_index` and one `RT_GROUP_ICON` resource at `resource_id`; there is no collision detection, so callers must choose IDs that do not conflict with resources they need to preserve. It writes resources with the English/default language ID. `delete_existing_resources=true` is passed as Win32's delete-existing flag and removes all existing resources before adding the icon, not only an earlier icon. Both APIs return false and log when resource-update calls fail. Malformed ICO data is outside the input contract; a safe implementation may reject it before touching the executable.

`Manifest_Options` has defaults `dpi_aware=true`, `require_administrator=false`, and `use_segment_heap=true`. `generate_manifest` returns an allocated string that the caller releases. Its XML requests `asInvoker` by default and `requireAdministrator` when requested. DPI awareness enabled emits the legacy per-monitor declaration plus the modern `permonitorv2,permonitor` declaration; disabled emits the legacy `unaware` declaration. Segment-heap enabled adds the `SegmentHeap` setting; disabled omits that setting. The manifest also emits the source's fixed supported-Windows compatibility entries and UTF-8 active code page setting. It does not emit an `assemblyIdentity`; the options provide no application name or version. `add_manifest_to_executable` inserts the generated XML as an `RT_MANIFEST` resource using the requested numeric ID (default `1`) and English/default language, preserving other resources. It returns false and logs on resource-update failure. Native linker/resource behavior remains Windows-only; behavior on non-Windows hosts is unspecified.

`find_windows_kit_root(version)` queries the registry's installed Windows Kits roots using `version` as the registry value name (for example, `KitsRoot10`) and returns an empty string on lookup failure. Its returned converted string uses temporary storage. The no-argument overload checks Windows 10 first and selects the lexically parsed greatest four-part numeric version directory under its `Lib` directory; if unavailable it checks Windows 8.1 and selects the greatest `winv<major>.<minor>` directory. It returns the selected root and `windows_version` value `10` or `8`; if neither is found it returns `"", 0`. The selected path is copied into allocated storage for the caller to release.

`find_visual_studio_in_a_ridiculous_garbage_way` returns directory paths, not filenames: `library_path` contains the checked `vcruntime.lib`, and `linker_path` names the corresponding linker executable directory. Its COM discovery accepts the first enumerated Visual Studio instance with a readable MSVC tools version and the expected x64 runtime library; it does not rank installations by version. If that search fails, it checks registry versions `14.0`, `12.0`, `11.0`, then `10.0`, using the corresponding `VC\Lib\amd64` and `VC\bin` directories. The modern and fallback paths target x64; there is no architecture selector. Returned paths use temporary storage, and empty paths indicate discovery failure.

`disable_runtime_console(w: Workspace = -1)` is intended for compile-time `#run` use. It appends `/SUBSYSTEM:windows` and `/ENTRY:mainCRTStartup` to the selected workspace's linker arguments; `-1` selects the current workspace. This suppresses automatic console creation for the executable and may also prevent standard output from appearing in a console unless the application attaches to one at runtime.
