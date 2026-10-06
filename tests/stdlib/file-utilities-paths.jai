// File_Utilities paths: parse_path with drives, leading and trailing separators, backslashes,
// "." and ".."; reduce_path, path_contains, path_to_string, how much of a path exists, and
// permanent copies.
#import "Basic";
#import "File";
#import "File_Utilities";

ROOT :: ".scratch/file-utilities-paths";

words_are :: (path: Path, expected: [] string, loc := #caller_location) {
    assert(path.words.count == expected.count, "% words, expected %", path.words.count, expected.count, loc = loc);
    for expected assert(path.words[it_index] == it, "word % is '%', expected '%'", it_index, path.words[it_index], it, loc = loc);
}

main :: () {
    {
        p := parse_path("/usr/local/bin/");
        assert(p.header_string == "" && p.number_of_leading_slashes == 1 && p.trailing_slash);
        words_are(p, .["usr", "local", "bin"]);
        assert(path_to_string(p) == "/usr/local/bin/");
        assert(path_to_string(p, 2) == "/usr/local/");
        assert(path_to_string(p, 0) == "/");
    }
    {
        // Windows drive and backslashes; output uses forward slashes.
        p := parse_path("C:\\Users\\me\\file.txt");
        assert(p.header_string == "C:" && p.number_of_leading_slashes == 1 && !p.trailing_slash);
        words_are(p, .["Users", "me", "file.txt"]);
        assert(path_to_string(p) == "C:/Users/me/file.txt");
        // A drive-relative path has no leading separator.
        q := parse_path("D:dir/x");
        assert(q.header_string == "D:" && q.number_of_leading_slashes == 0);
        words_are(q, .["dir", "x"]);
        assert(path_to_string(q) == "D:dir/x");
        // A colon after the first separator is just part of a word.
        r := parse_path("a/b:c");
        assert(r.header_string == "");
        words_are(r, .["a", "b:c"]);
    }
    {
        // UNC-style double separators, and repeated separators inside.
        p := parse_path("\\\\server\\share//dir///");
        assert(p.number_of_leading_slashes == 2 && p.trailing_slash);
        words_are(p, .["server", "share", "dir"]);
        assert(path_to_string(p) == "//server/share/dir/");
    }
    {
        // Reduction of "." and "..".
        words_are(parse_path("a/./b/../c"), .["a", "c"]);
        words_are(parse_path("../../x/./y/.."), .["..", "..", "x"]);
        words_are(parse_path("a/b/../../.."), .[".."]);
        // Above the root of an absolute path there is nothing to climb to.
        p := parse_path("/../x/../../y");
        assert(p.number_of_leading_slashes == 1);
        words_are(p, .["y"]);
        assert(path_to_string(p) == "/y");
        // Unreduced keeps every word.
        raw := parse_path("a/./b/../c", reduce = false);
        words_are(raw, .["a", ".", "b", "..", "c"]);
        words_are(reduce_path(raw), .["a", "c"]);
    }
    {
        // Empty and trivial inputs.
        e := parse_path("");
        assert(e.words.count == 0 && e.number_of_leading_slashes == 0 && !e.trailing_slash);
        assert(path_to_string(e) == "");
        root := parse_path("/");
        assert(root.number_of_leading_slashes == 1 && root.words.count == 0);
        assert(path_to_string(root) == "/");
        dot := parse_path(".");
        assert(dot.words.count == 0);
        single := parse_path("name");
        words_are(single, .["name"]);
        assert(path_to_string(single) == "name");
    }
    {
        p := parse_path("/a/b/c/b/c/d");
        found, at := path_contains(p, .["b", "c"]);
        assert(found && at == 1, "found % at %", found, at);
        found, at = path_contains(p, .["c", "d"]);
        assert(found && at == 4);
        found, at = path_contains(p, .["c", "a"]);
        assert(!found && at == -1);
        found, at = path_contains(p, .[]);
        assert(found && at == 0);
        found, at = path_contains(p, .["a", "b", "c", "b", "c", "d", "e"]);
        assert(!found && at == -1);
        // Whole words only.
        found = path_contains(p, .["ab"]);
        assert(!found);
    }
    {
        // Permanent copies survive the temporary storage being reset.
        p := parse_path("/keep/these/words/");
        kept := copy_to_permanent(p);
        reset_temporary_storage();
        assert(kept.number_of_leading_slashes == 1 && kept.trailing_slash);
        words_are(kept, .["keep", "these", "words"]);
        assert(path_to_string(kept) == "/keep/these/words/");
        free_permanent(kept);
        drive := copy_to_permanent(parse_path("E:/x"));
        assert(drive.header_string == "E:");
        free_permanent(drive);
    }
    {
        delete_directory(ROOT);
        assert(make_directory_if_it_does_not_exist(tprint("%/one/two", ROOT), recursive = true));
        full, count := how_much_of_path_exists_on_the_local_filesystem(parse_path(tprint("%/one/two", ROOT)));
        assert(full && count == 4, "existing path: % %", full, count);
        full, count = how_much_of_path_exists_on_the_local_filesystem(parse_path(tprint("%/one/missing/deeper", ROOT)));
        assert(!full && count == 3, "partly existing path: % %", full, count);
        full, count = how_much_of_path_exists_on_the_local_filesystem(parse_path("no-such-top-level-dir/x"));
        assert(!full && count == 0);
        // An absolute path keeps its root while being probed.
        absolute := get_absolute_path(tprint("%/one", ROOT));
        full, count = how_much_of_path_exists_on_the_local_filesystem(parse_path(absolute));
        assert(full && count == parse_path(absolute).words.count, "absolute '%': % %", absolute, full, count);
        assert(delete_directory(ROOT));
    }
    print("ok\n");
}
