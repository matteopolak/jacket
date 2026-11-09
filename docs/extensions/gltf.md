# Gltf

`Extensions/Gltf` loads and writes glTF 2.0 JSON and GLB containers, with typed core structures and bounded buffer access. Core validation rejects the pinned Khronos invalid fixtures, including image resources, skinning data, animation target collisions and non-TRS matrices.

```jai
#import "Extensions/Gltf";

asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"}}");
defer gltf_error_free(*error);
if error.kind == .NONE {
    defer gltf_free(*asset);
    bytes, failure := gltf_write(*asset, binary = true);
    defer gltf_error_free(*failure);
    if failure.kind == .NONE free(bytes);
}
```

## Decoding and ownership

`gltf_load` accepts a `string` of bytes and returns an owned `Gltf_Asset` and `Gltf_Error`. Check `error.kind == .NONE`. Failed loads release their partial asset. `gltf_free` releases successful loads; `gltf_error_free` releases the error's message and path. Both reset their argument. Resolver results must use `context.allocator`, including results returned with failure.

Core objects live in `asset.document`. `types.jai` uses Json's typed `from_json` and `@json` names and defaults. Optional indices use `Json_Maybe(u32)` so omission differs from index zero. The spec's closed sets are enums with the spec's names and values: `Gltf_Component` (`FLOAT` is 5126), `Gltf_Accessor_Type` (`VEC3`), `Gltf_Mode`, `Gltf_Target`, `Gltf_Filter`, `Gltf_Wrap`, `Gltf_Alpha_Mode`, `Gltf_Interpolation`, `Gltf_Path` and `Gltf_Camera_Type`. Numeric ones are written as numbers and string ones as strings, so output stays valid glTF. Defaults are the spec's (`.TRIANGLES`, `.REPEAT`, `.OPAQUE`, `.LINEAR`). A value outside a set fails the load with a `JSON` error whose path names the field; `gltf_validate` and `gltf_write` reject such values in hand-built assets. `Gltf_Path.POINTER` (KHR_animation_pointer) is accepted only with the extension object on the target. Unknown `extensions` and `extras` remain owned `Json_Value`s. A separate shape check rejects empty core arrays, null properties and undeclared extension objects before defaults erase presence information. Extras and extension contents are opaque.

The loader checks GLB headers, chunk order, alignment and lengths, then resolves external or base64 `data:` buffers. It validates references, buffer spans, strides, matrix column padding, sparse indices, and an iterative node forest. Float accessor bounds are compared in their storage precision. Additional checks cover core attribute formats, indices, animation formats and camera projections. Skin joints and weights must have matching attribute sets, nonnegative weights summing to one across sets, distinct weighted joints and indices valid for every skin using the mesh. Integer-only weight sets must sum exactly to one after dequantization; sets including float weights use the specification's recommended epsilon per nonzero weight. Each scene containing a skinned node must also contain its joints. Inverse-bind matrices must be affine; node matrices must have orthogonal basis columns so they decompose into TRS, allowing reflections and zero scales. Animation channels cannot repeat a node and core target path within one animation.

After structural validation, the loader resolves each core image and checks its header with `Extensions/Image.read_image_info`, as Khronos's validator does. PNG/JPEG signatures, declared MIME types and data-URI MIME types must agree, and the PNG header (checksum, size, bit depth and colour type) or the JPEG frame header must be valid. Pixel data is not decoded at load, so an image with a good header and damaged data still loads; call `Extensions/Image.decode_image` on the bytes when you need the pixels. Extension-only sources remain opaque. `gltf_validate` checks in-memory document and buffer semantics; image IO and header checks happen only in `gltf_load`.

## Resolving files

No resolver means no filesystem access. Supply `Gltf_Resolver.{callback, user}` for external buffers and images. Its callback takes a URI and a byte limit and returns allocated bytes and a success flag. It must enforce the limit while reading; the loader also checks its result.

```jai
folder := "assets/robot";
resolver := Gltf_Resolver.{gltf_local_resolve, *folder};
asset, error := gltf_load(text, resolver);
```

`gltf_local_resolve` is not available on wasm (it needs POSIX directory handles or a Windows file API). Everything else works there: `gltf_load`, `gltf_validate`, `gltf_write`, accessors, GLB and data URIs. On wasm pass a `Gltf_Resolver` of your own for external files, or none to accept only embedded data. Tests that read files (`tests/files.jai`, `tests/conformance.jai`) are skipped on wasm; the rest run under `jacket test --interp -- -os=wasm`.

