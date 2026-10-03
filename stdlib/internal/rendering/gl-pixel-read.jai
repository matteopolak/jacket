pixel_read_begin :: (source: *Texture, dest_format: Texture_Format, width: s32 = -1, height: s32 = -1, srgb := false, flip := false) -> *Pixel_Read {
    immediate_flush();
    require_gl_texture_context();
    assert(source != null && source.gl_handle != 0 && source.width > 0 && source.height > 0);
    if width == -1 width = source.width;
    if height == -1 height = source.height;

    assert(width > 0 && height > 0);
    description, known := describe_gl_format(dest_format, srgb);

    assert(known && !description.compressed && get_image_bytes_per_texel(dest_format) > 0);
    previous := save_pixel_pack_state();

    defer restore_pixel_pack_state(previous);
    reader := Basic.New(Pixel_Read);
    reader.dest_format = dest_format;
    reader.dest_srgb = srgb;
    reader.dest_width = width;
    reader.dest_height = height;
    reader.flip = flip;
    reader.dest_size = cast(int) width * height * get_image_bytes_per_texel(dest_format);
    flags: Texture_Flags;
    if srgb flags |= .sRGB;

    reader.intermediate_texture = texture_create_render_target(width, height, dest_format, flags);
    GL.gl.glGenFramebuffers(1, *reader.src_fbo_handle);
    GL.gl.glGenFramebuffers(1, *reader.dest_fbo_handle);
    GL.gl.glBindFramebuffer(GL.GL_READ_FRAMEBUFFER, reader.src_fbo_handle);
    attach_texture_framebuffer(GL.GL_READ_FRAMEBUFFER, source);
    assert(GL.gl.glCheckFramebufferStatus(GL.GL_READ_FRAMEBUFFER) == GL.GL_FRAMEBUFFER_COMPLETE);
    GL.gl.glBindFramebuffer(GL.GL_DRAW_FRAMEBUFFER, reader.dest_fbo_handle);
    attach_texture_framebuffer(GL.GL_DRAW_FRAMEBUFFER, *reader.intermediate_texture);
    assert(GL.gl.glCheckFramebufferStatus(GL.GL_DRAW_FRAMEBUFFER) == GL.GL_FRAMEBUFFER_COMPLETE);
    if srgb GL.gl.glEnable(GL.GL_FRAMEBUFFER_SRGB);
    else GL.gl.glDisable(GL.GL_FRAMEBUFFER_SRGB);

    filter := GL.GL_LINEAR;
    if description.integer filter = GL.GL_NEAREST;

    mask: u32 = GL.GL_COLOR_BUFFER_BIT;

    if depth_texture_format(source.format) || depth_texture_format(dest_format) {
        assert(source.format == dest_format, "native depth blit requires matching source/destination storage formats");
        mask = GL.GL_DEPTH_BUFFER_BIT;
        if stencil_texture_format(dest_format) mask |= GL.GL_STENCIL_BUFFER_BIT;
        filter = GL.GL_NEAREST;
    }

    GL.gl.glBlitFramebuffer(0, 0, source.width, source.height, 0, 0, width, height, mask, cast(u32) filter);
    GL.gl.glBindFramebuffer(GL.GL_READ_FRAMEBUFFER, reader.dest_fbo_handle);
    if depth_texture_format(dest_format) GL.gl.glReadBuffer(GL.GL_NONE);
    else GL.gl.glReadBuffer(GL.GL_COLOR_ATTACHMENT0);

    GL.gl.glGenBuffers(1, *reader.pbo);
    assert(reader.pbo != 0);
    GL.gl.glBindBuffer(GL.GL_PIXEL_PACK_BUFFER, reader.pbo);
    GL.gl.glBufferData(GL.GL_PIXEL_PACK_BUFFER, reader.dest_size, null, GL.GL_STREAM_READ);
    GL.gl.glPixelStorei(GL.GL_PACK_ALIGNMENT, 1);
    GL.gl.glPixelStorei(GL.GL_PACK_ROW_LENGTH, 0);
    GL.gl.glPixelStorei(GL.GL_PACK_SKIP_PIXELS, 0);
    GL.gl.glPixelStorei(GL.GL_PACK_SKIP_ROWS, 0);
    GL.gl.glPixelStorei(GL.GL_PACK_SWAP_BYTES, 0);
    GL.gl.glReadPixels(0, 0, cast(GL.GLsizei) width, cast(GL.GLsizei) height, description.external, description.scalar, null);
    fence := GL.gl.glFenceSync(GL.GL_SYNC_GPU_COMMANDS_COMPLETE, 0);

    assert(fence != 0 && GL.gl.glGetError() == GL.GL_NO_ERROR, "native asynchronous pixel read failed");
    if !pixel_readers.allocator.proc pixel_readers.allocator = context.default_allocator;

    Basic.array_add(*pixel_readers, Reader_Storage.{ reader, fence, Contexts.capture(), context.allocator });

    return reader;
}

