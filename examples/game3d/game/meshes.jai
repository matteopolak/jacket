// Meshes are generated as triangle soup, then run through meshoptimizer: duplicate vertices are
// merged, triangles are reordered for the vertex cache, overdraw and vertex fetch, and a coarser
// level of detail is made by simplifying. All meshes share one vertex buffer and one index buffer.

Vertex :: struct {
    position: [3] float;
    normal: [3] float;
}

Mesh_Kind :: enum s32 {
    CUBE;
    SPHERE;
    TORUS;
    FLOOR;
}

MESH_KINDS :: 4;
LOD_COUNT :: 2;

Mesh_Range :: struct {
    first_index: u32;
    index_count: u32;
    base_vertex: s32;
}

// What optimising one mesh did, for the log and the smoke test.
Mesh_Stats :: struct {
    soup_vertices: s64;
    vertices: s64;
    indices: s64;
    lod_indices: s64;
    acmr_before: float;
    acmr_after: float;
    encoded_bytes: s64;
    roundtrip_ok: bool;
}

Mesh_Set :: struct {
    vertices: [..] Vertex;
    indices: [..] u32;
    ranges: [MESH_KINDS][LOD_COUNT] Mesh_Range;
    radius: [MESH_KINDS] float;
    stats: [MESH_KINDS] Mesh_Stats;
}

Mesh_Data :: struct {
    vertices: [] Vertex;
    indices: [] u32;
    lod_indices: [] u32;
    stats: Mesh_Stats;
}

build_meshes :: (set: *Mesh_Set) {
    soups: [MESH_KINDS][] Vertex;

    soups[Mesh_Kind.CUBE] = cube_soup();
    soups[Mesh_Kind.SPHERE] = sphere_soup(28, 18);
    soups[Mesh_Kind.TORUS] = torus_soup(28, 14, 0.62, 0.34);
    soups[Mesh_Kind.FLOOR] = floor_soup();
    set.radius = .[1.74, 1.0, 1.0, 1.0];

    for kind: 0..MESH_KINDS - 1 {
        shrinks := kind == cast(s64) Mesh_Kind.SPHERE || kind == cast(s64) Mesh_Kind.TORUS;
        mesh := optimize_mesh(soups[kind], ifx shrinks then 0.2 else 1.0);

        for lod: 0..LOD_COUNT - 1 {
            source := ifx lod == 0 then mesh.indices else mesh.lod_indices;
            range := *set.ranges[kind][lod];

            range.first_index = cast(u32) set.indices.count;
            range.index_count = cast(u32) source.count;
            range.base_vertex = cast(s32) set.vertices.count;
            array_add(*set.indices, ..source);
        }

        // Both levels use the vertices of the full-detail mesh.
        array_add(*set.vertices, ..mesh.vertices);
        set.stats[kind] = mesh.stats;

        array_free(soups[kind]);
        array_free(mesh.vertices);
        array_free(mesh.indices);
        array_free(mesh.lod_indices);
    }
}

free_meshes :: (set: *Mesh_Set) {
    array_free(set.vertices);
    array_free(set.indices);
}

