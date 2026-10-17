// Canonical dependency check, independent of GetRect and rendering callbacks.
S :: #import "Simp";
#import "Basic";

main :: () {
    texture: S.Texture;
    assert(texture.width == 1);
}
