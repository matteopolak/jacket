// Regenerate with tools/generators/window_render_gl_formats.py.
#scope_module
describe_gl_format :: (tf: Texture_Format, srgb: bool) -> GL_Image_Format, bool {
    result: GL_Image_Format;
    known := false;

    if tf == .R8 {
        result = .{ internal = GL.GL_R8, external = GL.GL_RED, scalar = GL.GL_UNSIGNED_BYTE };
        known = true;
    }

    if tf == .RG8 {
        result = .{ internal = GL.GL_RG8, external = GL.GL_RG, scalar = GL.GL_UNSIGNED_BYTE };
        known = true;
    }

    if tf == .RGB8 {
        result = .{ internal = GL.GL_RGB8, external = GL.GL_RGB, scalar = GL.GL_UNSIGNED_BYTE };
        known = true;
    }

    if tf == .RGBA8 {
        result = .{ internal = GL.GL_RGBA8, external = GL.GL_RGBA, scalar = GL.GL_UNSIGNED_BYTE };
        known = true;
    }

    if tf == .RGBX8 {
        result = .{ internal = GL.GL_RGB8, external = GL.GL_RGBA, scalar = GL.GL_UNSIGNED_BYTE };
        known = true;
    }

    if tf == .BGRA8 {
        result = .{ internal = GL.GL_RGBA8, external = GL.GL_BGRA, scalar = GL.GL_UNSIGNED_BYTE };
        known = true;
    }

    if tf == .B5G6R5 {
        result = .{ internal = GL.GL_RGB565, external = GL.GL_RGB, scalar = GL.GL_UNSIGNED_SHORT_5_6_5 };
        known = true;
    }

    if tf == .RGB10A2 {
        result = .{ internal = GL.GL_RGB10_A2, external = GL.GL_RGBA, scalar = GL.GL_UNSIGNED_INT_2_10_10_10_REV };
        known = true;
    }

    if tf == .R16 {
        result = .{ internal = GL.GL_R16, external = GL.GL_RED, scalar = GL.GL_UNSIGNED_SHORT };
        known = true;
    }

    if tf == .R16F {
        result = .{ internal = GL.GL_R16F, external = GL.GL_RED, scalar = GL.GL_HALF_FLOAT };
        known = true;
    }

    if tf == .RG16F {
        result = .{ internal = GL.GL_RG16F, external = GL.GL_RG, scalar = GL.GL_HALF_FLOAT };
        known = true;
    }

    if tf == .RGBA16F {
        result = .{ internal = GL.GL_RGBA16F, external = GL.GL_RGBA, scalar = GL.GL_HALF_FLOAT };
        known = true;
    }

    if tf == .R32F {
        result = .{ internal = GL.GL_R32F, external = GL.GL_RED, scalar = GL.GL_FLOAT };
        known = true;
    }

    if tf == .RG32F {
        result = .{ internal = GL.GL_RG32F, external = GL.GL_RG, scalar = GL.GL_FLOAT };
        known = true;
    }

    if tf == .RGBA32F {
        result = .{ internal = GL.GL_RGBA32F, external = GL.GL_RGBA, scalar = GL.GL_FLOAT };
        known = true;
    }

    if tf == .RG11B10F {
        result = .{ internal = GL.GL_R11F_G11F_B10F, external = GL.GL_RGB, scalar = GL.GL_UNSIGNED_INT_10F_11F_11F_REV };
        known = true;
    }

    if tf == .RGB9E5F {
        result = .{ internal = GL.GL_RGB9_E5, external = GL.GL_RGB, scalar = GL.GL_UNSIGNED_INT_5_9_9_9_REV };
        known = true;
    }

    if tf == .R16I {
        result = .{ internal = GL.GL_R16I, external = GL.GL_RED_INTEGER, scalar = GL.GL_SHORT };
        known = true;
        result.integer = true;
    }

    if tf == .RG16I {
        result = .{ internal = GL.GL_RG16I, external = GL.GL_RG_INTEGER, scalar = GL.GL_SHORT };
        known = true;
        result.integer = true;
    }

    if tf == .RGBA16I {
        result = .{ internal = GL.GL_RGBA16I, external = GL.GL_RGBA_INTEGER, scalar = GL.GL_SHORT };
        known = true;
        result.integer = true;
    }

    if tf == .R32I {
        result = .{ internal = GL.GL_R32I, external = GL.GL_RED_INTEGER, scalar = GL.GL_INT };
        known = true;
        result.integer = true;
    }

    if tf == .RG32I {
        result = .{ internal = GL.GL_RG32I, external = GL.GL_RG_INTEGER, scalar = GL.GL_INT };
        known = true;
        result.integer = true;
    }

    if tf == .RGBA32I {
        result = .{ internal = GL.GL_RGBA32I, external = GL.GL_RGBA_INTEGER, scalar = GL.GL_INT };
        known = true;
        result.integer = true;
    }

    if tf == .D16 {
        result = .{ internal = GL.GL_DEPTH_COMPONENT16, external = GL.GL_DEPTH_COMPONENT, scalar = GL.GL_UNSIGNED_SHORT };
        known = true;
    }

    if tf == .D24S8 {
        result = .{ internal = GL.GL_DEPTH24_STENCIL8, external = GL.GL_DEPTH_STENCIL, scalar = GL.GL_UNSIGNED_INT_24_8 };
        known = true;
    }

    if tf == .D32F {
        result = .{ internal = GL.GL_DEPTH_COMPONENT32F, external = GL.GL_DEPTH_COMPONENT, scalar = GL.GL_FLOAT };
        known = true;
    }

    if tf == .D32FS8 {
        result = .{ internal = GL.GL_DEPTH32F_STENCIL8, external = GL.GL_DEPTH_STENCIL, scalar = GL.GL_FLOAT_32_UNSIGNED_INT_24_8_REV };
        known = true;
    }

    if tf == .BC1 {
        result.internal = GL.GL_COMPRESSED_RGBA_S3TC_DXT1_EXT;
        result.compressed = true;
        known = true;
    }

    if tf == .BC3 {
        result.internal = GL.GL_COMPRESSED_RGBA_S3TC_DXT5_EXT;
        result.compressed = true;
        known = true;
    }

    if tf == .BC4 {
        result.internal = GL.GL_COMPRESSED_RED_RGTC1;
        result.compressed = true;
        known = true;
    }

    if tf == .BC5 {
        result.internal = GL.GL_COMPRESSED_RG_RGTC2;
        result.compressed = true;
        known = true;
    }

    if tf == .BC6 {
        result.internal = GL.GL_COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT;
        result.compressed = true;
        known = true;
    }

    if tf == .BC7 {
        result.internal = GL.GL_COMPRESSED_RGBA_BPTC_UNORM;
        result.compressed = true;
        known = true;
    }

    if tf == .ETC_RGB {
        result.internal = GL.GL_COMPRESSED_RGB8_ETC2;
        result.compressed = true;
        known = true;
    }

    if tf == .ETC_RGBA {
        result.internal = GL.GL_COMPRESSED_RGBA8_ETC2_EAC;
        result.compressed = true;
        known = true;
    }

    if tf == .ETC_R {
        result.internal = GL.GL_COMPRESSED_R11_EAC;
        result.compressed = true;
        known = true;
    }

    if tf == .ETC_RG {
        result.internal = GL.GL_COMPRESSED_RG11_EAC;
        result.compressed = true;
        known = true;
    }

    if tf == .ETC_RGB_A1 {
        result.internal = GL.GL_COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2;
        result.compressed = true;
        known = true;
    }

    if !known return .{}, false;

    if srgb {
        if tf == .RGB8 result.internal = GL.GL_SRGB8;
        else if tf == .RGBX8 result.internal = GL.GL_SRGB8;
        else if tf == .RGBA8 result.internal = GL.GL_SRGB8_ALPHA8;
        else if tf == .BGRA8 result.internal = GL.GL_SRGB8_ALPHA8;
        else if tf == .BC1 result.internal = GL.GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT;
        else if tf == .BC3 result.internal = GL.GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT;
        else if tf == .BC7 result.internal = GL.GL_COMPRESSED_SRGB_ALPHA_BPTC_UNORM;
        else if tf == .ETC_RGB result.internal = GL.GL_COMPRESSED_SRGB8_ETC2;
        else if tf == .ETC_RGBA result.internal = GL.GL_COMPRESSED_SRGB8_ALPHA8_ETC2_EAC;
        else if tf == .ETC_RGB_A1 result.internal = GL.GL_COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2;
        else return .{}, false;
    }

    return result, true;
}
