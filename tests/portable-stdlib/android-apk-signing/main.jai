// Independent wire fixtures. Framing strings are deliberately not DER credentials.
using #import "Basic";
F :: #import "File";

Apk_Test_Allocation_State :: struct {
    original: Allocator;
    live: s64;
}

apk_test_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, user_data: *void) -> *void {
    state := cast(*Apk_Test_Allocation_State) user_data;
    result := state.original.proc(mode, size, old_size, old_memory, state.original.data);

    if (mode == .ALLOCATE || mode == .RESIZE) && old_memory == null && result != null state.live += 1;
    if mode == .FREE && old_memory != null state.live -= 1;

    return result;
}

apk_test_assertions: s64;
apk_test_errors: s64;

apk_test_capture_error :: (message: string, user_data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 apk_test_errors += 1;
}

apk_test_continue_assertion :: (location: Source_Code_Location, message: string) -> bool {
    apk_test_assertions += 1;
    return false;
}

fixture_block :: (cursor: *string) -> string {
    ok, value := apk_v2_take_length32(cursor);
    assert(ok);
    return value;
}

fixture_count :: (cursor: *string) -> u32 {
    ok, data := apk_v2_take(cursor, 4);
    assert(ok);
    return apk_v2_u32(data.data);
}

load_fixture :: (directory: string, name: string) -> string {
    path := sprint("%/%", directory, name);

    defer free(path);
    data, loaded := F.read_entire_file(path);

    assert(loaded);

    return data;
}

