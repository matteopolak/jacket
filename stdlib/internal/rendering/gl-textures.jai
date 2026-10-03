texture_load_from_bitmap :: (using texture: *Texture, bitmap: *Bitmap) -> bool {
    return upload_bitmap(texture, bitmap);
}

texture_load_from_file :: (map: *Texture, filename: string, srgb := false, build_mipmaps := false) -> bool {
    bitmap: Bitmap;
    if !bitmap_load(*bitmap, filename) return false;

    defer deinit(*bitmap);
    if srgb bitmap.flags |= .IS_SRGB;

    if build_mipmaps bitmap_build_mipmaps(*bitmap);

    return upload_bitmap(map, *bitmap);
}

texture_load_from_memory :: (texture: *Texture, memory: [] u8, srgb := false, build_mipmaps := false) -> bool {
    bitmap: Bitmap;
    if !bitmap_load_from_memory(*bitmap, memory) return false;

    defer deinit(*bitmap);
    if srgb bitmap.flags |= .IS_SRGB;

    if build_mipmaps bitmap_build_mipmaps(*bitmap);

    return upload_bitmap(texture, *bitmap);
}

texture_destroy :: (texture: *Texture) {
    if texture.gl_handle != 0 {
        require_gl_texture_context();
        release_texture_framebuffers(texture.gl_handle);
        GL.gl.glDeleteTextures(1, *texture.gl_handle);
        assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL texture deletion failed");
    }

    texture.* = .{};
}

backend_realloc_texture :: (texture: *Texture, bitmap: *Bitmap) {
    flush_texture_use(texture.gl_handle);

    if texture.gl_handle == 0 || texture.mipmap_count != 1 || bitmap.mipmap_count != 1 {
        uploaded := upload_bitmap(texture, bitmap);
        if !uploaded {
            context.logger("OpenGL texture reallocation failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            assert(false, "OpenGL texture reallocation failed");
        }

        return;
    }

    assert(texture.depth == 1 && texture.array_length == 1);
    assert((texture.flags & ~(.sRGB | .Render_Target | .No_Skip_Mipmaps)) == 0, "only ordinary 2D texture reallocation is implemented");
    flags := texture.flags & ~ .sRGB;
    if bitmap.flags & .IS_SRGB flags |= .sRGB;

    description, known := describe_gl_format(bitmap.format, (flags & .sRGB) != 0);

    assert(known && is_supported(bitmap.format), "unsupported texture reallocation format");
    assert((flags & .Render_Target) == 0 || !description.compressed, "compressed render targets are unsupported");
    packed, ok := pack_bitmap(bitmap);

    assert(ok, "invalid bitmap storage for texture reallocation");
    defer Basic.free(packed.data);
    previous := save_texture_state();

    defer restore_texture_state(previous);
    GL.gl.glBindTexture(GL.GL_TEXTURE_2D, texture.gl_handle);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ALIGNMENT, 1);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ROW_LENGTH, 0);
    prepare_client_unpack();

    if description.compressed {
        assert(GL.gl.glCompressedTexImage2D != null, "compressed texture entry point is unavailable");
        GL.gl.glCompressedTexImage2D(GL.GL_TEXTURE_2D, 0, description.internal, cast(GL.GLsizei) bitmap.width, cast(GL.GLsizei) bitmap.height, 0, cast(GL.GLsizei) packed.count, packed.data);
    } else {
        GL.gl.glTexImage2D(GL.GL_TEXTURE_2D, 0, cast(s32) description.internal, cast(GL.GLsizei) bitmap.width, cast(GL.GLsizei) bitmap.height, 0, description.external, description.scalar, packed.data);
    }

    filter := GL.GL_LINEAR;
    if description.integer || (flags & .Render_Target) != 0 filter = GL.GL_NEAREST;

    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MIN_FILTER, cast(s32) filter);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MAG_FILTER, cast(s32) filter);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL texture storage reallocation failed");
    // Preserve the native name, including existing framebuffer attachments.
    texture.width = bitmap.width;
    texture.height = bitmap.height;
    texture.format = bitmap.format;
    texture.flags = flags;
    texture.mipmap_count = 1;
}

