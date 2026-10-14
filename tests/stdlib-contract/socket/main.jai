// A zero IPv4 endpoint has stable numeric formatting independent of DNS.
#import "Basic";
Socket :: #import "Socket";

main :: () {
    #if OS_IS_UNIX {
        address: Socket.sockaddr_in = .{};
        address.sin_port = 0;
        address.sin_addr.s_addr = 0;
        rendered := Socket.to_string(address);
        assert(rendered == "0.0.0.0:0");
    }
    #if OS == .WINDOWS || OS == .LINUX || OS == .MACOS {
        address6: Socket.sockaddr_in6 = .{};
        assert(Socket.to_string(address6) == "0:0:0:0:0:0:0:0:0");
    }
    print("ok\n");
}
