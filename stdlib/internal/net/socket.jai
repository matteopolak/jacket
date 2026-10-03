#if OS == .WINDOWS #load "socket-windows.jai";
else #load "socket-posix.jai";

append_address :: (builder: *String_Builder, address: u32) {
    bytes := cast(*u8) *address;

    for i: 0..3 {
        if i > 0 append(builder, cast(u8) 46);
        text := sprint("%", bytes[i]);
        defer free(text);
        append(builder, text);
    }
}

append_address :: (builder: *String_Builder, address: [16] u8) {
    for i: 0..7 {
        if i > 0 append(builder, cast(u8) 58);
        value := (cast(u32) address[i * 2]) * 256 + cast(u32) address[i * 2 + 1];
        append_hex(builder, value);
    }
}

to_string :: (addr: ABI.sockaddr_in) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append_address(*builder, read_ipv4(*addr.sin_addr));
    append(*builder, cast(u8) 58);
    port := sprint("%", host_port(addr.sin_port));

    defer free(port);
    append(*builder, port);

    return builder_to_string(*builder);
}

to_string :: (addr: ABI.sockaddr_in6) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    bytes: [16] u8;

    memcpy(bytes.data, *addr.sin6_addr, 16);
    append_address(*builder, bytes);
    append(*builder, cast(u8) 58);
    port := sprint("%", host_port(addr.sin6_port));

    defer free(port);
    append(*builder, port);

    return builder_to_string(*builder);
}

accept :: (s: Socket) -> Socket, ABI.sockaddr_in {
    addr: ABI.sockaddr_in;
    size: ABI.socklen_t = size_of(ABI.sockaddr_in);
    socket := ABI.accept(s, cast(*ABI.sockaddr) *addr, *size);
    return socket, addr;
}

accept_v6 :: (s: Socket) -> Socket, ABI.sockaddr_in6 {
    addr: ABI.sockaddr_in6;
    size: ABI.socklen_t = size_of(ABI.sockaddr_in6);
    socket := ABI.accept(s, cast(*ABI.sockaddr) *addr, *size);
    return socket, addr;
}

bind :: (s: Socket, address: string, port: u16, family: type_of(ABI.sockaddr_in.sin_family) = ABI.AF_INET) -> s32 {
    cname := to_c_string(address);

    defer free(cname);

    if family == ABI.AF_INET {
        addr: ABI.sockaddr_in;
        if ABI.inet_pton(ABI.AF_INET, cname, *addr.sin_addr) != 1 return -1;
        return bind(s, read_ipv4(*addr.sin_addr), port);
    }

    if family == ABI.AF_INET6 {
        bytes: [16] u8;
        if ABI.inet_pton(ABI.AF_INET6, cname, bytes.data) != 1 return -1;
        return bind(s, bytes, port);
    }

    return -1;
}

bind :: (s: Socket, address: u32, port: u16) -> s32 {
    addr: ABI.sockaddr_in;

    #if OS == .MACOS addr.sin_len = size_of(ABI.sockaddr_in);
    addr.sin_family = ABI.AF_INET;
    addr.sin_port = host_port(port);
    write_ipv4(*addr.sin_addr, address);

    return ABI.bind(s, cast(*ABI.sockaddr) *addr, size_of(ABI.sockaddr_in));
}

bind :: (s: Socket, address: [16] u8, port: u16) -> s32 {
    addr: ABI.sockaddr_in6;

    #if OS == .MACOS addr.sin6_len = size_of(ABI.sockaddr_in6);
    addr.sin6_family = ABI.AF_INET6;
    addr.sin6_port = host_port(port);
    memcpy(*addr.sin6_addr, address.data, 16);

    return ABI.bind(s, cast(*ABI.sockaddr) *addr, size_of(ABI.sockaddr_in6));
}

#scope_module
#import "Basic";
ABI :: #import "internal/net/socket-abi";

host_port :: (value: u16) -> u16 {
    return (value >> 8) | (value << 8);
}

append_hex :: (builder: *String_Builder, value: u32) {
    bytes: [8] u8;
    cursor: s64 = 8;
    n := value;

    while true {
        cursor -= 1;
        bytes[cursor] = "0123456789abcdef"[n & 15];
        n >>= 4;
        if n == 0 break;
    }

    append(builder, bytes.data + cursor, 8 - cursor);
}

report_error :: () {
    code := get_last_socket_error();
    text := sprint("socket error %\n", cast(s32) code);
    defer free(text);
    context.logger(text, context.logger_data, Log_Info.{ common_flags = .ERROR });
}

read_ipv4 :: (address: *ABI.in_addr) -> u32 {
    #if OS == .WINDOWS return address.S_un.S_addr;
    else return address.s_addr;
}

write_ipv4 :: (address: *ABI.in_addr, value: u32) {
    #if OS == .WINDOWS address.S_un.S_addr = value;
    else address.s_addr = value;
}
