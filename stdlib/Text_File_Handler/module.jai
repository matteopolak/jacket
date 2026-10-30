// A line reader for simple text and configuration files.
//
// It skips blank lines and `#` comments, can read a `[version]` header, and reports problems with
// the file and line number through the context logger. Use `file_to_array` or `file_to_table` to
// load a whole file, or a `Text_File_Handler` to go line by line.
//
// ```jai
// ok, lines := file_to_array("x", use_data_from_memory = true, data_from_memory = "a\n# c\nb\n");
// assert(ok && lines.count == 2);
// for line: lines free(line);
// array_free(lines);
// ```

Text_File_Handler :: Impl.Text_File_Handler;
start_from_memory :: Impl.start_from_memory;
start_file :: Impl.start_file;
deinit :: Impl.deinit;
consume_next_line :: Impl.consume_next_line;
break_by_spaces :: Impl.break_by_spaces;
error :: Impl.error;
warn :: Impl.warn;
file_to_array :: Impl.file_to_array;
file_to_table :: Impl.file_to_table;

#scope_file
Impl :: #import "internal/files/text-handler";
