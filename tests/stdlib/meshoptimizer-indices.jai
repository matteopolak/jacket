// meshoptimizer against the pinned C++ library (tools/build_native_libs.py): take a shuffled,
// unindexed grid mesh, index it, optimize it for the vertex cache and for vertex fetch, and check
// the invariants that must survive: the same triangles (as a set), every index in range, a better
// cache hit rate and fetch overfetch. Also strip and index/vertex buffer codecs round-trip.
#import "Basic";
#import "Sort";
#import "meshoptimizer";

GRID :: 12;

Vertex :: struct {
    x, y, z: float;
}

// A flat GRID x GRID grid of quads, two triangles each, as a triangle list with a vertex per
// corner (so every grid point appears several times), triangles in a scrambled order.
make_soup :: () -> [] Vertex {
    soup: [..] Vertex;
    for cell: 0..GRID * GRID - 1 {
        x := cell % GRID;
        y := cell / GRID;
        // Two triangles per cell: (0,0) (1,0) (1,1) and (0,0) (1,1) (0,1).
        for offset, k: s32.[0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1]
            if k % 2 == 0 array_add(*soup, Vertex.{ cast(float) (x + offset), 0, 0 });
        else soup[soup.count - 1].y = cast(float) (y + offset);
    }
    triangles := soup.count / 3;
    state: u32 = 99;
    for i: 0..triangles - 1 {
        state = state * 1664525 + 1013904223;
        j := cast(s64) ((state >> 8) % cast(u32) triangles);
        for k: 0..2 {
            tmp := soup[i * 3 + k];
            soup[i * 3 + k] = soup[j * 3 + k];
            soup[j * 3 + k] = tmp;
        }
    }
    return soup;
}

// The triangle set of an index list, each triangle as one number with its corners rotated so the
// smallest comes first (the winding is kept), sorted.
triangle_keys :: (indices: [] u32) -> [] u64 {
    keys: [..] u64;
    for t: 0..indices.count / 3 - 1 {
        a, b, c := indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2];
        if b < a && b < c {
            a, b, c = b, c, a;
        } else if c < a && c < b {
            a, b, c = c, a, b;
        }
        array_add(*keys, (cast(u64) a << 40) | (cast(u64) b << 20) | cast(u64) c);
    }
    quick_sort(keys, (x: u64, y: u64) => (cast(s64) x - cast(s64) y));
    return keys;
}

