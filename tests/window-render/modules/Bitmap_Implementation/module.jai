// Exercise the exact CPU implementation and canonical image declaration files
// without requiring unresolved native window class dependencies in other Simp records.
#load "../../../../stdlib/Simp/image-types.jai";
#load "../../../../stdlib/internal/rendering/images.jai";
#load "../../../../stdlib/internal/rendering/image-codecs.jai";

#scope_module
Basic :: #import "Basic";

assert :: Basic.assert;

Image :: #import "stb_image";
Image_Write :: #import "stb_image_write";
