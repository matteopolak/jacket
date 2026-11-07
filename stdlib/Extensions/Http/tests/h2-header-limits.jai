// test: skip_on(.WASM, reason = "native HTTP/2 scheduler")
// A compressed block must be rejected as a whole without losing HPACK state.
#load "../module.jai";

check_limit :: (server: bool, trailers: bool, count_limit: bool) -> bool {
    conn: H2_Conn;
    rejected: H2_Stream;
    sibling: H2_Stream;
    encoder: Hpack_Encoder;

    conn.role = ifx server then .SERVER else .CLIENT;
    conn.next_stream = 5;
    conn.last_peer_stream = 3;
    conn.max_header_list = ifx count_limit then 8192 else 256;
    conn.max_headers = ifx count_limit then 2 else 100;
    if server conn.open_remote = 2;
    else conn.open_local = 2;

    hpack_decoder_init(*conn.decoder);
    hpack_encoder_init(*encoder);
    defer hpack_decoder_free(*conn.decoder);
    defer hpack_encoder_free(*encoder);
    defer array_reset(*conn.streams);
    defer array_reset(*conn.out);
    rejected.id = 1;
    sibling.id = 3;
    rejected.conn = *conn;
    sibling.conn = *conn;
    rejected.refs = 2;
    sibling.refs = 2;
    rejected.has_head = trailers;
    sibling.has_head = trailers;
    array_add(*conn.streams, *rejected);
    array_add(*conn.streams, *sibling);
    defer hpack_block_free(*rejected.head);
    defer hpack_block_free(*rejected.trailers);
    defer hpack_block_free(*sibling.head);
    defer hpack_block_free(*sibling.trailers);
    block: [..] u8;

    defer array_reset(*block);
    hpack_encode_start(*encoder, *block);
    if trailers hpack_encode_field(*encoder, *block, "grpc-status", "0");
    else hpack_encode_field(*encoder, *block, ":status", "200");
    for 0..19 hpack_encode_field(*encoder, *block, "x-fill", "01234567890123456789012345678901");

    // This insertion follows the decoded limit. The next stream references it.
    hpack_encode_field(*encoder, *block, "x-sync", "after-limit");
    assert(block.count < 256, "small wire block exceeds decoded-list limit");
    error := h2_headers_complete(*conn, 1, block, trailers);
    refused := error == .NO_ERROR && rejected.reset_sent && rejected.reset_code == cast(u32) H2_Error.ENHANCE_YOUR_CALM;
    refused = refused && rejected.trailers.fields.count == 0 && (trailers || !rejected.has_head);
    block.count = 0;
    hpack_encode_start(*encoder, *block);
    if trailers hpack_encode_field(*encoder, *block, "grpc-status", "0");
    else hpack_encode_field(*encoder, *block, ":status", "200");

    hpack_encode_field(*encoder, *block, "x-sync", "after-limit");
    assert(block.count < 8, "sibling uses the updated dynamic table");
    error = h2_headers_complete(*conn, 3, block, trailers);
    fields := ifx trailers then sibling.trailers.fields else sibling.head.fields;
    healthy := error == .NO_ERROR && !conn.dead && !sibling.reset_sent && fields.count == 2;

    if healthy healthy = fields[1].name == "x-sync" && fields[1].value == "after-limit";

    print("server=% trailers=% count_limit=% refused=% sibling=%\n", server, trailers, count_limit, refused, healthy);

    return refused && healthy;
}

main :: () {
    entry :: (_: *void) {
        failures := 0;
        if !check_limit(false, false, false) failures += 1;
        if !check_limit(false, true, false) failures += 1;
        if !check_limit(true, true, false) failures += 1;
        if !check_limit(true, true, true) failures += 1;

        assert(failures == 0, "all oversized blocks are refused and siblings survive");
    }

    Io.io_run(entry, null);
}
