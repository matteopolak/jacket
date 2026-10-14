M :: #import "MojoShader";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

probe :: #library "../../.work/native/abi_probe";
check_size :: (which: s32) -> u64 #foreign probe "binding_size";
vendor_version :: () -> s32 #foreign probe "binding_mojoshader_version_constant";

main :: () {
    check(M.MOJOSHADER_version() == vendor_version());
    check(M.MOJOSHADER_changeset() != null);
    check(M.MOJOSHADER_maxShaderModel("glsl") >= 2);
    check(size_of(M.MOJOSHADER_uniform) == check_size(14));
    check(size_of(M.MOJOSHADER_constant) == check_size(15));
    check(size_of(M.MOJOSHADER_parseData) == check_size(16));
    check(size_of(M.MOJOSHADER_preprocessData) == check_size(17));
    text := "#define VALUE 7\nVALUE\n";
    processed := M.MOJOSHADER_preprocess("binding-test", text.data, cast(u32) text.count,
                                       null, 0, null, null, null, null, null);

    check(processed != null && processed.error_count == 0);
    defer M.MOJOSHADER_freePreprocessData(processed);
    check(processed.output != null && processed.output_len > 0);
    seven := false;
    for i: 0..processed.output_len - 1 if processed.output[i] == #char "7" seven = true;

    check(seven);
    shader := "vs_2_0\ndcl_position v0\nmov oPos, v0\n";
    assembled := M.MOJOSHADER_assemble("binding-test", shader.data, cast(u32) shader.count,
                                     null, 0, null, 0, null, 0, null, null, null, null, null);

    check(assembled != null && assembled.error_count == 0);
    defer M.MOJOSHADER_freeParseData(assembled);
    check(assembled.output != null && assembled.output_len > 0);
    parsed := M.MOJOSHADER_parse("glsl", "main", assembled.output, cast(u32) assembled.output_len,
                                null, 0, null, 0, null, null, null);

    check(parsed != null && parsed.error_count == 0 && parsed.output_len > 0);
    M.MOJOSHADER_freeParseData(parsed);
}
