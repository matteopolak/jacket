shader_text: Shader;
shader_color: Shader;
shader_color_with_blend: Shader;
shader_sprite_right_handed: Shader;
shader_sprite_left_handed: Shader;

prepare_window :: (window: Window_Type, wanted_msaa: int) -> chosen_msaa: s32 {
    for *info: window_infos if info.window == window {
        backend_init(info);
        return info.msaa;
    }

    info: Window_Info;
    info.window = window;
    info.wanted_msaa = cast(s32) wanted_msaa;
    backend_init(*info);
    if !window_infos.allocator.proc window_infos.allocator = context.default_allocator;

    Basic.array_add(*window_infos, info);

    return info.msaa;
}

backend_init :: (info: *Window_Info) {
    if info.backend_initted_for_this_window return;

    Contexts.on_closing = retire_window_renderer;

    info.msaa = Contexts.prepare(info.window, info.wanted_msaa);
    _, _, width, height, ok := Windows_API.get_dimensions(info.window, false);

    assert(ok, "native render window dimension query failed");
    info.window_width = width;
    info.window_height = height;
    info.backend_initted_for_this_window = true;
}

set_render_target :: (window: Window_Type, coords := Simp_Coordinate_System.RIGHT_HANDED) {
    immediate_flush();
    prepare_window(window, 4);
    Contexts.make_current(window);

    for info: window_infos if info.window == window {
        immediate_state.current_window_info = info;
        break;
    }

    immediate_state.coordinate_system = coords;
    immediate_state.texture_render_target = null;
    backend_set_render_target(*immediate_state.current_window_info);
    immediate_set_2d_projection(immediate_state.current_window_info.window_width, immediate_state.current_window_info.window_height);
}

set_render_target :: (texture: *Texture, coords := Simp_Coordinate_System.RIGHT_HANDED) {
    immediate_flush();
    assert(texture != null && (texture.flags & .Render_Target) != 0);
    immediate_state.coordinate_system = coords;
    immediate_state.texture_render_target = texture;
    backend_set_render_target(texture);
    immediate_set_2d_projection(texture.width, texture.height);
}

backend_set_render_target :: (info: *Window_Info) {
    Contexts.make_current(info.window);
    require_gl_texture_context();
    GL.gl.glBindFramebuffer(GL.GL_FRAMEBUFFER, 0);
    GL.gl.glViewport(0, 0, cast(GL.GLsizei) info.window_width, cast(GL.GLsizei) info.window_height);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native window target binding failed");
}

backend_set_render_target :: (texture: *Texture) {
    require_gl_texture_context();
    key := Contexts.current_native();
    handle: u32;

    for entry: framebuffers if entry.selection.native == key && entry.texture == texture.gl_handle {
        handle = entry.framebuffer;
        break;
    }

    if handle == 0 {
        GL.gl.glGenFramebuffers(1, *handle);
        assert(handle != 0);
        if !framebuffers.allocator.proc framebuffers.allocator = context.default_allocator;
        Basic.array_add(*framebuffers, Framebuffer.{ Contexts.capture(), texture.gl_handle, handle });
    }

    GL.gl.glBindFramebuffer(GL.GL_FRAMEBUFFER, handle);
    attach_texture_framebuffer(GL.GL_FRAMEBUFFER, texture);
    assert(GL.gl.glCheckFramebufferStatus(GL.GL_FRAMEBUFFER) == GL.GL_FRAMEBUFFER_COMPLETE, "native texture framebuffer is incomplete");
    GL.gl.glViewport(0, 0, cast(GL.GLsizei) texture.width, cast(GL.GLsizei) texture.height);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native texture target binding failed");
}

update_window :: (window: Window_Type) {
    for *info: window_infos if info.window == window {
        _, _, width, height, ok := Windows_API.get_dimensions(window, false);

        assert(ok);
        info.window_width = width;
        info.window_height = height;
        Contexts.update(window);

        if immediate_state.texture_render_target == null && immediate_state.current_window_info.window == window {
            immediate_flush();
            immediate_state.current_window_info = <<info;
            backend_set_render_target(info);
            immediate_set_2d_projection(width, height);
        }

        return;
    }

    prepare_window(window, 4);
}

