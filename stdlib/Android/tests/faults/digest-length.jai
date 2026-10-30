// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") should_fail("SHA input exceeds the 64-bit bit-length domain")
// Independent signed-extent safety, not a reference invalid-input guarantee.
A :: #import "Android/Toolchain";

main :: () {
    invalid: string = .{ -1, null };
    A.sha1(invalid);
}
