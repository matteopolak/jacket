#import "Basic";
#import "Store";

verify :: () -> bool {
    values: [3] s64;
    values[0] = 7;
    holder: Holder(s64, 1024);
    holder.storage = *values[0];
    before := holder[0];
    address := *holder[0];

    holder[1] = 13;

    return before == 7 && values[1] == 13 && address == *values[0];
}

#assert verify();

main :: () {
    assert(verify());
}
