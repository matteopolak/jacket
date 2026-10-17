#load "fault-setup.jai";

#assert invalid_index(0, true, true);
#assert invalid_index(1, true, true);

main :: () {
    assert(invalid_index(0, true, true) && invalid_index(1, true, true));
    print("ok\n");
}