apk_signing_test_main :: () {
    assert(size_of(Algorithm_Id) == 4);
    assert(cast(u32) Algorithm_Id.RSASSA_PSS_SHA256 == 0x0101);
    assert(cast(u32) Algorithm_Id.RSASSA_PSS_SHA512 == 0x0102);
    assert(cast(u32) Algorithm_Id.RSASSA_PKCS1_v1_5_SHA256 == 0x0103);
    assert(cast(u32) Algorithm_Id.RSASSA_PKCS1_v1_5_SHA512 == 0x0104);
    assert(cast(u32) Algorithm_Id.ECDSA_SHA256 == 0x0201);
    assert(cast(u32) Algorithm_Id.ECDSA_SHA512 == 0x0202);
    assert(cast(u32) Algorithm_Id.DSA_SHA256 == 0x0301);
    assert(cast(int) Digest_Hash_Algorithm.SHA256 == 0 && cast(int) Digest_Hash_Algorithm.SHA512 == 1);
    assert(SIGNING_BLOCK_ID_V2_SIGNATURE == 0x7109871a && SIGNING_BLOCK_ID_V3_SIGNATURE == 0xf05368c0);
    assert(SIGNING_BLOCK_ID_V31_SIGNATURE == 0x1b93ad61 && SIGNING_BLOCK_ID_VERITY_PADDING == 0x42726577);
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);

    fixture := load_fixture(args[1], "framing.bin");

    defer free(fixture);
    cursor := fixture;
    container := fixture_block(*cursor);
    signed_data := fixture_block(*cursor);
    digest := fixture_block(*cursor);
    certificate := fixture_block(*cursor);
    signature := fixture_block(*cursor);
    spki := fixture_block(*cursor);

    assert(cursor.count == 0);
    ok, model := apk_v2_read_container(container);

    assert(ok);
    defer apk_v2_free_arrays(*model);
    assert(model.signers_count == 1);
    assert(model.digests.count == 1 && model.signatures.count == 1 && model.certificates.count == 1);
    assert(model.digests[0].signature_algorithm_id == .ECDSA_SHA256);
    assert(model.signatures[0].algorithm_id == .ECDSA_SHA256);
    assert(model.signed_data == signed_data && model.digests[0].data == digest);
    assert(model.certificates[0].data == certificate && model.signatures[0].data == signature);
    assert(model.subject_public_key_info == spki);
    // Pair value ends immediately before size/magic; these later bytes survive.
    public_cursor := string.{ container.count - 20, container.data + 20 };
    public_ok, public_model := apk_read_signing_block_v2(*public_cursor);

    assert(public_ok && public_cursor.count == 24);
    assert(public_cursor.data == container.data + container.count - 24);
    assert(public_model.signed_data.data == model.signed_data.data);
    apk_v2_free_arrays(*public_model);
    assert(model.signed_data.data >= container.data);
    assert(model.signed_data.data + model.signed_data.count <= container.data + container.count);
    old := model.digests[0].data[0];

    model.digests[0].data.data[0] = 199;
    assert(container[model.digests[0].data.data - container.data] == 199);
    model.digests[0].data.data[0] = old;

    serialized := apk_v2_serialize_block(digest, certificate, signature, spki);

    defer free(serialized);
    assert(serialized == container);

    // Arrays remember the active allocator; borrowed wire strings remain alive.
    allocation_state := Apk_Test_Allocation_State.{ original = context.allocator };

    context.allocator = .{ apk_test_allocator, *allocation_state };
    allocation_ok, allocated := apk_v2_read_container(container);

    context.allocator = allocation_state.original;
    assert(allocation_ok && allocation_state.live == 3);
    apk_v2_free_arrays(*allocated);
    assert(allocation_state.live == 0 && container.count > 0);
    apk_v2_free_arrays(*allocated);
    assert(allocation_state.live == 0);

    unsigned_path := sprint("%/unsigned.zip", args[1]);

    defer free(unsigned_path);
    unsigned := apk_load(unsigned_path);

    defer apk_deinit(*unsigned);
    assert(!unsigned.is_signed && unsigned.zip_dir.entries.count == 1);
    expected_zip_digest := load_fixture(args[1], "zip-digest.bin");

    defer free(expected_zip_digest);
    zip_digest := apk_compute_digest_libressl(unsigned, .SHA256);

    assert(string.{ 32, zip_digest.data } == expected_zip_digest);
    assertion := context.assertion_failed;

    context.assertion_failed = apk_test_continue_assertion;
    unsupported := apk_compute_digest_libressl(unsigned, .SHA512);

    context.assertion_failed = assertion;
    assert(apk_test_assertions == 1);
    for byte: unsupported assert(byte == 0);

    assembled := apk_v2_assemble(unsigned, serialized);

    defer free(assembled);
    framed := load_fixture(args[1], "framed.zip");

    defer free(framed);
    assert(assembled == framed);
    signed_path := sprint("%/framed.zip", args[1]);

    defer free(signed_path);
    signed_apk := apk_load(signed_path);

    defer apk_deinit(*signed_apk);
    assert(signed_apk.is_signed && signed_apk.signing_block == container);
    signed_digest := apk_compute_digest_libressl(signed_apk, .SHA256);

    assert(string.{ 32, signed_digest.data } == expected_zip_digest);

    // Genuine native EC/SHA256 vector, generated independently of this code.
    native_fixture := load_fixture(args[1], "native-framing.bin");

    defer free(native_fixture);
    cursor = native_fixture;
    native_block := fixture_block(*cursor);
    native_data := fixture_block(*cursor);
    native_digest := fixture_block(*cursor);
    native_certificate := fixture_block(*cursor);
    native_signature := fixture_block(*cursor);
    native_spki := fixture_block(*cursor);

    assert(cursor.count == 0);
    native_ok, native_model := apk_v2_read_container(native_block);

    assert(native_ok);
    defer apk_v2_free_arrays(*native_model);
    assert(native_model.signed_data == native_data && native_model.digests[0].data == native_digest);
    assert(native_model.certificates[0].data == native_certificate && native_model.signatures[0].data == native_signature);
    assert(native_model.subject_public_key_info == native_spki);
    certificate_spki := x509_get_subject_public_key_info(native_certificate);

    assert(certificate_spki.value_data == native_spki);
    signature_cursor := native_signature;
    signature_value := asn1_read_value(*signature_cursor);

    assert(signature_cursor.count == 0 && signature_value.universal_type == .SEQUENCE);
    components := signature_value.content;
    r := asn1_read_value(*components);
    s := asn1_read_value(*components);

    assert(components.count == 0 && r.universal_type == .INTEGER && s.universal_type == .INTEGER);
    assert(r.content.count >= 1 && r.content.count <= 33 && s.content.count >= 1 && s.content.count <= 33);
    native_serialized := apk_v2_serialize_block(native_digest, native_certificate, native_signature, native_spki);

    defer free(native_serialized);
    assert(native_serialized == native_block);
    native_assembled := apk_v2_assemble(unsigned, native_block);

    defer free(native_assembled);
    native_signed := load_fixture(args[1], "native-signed.zip");

    defer free(native_signed);
    assert(native_assembled == native_signed);

    cases := load_fixture(args[1], "reader-cases.bin");

    defer free(cases);
    cursor = cases;
    good_count := fixture_count(*cursor);
    for 0..cast(s64) good_count - 1 {
        input := fixture_block(*cursor);
        valid, parsed := apk_v2_read_container(input);

        assert(valid);
        apk_v2_free_arrays(*parsed);
        assert(parsed.digests.count == 0 && parsed.signatures.count == 0 && parsed.certificates.count == 0);
    }

    bad_count := fixture_count(*cursor);
    for 0..cast(s64) bad_count - 1 {
        input := fixture_block(*cursor);
        valid, parsed := apk_v2_read_container(input);
        assert(!valid);
        assert(parsed.digests.count == 0 && parsed.signatures.count == 0 && parsed.certificates.count == 0);
    }

    bad_payload_count := fixture_count(*cursor);
    for 0..cast(s64) bad_payload_count - 1 {
        input := fixture_block(*cursor);
        valid, parsed := apk_v2_read_payload(input);

        assert(!valid);
        assert(parsed.digests.count == 0 && parsed.signatures.count == 0 && parsed.certificates.count == 0);
        logger := context.logger;
        original_assertion := context.assertion_failed;
        before_assertions := apk_test_assertions;
        before_errors := apk_test_errors;

        context.logger = apk_test_capture_error;
        context.assertion_failed = apk_test_continue_assertion;
        rejected := input;
        public_valid, public_parsed := apk_read_signing_block_v2(*rejected);

        context.logger = logger;
        context.assertion_failed = original_assertion;

        // A valid first value with later bytes can succeed even when the
        // whole-input internal reader rejects trailing data.
        if public_valid {
            assert(rejected.count > 0);
            assert(apk_test_assertions == before_assertions && apk_test_errors == before_errors);
            apk_v2_free_arrays(*public_parsed);
        } else {
            assert(apk_test_assertions == before_assertions + 1);
            assert(apk_test_errors == before_errors + 1);
            assert(public_parsed.digests.count == 0 && public_parsed.signatures.count == 0 && public_parsed.certificates.count == 0);
        }
    }

    assert(cursor.count == 0);

    // Failed length-prefixed reads must leave their caller's cursor unchanged.
    short_prefix := string.{ 3, container.data };
    initial := short_prefix;
    short_ok := apk_v2_take_length32(*short_prefix);

    assert(!short_ok && short_prefix.data == initial.data && short_prefix.count == initial.count);
    oversized_prefix: [4] u8;

    apk_v2_write_u32(oversized_prefix.data, 0xffffffff);
    oversized := string.{ 4, oversized_prefix.data };

    initial = oversized;
    oversized_ok := apk_v2_take_length32(*oversized);

    assert(!oversized_ok && oversized.data == initial.data && oversized.count == initial.count);
    null_input := string.{ count = 4 };
    null_ok := apk_v2_take_length32(*null_input);

    assert(!null_ok);
    negative := string.{ count = -1 };
    negative_ok := apk_v2_take(*negative, 0);

    assert(!negative_ok);

    policies := load_fixture(args[1], "verification-structure.bin");

    defer free(policies);
    cursor = policies;
    accepted_count := fixture_count(*cursor);
    for 0..cast(s64) accepted_count - 1 {
        input := fixture_block(*cursor);
        logger := context.logger;

        context.logger = apk_test_capture_error;
        accepted, accepted_model := apk_v2_read_container(input, verify_structure = true);

        context.logger = logger;
        assert(accepted);
        apk_v2_free_arrays(*accepted_model);
    }

    rejected_count := fixture_count(*cursor);
    for 0..cast(s64) rejected_count - 1 {
        input := fixture_block(*cursor);
        accepted, rejected_model := apk_v2_read_container(input, verify_structure = true);
        assert(!accepted && rejected_model.digests.count == 0);
    }

    assert(cursor.count == 0);

    vectors := load_fixture(args[1], "digests.bin");

    defer free(vectors);
    cursor = vectors;
    count := fixture_count(*cursor);
    for 0..cast(s64) count - 1 {
        apk: Android_Apk;
        apk.content = fixture_block(*cursor);
        apk.central_directory = fixture_block(*cursor);
        apk.end_of_central_directory = fixture_block(*cursor);
        expected_ok, expected := apk_v2_take(*cursor, 32);

        assert(expected_ok);
        before := zip_eocd_get_start_of_central_directory_offset(apk.end_of_central_directory);
        actual := apk_compute_digest_libressl(apk, .SHA256);
        actual_bytes := string.{ 32, actual.data };

        assert(actual_bytes == expected);
        assert(zip_eocd_get_start_of_central_directory_offset(apk.end_of_central_directory) == before);
        apk.is_signed = true;
        again := apk_compute_digest_libressl(apk, .SHA256);

        assert(string.{ 32, again.data } == expected);
        assert(zip_eocd_get_start_of_central_directory_offset(apk.end_of_central_directory) == before);
    }

    assert(cursor.count == 0);
    print("ok\n");
}
