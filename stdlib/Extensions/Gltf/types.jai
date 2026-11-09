// The type of an accessor's components, with the glTF numeric codes. The enums in this file mirror
// the spec's closed sets: numeric ones are written as numbers, string ones in the spec's spelling,
// and a value outside the set fails to decode.
Gltf_Component :: enum u32 {
    BYTE :: 5120;
    UNSIGNED_BYTE :: 5121;
    SHORT :: 5122;
    UNSIGNED_SHORT :: 5123;
    UNSIGNED_INT :: 5125;
    FLOAT :: 5126;
}

// How many components make one accessor element: a scalar, a vector of 2 to 4, or a 2x2 to 4x4
// matrix.
Gltf_Accessor_Type :: enum u8 {
    SCALAR;
    VEC2;
    VEC3;
    VEC4;
    MAT2;
    MAT3;
    MAT4;
}

// How a primitive's vertices are assembled into shapes. The default is `.TRIANGLES`.
Gltf_Mode :: enum u32 {
    POINTS :: 0;
    LINES :: 1;
    LINE_LOOP :: 2;
    LINE_STRIP :: 3;
    TRIANGLES :: 4;
    TRIANGLE_STRIP :: 5;
    TRIANGLE_FAN :: 6;
}

// The GPU buffer a buffer view is meant for: vertex attributes or indices.
Gltf_Target :: enum u32 {
    ARRAY_BUFFER :: 34962;
    ELEMENT_ARRAY_BUFFER :: 34963;
}

// A sampler's texture filtering mode, with the glTF numeric codes.
Gltf_Filter :: enum u32 {
    NEAREST :: 9728;
    LINEAR :: 9729;
    NEAREST_MIPMAP_NEAREST :: 9984;
    LINEAR_MIPMAP_NEAREST :: 9985;
    NEAREST_MIPMAP_LINEAR :: 9986;
    LINEAR_MIPMAP_LINEAR :: 9987;
}

// A sampler's texture wrapping mode, with the glTF numeric codes. The default is `.REPEAT`.
Gltf_Wrap :: enum u32 {
    CLAMP_TO_EDGE :: 33071;
    MIRRORED_REPEAT :: 33648;
    REPEAT :: 10497;
}

// How a material's alpha is interpreted: ignored, thresholded by `alpha_cutoff`, or blended.
Gltf_Alpha_Mode :: enum u8 {
    OPAQUE;
    MASK;
    BLEND;
}

// How an animation sampler blends between keyframes. The default is `.LINEAR`.
Gltf_Interpolation :: enum u8 {
    LINEAR;
    STEP;
    CUBICSPLINE;
}

// Which property of a node an animation channel drives. `POINTER` is KHR_animation_pointer's target
// and needs the extension object on the target.
Gltf_Path :: enum u8 {
    TRANSLATION;
    ROTATION;
    SCALE;
    WEIGHTS;
    POINTER;
}

// Whether a camera is a [Gltf_Perspective] or a [Gltf_Orthographic] projection.
Gltf_Camera_Type :: enum u8 {
    PERSPECTIVE;
    ORTHOGRAPHIC;
}

