// The client side of HTTP/2.

h2_client_headers :: (conn: *H2_Conn, id: u32, stream: *H2_Stream, block: Hpack_Block, end_stream: bool) -> H2_Error {
    return .PROTOCOL_ERROR;
}
