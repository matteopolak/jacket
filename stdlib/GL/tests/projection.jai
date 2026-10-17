// Root-reviewed public projection; no graphics operation for address identity.
#import "Basic";
G :: #import "GL";
using #import "GL";

main :: () {
    procedure := glTexParameteri;
    if !procedure exit(2);
    if *G.glCreateShader != *G.gl.glCreateShader exit(3);
    if G.glCreateShader != G.gl.glCreateShader exit(4);

    write_string("ok\n");
}
