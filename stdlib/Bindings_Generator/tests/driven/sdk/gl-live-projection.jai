// Reviewed using-import projection uses the live canonical procedure table.
using #import "GL";
GL :: #import "GL";

libc :: #library,system "c";
abort :: () #foreign libc;
observed: u32;

first :: (texture: u32) #c_call {
    observed = texture;
}

second :: (texture: u32) #c_call {
    observed = texture + 1;
}

check :: (condition: bool) {
    if !condition abort();
}

main :: () {
    // Take the SDK procedure address; no graphics context or SDK call needed.
    sdk_procedure := glTexParameteri;

    check(cast(*void) sdk_procedure != null);
    check(cast(*void) *GL.gl == cast(*void) *gl);
    GL.gl.glActiveTexture = first;
    glActiveTexture(71);
    check(observed == 71);
    glActiveTexture = second;
    GL.gl.glActiveTexture(73);
    check(observed == 74);
    check(cast(*void) *GL.gl.glActiveTexture == cast(*void) *glActiveTexture);
    GL.gl = .{};
    check(cast(*void) glActiveTexture == null);
    GL.gl.glActiveTexture = first;
    glActiveTexture(47);
    check(observed == 47);
}