// The glTF root object: every array of scene data, which refer to each other by index. Decoded from
// and encoded to JSON through the `@json` notes; an optional index is a `Json_Maybe` so that
// omission differs from index 0. `extensions` and `extras` keep unknown data as `Json_Value`.
Gltf_Document :: struct @json(rename_all=camelCase) {
    asset: Gltf_Metadata;
    scene: Json_Maybe(u32);
    scenes: [] Gltf_Scene; @json(default,skip_serializing_if=empty)
    nodes: [] Gltf_Node; @json(default,skip_serializing_if=empty)
    meshes: [] Gltf_Mesh; @json(default,skip_serializing_if=empty)
    accessors: [] Gltf_Accessor; @json(default,skip_serializing_if=empty)
    buffer_views: [] Gltf_Buffer_View; @json(default,skip_serializing_if=empty)
    buffers: [] Gltf_Buffer; @json(default,skip_serializing_if=empty)
    materials: [] Gltf_Material; @json(default,skip_serializing_if=empty)
    textures: [] Gltf_Texture; @json(default,skip_serializing_if=empty)
    images: [] Gltf_Image; @json(default,skip_serializing_if=empty)
    samplers: [] Gltf_Sampler; @json(default,skip_serializing_if=empty)
    skins: [] Gltf_Skin; @json(default,skip_serializing_if=empty)
    animations: [] Gltf_Animation; @json(default,skip_serializing_if=empty)
    cameras: [] Gltf_Camera; @json(default,skip_serializing_if=empty)
    extensions_used: [] string; @json(default,skip_serializing_if=empty)
    extensions_required: [] string; @json(default,skip_serializing_if=empty)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The `asset` object: the glTF `version` the file targets plus optional authoring details.
Gltf_Metadata :: struct @json(rename_all=camelCase) {
    version: string;
    min_version: string; @json(default,skip_serializing_if=empty)
    generator: string; @json(default,skip_serializing_if=empty)
    copyright: string; @json(default,skip_serializing_if=empty)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A set of root nodes, by index into `nodes`.
Gltf_Scene :: struct @json(rename_all=camelCase) {
    nodes: [] u32; @json(default,skip_serializing_if=empty)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A node in the scene hierarchy: it may place a mesh, skin or camera and has either a `matrix` or
// translation, rotation and scale. `children` and the references are indices.
Gltf_Node :: struct @json(rename_all=camelCase) {
    children: [] u32; @json(default,skip_serializing_if=empty)
    mesh: Json_Maybe(u32);
    skin: Json_Maybe(u32);
    camera: Json_Maybe(u32);
    matrix: [] float32; @json(default,skip_serializing_if=empty)
    rotation: [] float32; @json(default,skip_serializing_if=empty)
    scale: [] float32; @json(default,skip_serializing_if=empty)
    translation: [] float32; @json(default,skip_serializing_if=empty)
    weights: [] float32; @json(default,skip_serializing_if=empty)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A list of primitives drawn together, with optional default morph target `weights`.
Gltf_Mesh :: struct @json(rename_all=camelCase) {
    primitives: [] Gltf_Primitive;
    weights: [] float32; @json(default,skip_serializing_if=empty)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// One draw call of a mesh: vertex `attributes` by name (such as `POSITION`) to accessor index,
// optional `indices` and `material`, the assembly `mode`, and morph `targets`.
Gltf_Primitive :: struct @json(rename_all=camelCase) {
    attributes: Table(string, u32);
    indices: Json_Maybe(u32);
    material: Json_Maybe(u32);
    mode: Gltf_Mode = .TRIANGLES; @json(default,as_number)
    targets: [] Table(string, u32); @json(default,skip_serializing_if=empty)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A block of binary data: `byte_length` bytes, from `uri` or from the GLB binary chunk when `uri` is
// empty.
Gltf_Buffer :: struct @json(rename_all=camelCase) {
    uri: string; @json(default,skip_serializing_if=empty)
    byte_length: u32;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A byte range of a [Gltf_Buffer], optionally strided and tagged with its GPU `target`.
Gltf_Buffer_View :: struct @json(rename_all=camelCase) {
    buffer: u32;
    byte_offset: u32 = 0; @json(default,skip_serializing_if=zero)
    byte_length: u32;
    byte_stride: Json_Maybe(u32);
    target: Json_Maybe(Gltf_Target); @json(as_number)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A typed view onto a buffer view: `count` elements of `type` made of `component_type`, with
// optional `min` and `max` bounds and a `sparse` override. Read one with [gltf_accessor].
Gltf_Accessor :: struct @json(rename_all=camelCase) {
    buffer_view: Json_Maybe(u32);
    byte_offset: u32 = 0; @json(default,skip_serializing_if=zero)
    component_type: Gltf_Component; @json(as_number)
    normalized: bool = false; @json(default)
    count: u32;
    type: Gltf_Accessor_Type;
    min: [] float64; @json(default,skip_serializing_if=empty)
    max: [] float64; @json(default,skip_serializing_if=empty)
    sparse: *Gltf_Sparse; @json(default,skip_serializing_if=null)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// An accessor's sparse storage: `count` replaced elements, their `indices` and new `values`.
Gltf_Sparse :: struct @json(rename_all=camelCase) {
    count: u32;
    indices: Gltf_Sparse_Indices;
    values: Gltf_Sparse_Values;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The positions replaced by a [Gltf_Sparse] accessor, read from a buffer view.
Gltf_Sparse_Indices :: struct @json(rename_all=camelCase) {
    buffer_view: u32;
    byte_offset: u32 = 0; @json(default,skip_serializing_if=zero)
    component_type: Gltf_Component; @json(as_number)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The replacement elements of a [Gltf_Sparse] accessor, read from a buffer view.
Gltf_Sparse_Values :: struct @json(rename_all=camelCase) {
    buffer_view: u32;
    byte_offset: u32 = 0; @json(default,skip_serializing_if=zero)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A sampler and an image `source`, by index.
Gltf_Texture :: struct @json(rename_all=camelCase) {
    sampler: Json_Maybe(u32);
    source: Json_Maybe(u32);
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// Image data from a `uri` or from a buffer view, in which case `mime_type` must be `image/png` or
// `image/jpeg`. URIs are kept as written.
Gltf_Image :: struct @json(rename_all=camelCase) {
    uri: string; @json(default,skip_serializing_if=empty)
    mime_type: string; @json(default,skip_serializing_if=empty)
    buffer_view: Json_Maybe(u32);
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// How a texture is filtered and wrapped.
Gltf_Sampler :: struct @json(rename_all=camelCase) {
    mag_filter: Json_Maybe(Gltf_Filter); @json(as_number)
    min_filter: Json_Maybe(Gltf_Filter); @json(as_number)
    wrap_s: Gltf_Wrap = .REPEAT; @json(default,as_number)
    wrap_t: Gltf_Wrap = .REPEAT; @json(default,as_number)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A texture reference from a material: the texture `index`, the UV set `tex_coord`, and the `scale`
// (normal) or `strength` (occlusion) that applies.
Gltf_Texture_Info :: struct @json(rename_all=camelCase) {
    index: u32;
    tex_coord: u32 = 0; @json(default)
    scale: float32 = 1.0; @json(default)
    strength: float32 = 1.0; @json(default)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The metallic-roughness parameters of a [Gltf_Material]: base color, metalness and roughness, with
// their textures.
Gltf_Pbr :: struct @json(rename_all=camelCase) {
    base_color_factor: [4] float32 = .[1, 1, 1, 1]; @json(default)
    base_color_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    metallic_factor: float32 = 1.0; @json(default)
    roughness_factor: float32 = 1.0; @json(default)
    metallic_roughness_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// How a surface looks: PBR parameters, normal, occlusion and emissive inputs, and alpha handling.
Gltf_Material :: struct @json(rename_all=camelCase) {
    pbr_metallic_roughness: *Gltf_Pbr; @json(default,skip_serializing_if=null)
    normal_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    occlusion_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    emissive_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    emissive_factor: [3] float32 = .[0, 0, 0]; @json(default)
    alpha_mode: Gltf_Alpha_Mode = .OPAQUE; @json(default)
    alpha_cutoff: float32 = 0.5; @json(default)
    double_sided: bool = false; @json(default)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The joints that deform a mesh: the joint node indices, the optional `skeleton` root and inverse
// bind matrices accessor.
Gltf_Skin :: struct @json(rename_all=camelCase) {
    inverse_bind_matrices: Json_Maybe(u32);
    skeleton: Json_Maybe(u32);
    joints: [] u32;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// Keyframed motion: `channels` that connect `samplers` to node properties.
Gltf_Animation :: struct @json(rename_all=camelCase) {
    channels: [] Gltf_Channel;
    samplers: [] Gltf_Animation_Sampler;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// Connects the animation sampler at index `sampler` to the property in `target`.
Gltf_Channel :: struct @json(rename_all=camelCase) {
    sampler: u32;
    target: Gltf_Animation_Target;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The node and property (`path`) an animation channel writes.
Gltf_Animation_Target :: struct @json(rename_all=camelCase) {
    node: Json_Maybe(u32);
    path: Gltf_Path; @json(rename_all=lowercase)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// Keyframes: the accessor of times (`input`), the accessor of values (`output`) and the
// `interpolation` between them.
Gltf_Animation_Sampler :: struct @json(rename_all=camelCase) {
    input: u32;
    output: u32;
    interpolation: Gltf_Interpolation = .LINEAR; @json(default)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A camera, with the projection for its `type` filled in.
Gltf_Camera :: struct @json(rename_all=camelCase) {
    type: Gltf_Camera_Type; @json(rename_all=lowercase)
    perspective: *Gltf_Perspective; @json(default,skip_serializing_if=null)
    orthographic: *Gltf_Orthographic; @json(default,skip_serializing_if=null)
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// A perspective projection; a missing `zfar` means an infinite far plane.
Gltf_Perspective :: struct @json(rename_all=camelCase) {
    aspect_ratio: Json_Maybe(float32);
    yfov: float32;
    zfar: Json_Maybe(float32);
    znear: float32;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// An orthographic projection with half-widths `xmag` and `ymag`.
Gltf_Orthographic :: struct @json(rename_all=camelCase) {
    xmag: float32;
    ymag: float32;
    zfar: float32;
    znear: float32;
    name: string; @json(default,skip_serializing_if=empty)
    extensions: Json_Value; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}
