// load_asset in the browser: the page fetches the path relative to its asset base.
#import "Basic";
#import "Extensions/Assets";

main :: () {
    bytes, ok := load_asset("textures/a.bin");

    print("found % %\n", ok, bytes.count);
    if ok print("first % last %\n", bytes[0], bytes[bytes.count - 1]);

    _, missing := load_asset("textures/missing.bin");

    print("missing %\n", missing);
    _, escaped := load_asset("../secret.bin");

    print("escaped %\n", escaped);
}
