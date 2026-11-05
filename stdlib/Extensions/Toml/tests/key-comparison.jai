#import "Basic";
#import "Extensions/Toml";

// Exercise the actual private comparator without adding a testing API.
Toml_Index :: type_of((Toml_Value.{}).internal_lookup.*);
Toml_Index_Node :: type_of((Toml_Index.{}).nodes.data.*);

#load "../index.jai";

main :: () {
    left: [80] u8;
    right: [80] u8;

    for 0..79 {
        left[it] = 97;
        right[it] = 97;
    }

    // Common prefixes, terminal differences, aliasing, empty views and unaligned starts.
    for offset: 0..7 {
        for length: 0..65 {
            a := string.{ length, left.data + offset };
            b := string.{ length, right.data + offset };

            assert(toml_key_compare(a, b) == 0);
            assert(toml_key_compare(a, a) == 0);
            longer := string.{ length + 1, left.data + offset };

            assert(toml_key_compare(a, longer) == -1);
            assert(toml_key_compare(longer, a) == 1);

            if length > 0 {
                right[offset + length - 1] = 98;
                assert(toml_key_compare(a, b) == -1);
                assert(toml_key_compare(b, a) == 1);
                right[offset + length - 1] = 97;
            }
        }
    }

    empty: string;

    assert(empty.data == null);
    assert(toml_key_compare(empty, empty) == 0);
    assert(toml_key_compare(empty, "a") == -1);
    assert(toml_key_compare("a", empty) == 1);
    shared := "aaaab";
    overlap_left := string.{ 3, shared.data };
    overlap_right := string.{ 3, shared.data + 2 };

    assert(toml_key_compare(overlap_left, overlap_right) == -1);
    assert(toml_key_compare(overlap_right, overlap_left) == 1);
    left[0] = 127;
    right[0] = 128;
    assert(toml_key_compare(string.{ 1, left.data }, string.{ 1, right.data }) == -1);
    assert(toml_key_compare(string.{ 1, right.data }, string.{ 1, left.data }) == 1);
    right[0] = 255;
    assert(toml_key_compare(string.{ 1, left.data }, string.{ 1, right.data }) == -1);
    assert(toml_key_compare(string.{ 1, right.data }, string.{ 1, left.data }) == 1);

    // Comparison is unsigned byte order and does not stop at embedded NUL bytes.
    left[0] = 0;
    right[0] = 0;
    left[1] = 128;
    right[1] = 255;
    assert(toml_key_compare(string.{ 2, left.data }, string.{ 2, right.data }) == -1);
    assert(toml_key_compare(string.{ 2, right.data }, string.{ 2, left.data }) == 1);
    assert(toml_key_compare("", "") == 0);
    assert(toml_key_compare("a", "b") == -1);
    assert(toml_key_compare("a", "aa") == -1);
    assert(toml_key_compare("é", "🙂") == -1);
}
