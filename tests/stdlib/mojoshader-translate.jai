// MojoShader against the pinned library (tools/build_native_libs.py): Direct3D shader bytecode in,
// GLSL and ARB assembly out, no graphics context. Checks the library identifies itself, a
// hand-written pixel shader token stream translates, the bytecode assembler produces that same
// stream from text, the HLSL compiler (built with its Lemon-generated parser) accepts a small
// shader, and bad input is reported as errors, not a crash.
#import "Basic";
#import "String";
#import "MojoShader";

// ps_2_0: mov oC0, c0 (the version token, the instruction, its destination and source, END).
BYTECODE :: u32.[0xFFFF0200, 0x02000001, 0x800F0800, 0xA0E40000, 0x0000FFFF];

output_of :: (data: *MOJOSHADER_parseData) -> string {
    return string.{ data.output_len, data.output };
}

first_error :: (count: s32, errors: *MOJOSHADER_error) -> string {
    if count == 0 return "";
    return to_string(errors[0].error);
}

main :: () {
    assert(MOJOSHADER_version() == -1 || MOJOSHADER_version() > 0, "version %", MOJOSHADER_version());
    assert(starts_with(to_string(MOJOSHADER_changeset()), "git-"), "changeset '%'", to_string(MOJOSHADER_changeset()));
    assert(MOJOSHADER_maxShaderModel("glsl") >= 2, "glsl shader model %", MOJOSHADER_maxShaderModel("glsl"));

    // Hand-written bytecode, translated to GLSL and to ARB assembly.
    bytes := cast(*u8) BYTECODE.data;
    size := cast(u32) (BYTECODE.count * 4);
    for profile: string.["glsl", "glsl120", "arb1"] {
        data := MOJOSHADER_parse(temp_c_string(profile), null, bytes, size, null, 0, null, 0, null, null, null);
        assert(data.error_count == 0, "% parse: %", profile, first_error(data.error_count, data.errors));
        assert(data.shader_type == .MOJOSHADER_TYPE_PIXEL, "% shader type %", profile, data.shader_type);
        assert(data.major_ver == 2 && data.minor_ver == 0, "% model %.%", profile, data.major_ver, data.minor_ver);
        assert(data.output_len > 0, "% produced no code", profile);
        text := output_of(data);
        if profile == "arb1" {
            assert(contains(text, "!!ARBfp1.0"), "ARB program:\n%", text);
        } else {
            assert(contains(text, "void main()"), "GLSL program:\n%", text);
            assert(contains(text, "gl_FragData[0]") || contains(text, "gl_FragColor"), "GLSL writes the colour:\n%", text);
        }
        MOJOSHADER_freeParseData(data);
    }

    // The bytecode assembler reads the text of the same program and writes the same token stream
    // (it may add comment blocks, so compare the translation instead of the bytes).
    asm_source := "ps_2_0\nmov oC0, c0\n";
    assembled := MOJOSHADER_assemble("assembled.psh", asm_source.data, cast(u32) asm_source.count, null, 0, null, 0, null, 0, null, null, null, null, null);
    assert(assembled.error_count == 0, "assemble: %", first_error(assembled.error_count, assembled.errors));
    assert(assembled.output_len >= cast(s32) size, "assembled % bytes", assembled.output_len);
    from_text := MOJOSHADER_parse("glsl", null, assembled.output, cast(u32) assembled.output_len, null, 0, null, 0, null, null, null);
    assert(from_text.error_count == 0, "parse assembled: %", first_error(from_text.error_count, from_text.errors));
    from_bytes := MOJOSHADER_parse("glsl", null, bytes, size, null, 0, null, 0, null, null, null);
    assert(output_of(from_text) == output_of(from_bytes), "assembled and hand-written programs translate differently");
    MOJOSHADER_freeParseData(from_text);
    MOJOSHADER_freeParseData(from_bytes);
    MOJOSHADER_freeParseData(assembled);

    // HLSL source goes through the compiler front end (lexer, Lemon-generated parser, semantic
    // analysis). At this revision the compiler stops at an intermediate form that it prints to
    // standard output and returns no assembly, so a clean compile is all there is to check.
    hlsl := "float4 main(float4 c : COLOR0) : COLOR0 { return c * 0.5; }\n";
    compiled := MOJOSHADER_compile("hlsl_ps_2_0", "shader.hlsl", hlsl.data, cast(u32) hlsl.count, null, 0, null, null, null, null, null);
    assert(compiled.error_count == 0, "compile: %", first_error(compiled.error_count, compiled.errors));
    assert(to_string(compiled.source_profile) == "hlsl_ps_2_0", "source profile '%'", to_string(compiled.source_profile));
    MOJOSHADER_freeCompileData(compiled);

    // Errors are reported, not fatal: malformed HLSL, and bytes that are no shader.
    broken := "float4 main( : COLOR0 { return; }\n";
    bad := MOJOSHADER_compile("hlsl_ps_2_0", "broken.hlsl", broken.data, cast(u32) broken.count, null, 0, null, null, null, null, null);
    assert(bad.error_count > 0, "malformed HLSL was accepted");
    MOJOSHADER_freeCompileData(bad);
    junk := u32.[1, 2, 3, 4];
    nonsense := MOJOSHADER_parse("glsl", null, cast(*u8) junk.data, 16, null, 0, null, 0, null, null, null);
    assert(nonsense.error_count > 0, "junk bytecode was accepted");
    MOJOSHADER_freeParseData(nonsense);
    print("mojoshader ok\n");
}
