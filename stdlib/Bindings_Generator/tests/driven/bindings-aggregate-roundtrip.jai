#import "Basic";
G :: #import "developer-tools-aggregate-generated";

main :: () {
    assert(G.default_packet() == 33);
    packet := G.make_packet();

    assert(packet.first == 11 && packet.second == 22);
    assert(G.default_packet(packet) == 33);
    assert(G.default_packet(.{ 7, 9 }) == 16);
    print("ok\n");
}