get_render_dimensions :: (window: Window_Type) -> (width: s32, height: s32) {
    _, _, width, height, ok := Windows_API.get_dimensions(window, false);
    assert(ok, "native render dimension query failed");
    return width, height;
}

disable_window :: (window: Window_Type) {
    for item: disabled_windows if item == window return;
    Basic.array_add(*disabled_windows, window);
}

enable_window :: (window: Window_Type) {
    for item, index: disabled_windows if item == window {
        Basic.array_unordered_remove_by_index(*disabled_windows, index);
        break;
    }
}

swap_buffers :: (window: Window_Type, vsync := true) {
    immediate_flush();
    for item: disabled_windows if item == window return;
    Contexts.swap(window, vsync);
}

backend_resize_render_target :: (info: Window_Info) {
    GL.gl.glViewport(0, 0, cast(GL.GLsizei) info.window_width, cast(GL.GLsizei) info.window_height);
}

backend_flush :: () {
    GL.gl.glFlush();
}

backend_tweak_projection_matrix :: (m: *Matrix4) {
    // Projection is already constructed in native OpenGL clip coordinates.
}

clear_render_target :: (r: float, g: float, b: float, a: float) {
    immediate_flush();
    require_gl_texture_context();
    GL.gl.glClearColor(r, g, b, a);
    GL.gl.glClear(GL.GL_COLOR_BUFFER_BIT | GL.GL_DEPTH_BUFFER_BIT | GL.GL_STENCIL_BUFFER_BIT);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native render-target clear failed");
}

set_scissor :: (p0: Vector2, p1: Vector2) {
    set_scissor(cast(s32) p0.x, cast(s32) p0.y, cast(s32) p1.x, cast(s32) p1.y);
}

set_scissor :: (x0: s32, y0: s32, x1: s32, y1: s32) {
    immediate_flush();
    assert(x1 >= x0 && y1 >= y0);

    if immediate_state.coordinate_system == .LEFT_HANDED {
        height := immediate_state.current_window_info.window_height;
        if immediate_state.texture_render_target height = immediate_state.texture_render_target.height;

        bottom := height - y1;

        y1 = height - y0;
        y0 = bottom;
    }

    GL.gl.glEnable(GL.GL_SCISSOR_TEST);
    GL.gl.glScissor(x0, y0, cast(GL.GLsizei) (x1 - x0), cast(GL.GLsizei) (y1 - y0));
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native scissor update failed");
}

clear_scissor :: () {
    immediate_flush();
    GL.gl.glDisable(GL.GL_SCISSOR_TEST);
}

immediate_reset :: () {
    immediate_state.num_vertices = 0;
}

immediate_begin :: () {
    immediate_reset();
}

immediate_flush :: () {
    if immediate_state.num_vertices == 0 return;
    backend_draw_immediate_vertices(immediate_state.num_vertices);
    immediate_state.num_vertices = 0;
}

immediate_set_shader :: (s: *Shader) {
    if immediate_state.current_shader == s return;
    immediate_flush();
    immediate_state.current_shader = s;
}

immediate_set_texture :: (texture: *Texture) {
    if immediate_texture == texture return;
    immediate_flush();
    immediate_texture = texture;
}

immediate_vertex :: (position: Vector3, color_scale: Vector4, uv: Vector2 = .{}, normal: Vector3 = .{}) {
    if immediate_state.num_vertices == Immediate_State.MAX_VERTICES immediate_flush();

    vertex := *immediate_state.vertices[immediate_state.num_vertices];
    vertex.* = .{};
    vertex.position = position;
    vertex.color_scale = color_scale;
    vertex.uv0 = uv;
    vertex.normal = normal;
    immediate_state.num_vertices += 1;
}

immediate_set_2d_projection :: (render_target_width: s32, render_target_height: s32) {
    assert(render_target_width > 0 && render_target_height > 0);
    projection = .{};
    projection.floats[0] = 2.0 / cast(float) render_target_width;
    projection.floats[5] = 2.0 / cast(float) render_target_height;
    projection.floats[10] = -1;
    projection.floats[12] = -1;
    projection.floats[13] = -1;
    projection.floats[15] = 1;

    if immediate_state.coordinate_system == .LEFT_HANDED {
        projection.floats[5] = -projection.floats[5];
        projection.floats[13] = 1;
    }
}