`gltf_local_resolve` decodes percent escapes, rejects absolute paths, schemes and NULs, and restricts files to the supplied folder. Unix lookups use directory descriptors and refuse symlinks and parent components. Windows checks the opened handle's final path against the asset folder's final path, ignoring the `\\?\` prefix and the separator style.

## Accessors and output

`gltf_accessor(*asset, index)` returns a cursor over an asset already checked by `gltf_load` or `gltf_validate`. It does not repeat whole-asset validation for each accessor. Keep the asset alive and unchanged while using it. `gltf_next_floats(*cursor)` returns sixteen float32 lanes, a boolean indicating an element, and an error. Use `cursor.components` lanes; matrices are column-major. Signed normalization clamps the minimum value to -1. Sparse replacement is merged in a single forward pass; absent base storage reads as zero. `gltf_next_index` reads unsigned SCALAR values directly into `u32`, avoiding float rounding. End of iteration returns false without an error.

`gltf_unpack_floats(*asset, index)` reads a whole accessor into a flat `[] float32` of `count * components`, by stepping the same cursor, so component types, normalization, padding and sparse replacement behave exactly as above. The caller frees `floats.data`.

## Node transforms

`gltf_node_local(node)` gives a `Matrix4` (`Math`, row-major, column vectors) from `matrix` (converted from glTF's column-major order) or from translation, rotation and scale. `gltf_node_world(*asset, index)` multiplies the local matrices from the root down and returns false for a bad index. Validation records each node's parent in `asset.parents` once, so a world matrix only walks its own ancestors; `gltf_validate` clears the index when it fails, and a hand-built asset has none until it is validated.

## Extensions

`extensions` and `extras` stay raw `Json_Value`s. `gltf_extension(*object.extensions, "KHR_materials_ior", Gltf_Ior)` decodes one named extension object into a typed struct by reading the Json tree directly with Json's generated decoder (`from_json(T, *tree)`), and returns `found` and a `.JSON` error (path like `$.ior`, offset 0) for a mistyped object. Typed structs exist for `KHR_materials_emissive_strength` (`Gltf_Emissive_Strength`), `KHR_materials_ior` (`Gltf_Ior`), `KHR_materials_transmission` (`Gltf_Transmission`), `KHR_materials_clearcoat` (`Gltf_Clearcoat`), `KHR_materials_sheen` (`Gltf_Sheen`), `KHR_materials_specular` (`Gltf_Specular`), `KHR_materials_volume` (`Gltf_Volume`; an unset `attenuation_distance` means infinity), `KHR_materials_iridescence` (`Gltf_Iridescence`), `KHR_materials_anisotropy` (`Gltf_Anisotropy`), `KHR_materials_unlit` (`Gltf_Unlit`, no properties), `KHR_texture_transform` (`Gltf_Texture_Transform`) and `KHR_lights_punctual` (`Gltf_Lights` on the document, `Gltf_Node_Light` on a node). Others are read with `json_get`. Structs with pointers (texture infos, lights) are released with `free_json`.

The decoded struct is a copy. To keep an edit, store it back with `gltf_set_extension(*object.extensions, name, value)`, which encodes it into the Json object (an absent `extensions` becomes an object, and an existing entry of that name is replaced). `gltf_write` writes the raw `extensions`, so the edit is then written. An extension the asset did not use before must be listed with `gltf_use_extension(*asset, name)`; `gltf_write` fails with `.INVALID` when an extension object is not in `extensionsUsed`, as the output would not load. Add a new extension by declaring a struct with `@json` notes in `extensions.jai`; the defaults and names come from the Khronos extension text.

`gltf_write(*asset)` embeds buffers in JSON data URIs. `gltf_write(*asset, true)` puts buffer zero in a padded BIN chunk and embeds subsequent buffers as data URIs, preserving buffer indices used by opaque extension payloads. Image URIs remain unchanged. Output bytes belong to the caller. Caller-created structures may borrow strings and arrays for validation and writing. Free a hand-built asset with `gltf_free` only if the document holds allocated strings and arrays (`copy_string`), as `free_json` needs; the exception is `extensions_used` and `extensions_required`, which may be literals. `Gltf_Asset.owns_extension_lists` says whether they are the asset's: `gltf_load` sets it, and `gltf_free` leaves a borrowed list alone. `gltf_use_extension` copies a borrowed list into allocated storage before changing it, so it never frees a literal.

## Limits and changes

`Gltf_Options` defaults to 64 MiB of document/container bytes, 512 MiB of aggregate resolved buffer and image bytes, one million objects or accessor elements per count, and JSON depth 64. Pass options to loading, validation or writing. An image's width times height may not exceed a quarter of the remaining byte budget, capped by `Extensions/Image.DEFAULT_MAX_PIXELS`. Embedded image bytes already belong to the buffer budget. There are no environment switches in the module.

Add core fields in `types.jai`, reference and layout rules in `validate.jai`, and binary interpretation in `accessor.jai`. Preserve validation order: storage precedes accessor reads, which precede graph and semantic checks. `schema.jai` handles constraints that cannot survive typed defaults. Keep regression fixtures in `tests/`; do not weaken the invalid-fixture expectation to hide missing checks.

Dependencies are `Extensions/Json`, `Base64`, `Hash_Table`, `Basic`, `String`, `Math`, `Extensions/Image`, and, off wasm, `File`; the Unix file resolver uses the repository's POSIX bindings. There is no renderer, Draco or meshopt decompression.

## Verification

Fetch the pinned Khronos sample archive and core validator fixtures through the portable data workflow:

```sh
PYTHONPATH=tools python3 tools/portable/build_portable.py data
PYTHONPATH=tools python3 tools/gltf_conformance/fetch.py --samples
jacket test --path stdlib/Extensions/Gltf --needs gltf_samples
jacket test --path stdlib/Extensions/Gltf --interp
jailint -D warnings stdlib/Extensions/Gltf/*.jai
jaifmt stdlib/Extensions/Gltf
```

`tools/gltf_conformance/fetch.py` uses SHA-256-pinned archives and a gitignored download cache. Generated fixtures go under `target/gltf-conformance`, or `JACKET_OUTPUT_DIR/gltf-conformance`. Missing fixtures fail with the command that fetches them. The sample test checks both `glTF` and `glTF-Binary` variants. Its three skips are both `SheenWoodLeatherSofa` variants (WebP image sources) and `MeshoptCubeTest/glTF` (quantized attributes); their extensions are not interpreted. There are no expected core failures. The same test decodes every typed material extension the samples use and fails if one does not decode. The portable data command fetches only the validator fixtures. The sample models are a 1.1 GB download, fetched with `--samples` and tested only under `--needs gltf_samples`; CI does both in one Linux leg (`gltf_samples` in `tools/ci/ci_changes.py`). The sample test runs natively only (`@only(.NATIVE)`); the interpreter runs the focused tests. The validator test requires rejection of every core fixture whose pinned report contains errors; extension-specific validator fixtures are not checked.

Run `jacket bench --path stdlib/Extensions/Gltf` to measure loading the pinned BoxTextured JSON asset, including its external buffer and PNG image. The benchmark reports JSON input throughput and includes resource IO, validation and freeing. The development cross-check uses Khronos's `gltf-validator` npm package through `npx`; it is not a runtime or CI dependency.

## Compared with the community libraries

[kooparse/gltf_parser](https://github.com/kooparse/gltf_parser) is a pure-Jai parser (about 1,600 lines plus the `jaison` JSON module it bundles). [napina/jai_cgltf](https://github.com/napina/jai_cgltf) is Jai bindings, not a port: it compiles the C library cgltf 1.15 (`src/cgltf.h`, `cgltf_write.h`) into a static library at build time. This comparison is from reading their source; none of it was built or run.

| Feature | Gltf | gltf_parser | jai_cgltf |
| --- | --- | --- | --- |
| Language and dependencies | pure Jai; `Extensions/Json`, `Image` | pure Jai; `jaison`, `unicode_utils` | C library built with a C++ compiler |
| `.gltf` and `.glb` | both | both; GLB must be one JSON chunk then one BIN chunk, one buffer | both |
| Data-URI buffers | yes, base64 decoded and bounds-checked | no, the URI is joined to the file path and read | yes |
| External buffers | via `Gltf_Resolver`, no filesystem unless supplied; `gltf_local_resolve` stays inside the folder | `gltf_load_buffers` joins the buffer URI to the file path, no escape check | `cgltf_load_buffers` reads the path next to the file, no escape check |
| Accessor reading | cursor: strides, normalized, sparse, matrices, `u32` indices; `gltf_unpack_floats` for a whole accessor | copies into a `[..]` of your type, cast per component; no sparse or normalized handling | `cgltf_accessor_unpack_floats` (sparse applied, normalized converted); `read_float` rejects sparse |
| Validation | full structural and semantic checks against the Khronos invalid fixtures | none; asserts on missing fields | `cgltf_validate`: accessor, view and sparse ranges, indices, skins; much less than Gltf |
| Errors | kind, message, byte offset, JSON pointer | log and `exit(-1)` | result code only |
| Extensions | `extensions` and `extras` kept as `Json_Value`; typed `gltf_extension` and `gltf_set_extension` for lights_punctual, texture_transform and the KHR_materials extensions emissive_strength, ior, transmission, clearcoat, sheen, specular, volume, iridescence, anisotropy and unlit | typed: lights_punctual, emissive_strength, ior, transmission, texture_transform | typed: lights, texture_transform, ~15 KHR_materials_*, variants, Draco/meshopt/basisu/webp records, mesh_gpu_instancing |
| Animation, skins | yes, validated; KHR_animation_pointer accepted | yes | yes |
| Morph targets | yes, validated | not parsed | yes |
| Cameras, lights | cameras typed and validated; lights typed (KHR_lights_punctual) | both typed | both typed |
| Node transforms | `gltf_node_local` and `gltf_node_world` | local and world matrices | local and world matrices |
| Writing | `gltf_write`, JSON or GLB | no | bound (`cgltf_write`, `cgltf_write_file`); its README says no write API |
| Limits on untrusted input | bytes, buffer bytes, object count, depth | none | none (only a JSON token count option) |
| Allocation | one owned `Gltf_Asset`, `context.allocator`; failed loads free themselves | arrays on the `GLTF_Data` allocator; accessor reads allocate temporary arrays | C `malloc` by default, replaceable through `cgltf_memory_options`; `cgltf_free` |

Gltf is the only one that treats the file as hostile: it bounds sizes, validates before any accessor read, reports where the problem is, and returns errors instead of exiting. gltf_parser's unchecked offsets and `exit` on bad input make it unsuitable for untrusted files. cgltf has the widest extension coverage; Gltf types twelve of its extensions and leaves the rest as raw Json.
