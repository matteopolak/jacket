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

`gltf_local_resolve` decodes percent escapes, rejects absolute paths, schemes and NULs, and restricts files to the supplied folder. Unix lookups use directory descriptors and refuse symlinks and parent components. Windows checks the opened handle's canonical path.

## Accessors and output

`gltf_accessor(*asset, index)` returns a cursor over an asset already checked by `gltf_load` or `gltf_validate`. It does not repeat whole-asset validation for each accessor. Keep the asset alive and unchanged while using it. `gltf_next_floats(*cursor)` returns sixteen float32 lanes, a boolean indicating an element, and an error. Use `cursor.components` lanes; matrices are column-major. Signed normalization clamps the minimum value to -1. Sparse replacement is merged in a single forward pass; absent base storage reads as zero. `gltf_next_index` reads unsigned SCALAR values directly into `u32`, avoiding float rounding. End of iteration returns false without an error.

`gltf_write(*asset)` embeds buffers in JSON data URIs. `gltf_write(*asset, true)` puts buffer zero in a padded BIN chunk and embeds subsequent buffers as data URIs, preserving buffer indices used by opaque extension payloads. Image URIs remain unchanged. Output bytes belong to the caller. Caller-created structures may borrow strings and arrays for validation and writing; `gltf_free` is for loaded assets.

## Limits and changes

`Gltf_Options` defaults to 64 MiB of document/container bytes, 512 MiB of aggregate resolved buffer and image bytes, one million objects or accessor elements per count, and JSON depth 64. Pass options to loading, validation or writing. An image's width times height may not exceed a quarter of the remaining byte budget, capped by `Extensions/Image.DEFAULT_MAX_PIXELS`. Embedded image bytes already belong to the buffer budget. There are no environment switches in the module.

Add core fields in `types.jai`, reference and layout rules in `validate.jai`, and binary interpretation in `accessor.jai`. Preserve validation order: storage precedes accessor reads, which precede graph and semantic checks. `schema.jai` handles constraints that cannot survive typed defaults. Keep regression fixtures in `tests/`; do not weaken the invalid-fixture expectation to hide missing checks.

Dependencies are `Extensions/Json`, `Base64`, `Hash_Table`, `Basic`, `String`, `Math`, `Extensions/Image`, and `File`; the optional Unix file resolver uses the repository's POSIX bindings. There is no renderer, extension interpretation, Draco or meshopt decompression.

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

`tools/gltf_conformance/fetch.py` uses SHA-256-pinned archives and a gitignored download cache. Generated fixtures go under `target/gltf-conformance`, or `JACKET_OUTPUT_DIR/gltf-conformance`. Missing fixtures fail with the command that fetches them. The sample test checks both `glTF` and `glTF-Binary` variants. Its three skips are both `SheenWoodLeatherSofa` variants (WebP image sources) and `MeshoptCubeTest/glTF` (quantized attributes); their extensions are not interpreted. There are no expected core failures. The portable data command fetches only the validator fixtures. The sample models are a 1.1 GB download, fetched with `--samples` and tested only under `--needs gltf_samples`; CI does both in one Linux leg (`gltf_samples` in `tools/ci/ci_changes.py`). The sample test runs natively only (`@only(.NATIVE)`); the interpreter runs the focused tests. The validator test requires rejection of every core fixture whose pinned report contains errors; extension-specific validator fixtures are not checked.

Run `jacket bench --path stdlib/Extensions/Gltf` to measure loading the pinned BoxTextured JSON asset, including its external buffer and PNG image. The benchmark reports JSON input throughput and includes resource IO, validation and freeing. The development cross-check uses Khronos's `gltf-validator` npm package through `npx`; it is not a runtime or CI dependency.
