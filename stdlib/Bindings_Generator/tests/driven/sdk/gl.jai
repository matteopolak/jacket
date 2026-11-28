G :: #import "GL";

libc :: #library,system "c";

#import "internal/libc";

abort :: c_abort;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    major, minor := G.gl_parse_version("GL_VERSION_4_6");

    check(major == 4 && minor == 6);
    major, minor = G.gl_parse_version("4.6");
    check(major == 0 && minor == 0);
    major, minor = G.gl_parse_version("GL_VERSION_4_");
    check(major == 0 && minor == 0);
    check(!G.DumpGLErrors("disabled diagnostics"));
}
