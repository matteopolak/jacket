get_shader_program :: (shader_text: string) -> GLuint {
    require_gl_texture_context();
    vertex := compile_stage(shader_text, GL.GL_VERTEX_SHADER, "#version 330 core\n#define VERTEX_SHADER\n#define OUT_IN out\n");
    if vertex == 0 return 0;

    defer GL.gl.glDeleteShader(vertex);
    fragment := compile_stage(shader_text, GL.GL_FRAGMENT_SHADER, "#version 330 core\n#define FRAGMENT_SHADER\n#define OUT_IN in\n");
    if fragment == 0 return 0;

    defer GL.gl.glDeleteShader(fragment);
    program := GL.gl.glCreateProgram();
    if program == 0 return 0;

    GL.gl.glAttachShader(program, vertex);
    GL.gl.glAttachShader(program, fragment);
    GL.gl.glLinkProgram(program);
    status: s32;

    GL.gl.glGetProgramiv(program, GL.GL_LINK_STATUS, *status);

    if status == 0 {
        log_shader_diagnostic(program, false);
        GL.gl.glDeleteProgram(program);
        return 0;
    }

    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL shader linkage operation failed");

    return program;
}

backend_use_shader :: (s: *Shader) {
    require_gl_texture_context();
    GL.gl.glUseProgram(s.gl_handle);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL shader selection failed");
}

backend_update_shader_parameter_locations :: (s: *Shader) {
    require_gl_texture_context();
    s.position_loc = GL._glGetAttribLocation(s.gl_handle, "position");
    s.color_scale_loc = GL._glGetAttribLocation(s.gl_handle, "color_scale");
    s.normal_loc = GL._glGetAttribLocation(s.gl_handle, "normal");
    s.tangent_loc = GL._glGetAttribLocation(s.gl_handle, "tangent");
    s.uv_0_loc = GL._glGetAttribLocation(s.gl_handle, "uv_0");
    s.uv_1_loc = GL._glGetAttribLocation(s.gl_handle, "uv_1");
    s.diffuse_texture_loc = GL._glGetUniformLocation(s.gl_handle, "diffuse_texture");
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL uniform query failed");
}

set_parameter :: (shader: *Shader, name: *u8, m: Matrix4) {
    require_gl_texture_context();
    location := GL.gl.glGetUniformLocation(shader.gl_handle, name);
    GL.gl.glUniformMatrix4fv(location, 1, false, cast(*float) *m);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL matrix uniform update failed");
}

#scope_module
compile_stage :: (source: string, stage: u32, preamble: string) -> u32 {
    shader := GL.gl.glCreateShader(stage);
    if shader == 0 return 0;

    parts := string.[preamble, source];

    GL._glShaderSource(shader, parts);
    GL.gl.glCompileShader(shader);
    status: s32;

    GL.gl.glGetShaderiv(shader, GL.GL_COMPILE_STATUS, *status);

    if status == 0 {
        log_shader_diagnostic(shader, true);
        GL.gl.glDeleteShader(shader);
        return 0;
    }

    return shader;
}

log_shader_diagnostic :: (object: u32, shader: bool) {
    capacity: s32;
    if shader GL.gl.glGetShaderiv(object, GL.GL_INFO_LOG_LENGTH, *capacity);
    else GL.gl.glGetProgramiv(object, GL.GL_INFO_LOG_LENGTH, *capacity);

    if capacity <= 0 {
        context.logger("OpenGL compilation/linkage failed without a driver log\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return;
    }

    bytes := cast(*u8) Basic.alloc(capacity);

    defer Basic.free(bytes);
    written: GL.GLsizei;
    if shader GL.gl.glGetShaderInfoLog(object, cast(GL.GLsizei) capacity, *written, bytes);
    else GL.gl.glGetProgramInfoLog(object, cast(GL.GLsizei) capacity, *written, bytes);

    context.logger(string.{ cast(int) written, bytes }, context.logger_data, Log_Info.{ common_flags = .ERROR });
}
