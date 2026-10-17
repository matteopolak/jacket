// Independent Python hashlib fixtures; public reviewed success/ownership contract.
#import "Basic";
F :: #import "File";
A :: #import "Android/Toolchain";

Tracker :: struct {
    upstream: Allocator;
    live: s64;
}

tracking_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    tracker := cast(*Tracker) allocator_data;
    result := tracker.upstream.proc(mode, size, old_size, old_memory, tracker.upstream.data);

    if mode == .ALLOCATE && result != null tracker.live += 1;
    if mode == .FREE && old_memory != null tracker.live -= 1;

    return result;
}

main :: () {
    root := FIXTURE_ROOT;

    assert(root.count > 0);
    path := tprint("%/vectors.bin", root);
    data, ok := F.read_entire_file(path);

    assert(ok);
    defer free(data);
    cursor := data;
    count := A.read(u32, *cursor);

    for 0..cast(s64) count - 1 {
        password := A.read_length_prefixed_block(*cursor);
        plain := A.read_length_prefixed_block(*cursor);
        protected := A.read_length_prefixed_block(*cursor);
        wrapped := A.read_length_prefixed_block(*cursor);
        store := A.read_length_prefixed_block(*cursor);
        expected_certificate := A.read_length_prefixed_block(*cursor);
        key_offset := A.read(u64, *cursor);
        certificate_offset := A.read(u64, *cursor);
        success, certificate, encrypted := A.jks_read(store, password);

        assert(success && certificate == expected_certificate && encrypted == wrapped);
        assert(encrypted.data == store.data + key_offset);
        assert(certificate.data == store.data + certificate_offset);
        tracker := Tracker.{ upstream = context.allocator };

        context.allocator = .{ tracking_allocator, *tracker };
        decrypted_ok, decrypted := A.jdk_decrypt_private_key(protected, password);

        context.allocator = tracker.upstream;
        assert(decrypted_ok && decrypted == plain);
        if plain.count > 0 assert(decrypted.data != plain.data && decrypted.data != protected.data + 20);
        if plain.count > 0 assert(tracker.live == 1);

        context.allocator = .{ tracking_allocator, *tracker };
        free(decrypted);
        decrypted_ok, decrypted = A.decrypt_private_key(wrapped, password);
        context.allocator = tracker.upstream;
        assert(decrypted_ok && decrypted == plain);
        if plain.count > 0 assert(tracker.live == 1);

        context.allocator = .{ tracking_allocator, *tracker };
        free(decrypted);
        context.allocator = tracker.upstream;
        assert(tracker.live == 0);
    }

    assert(cursor.count == 0);
    v1, loaded_v1 := F.read_entire_file(tprint("%/version1.jks", root));

    assert(loaded_v1);
    defer free(v1);
    success, certificate, encrypted := A.jks_read(v1, "password");

    assert(success && certificate.count == 0 && encrypted == "opaque key");
    print("ok\n");
}
