#import "Basic";
#load "tagged-choice-types.jai";

main :: () {
    value := Probe_Choice.{ discriminator = .COUNT, quantity = 37 };

    assert(value.discriminator == .COUNT && value.quantity == 37);
    info := cast(*Type_Info_Struct) type_info(Probe_Choice);

    assert((info.textual_flags & (.UNION | .UNION_IS_TAGGED)) == (.UNION | .UNION_IS_TAGGED));
    assert(info.runtime_size == size_of(Probe_Choice));
    quantity_index: s64 = -1;
    phrase_index: s64 = -1;

    for member: info.members {
        if member.name == "quantity" {
            quantity_index = it_index;
            assert(member.type == type_info(s64));
            assert(member.offset_in_bytes == offset_of(Probe_Choice, "quantity"));
        }

        if member.name == "phrase" {
            phrase_index = it_index;
            assert(member.type == type_info(string));
            assert(member.offset_in_bytes == offset_of(Probe_Choice, "phrase"));
        }
    }

    assert(quantity_index >= 0 && phrase_index >= 0);
    assert(offset_of(Probe_Choice, "quantity") == offset_of(Probe_Choice, "phrase"));
    count_binding := false;
    text_binding := false;

    for binding: info.tagged_union_bindings {
        if binding.constant_value == 3 {
            assert(binding.member_index == quantity_index);
            count_binding = true;
        }

        if binding.constant_value == 17 {
            assert(binding.member_index == phrase_index);
            text_binding = true;
        }
    }

    assert(count_binding && text_binding);
    value = .{ discriminator = .TEXT, phrase = "chosen" };
    assert(value.discriminator == .TEXT && value.phrase == "chosen");
    print("union metadata/layout ok\n");
}