set_projection :: () {
    assert(immediate_state.current_shader != null);
    set_parameter(immediate_state.current_shader, "transform", projection);
}

set_shader_for_color :: (enable_blend := false) {
    ensure_renderer();
    immediate_set_texture(null);
    if enable_blend immediate_set_shader(*shader_color_with_blend);
    else immediate_set_shader(*shader_color);
}

set_shader_for_text :: () {
    ensure_renderer();
    immediate_set_shader(*shader_text);
}

set_shader_for_images :: (texture: *Texture) {
    ensure_renderer();
    immediate_set_texture(texture);
    if immediate_state.coordinate_system == .LEFT_HANDED immediate_set_shader(*shader_sprite_left_handed);
    else immediate_set_shader(*shader_sprite_right_handed);
}

backend_set_texture :: (shader: *Shader, texture: *Texture) {
    GL.gl.glActiveTexture(GL.GL_TEXTURE0);
    GL.gl.glBindTexture(GL.GL_TEXTURE_2D, texture.gl_handle);
    GL.gl.glUniform1i(shader.diffuse_texture_loc, 0);
}

backend_draw_immediate_vertices :: (num_vertices: int) {
    ensure_renderer();
    if !immediate_state.current_shader immediate_state.current_shader = *shader_color;

    shader := immediate_state.current_shader;

    backend_use_shader(shader);
    set_projection();

    if shader.alpha_blend {
        GL.gl.glEnable(GL.GL_BLEND);
        GL.gl.glBlendFuncSeparate(GL.GL_SRC_ALPHA, GL.GL_ONE_MINUS_SRC_ALPHA, GL.GL_ONE, GL.GL_ONE_MINUS_SRC_ALPHA);
    } else GL.gl.glDisable(GL.GL_BLEND);

    GL.gl.glBindVertexArray(renderer_vao);
    GL.gl.glBindBuffer(GL.GL_ARRAY_BUFFER, renderer_vbo);
    GL.gl.glBufferData(GL.GL_ARRAY_BUFFER, num_vertices * size_of(Immediate_Vertex), immediate_state.vertices.data, GL.GL_STREAM_DRAW);
    vertex_attribute(shader.position_loc, 3, offset_of(Immediate_Vertex.position));
    vertex_attribute(shader.color_scale_loc, 4, offset_of(Immediate_Vertex.color_scale));
    vertex_attribute(shader.normal_loc, 3, offset_of(Immediate_Vertex.normal));
    vertex_attribute(shader.tangent_loc, 4, offset_of(Immediate_Vertex.tangent));
    vertex_attribute(shader.uv_0_loc, 2, offset_of(Immediate_Vertex.uv0));
    vertex_attribute(shader.uv_1_loc, 2, offset_of(Immediate_Vertex.uv1));
    if immediate_texture backend_set_texture(shader, immediate_texture);

    GL.gl.glDrawArrays(GL.GL_TRIANGLES, 0, cast(GL.GLsizei) num_vertices);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native immediate rendering failed");
}

draw_text :: (font: *Dynamic_Font, x: s64, y: s64, text: string, color := Vector4.{ 1, 1, 1, 1 }, effects: Font_Effects = 0) -> width: s64 {
    width := prepare_text(font, text, effects);
    draw_prepared_text(font, x, y, color, effects);
    return width;
}

draw_prepared_text :: (font: *Dynamic_Font, x: s64, y: s64, color := Vector4.{ 1, 1, 1, 1 }, effects: Font_Effects = 0) {
    baseline := y;
    if immediate_state.coordinate_system == .LEFT_HANDED baseline = -baseline;
    generate_quads_for_prepared_text(font, x, baseline, effects);
    draw_generated_quads(font, color);
}

