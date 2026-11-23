// Text_Buffer line index after inserts, erases and replaces that add or remove breaks, and text queries on the result.
#import "Basic";
#import "Extensions/Text_Buffer";

main :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);
    set_text(*buffer, "one\r\ntwo\nthree");
    print("start lines=% length=%\n", line_count(*buffer), buffer.length);
    assert(line_count(*buffer) == 3);

    // Adding a break in the middle of "two" splits it; the new line starts after the inserted \n.
    insert(*buffer, 7, "\n");
    print("after split lines=%\n", line_count(*buffer));
    assert(line_count(*buffer) == 4);
    assert(line_text(*buffer, 1) == "tw" && line_text(*buffer, 2) == "o" && line_text(*buffer, 3) == "three");

    // Removing the CRLF joins lines 0 and 1, so the line count drops.
    erase(*buffer, 3, 2);
    print("after join lines=% first=[%]\n", line_count(*buffer), line_text(*buffer, 0));
    assert(line_count(*buffer) == 3 && line_text(*buffer, 0) == "onetw");

    // Replacing a whole word with a longer one keeps the line structure intact.
    replace(*buffer, find_text(*buffer, "three"), 5, "THREE!");
    assert(line_text(*buffer, 1) == "o" && line_text(*buffer, 2) == "THREE!");
    print("text=%|\n", buffer_to_string(*buffer,, temp));

    // A range query and a search from an offset.
    print("range=[%] from=% missing=%\n", text_range(*buffer, 0, 5), find_text(*buffer, "o", 4), find_text(*buffer, "zzz"));
    assert(text_range(*buffer, 0, 5) == "onetw");
    assert(find_text(*buffer, "o", 4) == 6 && find_text(*buffer, "zzz") == -1);

    // A line past the end clamps to the last line.
    print("last=[%] count=%\n", line_text(*buffer, 50), line_count(*buffer));
    assert(line_text(*buffer, 50) == "THREE!");

    // Copying a range gives an independent string.
    copy := copy_range(*buffer, 0, 5, temp);

    erase(*buffer, 0, 3);
    print("copy=[%] buffer=[%]\n", copy, buffer_to_string(*buffer,, temp));
    assert(copy == "onetw" && buffer_to_string(*buffer,, temp) == "tw\no\nTHREE!");
    print("ok\n");
}
