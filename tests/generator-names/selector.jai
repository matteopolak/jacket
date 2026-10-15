#import "Basic";
Names :: #import "Bindings_Generator";

main :: () {
    unchanged := "description";

    Names.convert_objc_selector_name_to_function_identifier(*unchanged);
    assert(unchanged == "description");

    unary := "setValue:";
    original := unary;

    Names.convert_objc_selector_name_to_function_identifier(*unary);
    assert(unary == "setValue");
    assert(original == "setValue:");
    assert(unary.data == original.data);

    empty := "";

    Names.convert_objc_selector_name_to_function_identifier(*empty);
    assert(empty == "");

    print("generator selector contract subset passed\n");
}
