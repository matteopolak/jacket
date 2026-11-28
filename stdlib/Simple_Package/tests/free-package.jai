// `free_package` releases everything a reader allocated, for a package read from a file and for
// one that holds only its table of contents.
B :: #import "Basic";
F :: #import "File";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
P :: #import "Simple_Package";

main :: () {
    creator: P.Create_Package;

    P.init(*creator);
    P.add(*creator, "one", cast([] u8) "1");
    P.add(*creator, "two", cast([] u8) "22");

    path := "simple-package-free.simp";

    B.assert(P.write(*creator, path));
    P.deinit(*creator);
    D.allocations_before_this_call_will_not_be_considered_leaks();

    whole: P.Load_Package;

    B.assert(P.init_from_file(*whole, path));
    B.assert(whole.entries.count == 2);
    P.free_package(*whole);
    B.assert(whole.entries.count == 0);

    ok, table := P.load_table_of_contents_only(path);

    B.assert(ok && table.entries.count == 2);
    P.free_package(*table);

    F.file_delete(path);

    report := D.make_leak_report();

    defer D.deinit(*report);
    B.assert(report.sorted_summaries.count == 0, "% leaks", report.sorted_summaries.count);
}