draw_generated_quads :: (font: *Dynamic_Font, color := Vector4.{ 1, 1, 1, 1 }) {
    set_shader_for_text();

    for quad: font.current_quads {
        page := quad.glyph.page;
        if page.dirty {
            immediate_flush();
            backend_update_texture(*page.texture, page.bitmap_data);
            page.dirty = false;
        }

        immediate_set_texture(*page.texture);
        p0 := quad.p0;
        p1 := quad.p1;
        p2 := quad.p2;
        p3 := quad.p3;

        if immediate_state.coordinate_system == .LEFT_HANDED {
            p0.y = -p0.y;
            p1.y = -p1.y;
            p2.y = -p2.y;
            p3.y = -p3.y;
        }

        immediate_quad(p0, p1, p2, p3, color, .{ quad.u0, quad.v1 }, .{ quad.u1, quad.v1 }, .{ quad.u1, quad.v0 }, .{ quad.u0, quad.v0 });
    }

    immediate_flush();
}

#scope_module
Windows_API :: #import "Window_Creation";
Contexts :: #import "internal/rendering/window-contexts";

immediate_state: Immediate_State;
immediate_texture: *Texture;
projection: Matrix4;
renderer_vao, renderer_vbo: u32;
renderer_native: *void;

Renderer_Objects :: struct {
    native: *void;
    vao: u32;
    vbo: u32;
}

renderer_objects: [..] Renderer_Objects;

Framebuffer :: struct {
    selection: Contexts.Selection;
    texture: u32;
    framebuffer: u32;
}

framebuffers: [..] Framebuffer;
disabled_windows: [..] Window_Type;

depth_texture_format :: (format: Texture_Format) -> bool {
    return format == .D16 || format == .D24S8 || format == .D32F || format == .D32FS8;
}

stencil_texture_format :: (format: Texture_Format) -> bool {
    return format == .D24S8 || format == .D32FS8;
}

attach_texture_framebuffer :: (target: u32, texture: *Texture) {
    GL.gl.glFramebufferTexture2D(target, GL.GL_COLOR_ATTACHMENT0, GL.GL_TEXTURE_2D, 0, 0);
    GL.gl.glFramebufferTexture2D(target, GL.GL_DEPTH_STENCIL_ATTACHMENT, GL.GL_TEXTURE_2D, 0, 0);
    attachment: u32 = GL.GL_COLOR_ATTACHMENT0;
    buffer: u32 = GL.GL_COLOR_ATTACHMENT0;

    if depth_texture_format(texture.format) {
        attachment = GL.GL_DEPTH_ATTACHMENT;
        buffer = GL.GL_NONE;
        if stencil_texture_format(texture.format) attachment = GL.GL_DEPTH_STENCIL_ATTACHMENT;
    }

    GL.gl.glFramebufferTexture2D(target, attachment, GL.GL_TEXTURE_2D, texture.gl_handle, 0);
    if target != GL.GL_DRAW_FRAMEBUFFER GL.gl.glReadBuffer(buffer);
    if target != GL.GL_READ_FRAMEBUFFER GL.gl.glDrawBuffer(buffer);
}

retire_window_renderer :: (window: Window_Type, native: *void) {
    immediate_flush();
    Contexts.make_current(window);
    retire_window_pixel_readers(native);

    for entry, index: renderer_objects if entry.native == native {
        GL.gl.glDeleteVertexArrays(1, *entry.vao);
        GL.gl.glDeleteBuffers(1, *entry.vbo);
        Basic.array_unordered_remove_by_index(*renderer_objects, index);
        break;
    }

    index := framebuffers.count;
    while index > 0 {
        index -= 1;
        entry := framebuffers[index];
        if entry.selection.native != native continue;

        GL.gl.glDeleteFramebuffers(1, *entry.framebuffer);
        Basic.array_unordered_remove_by_index(*framebuffers, index);
    }

    for info, position: window_infos if info.window == window {
        Basic.array_unordered_remove_by_index(*window_infos, position);
        break;
    }

    for item, position: disabled_windows if item == window {
        Basic.array_unordered_remove_by_index(*disabled_windows, position);
        break;
    }

    if immediate_state.current_window_info.window == window {
        immediate_state.current_window_info = .{};
        immediate_state.texture_render_target = null;
    }

    if renderer_native == native {
        renderer_native = null;
        renderer_vao = 0;
        renderer_vbo = 0;
    }

    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native window renderer resource release failed");
}

