// Reviewed macOS network database shapes, canonical names and real SDK calls.
using #import "Basic";
S :: #import "Socket";

main :: () {
    h: S.hostent;
    n: S.netent;
    p: S.protoent;
    s: S.servent;
    r: S.rpcent;

    assert(type_info(S.hostent).name == "hostent");
    assert(type_info(S.netent).name == "netent");
    assert(type_info(S.protoent).name == "protoent");
    assert(type_info(S.servent).name == "servent");
    assert(type_info(S.rpcent).name == "rpcent");
    assert(type_of(h.h_addrtype) == s32 && type_of(h.h_length) == s32);
    assert(type_of(n.n_addrtype) == s32 && type_of(n.n_net) == u32);
    assert(type_of(p.p_proto) == s32 && type_of(s.s_port) == s32 && type_of(r.r_number) == s32);
    print("hostent % % % % % %\n", size_of(S.hostent), offset(*h, *h.h_name), offset(*h, *h.h_aliases), offset(*h, *h.h_addrtype), offset(*h, *h.h_length), offset(*h, *h.h_addr_list));
    print("netent % % % % %\n", size_of(S.netent), offset(*n, *n.n_name), offset(*n, *n.n_aliases), offset(*n, *n.n_addrtype), offset(*n, *n.n_net));
    print("protoent % % % %\n", size_of(S.protoent), offset(*p, *p.p_name), offset(*p, *p.p_aliases), offset(*p, *p.p_proto));
    print("servent % % % % %\n", size_of(S.servent), offset(*s, *s.s_name), offset(*s, *s.s_aliases), offset(*s, *s.s_port), offset(*s, *s.s_proto));
    print("rpcent % % % %\n", size_of(S.rpcent), offset(*r, *r.r_name), offset(*r, *r.r_aliases), offset(*r, *r.r_number));
    host := S.gethostbyname(temp_c_string("127.0.0.1"));

    assert(host != null && host.h_addrtype == S.AF_INET && host.h_length == 4);
    assert(host.h_addr_list != null && host.h_addr_list[0] != null);
    bytes := host.h_addr_list[0];

    assert(bytes[0] == 127 && bytes[1] == 0 && bytes[2] == 0 && bytes[3] == 1);
    protocol := S.getprotobyname(temp_c_string("tcp"));

    assert(protocol != null && protocol.p_proto == cast(s32) S.IPPROTO.TCP);
    service := S.getservbyname(temp_c_string("http"), temp_c_string("tcp"));

    assert(service != null && S.ntohs(cast(u16) service.s_port) == 80);
    assert(to_string(service.s_proto) == "tcp");
}

#scope_file
offset :: (record: *void, field: *void) -> u64 {
    return cast(u64) field - cast(u64) record;
}
