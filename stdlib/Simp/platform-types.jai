#if render_api == .OPENGL {
    Texture_Platform_Specific :: struct {
        gl_handle: GLuint;
    }

    Shader_Platform :: struct {
        gl_handle: GLuint;
        position_loc: GLint = -1;
        color_scale_loc: GLint = -1;
        normal_loc: GLint = -1;
        tangent_loc: GLint = -1;
        uv_0_loc: GLint = -1;
        uv_1_loc: GLint = -1;
        diffuse_texture_loc: GLint = -1;
        alpha_blend := false;
    }

    Pixel_Read_Platform :: struct {
        dest_size: s64;
        intermediate_texture: Texture;
        pbo: GLuint;
        src_fbo_handle: GLuint;
        dest_fbo_handle: GLuint;
    }
} else {
    #assert render_api == .NONE, "the Simp spec supplies texture layouts for OPENGL and NONE only";
    Texture_Platform_Specific :: struct {}
    Shader_Platform :: struct {}
    Pixel_Read_Platform :: struct {}
}