flush_texture_use :: (texture: u32) {
    if texture == 0 return;
    if (immediate_texture && immediate_texture.gl_handle == texture) || (immediate_state.texture_render_target && immediate_state.texture_render_target.gl_handle == texture) immediate_flush();
}

release_texture_framebuffers :: (texture: u32) {
    flush_texture_use(texture);
    if immediate_state.texture_render_target && immediate_state.texture_render_target.gl_handle == texture immediate_state.texture_render_target = null;

    if immediate_texture && immediate_texture.gl_handle == texture {
        immediate_texture = null;
    }

    previous := Contexts.capture();
    index := framebuffers.count;

    while index > 0 {
        index -= 1;
        entry := framebuffers[index];
        if entry.texture != texture continue;

        Contexts.restore(entry.selection);
        GL.gl.glDeleteFramebuffers(1, *entry.framebuffer);
        assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "texture framebuffer release failed");
        Basic.array_unordered_remove_by_index(*framebuffers, index);
    }

    Contexts.restore(previous);
} @Releases

vertex_attribute :: (location: s32, components: s32, offset: int) {
    if location < 0 return;
    GL.gl.glEnableVertexAttribArray(cast(u32) location);
    GL.gl.glVertexAttribPointer(cast(u32) location, components, GL.GL_FLOAT, false, cast(GL.GLsizei) size_of(Immediate_Vertex), cast(*void) offset);
}

ensure_renderer :: () {
    require_gl_texture_context();
    native := Contexts.current_native();
    if renderer_native == native && renderer_vao != 0 && GL.gl.glIsVertexArray(renderer_vao) return;

    renderer_native = native;
    renderer_vao = 0;
    renderer_vbo = 0;

    for entry: renderer_objects if entry.native == native {
        renderer_vao = entry.vao;
        renderer_vbo = entry.vbo;
        break;
    }

    if renderer_vao == 0 {
        GL.gl.glGenVertexArrays(1, *renderer_vao);
        GL.gl.glGenBuffers(1, *renderer_vbo);
        if !renderer_objects.allocator.proc renderer_objects.allocator = context.default_allocator;
        Basic.array_add(*renderer_objects, Renderer_Objects.{ native, renderer_vao, renderer_vbo });
    }

    if shader_color.gl_handle != 0 && GL.gl.glIsProgram(shader_color.gl_handle) return;

    initialize_builtin_shader(*shader_color, false, false, false);
    initialize_builtin_shader(*shader_color_with_blend, true, false, false);
    initialize_builtin_shader(*shader_text, true, true, false);
    initialize_builtin_shader(*shader_sprite_right_handed, true, true, true);
    initialize_builtin_shader(*shader_sprite_left_handed, true, true, false);
}

initialize_builtin_shader :: (shader: *Shader, blend: bool, texture: bool, flip: bool) {
    builder: Basic.String_Builder;

    Basic.append(*builder, "OUT_IN vec4 vertex_color;\nOUT_IN vec2 texture_uv;\n#ifdef VERTEX_SHADER\nin vec3 position; in vec4 color_scale; in vec2 uv_0; uniform mat4 transform;\nvoid main(){gl_Position=transform*vec4(position,1);vertex_color=color_scale;texture_uv=uv_0;");
    if flip Basic.append(*builder, "texture_uv.y=1.0-texture_uv.y;");

    Basic.append(*builder, "}\n#else\nout vec4 result; uniform sampler2D diffuse_texture;\nvoid main(){result=vertex_color;");
    if texture Basic.append(*builder, "result*=texture(diffuse_texture,texture_uv);");

    Basic.append(*builder, "}\n#endif\n");
    source := Basic.builder_to_string(*builder);

    defer Basic.free(source);
    shader.gl_handle = get_shader_program(source);
    assert(shader.gl_handle != 0, "built-in shader compilation failed");
    shader.alpha_blend = blend;
    backend_update_shader_parameter_locations(shader);
}
