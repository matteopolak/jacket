#import "Basic";
#load "tagged-choice-types.jai";

main :: () {
    count := Probe_Choice.{ discriminator = .COUNT, quantity = 37 };
    text := Probe_Choice.{ discriminator = .TEXT, phrase = "chosen" };

    assert(tprint("%", count) == "{COUNT, 37}");
    assert(tprint("%", text) == "{TEXT, chosen}");
    assert(tprint("%", formatStruct(count, use_long_form_if_more_than_this_many_members = -1)) == "{COUNT, 37}");
    assert(tprint("%", formatStruct(text, use_long_form_if_more_than_this_many_members = -1)) == "{TEXT, chosen}");

    print("count short: %\n", count);
    print("text short: %\n", text);
    print("count long: %\n", formatStruct(count, use_long_form_if_more_than_this_many_members = -1));
    print("text long: %\n", formatStruct(text, use_long_form_if_more_than_this_many_members = -1));
}