backend_update_texture :: (texture: *Texture, bitmap: *Bitmap) {
    flush_texture_use(texture.gl_handle);

    // Reallocation is required if the public dimensions or format changed.
    color_space_changed := ((texture.flags & .sRGB) != 0) != ((bitmap.flags & .IS_SRGB) != 0);
    if texture.gl_handle == 0 || texture.width != bitmap.width || texture.height != bitmap.height || texture.format != bitmap.format || texture.mipmap_count != bitmap.mipmap_count || color_space_changed {
        backend_realloc_texture(texture, bitmap);
        return;
    }

    description, known := describe_gl_format(bitmap.format, (texture.flags & .sRGB) != 0);

    assert(known, "unknown texture subimage format");
    views, valid := texture_mip_views(bitmap);

    assert(valid, "invalid bitmap mip storage for texture update");
    defer Basic.array_free(views);
    require_gl_texture_context();
    previous := save_texture_state();

    defer restore_texture_state(previous);
    GL.gl.glBindTexture(GL.GL_TEXTURE_2D, texture.gl_handle);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ALIGNMENT, 1);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ROW_LENGTH, 0);
    prepare_client_unpack();

    for view, level: views {
        packed, ok := pack_bitmap(*view);

        assert(ok, "invalid bitmap storage for texture update");
        defer Basic.free(packed.data);

        if description.compressed {
            assert(GL.gl.glCompressedTexSubImage2D != null, "compressed texture update entry point is unavailable");
            GL.gl.glCompressedTexSubImage2D(GL.GL_TEXTURE_2D, cast(s32) level, 0, 0, cast(GL.GLsizei) view.width, cast(GL.GLsizei) view.height, description.internal, cast(GL.GLsizei) packed.count, packed.data);
        } else {
            GL.gl.glTexSubImage2D(GL.GL_TEXTURE_2D, cast(s32) level, 0, 0, cast(GL.GLsizei) view.width, cast(GL.GLsizei) view.height, description.external, description.scalar, packed.data);
        }
    }

    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL texture update failed");
}

texture_create_render_target :: (width: s32, height: s32, format: Texture_Format, flags: Texture_Flags = 0) -> Texture {
    result: Texture;
    result.width = width;
    result.height = height;
    result.format = format;
    result.flags = flags | .Render_Target;
    backend_create_render_target(*result);

    return result;
}

backend_create_render_target :: (texture: *Texture) {
    assert(texture.width > 0 && texture.height > 0 && texture.depth == 1 && texture.array_length == 1 && texture.mipmap_count == 1);
    assert((texture.flags & ~(.sRGB | .Render_Target | .No_Skip_Mipmaps)) == 0, "only ordinary 2D render-target texture storage is implemented");
    description, known := describe_gl_format(texture.format, (texture.flags & .sRGB) != 0);

    assert(known && !description.compressed, "compressed or unknown render-target format");
    assert(is_supported(texture.format), "texture format is unsupported by the current OpenGL context");
    require_gl_texture_context();
    previous := save_texture_state();

    defer restore_texture_state(previous);
    handle: GLuint;

    GL.gl.glGenTextures(1, *handle);
    assert(handle != 0 && GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL render-target texture allocation failed");
    GL.gl.glBindTexture(GL.GL_TEXTURE_2D, handle);
    prepare_client_unpack();
    GL.gl.glTexImage2D(GL.GL_TEXTURE_2D, 0, cast(s32) description.internal, cast(GL.GLsizei) texture.width, cast(GL.GLsizei) texture.height, 0, description.external, description.scalar, null);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MIN_FILTER, GL.GL_NEAREST);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MAG_FILTER, GL.GL_NEAREST);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_WRAP_S, GL.GL_CLAMP_TO_EDGE);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_WRAP_T, GL.GL_CLAMP_TO_EDGE);
    error := GL.gl.glGetError();
    if error != GL.GL_NO_ERROR {
        GL.gl.glDeleteTextures(1, *handle);
        assert(false, "OpenGL render-target texture storage failed");
    }

    if texture.gl_handle != 0 {
        old_handle := texture.gl_handle;
        release_texture_framebuffers(old_handle);
        GL.gl.glDeleteTextures(1, *old_handle);
        if cast,truncate(u32) previous.binding == old_handle previous.binding = cast,truncate(s32) handle;
    }

    texture.gl_handle = handle;
    texture.mipmap_count = 1;
    texture.flags |= .Render_Target;
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL render-target replacement failed");
}

