#import "Basic";
#import "String";

// A global whose type spells a mapped name is sized while the mapper still has to run.
Peer :: struct {
    address: *SDLNet_Address;
    count: s32;
}

peer: Peer;
using peer;

using,map(prefix_net) Net :: #import "Net_Pack";

prefix_net :: (names: [] string) {
    for *names {
        if begins_with(it.*, "NET_") it.* = tprint("SDLNet_%", slice(it.*, 4, it.*.count - 4));
    }
}

main :: () {
    address := SDLNet_Address.{ port = 8080 };
    peer.address = *address;
    assert(SDLNet_port_of(peer.address) == 8080, "mapped names resolve in types declared before the import");
    print("ok\n");
}
