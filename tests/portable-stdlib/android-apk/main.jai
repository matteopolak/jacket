// Provenance: reviewed APK whole-file views, local payload offsets and iteration.
using #import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    apk := A.apk_load(args[1]);

    assert(apk.entire_apk_is_loaded);
    assert(apk.filepath.data == args[1].data);
    assert(apk.data.count > apk.content.count);
    assert(apk.central_directory.data == apk.content.data + apk.content.count + apk.signing_block.count);
    assert(apk.end_of_central_directory.data == apk.central_directory.data + apk.central_directory.count);
    assert(apk.zip_dir.memory_block.data != apk.central_directory.data);
    assert(apk.zip_dir.entries.count == 3);

    if apk.is_signed {
        assert(apk.signing_block.count == 32);
        assert(apk.signing_block.data == apk.content.data + apk.content.count);
    } else assert(apk.signing_block.count == 0);

    count: s64;

    for apk {
        assert(it_index == count);
        assert(it.filename == apk.zip_dir.entries[count].filename);
        assert(it.offset > apk.zip_dir.entries[count].chunk_offset);
        assert(it.compressed_size == apk.zip_dir.entries[count].compressed_len);
        assert(it.uncompressed_size == apk.zip_dir.entries[count].chunk_len);
        assert(it.offset + it.compressed_size <= apk.content.count);
        if it_index == 0 assert(it.offset % 4096 == 0);

        count += 1;
    }

    assert(count == 3);
    expected := 2;

    for < apk {
        assert(it_index == expected);
        expected -= 1;
    }

    assert(expected == -1);
    A.apk_print_sections(apk);
    A.apk_deinit(*apk);
    assert(apk.data.data == null && apk.zip_dir.memory_block.data == null);
    assert(apk.filepath.count == 0 && apk.content.count == 0 && apk.signing_block.count == 0);
    assert(apk.central_directory.count == 0 && apk.end_of_central_directory.count == 0);
    assert(!apk.entire_apk_is_loaded && !apk.is_signed);
    A.apk_deinit(*apk);
    print("ok\n");
}
