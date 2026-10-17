// Independent signed-extent safety, not a reference invalid-input guarantee.
A :: #import "Android/Toolchain";

main :: () {
    invalid: string = .{ -1, null };
    A.sha1(invalid);
}
