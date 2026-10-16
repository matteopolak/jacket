#import "Basic";

Probe_Float :: #type,distinct float64;
Probe_Float_Child :: #type,isa Probe_Float;
Probe_Integer :: #type,distinct s64;
Probe_Integer_Child :: #type,isa Probe_Integer;

main :: () {
    args := get_command_line_arguments();
    mode := ifx args.count > 1 then args[1] else "base";

    if mode == "base" {
        assert(tprint("%", formatFloat(cast(float64) 8.5, trailing_width = 2, zero_removal = .NO)) == "8.50");
        assert(tprint("%", formatInt(cast(s64) 41, base = 16)) == "29");
        print("base numeric formatting ok\n");
        return;
    }

    number := cast(Probe_Float_Child) cast(float64) 8.5;
    integer := cast(Probe_Integer_Child) cast(s64) 41;
    floating := formatFloat(number, trailing_width = 2, zero_removal = .NO);
    integral := formatInt(integer, base = 16);

    assert(floating.value.type == type_info(Probe_Float_Child));
    assert(integral.value.type == type_info(Probe_Integer_Child));
    assert(floating.value.type.runtime_size == size_of(Probe_Float_Child));
    assert(integral.value.type.runtime_size == size_of(Probe_Integer_Child));
    assert((cast(*float64) floating.value.value_pointer).* == 8.5);
    assert((cast(*s64) integral.value.value_pointer).* == 41);
    info := floating.value.type;
    while info.type == .VARIANT info = (cast(*Type_Info_Variant) info).variant_of;

    assert(info == type_info(float64));
    info = integral.value.type;
    while info.type == .VARIANT info = (cast(*Type_Info_Variant) info).variant_of;

    assert(info == type_info(s64));

    if mode == "metadata" {
        print("numeric variant metadata/storage ok\n");
        return;
    }

    if mode == "float" {
        print("variant float: %\n", floating);
        assert(tprint("%", floating) == "8.50");
        assert(floating.value.type == type_info(Probe_Float_Child));
        return;
    }

    assert(mode == "int");
    print("variant integer: %\n", integral);
    assert(tprint("%", integral) == "29");
    assert(integral.value.type == type_info(Probe_Integer_Child));
}
