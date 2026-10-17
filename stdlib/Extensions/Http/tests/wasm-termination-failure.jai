// An independent failed check must use the real support counter and exit path.
#load "support/support.jai";

main :: () {
    check(false, "independent WASM termination control");
    assert(failures == 1);
    finish();
    print("ERROR: execution continued after failed finish\n");
}
