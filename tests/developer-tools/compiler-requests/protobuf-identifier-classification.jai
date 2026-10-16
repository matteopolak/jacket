// Original reduction pinned to the reviewed Basic byte-classification contract.
#import "Basic";

main :: () {
    assert(!is_alpha(95) && is_alnum(95));
    assert(is_alpha(65) && is_alnum(65));
    assert(!is_alpha(48) && is_alnum(48) && is_digit(48));
    print("underscore alpha=% alnum=%; letter alpha=% alnum=%; digit alpha=% alnum=%\n",
          is_alpha(95), is_alnum(95), is_alpha(65), is_alnum(65), is_alpha(48), is_alnum(48));
}