is_supported :: (tf: Texture_Format) -> bool {
    description, known := describe_gl_format(tf, false);
    if !known return false;
    if GL.gl.glGetString(GL.GL_VERSION) == null return false;

    require_gl_texture_context();
    major, minor := GL.gl_get_version();
    if major < 3 return false;

    if (major > 4 || (major == 4 && minor >= 3)) && GL.gl.glGetInternalformativ {
        supported: s32;
        GL.gl.glGetInternalformativ(GL.GL_TEXTURE_2D, description.internal, GL.GL_INTERNALFORMAT_SUPPORTED, 1, *supported);
        assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL internal-format query failed");
        return supported != 0;
    }

    if tf == .BC1 || tf == .BC3 return GL.gl_supports_extension("GL_EXT_texture_compression_s3tc");
    if tf == .BC4 || tf == .BC5 return major >= 3 || GL.gl_supports_extension("GL_ARB_texture_compression_rgtc");
    if tf == .BC6 || tf == .BC7 return major > 4 || (major == 4 && minor >= 2) || GL.gl_supports_extension("GL_ARB_texture_compression_bptc");
    if tf == .B5G6R5 return major > 4 || (major == 4 && minor >= 1) || GL.gl_supports_extension("GL_ARB_ES2_compatibility");
    if tf >= .ETC_RGB && tf <= .ETC_RGB_A1 return major > 4 || (major == 4 && minor >= 3) || GL.gl_supports_extension("GL_ARB_ES3_compatibility");

    // Sized scalar/integer/depth formats in this backend require desktop GL 3.
    return major >= 3;
}

#scope_module
#load "texture-packing.jai";

GL_Texture_State :: struct {
    binding, alignment, row_length: s32;
    unpack_buffer, skip_rows, skip_pixels, swap_bytes: s32;
}

require_gl_texture_context :: () {
    assert(GL.gl.glGetString(GL.GL_VERSION) != null, "a current desktop OpenGL context is required");
    Generate :: #type (n: GL.GLsizei, textures: *GLuint) #c_call;
    Bind :: #type (target: GL.GLenum, texture: GLuint) #c_call;
    Delete :: #type (n: GL.GLsizei, textures: *GLuint) #c_call;
    Bind_Buffer :: #type (target: GL.GLenum, buffer: GLuint) #c_call;

    GL.gl_load(*GL.gl);
    generate: Generate = GL.gl.glGenTextures;
    bind: Bind = GL.gl.glBindTexture;
    release: Delete = GL.gl.glDeleteTextures;
    bind_buffer: Bind_Buffer = GL.gl.glBindBuffer;

    assert(generate != null && bind != null && release != null && bind_buffer != null, "OpenGL texture entry points are unavailable");
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "pending OpenGL error before texture operation");
}

save_texture_state :: () -> GL_Texture_State {
    result: GL_Texture_State;

    GL.gl.glGetIntegerv(GL.GL_TEXTURE_BINDING_2D, *result.binding);
    GL.gl.glGetIntegerv(GL.GL_UNPACK_ALIGNMENT, *result.alignment);
    GL.gl.glGetIntegerv(GL.GL_UNPACK_ROW_LENGTH, *result.row_length);
    GL.gl.glGetIntegerv(GL.GL_PIXEL_UNPACK_BUFFER_BINDING, *result.unpack_buffer);
    GL.gl.glGetIntegerv(GL.GL_UNPACK_SKIP_ROWS, *result.skip_rows);
    GL.gl.glGetIntegerv(GL.GL_UNPACK_SKIP_PIXELS, *result.skip_pixels);
    GL.gl.glGetIntegerv(GL.GL_UNPACK_SWAP_BYTES, *result.swap_bytes);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL unpack state query failed");

    return result;
}

restore_texture_state :: (state: GL_Texture_State) {
    GL.gl.glPixelStorei(GL.GL_UNPACK_ALIGNMENT, state.alignment);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ROW_LENGTH, state.row_length);
    GL.gl.glPixelStorei(GL.GL_UNPACK_SKIP_ROWS, state.skip_rows);
    GL.gl.glPixelStorei(GL.GL_UNPACK_SKIP_PIXELS, state.skip_pixels);
    GL.gl.glPixelStorei(GL.GL_UNPACK_SWAP_BYTES, state.swap_bytes);
    GL.gl.glBindBuffer(GL.GL_PIXEL_UNPACK_BUFFER, cast,truncate(u32) state.unpack_buffer);
    GL.gl.glBindTexture(GL.GL_TEXTURE_2D, cast,truncate(u32) state.binding);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL texture state restoration failed");
}

