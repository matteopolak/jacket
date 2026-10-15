// Independent general compiler regression, no platform/module dependencies.
Packet :: struct {
    #overlay (storage) alternate: *void;
    storage: *u64;
    discriminator: u32;
}

#assert offset_of(Packet, "alternate") == 0;
#assert offset_of(Packet, "storage") == 0;
#assert offset_of(Packet, "discriminator") == 8;
#assert size_of(Packet) == 16;

libc :: #library,system "c";
abort :: () #foreign libc;

main :: () {
    payload: u64 = 19;
    packet: Packet;
    packet.storage = *payload;
    if packet.alternate != *payload abort();

    packet.alternate = null;
    if packet.storage != null abort();
}
