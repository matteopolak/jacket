// Real macOS OpenGL driver test; no GL entry point is replaced.
G :: #import "GL";

probe :: #library,system "apple_abi_probe";
token :: (which: u32) -> u32 #foreign probe "binding_cgl_token";
opengl :: #library,system "OpenGL";
choose_format :: (attributes: *u32, format: **void, count: *s32) -> s32 #foreign opengl "CGLChoosePixelFormat";
create_context :: (format: *void, share: *void, result: **void) -> s32 #foreign opengl "CGLCreateContext";
destroy_format :: (format: *void) -> s32 #foreign opengl "CGLDestroyPixelFormat";
destroy_context :: (value: *void) -> s32 #foreign opengl "CGLDestroyContext";
set_current :: (value: *void) -> s32 #foreign opengl "CGLSetCurrentContext";
libc :: #library,system "c";
abort :: () #foreign libc;
native_printf :: (fmt: *u8, args: ..Any) -> s32 #foreign libc "printf";
native_fflush :: (stream: *void) -> s32 #foreign libc "fflush";
native_strlen :: (text: *u8) -> u64 #foreign libc "strlen";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    attributes: [9] u32;
    attributes[0] = token(0);
    attributes[1] = token(1);
    attributes[2] = token(2);
    attributes[3] = 24;
    attributes[4] = token(3);
    attributes[5] = 8;
    attributes[6] = token(4);
    attributes[7] = 24;
    format: *void;
    count: s32;

    check(choose_format(attributes.data, *format, *count) == 0 && format != null && count > 0);
    defer check(destroy_format(format) == 0);
    native_context: *void;

    check(create_context(format, null, *native_context) == 0 && native_context != null);
    defer check(destroy_context(native_context) == 0);
    check(set_current(native_context) == 0);
    defer check(set_current(null) == 0);

    table := *G.gl;

    for member: type_info(G.GL_Procedures).members {
        if member.name == "glGetStringi" {
            check(native_printf("query member offset=%lld flags=%u kind=%u symbol=%p\n",
                member.offset_in_bytes, cast(u32) member.flags, cast(u32) member.type.type,
                G.nsgl_get_proc_address("glGetStringi")) >= 0);
            check(native_fflush(null) == 0);
            check((member.flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) != 0);
        }
    }

    G.gl_load(table);
    major, minor := G.gl_get_version();

    check(major > 3 || (major == 3 && minor >= 2));
    check(native_printf("Native OpenGL version %d.%d\n", major, minor) >= 0);

    vertex := table.glCreateShader(G.GL_VERTEX_SHADER);
    fragment := table.glCreateShader(G.GL_FRAGMENT_SHADER);

    check(vertex != 0 && fragment != 0);
    defer table.glDeleteShader(vertex);
    defer table.glDeleteShader(fragment);
    G._glShaderSource(vertex, "#version 150\nin vec3 position; uniform mat4 projection; void main() { gl_Position = projection * vec4(position, 1.0); }\n");
    G._glShaderSource(fragment, "#version 150\nout vec4 color; void main() { color = vec4(1.0); }\n");
    table.glCompileShader(vertex);
    table.glCompileShader(fragment);
    compiled: G.GLint;

    table.glGetShaderiv(vertex, G.GL_COMPILE_STATUS, *compiled);
    check(compiled != 0);
    table.glGetShaderiv(fragment, G.GL_COMPILE_STATUS, *compiled);
    check(compiled != 0);
    program := table.glCreateProgram();

    check(program != 0);
    defer table.glDeleteProgram(program);
    table.glAttachShader(program, vertex);
    table.glAttachShader(program, fragment);
    table.glLinkProgram(program);
    table.glGetProgramiv(program, G.GL_LINK_STATUS, *compiled);
    check(compiled != 0);

    bytes: [64] u8;
    buffer: string = .{ data = bytes.data, count = bytes.count };
    length: G.GLsizei;
    size: G.GLint;
    type: G.GLenum;

    G._glGetActiveAttrib(program, 0, cast(G.GLsizei) bytes.count, *length, *size, *type, buffer);
    name: string = .{ data = bytes.data, count = length };

    check(name == "position" && size == 1 && type == G.GL_FLOAT_VEC3);
    G._glGetActiveUniform(program, 0, cast(G.GLsizei) bytes.count, *length, *size, *type, buffer);
    name.count = length;
    check(name == "projection" && size == 1 && type == G.GL_FLOAT_MAT4);
    check(G._glGetUniformLocation(program, "projection") >= 0);
    check(G._glGetAttribLocation(program, "position") >= 0);
    extensions: G.GLint;

    table.glGetIntegerv(G.GL_NUM_EXTENSIONS, *extensions);
    native_arb, native_khr: bool;

    for i: 0..extensions - 1 {
        pointer := table.glGetStringi(G.GL_EXTENSIONS, cast(G.GLuint) i);

        check(pointer != null);
        extension: string = .{ data = pointer, count = cast(s64) native_strlen(pointer) };
        if extension == "GL_ARB_debug_output" native_arb = true;
        if extension == "GL_KHR_debug" native_khr = true;
    }

    check(native_arb == G.gl.GL_ARB_debug_output && native_khr == G.gl.GL_KHR_debug);
    check(G.gl.GL_VERSION_4_3 == (major > 4 || (major == 4 && minor >= 3)));

    if G.gl.GL_VERSION_4_3 || G.gl.GL_KHR_debug || G.gl.GL_ARB_debug_output {
        G.gl_enable_debug_output();
        message := "binding_debug_marker";

        if G.gl.GL_VERSION_4_3 && table.glDebugMessageInsert {
            table.glDebugMessageInsert(G.GL_DEBUG_SOURCE_APPLICATION, G.GL_DEBUG_TYPE_OTHER,
                123, G.GL_DEBUG_SEVERITY_LOW, cast(G.GLsizei) message.count, message.data);
        } else if G.gl.GL_KHR_debug && table.glDebugMessageInsertKHR {
            table.glDebugMessageInsertKHR(G.GL_DEBUG_SOURCE_APPLICATION, G.GL_DEBUG_TYPE_OTHER,
                123, G.GL_DEBUG_SEVERITY_LOW, cast(G.GLsizei) message.count, message.data);
        } else {
            check(table.glDebugMessageInsertARB != null);
            table.glDebugMessageInsertARB(G.GL_DEBUG_SOURCE_APPLICATION, G.GL_DEBUG_TYPE_OTHER,
                123, G.GL_DEBUG_SEVERITY_LOW, cast(G.GLsizei) message.count, message.data);
        }
    } else {
        // Explicitly unavailable runtime evidence; not a passed callback test.
        check(native_printf("SDK_DEBUG_OUTPUT_UNAVAILABLE\n") >= 0);
    }

    check(table.glGetError() == G.GL_NO_ERROR);
}