pixel_read_is_done :: (reader: *Pixel_Read) -> bool {
    storage := find_pixel_reader(reader);

    require_reader_context(storage.selection.native);
    result := GL.gl.glClientWaitSync(storage.fence, 0, 0);

    assert(result != GL.GL_WAIT_FAILED, "native pixel fence query failed");

    return result == GL.GL_ALREADY_SIGNALED || result == GL.GL_CONDITION_SATISFIED;
}

pixel_read_end :: (reader: *Pixel_Read) -> Bitmap {
    result: Bitmap;
    pixel_read_end(reader, *result);
    return result;
}

pixel_read_end :: (reader: *Pixel_Read, bitmap: *Bitmap) {
    stored := find_pixel_reader(reader).*;

    require_reader_context(stored.selection.native);
    result := GL.gl.glClientWaitSync(stored.fence, GL.GL_SYNC_FLUSH_COMMANDS_BIT, 0xffffffffffffffff);

    assert(result == GL.GL_ALREADY_SIGNALED || result == GL.GL_CONDITION_SATISFIED, "native pixel read wait failed");
    previous: s32;

    GL.gl.glGetIntegerv(GL.GL_PIXEL_PACK_BUFFER_BINDING, *previous);
    GL.gl.glBindBuffer(GL.GL_PIXEL_PACK_BUFFER, reader.pbo);
    source := cast(*u8) GL.gl.glMapBufferRange(GL.GL_PIXEL_PACK_BUFFER, 0, reader.dest_size, GL.GL_MAP_READ_BIT);

    assert(source != null, "native pixel buffer mapping failed");
    bitmap_alloc(bitmap, reader.dest_width, reader.dest_height, reader.dest_format);
    if reader.dest_srgb bitmap.flags |= .IS_SRGB;

    for row: 0..reader.dest_height - 1 {
        native_row := row;
        if reader.flip native_row = reader.dest_height - 1 - row;
        memcpy(bitmap.data.data + row * bitmap.stride, source + native_row * bitmap.stride, bitmap.stride);
    }

    unmapped := GL.gl.glUnmapBuffer(GL.GL_PIXEL_PACK_BUFFER);

    GL.gl.glBindBuffer(GL.GL_PIXEL_PACK_BUFFER, cast,force(u32) previous);
    GL.gl.glDeleteSync(stored.fence);
    GL.gl.glDeleteBuffers(1, *reader.pbo);

    if reader.src_fbo_handle != 0 || reader.dest_fbo_handle != 0 {
        selection := Contexts.capture();

        Contexts.restore(stored.selection);
        GL.gl.glDeleteFramebuffers(1, *reader.src_fbo_handle);
        GL.gl.glDeleteFramebuffers(1, *reader.dest_fbo_handle);
        Contexts.restore(selection);
    }

    texture_destroy(*reader.intermediate_texture);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native pixel reader destruction failed");

    for entry, index: pixel_readers if entry.reader == reader {
        Basic.array_unordered_remove_by_index(*pixel_readers, index);
        break;
    }

    stored.allocator.proc(.FREE, 0, 0, reader, stored.allocator.data);

    if !unmapped {
        // The SDK reports undefined data on corruption. Do not publish the copy.
        deinit(bitmap);
        context.logger("native pixel buffer became invalid\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "native pixel buffer became invalid");
    }
}

#scope_module
Reader_Storage :: struct {
    reader: *Pixel_Read;
    fence: GL.GLsync;
    selection: Contexts.Selection;
    allocator: Allocator;
}

pixel_readers: [..] Reader_Storage;

require_reader_context :: (native: *void) {
    current := Contexts.current_native();
    assert(current == native || (Contexts.is_shared_native(current) && Contexts.is_shared_native(native)), "pixel read requires its creating context or a context in the same native share group");
}

retire_window_pixel_readers :: (native: *void) {
    for *storage: pixel_readers if storage.selection.native == native {
        GL.gl.glDeleteFramebuffers(1, *storage.reader.src_fbo_handle);
        GL.gl.glDeleteFramebuffers(1, *storage.reader.dest_fbo_handle);
        storage.reader.src_fbo_handle = 0;
        storage.reader.dest_fbo_handle = 0;
        storage.selection = Contexts.share_owner_selection();
    }
}

Pixel_Pack_State :: struct {
    read_fbo, draw_fbo, read_buffer, pack_buffer, alignment, row_length, skip_pixels, skip_rows, swap_bytes: s32;
    srgb: bool;
}

find_pixel_reader :: (reader: *Pixel_Read) -> *Reader_Storage {
    for *storage: pixel_readers if storage.reader == reader return storage;
    assert(false, "pixel reader is invalid or already consumed");
    return null;
}

save_pixel_pack_state :: () -> Pixel_Pack_State {
    state: Pixel_Pack_State;

    GL.gl.glGetIntegerv(GL.GL_READ_FRAMEBUFFER_BINDING, *state.read_fbo);
    GL.gl.glGetIntegerv(GL.GL_DRAW_FRAMEBUFFER_BINDING, *state.draw_fbo);
    GL.gl.glGetIntegerv(GL.GL_READ_BUFFER, *state.read_buffer);
    GL.gl.glGetIntegerv(GL.GL_PIXEL_PACK_BUFFER_BINDING, *state.pack_buffer);
    GL.gl.glGetIntegerv(GL.GL_PACK_ALIGNMENT, *state.alignment);
    GL.gl.glGetIntegerv(GL.GL_PACK_ROW_LENGTH, *state.row_length);
    GL.gl.glGetIntegerv(GL.GL_PACK_SKIP_PIXELS, *state.skip_pixels);
    GL.gl.glGetIntegerv(GL.GL_PACK_SKIP_ROWS, *state.skip_rows);
    GL.gl.glGetIntegerv(GL.GL_PACK_SWAP_BYTES, *state.swap_bytes);
    state.srgb = GL.gl.glIsEnabled(GL.GL_FRAMEBUFFER_SRGB);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR);

    return state;
}

