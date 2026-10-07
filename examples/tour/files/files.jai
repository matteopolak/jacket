// Files: the File and File_Utilities modules. In the browser, reads see this
// workspace (as /workspace) and the standard library (as /stdlib); writes go to
// an in-memory disk that lasts for one run.

#import "File";
#import "File_Utilities";

tour_files :: () {
    // #filepath is the folder of the current source file, so `..` is the tour.
    tour_folder := tprint("%..", #filepath);

    // Visit every .jai file below it and count its lines.
    Census :: struct {
        files: int;
        lines: int;
        longest_name: string;
        longest_lines: int;
    }
    census: Census;
    visit_files(tour_folder, true, *census, (info: *File_Visit_Info, census: *Census) {
        if !ends_with(info.short_name, ".jai") return;
        text, ok := read_entire_file(info.full_name);
        if !ok return;
        lines := count_lines(text);
        census.files += 1;
        census.lines += lines;
        if lines > census.longest_lines {
            census.longest_lines = lines;
            census.longest_name = copy_temporary_string(info.short_name);
        }
    });
    print("The tour has % .jai files and % lines; the longest is % (% lines).\n",
        census.files, census.lines, census.longest_name, census.longest_lines);

    // Read a file of the workspace: the headings of tour.md.
    guide, found := read_entire_file(tprint("%/tour.md", tour_folder));
    if found {
        print("tour.md's headings:");
        for line: split(guide, "\n") {
            if begins_with(line, "#") print(" \"%\"", trim_left(line, "# "));
        }
        print("\n");
    }

    // Write a file a line at a time, read it back, then delete it.
    path := "/tmp/jai-tour-log.txt";
    file, opened := file_open(path, for_writing = true);
    if !opened return;
    for 1..3 file_write(*file, tprint("entry %: % squared is %\n", it, it, it * it));
    file_close(*file);

    log_text := read_entire_file(path);
    print("Read back % bytes from %:\n%", log_text.count, path, log_text);
    free(log_text);
    file_delete(path);
    print("After file_delete, file_exists says %.\n", file_exists(path));
}

count_lines :: (text: string) -> int {
    lines := 0;
    for 0..text.count - 1 if text[it] == #char "\n" lines += 1;
    return lines;
}
