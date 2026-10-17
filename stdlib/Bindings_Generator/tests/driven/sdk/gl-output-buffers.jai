// Source-level forwarding regression. Runtime SDK coverage is separate.
G :: #import "GL";

main :: () {
    buffer: [64] u8;
    name: string = .{ data = buffer.data, count = buffer.count };
    length: G.GLsizei;
    size: G.GLint;
    type: G.GLenum;

    G._glGetActiveAttrib(1, 0, cast(G.GLsizei) buffer.count, *length, *size, *type, name);
    G._glGetActiveUniform(1, 0, cast(G.GLsizei) buffer.count, *length, *size, *type, name);
}