restore_pixel_pack_state :: (state: Pixel_Pack_State) {
    GL.gl.glBindFramebuffer(GL.GL_READ_FRAMEBUFFER, cast,force(u32) state.read_fbo);
    GL.gl.glBindFramebuffer(GL.GL_DRAW_FRAMEBUFFER, cast,force(u32) state.draw_fbo);
    GL.gl.glReadBuffer(cast,force(u32) state.read_buffer);
    GL.gl.glBindBuffer(GL.GL_PIXEL_PACK_BUFFER, cast,force(u32) state.pack_buffer);
    GL.gl.glPixelStorei(GL.GL_PACK_ALIGNMENT, state.alignment);
    GL.gl.glPixelStorei(GL.GL_PACK_ROW_LENGTH, state.row_length);
    GL.gl.glPixelStorei(GL.GL_PACK_SKIP_PIXELS, state.skip_pixels);
    GL.gl.glPixelStorei(GL.GL_PACK_SKIP_ROWS, state.skip_rows);
    GL.gl.glPixelStorei(GL.GL_PACK_SWAP_BYTES, state.swap_bytes);
    if state.srgb GL.gl.glEnable(GL.GL_FRAMEBUFFER_SRGB);
    else GL.gl.glDisable(GL.GL_FRAMEBUFFER_SRGB);

    assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "native pixel-pack state restoration failed");
}
