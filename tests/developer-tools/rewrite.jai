#import "Basic";
#import "File";
R :: #import "Rewrite";

reports: s64;
errors: s64;

capture_report :: (text: string, _data: *void, info: Log_Info) {
    assert(text.count > 0);
    reports += 1;
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    plugin: R.My_Plugin;
    file: R.File_To_Rewrite;
    file.full_data = "alpha\r\nbeta\ngamma";
    first: R.Single_Rewrite = .{ l0 = 1, c0 = 1, l1 = 1, c1 = 6, new_text = "A" };
    second: R.Single_Rewrite = .{ l0 = 2, c0 = 2, l1 = 3, c1 = 1, new_text = "!" };
    edits: [2] *R.Single_Rewrite = .[*first, *second];

    assert(R.compare_single_rewrites(*first, *second) < 0);
    result := R.make_new_file_data(*plugin, *file, edits);

    defer free(result);
    assert(result == "A\r\nb!gamma");
    path := tprint("%../../target/developer-tools-rewrite.txt", #filepath);

    file.filename = path;
    assert(write_entire_file(path, file.full_data));
    array_add(*file.single_rewrites, second);
    array_add(*file.single_rewrites, first);

    push_context {
        context.logger = capture_report;
        R.rewrite_one_file(*plugin, *file);
        original, ok := read_entire_file(path);

        defer free(original);
        assert(ok && original == file.full_data);
        plugin.write_mode = true;
        R.rewrite_one_file(*plugin, *file);
        assert(reports == 4 && errors == 0);
        R.rewrite_one_file(*plugin, *file);
        assert(errors == 1); // The original snapshot is now stale.
        first.c1 = 4;
        second.l0 = 1;
        second.c0 = 3;
        invalid := R.make_new_file_data(*plugin, *file, edits);

        assert(invalid == "" && errors == 2);
        first.l0 = 0;
        invalid = R.make_new_file_data(*plugin, *file, edits);
        assert(invalid == "" && errors == 3);
    }

    changed, changed_ok := read_entire_file(path);

    defer free(changed);
    assert(changed_ok && changed == result);
    print("ok\n");
}
