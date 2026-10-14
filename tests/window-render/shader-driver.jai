#import "Basic";
S :: #import "Simp";
GL :: #import "GL";
W :: #import "Window_Creation";

probe :: #library "window_gl_probe";
begin_context :: () -> *void #foreign probe "window_gl_probe_begin";
end_context :: (context: *void) #foreign probe "window_gl_probe_end";
diagnostics: int;

capture_diagnostic :: (text: string, #discard data: *void, info: Log_Info) {
    assert(text.count > 0 && (info.common_flags & .ERROR) != 0);
    diagnostics += 1;
}

main :: () {
    W.init_mac_app();
    native := begin_context();

    assert(native != null);
    defer end_context(native);
    source := "OUT_IN vec4 color;\n#ifdef VERTEX_SHADER\nvoid main(){gl_Position=vec4(0,0,0,1);color=vec4(1);}\n#endif\n#ifdef FRAGMENT_SHADER\nout vec4 result;\nvoid main(){result=color;}\n#endif\n";
    program := S.get_shader_program(source);

    assert(program != 0);
    defer GL.gl.glDeleteProgram(program);
    linked: s32;

    GL.gl.glGetProgramiv(program, GL.GL_LINK_STATUS, *linked);
    assert(linked != 0);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
    scoped := context;
    scoped.logger = capture_diagnostic;

    {
        push_context, defer_pop scoped;
        assert(S.get_shader_program("this is deliberately invalid GLSL") == 0);
        incompatible := "#ifdef VERTEX_SHADER\nout vec2 mismatch;\nvoid main(){gl_Position=vec4(0);mismatch=vec2(1);}\n#else\nin vec3 mismatch; out vec4 result;\nvoid main(){result=vec4(mismatch,1);}\n#endif\n";
        assert(S.get_shader_program(incompatible) == 0);
    }

    assert(diagnostics == 2 && GL.gl.glGetError() == GL.GL_NO_ERROR);
    print("ok\n");
}