// Merges duplicate vertices, optimises the triangle order, makes a simplified index list
// (`target_ratio` of the triangles) and checks that the compressed form decodes to the same data.
optimize_mesh :: (soup: [] Vertex, target_ratio: float) -> Mesh_Data {
    mesh: Mesh_Data;
    index_count := cast(u64) soup.count;
    soup_count := cast(u64) soup.count;
    remap := NewArray(soup.count, u32);

    defer array_free(remap);

    vertex_count := meshopt_generateVertexRemap(remap.data, null, index_count, soup.data, soup_count, size_of(Vertex));
    indices := NewArray(soup.count, u32);

    meshopt_remapIndexBuffer(indices.data, null, index_count, remap.data);

    shuffled := NewArray(cast(s64) vertex_count, Vertex);

    defer array_free(shuffled);

    meshopt_remapVertexBuffer(shuffled.data, soup.data, soup_count, size_of(Vertex), remap.data);

    stats := *mesh.stats;

    stats.soup_vertices = soup.count;
    stats.acmr_before = meshopt_analyzeVertexCache(indices.data, index_count, vertex_count, 16, 0, 0).acmr;

    meshopt_optimizeVertexCache(indices.data, indices.data, index_count, vertex_count);
    meshopt_optimizeOverdraw(indices.data, indices.data, index_count, cast(*float) shuffled.data, vertex_count, size_of(Vertex), 1.05);

    vertices := NewArray(cast(s64) vertex_count, Vertex);

    meshopt_optimizeVertexFetch(vertices.data, indices.data, index_count, shuffled.data, vertex_count, size_of(Vertex));
    stats.acmr_after = meshopt_analyzeVertexCache(indices.data, index_count, vertex_count, 16, 0, 0).acmr;
    stats.vertices = cast(s64) vertex_count;
    stats.indices = soup.count;

    lod := NewArray(soup.count, u32);
    target := cast(u64) max(cast(float) soup.count * target_ratio, 3.0);

    target -= target % 3;

    lod_count := simplify_triangles(lod.data, indices.data, index_count, cast(*float) vertices.data, vertex_count, size_of(Vertex), target, 0.05, 0, null);

    lod.count = cast(s64) lod_count;
    stats.lod_indices = cast(s64) lod_count;
    stats.encoded_bytes, stats.roundtrip_ok = compress_roundtrip(vertices, indices);
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.lod_indices = lod;

    return mesh;
}

// Encodes the mesh with meshoptimizer's compressed formats, decodes it again and compares.
// Returns the compressed size and whether the data survived.
compress_roundtrip :: (vertices: [] Vertex, indices: [] u32) -> s64, bool {
    index_bound := meshopt_encodeIndexBufferBound(cast(u64) indices.count, cast(u64) vertices.count);
    vertex_bound := meshopt_encodeVertexBufferBound(cast(u64) vertices.count, size_of(Vertex));
    index_bytes := NewArray(cast(s64) index_bound, u8);
    vertex_bytes := NewArray(cast(s64) vertex_bound, u8);

    defer array_free(index_bytes);
    defer array_free(vertex_bytes);

    index_size := meshopt_encodeIndexBuffer(index_bytes.data, index_bound, indices.data, cast(u64) indices.count);
    vertex_size := meshopt_encodeVertexBuffer(vertex_bytes.data, vertex_bound, vertices.data, cast(u64) vertices.count, size_of(Vertex));

    decoded_indices := NewArray(indices.count, u32);
    decoded_vertices := NewArray(vertices.count, Vertex);

    defer array_free(decoded_indices);
    defer array_free(decoded_vertices);

    index_result := meshopt_decodeIndexBuffer(decoded_indices.data, cast(u64) indices.count, 4, index_bytes.data, index_size);
    vertex_result := meshopt_decodeVertexBuffer(decoded_vertices.data, cast(u64) vertices.count, size_of(Vertex), vertex_bytes.data, vertex_size);
    same := index_result == 0 && vertex_result == 0 && same_triangles(decoded_indices, indices) && memcmp(decoded_vertices.data, vertices.data, vertices.count * size_of(Vertex)) == 0;

    return cast(s64) (index_size + vertex_size), same;
}

// The index codec keeps the order of the triangles and their winding, but may start each triangle
// at a different corner, so compare triangle by triangle with the corners rotated to start at the
// smallest index.
same_triangles :: (a: [] u32, b: [] u32) -> bool {
    if a.count != b.count return false;

    for triangle: 0..a.count / 3 - 1 {
        left := canonical_triangle(a, triangle * 3);
        right := canonical_triangle(b, triangle * 3);

        if memcmp(*left, *right, size_of([3] u32)) != 0 return false;
    }

    return true;
}

#scope_file

// The module's `meshopt_simplify` has the older eight-argument form, but the library that gets
// linked takes two more (`options` and `result_error`); call the real entry point directly.
simplify_library :: #library,system "meshoptimizer";
simplify_triangles :: (destination: *u32, indices: *u32, index_count: u64, positions: *float, vertex_count: u64, stride: u64, target_index_count: u64, target_error: float, options: u32, result_error: *float) -> u64 #foreign simplify_library "meshopt_simplify";

canonical_triangle :: (indices: [] u32, at: s64) -> [3] u32 {
    a, b, c := indices[at], indices[at + 1], indices[at + 2];

    if b < a && b <= c return .[b, c, a];
    if c < a && c < b return .[c, a, b];

    return .[a, b, c];
}