main :: () {
    soup := make_soup();
    index_count := cast(u64) soup.count;
    assert(index_count == GRID * GRID * 6);

    // Index it: the grid has (GRID + 1)^2 distinct points.
    remap := NewArray(soup.count, u32);
    unique := meshopt_generateVertexRemap(remap.data, null, index_count, soup.data, index_count, size_of(Vertex));
    assert(unique == (GRID + 1) * (GRID + 1), "% unique vertices", unique);
    indices := NewArray(soup.count, u32);
    meshopt_remapIndexBuffer(indices.data, null, index_count, remap.data);
    vertices := NewArray(cast(s64) unique, Vertex);
    meshopt_remapVertexBuffer(vertices.data, soup.data, index_count, size_of(Vertex), remap.data);
    for indices assert(it < unique, "index % out of range", it);
    for t: 0..indices.count / 3 - 1 {
        a, b, c := indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2];
        assert(vertices[a].x == soup[t * 3].x && vertices[b].y == soup[t * 3 + 1].y && vertices[c].x == soup[t * 3 + 2].x, "triangle % keeps its positions", t);
    }
    before_keys := triangle_keys(indices);

    // Optimize for the post-transform cache: same triangles, better hit rate.
    before := meshopt_analyzeVertexCache(indices.data, index_count, unique, 16, 0, 0);
    optimized := NewArray(soup.count, u32);
    meshopt_optimizeVertexCache(optimized.data, indices.data, index_count, unique);
    after := meshopt_analyzeVertexCache(optimized.data, index_count, unique, 16, 0, 0);
    assert(after.acmr < before.acmr, "ACMR %, was %", after.acmr, before.acmr);
    assert(after.acmr < 1.0, "a grid reaches an ACMR near 0.5, got %", after.acmr);
    for optimized assert(it < unique, "optimized index % out of range", it);
    after_keys := triangle_keys(optimized);
    assert(after_keys.count == before_keys.count);
    for before_keys assert(after_keys[it_index] == it, "the triangle set changed at %", it_index);

    // Overdraw optimization keeps the triangle set as well.
    overdraw := NewArray(soup.count, u32);
    meshopt_optimizeOverdraw(overdraw.data, optimized.data, index_count, *vertices[0].x, unique, size_of(Vertex), 1.05);
    overdraw_keys := triangle_keys(overdraw);
    for before_keys assert(overdraw_keys[it_index] == it, "overdraw changed the triangles at %", it_index);

    // Vertex fetch: renumber the vertices in the order they are first used.
    fetched := NewArray(soup.count, u32);
    memcpy(fetched.data, optimized.data, soup.count * size_of(u32));
    reordered := NewArray(cast(s64) unique, Vertex);
    fetch_unique := meshopt_optimizeVertexFetch(reordered.data, fetched.data, index_count, vertices.data, unique, size_of(Vertex));
    assert(fetch_unique == unique, "% vertices kept", fetch_unique);
    assert(fetched[0] == 0, "the first index fetches the first vertex");
    highest: u32;
    for fetched {
        assert(it <= highest + 1, "index % jumps past the vertices fetched so far (%)", it, highest);
        highest = max(highest, it);
    }
    fetch_stats := meshopt_analyzeVertexFetch(fetched.data, index_count, unique, size_of(Vertex));
    assert(fetch_stats.overfetch < 3.0, "overfetch %", fetch_stats.overfetch);

    // Strips: the list round-trips through a triangle strip with a restart index.
    strip := NewArray(cast(s64) meshopt_stripifyBound(index_count), u32);
    strip_size := meshopt_stripify(strip.data, optimized.data, index_count, unique, 0xFFFFFFFF);
    assert(strip_size > 0 && strip_size < index_count, "strip of % indices", strip_size);
    list := NewArray(cast(s64) meshopt_unstripifyBound(strip_size), u32);
    list_size := meshopt_unstripify(list.data, strip.data, strip_size, 0xFFFFFFFF);
    assert(list_size == index_count, "% indices back from the strip", list_size);
    list.count = cast(s64) list_size;
    list_keys := triangle_keys(list);
    for before_keys assert(list_keys[it_index] == it, "the strip changed the triangles at %", it_index);

    // Index buffer codec.
    encoded := NewArray(cast(s64) meshopt_encodeIndexBufferBound(index_count, unique), u8);
    encoded_size := meshopt_encodeIndexBuffer(encoded.data, cast(u64) encoded.count, optimized.data, index_count);
    assert(encoded_size > 0 && encoded_size < index_count * 4 / 2, "% bytes for % indices", encoded_size, index_count);
    decoded := NewArray(soup.count, u32);
    assert(meshopt_decodeIndexBuffer(decoded.data, index_count, 4, encoded.data, encoded_size) == 0);
    // The codec keeps the triangle order but may rotate the corners within a triangle.
    decoded_keys := triangle_keys(decoded);
    for before_keys assert(decoded_keys[it_index] == it, "index codec changed the triangles at %", it_index);
    corrupt := NewArray(cast(s64) encoded_size, u8);
    memcpy(corrupt.data, encoded.data, cast(s64) encoded_size);
    corrupt[0] = 0;
    assert(meshopt_decodeIndexBuffer(decoded.data, index_count, 4, corrupt.data, encoded_size) != 0, "a bad header is refused");

    // Vertex buffer codec.
    vencoded := NewArray(cast(s64) meshopt_encodeVertexBufferBound(unique, size_of(Vertex)), u8);
    vsize := meshopt_encodeVertexBuffer(vencoded.data, cast(u64) vencoded.count, vertices.data, unique, size_of(Vertex));
    assert(vsize > 0 && vsize < unique * size_of(Vertex), "% bytes for % vertices", vsize, unique);
    vdecoded := NewArray(cast(s64) unique, Vertex);
    assert(meshopt_decodeVertexBuffer(vdecoded.data, unique, size_of(Vertex), vencoded.data, vsize) == 0);
    for vertices assert(vdecoded[it_index].x == it.x && vdecoded[it_index].y == it.y && vdecoded[it_index].z == it.z, "vertex codec differs at %", it_index);

    print("ok\n");
}
