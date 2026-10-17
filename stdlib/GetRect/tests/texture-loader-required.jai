// Texture creation requires the caller's actual resource loader.
#import "GetRect";

main :: () {
    require_texture("caller-provided pixels");
}
