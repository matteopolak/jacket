#import "Hash";

Payload :: struct {
    value: u64;
}

Wrapped :: #type,isa Payload;

main :: () {
    key: Wrapped;
    get_hash(key);
}
