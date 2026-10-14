// The actual native face implementation needs no window or renderer substitution.
#load "../../../../stdlib/Simp/loaded-font-types.jai";
#load "../../../../stdlib/internal/rendering/loaded-fonts.jai";

#scope_module
#import "freetype";
Basic :: #import "Basic";

assert :: Basic.assert;

File :: #import "File";
