// Native SDK field assignments are the independent ABI oracle.
#import "Basic";
S :: #import "Socket";
F :: #import "File";
P :: #import "String";

main :: () {
    #if OS == .MACOS {
        filename := sprint("%/../../../target/portable-socket-fields/vectors.bin", P.path_strip_filename(#file));

        defer free(filename);
        bytes, loaded := F.read_entire_file(filename);

        assert(loaded && bytes.count == 16 + 256 * (size_of(S.tcphdr) + size_of(S.tcp_connection_info)));
        defer free(bytes);
        assert(read_word(bytes.data) == 0x54435031 && read_word(bytes.data + 4) == 256);
        assert(read_word(bytes.data + 8) == size_of(S.tcphdr) && read_word(bytes.data + 12) == size_of(S.tcp_connection_info));

        for i: 0..255 {
            scaled := cast,truncate(u32) (cast(u64) i * 0x01054321);
            pattern := scaled ^ 0xa5c31f00;
            packet: S.tcphdr;

            memset(*packet, 0xa5, size_of(S.tcphdr));
            S.set_th_off(*packet, pattern);
            S.set_th_x2(*packet, ~pattern);
            assert(S.get_th_off(packet) == (pattern & 15) && S.get_th_x2(packet) == ((~pattern) & 15));
            info: S.tcp_connection_info;

            memset(*info, 0xa5, size_of(S.tcp_connection_info));
            old_8 := info.__bitfield;

            S.set_tcpi_tfo_cookie_invalid(*info, (pattern & 256) != 0);
            assert(S.get_tcpi_tfo_cookie_invalid(info) == ((pattern & 256) != 0));
            assert(((old_8 ^ info.__bitfield) & ~cast(u32) 256) == 0);
            old_1 := info.__bitfield;

            S.set_tcpi_tfo_cookie_rcv(*info, (pattern & 2) != 0);
            assert(S.get_tcpi_tfo_cookie_rcv(info) == ((pattern & 2) != 0));
            assert(((old_1 ^ info.__bitfield) & ~cast(u32) 2) == 0);
            old_0 := info.__bitfield;

            S.set_tcpi_tfo_cookie_req(*info, (pattern & 1) != 0);
            assert(S.get_tcpi_tfo_cookie_req(info) == ((pattern & 1) != 0));
            assert(((old_0 ^ info.__bitfield) & ~cast(u32) 1) == 0);
            old_6 := info.__bitfield;

            S.set_tcpi_tfo_cookie_req_rcv(*info, (pattern & 64) != 0);
            assert(S.get_tcpi_tfo_cookie_req_rcv(info) == ((pattern & 64) != 0));
            assert(((old_6 ^ info.__bitfield) & ~cast(u32) 64) == 0);
            old_7 := info.__bitfield;

            S.set_tcpi_tfo_cookie_sent(*info, (pattern & 128) != 0);
            assert(S.get_tcpi_tfo_cookie_sent(info) == ((pattern & 128) != 0));
            assert(((old_7 ^ info.__bitfield) & ~cast(u32) 128) == 0);
            old_9 := info.__bitfield;

            S.set_tcpi_tfo_cookie_wrong(*info, (pattern & 512) != 0);
            assert(S.get_tcpi_tfo_cookie_wrong(info) == ((pattern & 512) != 0));
            assert(((old_9 ^ info.__bitfield) & ~cast(u32) 512) == 0);
            old_11 := info.__bitfield;

            S.set_tcpi_tfo_heuristics_disable(*info, (pattern & 2048) != 0);
            assert(S.get_tcpi_tfo_heuristics_disable(info) == ((pattern & 2048) != 0));
            assert(((old_11 ^ info.__bitfield) & ~cast(u32) 2048) == 0);
            old_10 := info.__bitfield;

            S.set_tcpi_tfo_no_cookie_rcv(*info, (pattern & 1024) != 0);
            assert(S.get_tcpi_tfo_no_cookie_rcv(info) == ((pattern & 1024) != 0));
            assert(((old_10 ^ info.__bitfield) & ~cast(u32) 1024) == 0);
            old_14 := info.__bitfield;

            S.set_tcpi_tfo_onebyte_proxy(*info, (pattern & 16384) != 0);
            assert(S.get_tcpi_tfo_onebyte_proxy(info) == ((pattern & 16384) != 0));
            assert(((old_14 ^ info.__bitfield) & ~cast(u32) 16384) == 0);
            old_13 := info.__bitfield;

            S.set_tcpi_tfo_recv_blackhole(*info, (pattern & 8192) != 0);
            assert(S.get_tcpi_tfo_recv_blackhole(info) == ((pattern & 8192) != 0));
            assert(((old_13 ^ info.__bitfield) & ~cast(u32) 8192) == 0);
            old_12 := info.__bitfield;

            S.set_tcpi_tfo_send_blackhole(*info, (pattern & 4096) != 0);
            assert(S.get_tcpi_tfo_send_blackhole(info) == ((pattern & 4096) != 0));
            assert(((old_12 ^ info.__bitfield) & ~cast(u32) 4096) == 0);
            old_4 := info.__bitfield;

            S.set_tcpi_tfo_syn_data_acked(*info, (pattern & 16) != 0);
            assert(S.get_tcpi_tfo_syn_data_acked(info) == ((pattern & 16) != 0));
            assert(((old_4 ^ info.__bitfield) & ~cast(u32) 16) == 0);
            old_5 := info.__bitfield;

            S.set_tcpi_tfo_syn_data_rcv(*info, (pattern & 32) != 0);
            assert(S.get_tcpi_tfo_syn_data_rcv(info) == ((pattern & 32) != 0));
            assert(((old_5 ^ info.__bitfield) & ~cast(u32) 32) == 0);
            old_3 := info.__bitfield;

            S.set_tcpi_tfo_syn_data_sent(*info, (pattern & 8) != 0);
            assert(S.get_tcpi_tfo_syn_data_sent(info) == ((pattern & 8) != 0));
            assert(((old_3 ^ info.__bitfield) & ~cast(u32) 8) == 0);
            old_2 := info.__bitfield;

            S.set_tcpi_tfo_syn_loss(*info, (pattern & 4) != 0);
            assert(S.get_tcpi_tfo_syn_loss(info) == ((pattern & 4) != 0));
            assert(((old_2 ^ info.__bitfield) & ~cast(u32) 4) == 0);
            S.set___pad2(*info, pattern);
            assert(S.get___pad2(info) == (pattern & 0x1ffff));
            expected := bytes.data + 16 + i * (size_of(S.tcphdr) + size_of(S.tcp_connection_info));
            for b: 0..size_of(S.tcphdr) - 1 assert((cast(*u8) *packet)[b] == expected[b]);

            expected += size_of(S.tcphdr);
            for b: 0..size_of(S.tcp_connection_info) - 1 assert((cast(*u8) *info)[b] == expected[b]);
        }

        print("ok\n");
    } else {
        assert(false, "Apple native field fixture requires macOS");
    }
}

#scope_file
read_word :: (p: *u8) -> u32 {
    result: u32;

    for i: 0..3 {
        part := cast(u32) p[i];
        result |= part << (i * 8);
    }

    return result;
}
