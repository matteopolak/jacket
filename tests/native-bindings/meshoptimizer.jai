M :: #import "meshoptimizer";

#assert M.MESHOPTIMIZER_VERSION == 110;
#assert size_of(M.meshopt_Stream) == 24;
#assert size_of(M.meshopt_Meshlet) == 636;
#assert size_of(M.meshopt_Bounds) == 48;
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

probe :: #library "../../.work/native/abi_probe";
check_size :: (which: s32) -> u64 #foreign probe "binding_size";

main :: () {
    check(type_info(M.meshopt_Stream).name == "meshopt_Stream");
    check(size_of(M.meshopt_Stream) == check_size(7));
    check(size_of(M.meshopt_Meshlet) == check_size(8));
    check(size_of(M.meshopt_Bounds) == check_size(9));
    vertices: [12] float = .[0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0];
    indices: [6] u32 = .[0, 1, 2, 3, 1, 2];
    remap: [4] u32;
    unique := M.meshopt_generateVertexRemap(remap.data, indices.data, 6, vertices.data, 4, 12);

    check(unique == 3 && remap[0] == remap[3]);
    remapped: [6] u32;

    M.meshopt_remapIndexBuffer(remapped.data, indices.data, 6, remap.data);
    check(remapped[0] == remapped[3]);
    encoded: [256] u8;
    n := M.meshopt_encodeIndexBuffer(encoded.data, 256, indices.data, 6);

    check(n > 0 && n <= M.meshopt_encodeIndexBufferBound(6, 4));
    decoded: [6] u32;

    check(M.meshopt_decodeIndexBuffer(decoded.data, 6, 4, encoded.data, n) == 0);
    for i: 0..5 check(indices[i] == decoded[i]);

    invalid := "invalid";

    check(M.meshopt_decodeIndexBuffer(decoded.data, 6, 4, invalid.data, cast(u64) invalid.count) != 0);
    stats := M.meshopt_analyzeVertexCache(indices.data, 6, 4, 16, 32, 0);

    check(stats.vertices_transformed > 0 && stats.acmr > 0);
    bounds := M.meshopt_computeClusterBounds(indices.data, 6, vertices.data, 4, 12);

    check(bounds.radius > 0);
}