make_vertex :: (position: Vector3, normal: Vector3) -> Vertex {
    return .{ position = .[position.x, position.y, position.z], normal = .[normal.x, normal.y, normal.z] };
}

add_triangle :: (soup: *[..] Vertex, a: Vertex, b: Vertex, c: Vertex) {
    array_add(soup, a);
    array_add(soup, b);
    array_add(soup, c);
}

// A cube from -1 to 1 with flat normals; each face is four corners and two triangles.
cube_soup :: () -> [] Vertex {
    soup: [..] Vertex;
    normals :: Vector3.[.{ 1, 0, 0 }, .{ -1, 0, 0 }, .{ 0, 1, 0 }, .{ 0, -1, 0 }, .{ 0, 0, 1 }, .{ 0, 0, -1 }];

    for normal: normals {
        // Two axes across the face.
        across := ifx abs(normal.y) > 0.5 then Vector3.{ 1, 0, 0 } else Vector3.{ 0, 1, 0 };
        up := cross(normal, across);

        a := make_vertex(normal - across - up, normal);
        b := make_vertex(normal + across - up, normal);
        c := make_vertex(normal + across + up, normal);
        d := make_vertex(normal - across + up, normal);

        add_triangle(*soup, a, b, c);
        add_triangle(*soup, a, c, d);
    }

    return soup;
}

sphere_point :: (segments: s64, rings: s64, segment: s64, ring: s64) -> Vertex {
    theta := cast(float) segment / cast(float) segments * TAU;
    phi := cast(float) ring / cast(float) rings * PI;
    p := Vector3.{ sin(phi) * cos(theta), cos(phi), sin(phi) * sin(theta) };

    return make_vertex(p, p);
}

// A sphere of radius 1 with smooth normals, as rings of quads split into triangles. The seam and
// the poles repeat vertices, which the remap step merges.
sphere_soup :: (segments: s64, rings: s64) -> [] Vertex {
    soup: [..] Vertex;

    for ring: 0..rings - 1 {
        for segment: 0..segments - 1 {
            a := sphere_point(segments, rings, segment, ring);
            b := sphere_point(segments, rings, segment + 1, ring);
            c := sphere_point(segments, rings, segment + 1, ring + 1);
            d := sphere_point(segments, rings, segment, ring + 1);

            if ring > 0 add_triangle(*soup, a, c, b);
            if ring < rings - 1 add_triangle(*soup, a, d, c);
        }
    }

    return soup;
}

torus_point :: (segments: s64, sides: s64, major: float, minor: float, segment: s64, side: s64) -> Vertex {
    u := cast(float) segment / cast(float) segments * TAU;
    v := cast(float) side / cast(float) sides * TAU;
    ring := Vector3.{ cos(u), 0, sin(u) };
    normal := ring * cos(v) + Vector3.{ 0, sin(v), 0 };

    return make_vertex(ring * major + normal * minor, normal);
}

// A torus lying in the xz plane: `major` is the distance from the centre to the middle of the
// tube and `minor` the tube's radius.
torus_soup :: (segments: s64, sides: s64, major: float, minor: float) -> [] Vertex {
    soup: [..] Vertex;

    for segment: 0..segments - 1 {
        for side: 0..sides - 1 {
            a := torus_point(segments, sides, major, minor, segment, side);
            b := torus_point(segments, sides, major, minor, segment + 1, side);
            c := torus_point(segments, sides, major, minor, segment + 1, side + 1);
            d := torus_point(segments, sides, major, minor, segment, side + 1);

            add_triangle(*soup, a, b, c);
            add_triangle(*soup, a, c, d);
        }
    }

    return soup;
}

// A flat square from -1 to 1 in the xz plane, facing up.
floor_soup :: () -> [] Vertex {
    soup: [..] Vertex;
    up := Vector3.{ 0, 1, 0 };
    a := make_vertex(.{ -1, 0, -1 }, up);
    b := make_vertex(.{ 1, 0, -1 }, up);
    c := make_vertex(.{ 1, 0, 1 }, up);
    d := make_vertex(.{ -1, 0, 1 }, up);

    add_triangle(*soup, a, d, c);
    add_triangle(*soup, a, c, b);

    return soup;
}
