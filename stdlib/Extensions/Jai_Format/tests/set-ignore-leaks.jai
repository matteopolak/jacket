#import "Basic";
#import "Extensions/Jai_Format";
#load "../../test-support/counting-allocator.jai";

// Replacing `ignore` through format_apply_setting releases the globs it replaces.
main :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        config, ok, error := parse_config("[format]\nignore = ['a', 'b']\n");

        assert(ok, error);
        assert(config.ignore.count == 2);

        set_ok, set_error := format_apply_setting(*config, "ignore=[\"c\"]");

        assert(set_ok, set_error);
        assert(config.ignore.count == 1 && config.ignore[0] == "c");

        for config.ignore free(it);

        array_free(config.ignore);
    }

    assert(tally.outstanding == 0, tprint("% allocations were not released", tally.outstanding));
    print("ok\n");
}