upload_bitmap :: (texture: *Texture, bitmap: *Bitmap) -> bool {
    flags := texture.flags & ~ .sRGB;
    if bitmap.flags & .IS_SRGB flags |= .sRGB;
    if (flags & ~(.sRGB | .No_Skip_Mipmaps | .Render_Target)) != 0 return false;

    description, known := describe_gl_format(bitmap.format, (flags & .sRGB) != 0);
    if !known return false;
    if (flags & .Render_Target) != 0 && description.compressed return false;
    if !is_supported(bitmap.format) return false;

    views, valid := texture_mip_views(bitmap);
    if !valid return false;

    defer Basic.array_free(views);
    previous := save_texture_state();
    handle: GLuint;
    committed := false;

    defer {
        if !committed && handle != 0 {
            GL.gl.glDeleteTextures(1, *handle);
            assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL failed-upload cleanup failed");
        }

        restore_texture_state(previous);
    }

    GL.gl.glGenTextures(1, *handle);
    error := GL.gl.glGetError();
    if handle == 0 || error != GL.GL_NO_ERROR return false;

    GL.gl.glBindTexture(GL.GL_TEXTURE_2D, handle);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ALIGNMENT, 1);
    GL.gl.glPixelStorei(GL.GL_UNPACK_ROW_LENGTH, 0);
    prepare_client_unpack();

    for view, level: views {
        packed, ok := pack_bitmap(*view);
        if !ok return false;

        defer Basic.free(packed.data);

        if description.compressed {
            if !GL.gl.glCompressedTexImage2D return false;
            GL.gl.glCompressedTexImage2D(GL.GL_TEXTURE_2D, cast(s32) level, description.internal, cast(GL.GLsizei) view.width, cast(GL.GLsizei) view.height, 0, cast(GL.GLsizei) packed.count, packed.data);
        } else {
            GL.gl.glTexImage2D(GL.GL_TEXTURE_2D, cast(s32) level, cast(s32) description.internal, cast(GL.GLsizei) view.width, cast(GL.GLsizei) view.height, 0, description.external, description.scalar, packed.data);
        }
    }

    filter := GL.GL_LINEAR;
    if description.integer filter = GL.GL_NEAREST;

    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MIN_FILTER, cast(s32) filter);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MAG_FILTER, cast(s32) filter);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_WRAP_S, GL.GL_CLAMP_TO_EDGE);
    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_WRAP_T, GL.GL_CLAMP_TO_EDGE);
    levels := cast(u16) views.count;

    GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MAX_LEVEL, cast(s32) levels - 1);

    if levels > 1 {
        mip_filter := GL.GL_LINEAR_MIPMAP_LINEAR;
        if description.integer mip_filter = GL.GL_NEAREST_MIPMAP_NEAREST;
        GL.gl.glTexParameteri(GL.GL_TEXTURE_2D, GL.GL_TEXTURE_MIN_FILTER, cast(s32) mip_filter);
    }

    error = GL.gl.glGetError();
    if error != GL.GL_NO_ERROR return false;

    old_handle := texture.gl_handle;
    if old_handle != 0 {
        release_texture_framebuffers(old_handle);
        GL.gl.glDeleteTextures(1, *old_handle);
        assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "OpenGL replaced-texture release failed");
        if cast,truncate(u32) previous.binding == old_handle previous.binding = cast,truncate(s32) handle;
    }

    texture.* = .{};
    texture.gl_handle = handle;
    texture.width = bitmap.width;
    texture.height = bitmap.height;
    texture.format = bitmap.format;
    texture.flags = flags;
    texture.mipmap_count = levels;
    committed = true;

    return true;
}

prepare_client_unpack :: () {
    GL.gl.glBindBuffer(GL.GL_PIXEL_UNPACK_BUFFER, 0);
    GL.gl.glPixelStorei(GL.GL_UNPACK_SKIP_ROWS, 0);
    GL.gl.glPixelStorei(GL.GL_UNPACK_SKIP_PIXELS, 0);
    GL.gl.glPixelStorei(GL.GL_UNPACK_SWAP_BYTES, 0);
}

texture_mip_views :: (bitmap: *Bitmap) -> [..] Bitmap, bool {
    views: [..] Bitmap;
    if bitmap.width <= 0 || bitmap.height <= 0 || bitmap.mipmap_count <= 0 || !bitmap.data.data return views, false;

    width := bitmap.width;
    height := bitmap.height;
    offset: int;

    for level: 0..bitmap.mipmap_count - 1 {
        _, row_bytes := get_image_size(bitmap.format, width, height);
        stride := (row_bytes + 3) & ~cast(int) 3;

        if level == 0 stride = bitmap.stride;

        rows := cast(int) height;
        if get_image_bytes_per_texel(bitmap.format) == 0 rows = (rows + 3) / 4;

        required := (rows - 1) * stride + row_bytes;
        if stride < row_bytes || offset + required > bitmap.data.count {
            Basic.array_free(views);
            return .{}, false;
        }

        view := bitmap.*;
        view.width = width;
        view.height = height;
        view.stride = cast(s32) stride;
        view.mipmap_count = 1;
        view.data = .{ required, bitmap.data.data + offset };
        Basic.array_add(*views, view);
        offset += stride * rows;

        if level + 1 < bitmap.mipmap_count && width == 1 && height == 1 {
            Basic.array_free(views);
            return .{}, false;
        }

        width = Basic.max(cast(s32) 1, width / 2);
        height = Basic.max(cast(s32) 1, height / 2);
    }

    return views, true;
}
