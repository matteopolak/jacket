// Actual public DER from independently generated native X509 certificates.
#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";

main :: () {
    arguments := get_command_line_arguments();

    assert(arguments.count == 2, "pass the independently generated public certificate vectors filename");
    filename := arguments[1];
    data, loaded := F.read_entire_file(filename);

    assert(loaded);
    defer free(data);
    cursor := data;
    count := A.read(u32, *cursor);

    assert(count == 4);

    for 0..cast(s64) count - 1 {
        certificate := A.read_length_prefixed_block(*cursor);
        expected := A.read_length_prefixed_block(*cursor);
        extracted := A.x509_get_subject_public_key_info(certificate);

        assert(extracted.value_data == expected && extracted.class == .UNIVERSAL && extracted.universal_type == .SEQUENCE);
        assert(extracted.length_value == .CONSTRUCTED);
        assert(cast(u64) extracted.value_data.data >= cast(u64) certificate.data);
        assert(cast(u64) extracted.value_data.data + cast(u64) extracted.value_data.count <= cast(u64) certificate.data + cast(u64) certificate.count);
        // Shared backing identity is observable when the caller edits its buffer.
        offset := extracted.content.count - 1;
        saved := extracted.content[offset];

        extracted.content.data[offset] ^= 1;
        assert(extracted.value_data[extracted.value_data.count - 1] == (saved ^ 1));
        extracted.content.data[offset] = saved;
        assert(extracted.value_data == expected);
    }

    assert(cursor.count == 0);
    print("ok\n");
}
